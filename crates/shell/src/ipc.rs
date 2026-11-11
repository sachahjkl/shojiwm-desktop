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

use anti_webui::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{Sender, channel};
use std::time::Duration;

/// A target's functions: the function's name and its arguments, to an answer or an error.
pub type Handler = Rc<dyn Fn(&str, &[String]) -> Result<String, String>>;

thread_local! {
    static HANDLERS: RefCell<HashMap<String, Handler>> = RefCell::new(HashMap::new());
}

/// Answers the calls to `target`. A second call for the same target replaces the first.
pub fn handle(target: &str, f: impl Fn(&str, &[String]) -> Result<String, String> + 'static) {
    HANDLERS.with(|h| h.borrow_mut().insert(target.to_owned(), Rc::new(f)));
}

fn socket() -> PathBuf {
    let display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".into());
    crate::paths::runtime_dir().join(format!("shojiwm-shell.{display}.sock"))
}

/// Sends a command to the running shell and prints what it says.
pub fn send(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
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
        Some(said) => {
            let said = said.trim_start();
            if !said.is_empty() {
                println!("{said}");
            }
            Ok(())
        }
        None => Err(answer.trim_start_matches("error ").to_owned().into()),
    }
}

struct Call {
    args: Vec<String>,
    reply: Sender<String>,
}

fn answer(call: Call) {
    let (target, function, rest) = match call.args.as_slice() {
        [target, function, rest @ ..] => (target.clone(), function.clone(), rest.to_vec()),
        _ => {
            let _ = call
                .reply
                .send("error usage: msg <target> <function> [args]".into());
            return;
        }
    };
    let handler = HANDLERS.with(|h| h.borrow().get(&target).cloned());
    let text = match handler {
        None => format!("error no target {target}"),
        Some(f) => match f(&function, &rest) {
            Ok(said) => format!("ok {said}"),
            Err(e) => format!("error {e}"),
        },
    };
    let _ = call.reply.send(text);
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
    let calls = ui_channel(answer);
    std::thread::Builder::new()
        .name("shojiwm-shell ipc".into())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                let calls = calls.clone();
                std::thread::spawn(move || {
                    let reader = match stream.try_clone() {
                        Ok(r) => BufReader::new(r),
                        Err(_) => return,
                    };
                    let args: Vec<String> = reader.lines().map_while(Result::ok).collect();
                    let (reply, answered) = channel();
                    calls.send(Call { args, reply });
                    let text = answered
                        .recv_timeout(Duration::from_secs(5))
                        .unwrap_or_else(|_| "error no answer".into());
                    let mut stream = stream;
                    let _ = writeln!(stream, "{text}");
                });
            }
        })
        .expect("a thread for the command socket");
}
