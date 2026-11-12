//! Commands from outside, such as the compositor's key bindings, over a socket in
//! `$XDG_RUNTIME_DIR` (`shojiwm-shell.$WAYLAND_DISPLAY.sock`). They are the QML shell's
//! `quickshell ipc call <target> <function> [args]`, with the same names:
//!
//! ```text
//! shojiwm-shell msg startmenu toggleOn DP-2
//! shojiwm-shell msg launcher toggleOn DP-2
//! shojiwm-shell msg run toggleOn DP-2
//! shojiwm-shell msg shutdown openOn DP-2
//! shojiwm-shell msg adminmenu toggleOn DP-2
//! shojiwm-shell msg lock lock
//! shojiwm-shell msg taskbar activate 3
//! shojiwm-shell msg appearance setDark true
//! ```
//!
//! A surface registers its target with [`handle`]; the call runs on the UI thread that
//! registered it, and what it returns is printed by `msg`.
//!
//! A target that answers once the user has acted (a dialog that asks) registers with
//! [`handle_later`] instead: it gets a [`Reply`] to send when it knows. Such a call waits with no
//! time limit. If the caller goes away first (its process was killed), the reply's
//! [`Reply::on_gone`] runs on the UI thread, so the dialog can close.

use anti_webui::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::Duration;

/// A target's functions: the function's name and its arguments, to an answer or an error.
pub type Handler = Rc<dyn Fn(&str, &[String]) -> Result<String, String>>;

/// A target's functions that answer later: the function's name, its arguments, and the reply to
/// send when the answer is known.
type LaterHandler = Rc<dyn Fn(&str, &[String], Reply)>;

#[derive(Clone)]
enum Target {
    Now(Handler),
    Later(LaterHandler),
}

/// How long an ordinary call has to answer.
const ANSWER_WITHIN: Duration = Duration::from_secs(5);
/// How often the socket thread looks whether the caller of a deferred call is still there.
const PEER_CHECK: Duration = Duration::from_millis(250);
/// What a deferred call answers when its reply is dropped unsent or its caller has gone.
const CANCELLED: &str = "error cancelled";

thread_local! {
    static HANDLERS: RefCell<HashMap<String, Target>> = RefCell::new(HashMap::new());
    /// What runs when the caller of a deferred call goes away, by call id.
    static GONE: RefCell<HashMap<u64, Box<dyn FnOnce()>>> = RefCell::new(HashMap::new());
}

/// Answers the calls to `target`. A second call for the same target replaces the first.
pub fn handle(target: &str, f: impl Fn(&str, &[String]) -> Result<String, String> + 'static) {
    HANDLERS.with(|h| {
        h.borrow_mut()
            .insert(target.to_owned(), Target::Now(Rc::new(f)))
    });
}

/// Answers the calls to `target` later: `f` gets a [`Reply`] that it sends once, from this UI
/// thread, when the answer is known. A reply dropped unsent answers "cancelled". A second call
/// for the same target replaces the first.
pub fn handle_later(target: &str, f: impl Fn(&str, &[String], Reply) + 'static) {
    HANDLERS.with(|h| {
        h.borrow_mut()
            .insert(target.to_owned(), Target::Later(Rc::new(f)))
    });
}

/// The answer to a deferred call, sent once. Dropping it unsent answers "cancelled".
pub struct Reply {
    id: u64,
    sender: Option<Sender<Heard>>,
}

impl Reply {
    /// Answers the call: `Ok` is printed by the caller, `Err` is its error.
    pub fn send(mut self, result: Result<String, String>) {
        self.finish(match result {
            Ok(said) => format!("ok {said}"),
            Err(e) => format!("error {e}"),
        });
    }

    /// Runs `f` on this UI thread if the caller goes away before the answer. A second call
    /// replaces the first.
    pub fn on_gone(&self, f: impl FnOnce() + 'static) {
        let id = self.id;
        GONE.with(|g| g.borrow_mut().insert(id, Box::new(f)));
    }

    fn finish(&mut self, text: String) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(Heard::Answer(text));
        }
        let id = self.id;
        // `try_with`: a reply kept in another thread-local can drop after this one is gone,
        // when the thread ends.
        let _ = GONE.try_with(|g| g.borrow_mut().remove(&id));
    }
}

impl Drop for Reply {
    fn drop(&mut self) {
        self.finish(CANCELLED.to_owned());
    }
}

fn socket() -> PathBuf {
    let display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".into());
    crate::paths::runtime_dir().join(format!("shojiwm-shell.{display}.sock"))
}

/// Sends a command to the running shell and prints what it says.
pub fn send(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let said = ask(args)?;
    if !said.is_empty() {
        println!("{said}");
    }
    Ok(())
}

/// Sends a command to the running shell and gives what it says. It waits as long as the shell
/// takes: a deferred call answers when the user has acted.
pub fn ask(args: &[String]) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(socket())?;
    // One argument per line: an output's or a file's name can hold a space.
    for arg in args {
        writeln!(stream, "{}", arg.replace('\n', " "))?;
    }
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut answer = String::new();
    BufReader::new(stream).read_line(&mut answer)?;
    let answer = answer.trim_end();
    match answer.strip_prefix("ok") {
        Some(said) => Ok(said.trim_start().to_owned()),
        None => Err(answer.trim_start_matches("error ").to_owned().into()),
    }
}

/// What the socket thread hears from the UI thread about a call.
enum Heard {
    /// The line to write back.
    Answer(String),
    /// The target answers later: wait with no time limit.
    Later,
}

struct Call {
    id: u64,
    args: Vec<String>,
    reply: Sender<Heard>,
}

/// What the socket threads send to the UI thread.
enum Message {
    Call(Call),
    /// The caller of a deferred call has gone.
    Gone(u64),
}

fn receive(message: Message) {
    match message {
        Message::Call(call) => answer(call),
        Message::Gone(id) => {
            // Taken out before it runs: it drops the reply, which removes its own entry.
            let gone = GONE.with(|g| g.borrow_mut().remove(&id));
            if let Some(gone) = gone {
                gone();
            }
        }
    }
}

fn answer(call: Call) {
    let (target, function, rest) = match call.args.as_slice() {
        [target, function, rest @ ..] => (target.clone(), function.clone(), rest.to_vec()),
        _ => {
            let _ = call.reply.send(Heard::Answer(
                "error usage: msg <target> <function> [args]".into(),
            ));
            return;
        }
    };
    let handler = HANDLERS.with(|h| h.borrow().get(&target).cloned());
    match handler {
        None => {
            let _ = call
                .reply
                .send(Heard::Answer(format!("error no target {target}")));
        }
        Some(Target::Now(f)) => {
            let text = match f(&function, &rest) {
                Ok(said) => format!("ok {said}"),
                Err(e) => format!("error {e}"),
            };
            let _ = call.reply.send(Heard::Answer(text));
        }
        Some(Target::Later(f)) => {
            // First, so that the socket thread lifts its time limit before any answer.
            let _ = call.reply.send(Heard::Later);
            let reply = Reply {
                id: call.id,
                sender: Some(call.reply),
            };
            f(&function, &rest, reply);
        }
    }
}

/// Whether the other end of `stream` has closed. The caller shuts its writing side once its
/// arguments are sent, which reads as an end of file here; only its full close (its process
/// ended) raises `POLLHUP` on a Unix socket.
fn peer_gone(stream: &UnixStream) -> bool {
    let mut fd = libc::pollfd {
        fd: stream.as_raw_fd(),
        events: 0,
        revents: 0,
    };
    // SAFETY: one valid `pollfd`, which `poll` writes to, and no wait.
    let ready = unsafe { libc::poll(&mut fd, 1, 0) };
    ready > 0 && fd.revents & (libc::POLLHUP | libc::POLLERR) != 0
}

/// Waits for the answer to the call `id`. An ordinary call has [`ANSWER_WITHIN`]; a deferred one
/// waits until its reply is sent or dropped, or until its caller goes away.
fn wait(
    stream: &UnixStream,
    heard: &Receiver<Heard>,
    id: u64,
    calls: &UiSender<Message>,
) -> String {
    match heard.recv_timeout(ANSWER_WITHIN) {
        Ok(Heard::Answer(text)) => return text,
        Ok(Heard::Later) => {}
        Err(_) => return "error no answer".into(),
    }
    loop {
        match heard.recv_timeout(PEER_CHECK) {
            Ok(Heard::Answer(text)) => return text,
            Ok(Heard::Later) => {}
            Err(RecvTimeoutError::Timeout) => {
                if peer_gone(stream) {
                    calls.send(Message::Gone(id));
                    return CANCELLED.into();
                }
            }
            Err(RecvTimeoutError::Disconnected) => return CANCELLED.into(),
        }
    }
}

/// Listens for commands. Call once, from the root.
pub fn serve() {
    let path = socket();
    // Someone else listening means another shell is running; a file with no one behind it is
    // left over from one that died.
    if UnixStream::connect(&path).is_ok() {
        eprintln!(
            "shojiwm-shell: another one is running at {}",
            path.display()
        );
        return;
    }
    let _ = std::fs::remove_file(&path);
    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "shojiwm-shell: no command socket at {}: {e}",
                path.display()
            );
            return;
        }
    };
    let calls = ui_channel(receive);
    std::thread::Builder::new()
        .name("shojiwm-shell ipc".into())
        .spawn(move || {
            // Each call's number, for the UI thread to tell a gone caller's dialog.
            for (id, stream) in (1u64..).zip(listener.incoming().flatten()) {
                let calls = calls.clone();
                std::thread::spawn(move || {
                    let reader = match stream.try_clone() {
                        Ok(r) => BufReader::new(r),
                        Err(_) => return,
                    };
                    let args: Vec<String> = reader.lines().map_while(Result::ok).collect();
                    let (reply, heard) = channel();
                    calls.send(Message::Call(Call { id, args, reply }));
                    let text = wait(&stream, &heard, id, &calls);
                    let mut stream = stream;
                    let _ = writeln!(stream, "{text}");
                });
            }
        })
        .expect("a thread for the command socket");
}
