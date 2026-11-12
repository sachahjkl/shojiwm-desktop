//! The shell's line to the compositor, for what the shell alone cannot do: Aero Peek, the
//! show-desktop toggle that brings the windows back, the screens' modes, and the settings the
//! window frames are drawn with (theme, corners, glass, animations, programs).
//!
//! `src/peek.tsx` listens and hands the path to the shell in `$SHOJIWM_DESKTOP_SOCKET` when it starts
//! it. The wire format is ShojiWM's: one JSON object per line, `{ "method", "params" }`, and
//! `{ "id", "method", "params" }` for a request, answered by `{ "id", "result" }`.
//!
//! The line lives on a thread of its own and is tried again each second while it is down: the
//! compositor can be reloaded, which closes its end. Each time it comes up, the settings the
//! frames are drawn with are sent again.

use crate::settings::{animations, settings};
use anti_webui::prelude::*;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

/// Where the answer to a request goes.
type Answer = Box<dyn FnOnce(Option<Value>) + Send>;

/// What goes to the line's thread.
enum Out {
    /// A message, and for a request its id and where its answer goes.
    Line(String, Option<(u64, Answer)>),
}

struct Line {
    out: Sender<Out>,
    up: SharedSignal<bool>,
    next_id: Mutex<u64>,
    /// What the frames are drawn with, as last sent: sent again when the line comes up.
    latest: Mutex<Vec<(String, Value)>>,
}

static LINE: OnceLock<Arc<Line>> = OnceLock::new();

fn path() -> Option<String> {
    std::env::var("SHOJIWM_DESKTOP_SOCKET")
        .ok()
        .filter(|p| !p.is_empty())
}

fn line() -> &'static Arc<Line> {
    LINE.get_or_init(|| {
        let (out, rx) = channel();
        let line = Arc::new(Line {
            out,
            up: shared_signal(false),
            next_id: Mutex::new(0),
            latest: Mutex::new(Vec::new()),
        });
        if let Some(path) = path() {
            let thread_line = line.clone();
            std::thread::Builder::new()
                .name("shojiwm-shell compositor".into())
                .spawn(move || run(path, rx, thread_line))
                .expect("a thread for the compositor's line");
        }
        line
    })
}

type Pending = Arc<Mutex<HashMap<u64, Answer>>>;

fn run(path: String, rx: Receiver<Out>, line: Arc<Line>) {
    let pending: Pending = Arc::default();
    loop {
        let Ok(stream) = UnixStream::connect(&path) else {
            std::thread::sleep(Duration::from_secs(1));
            // Messages sent while the line is down are lost, as the QML shell's were; the
            // settings are sent again when it comes up.
            while let Ok(Out::Line(_, request)) = rx.try_recv() {
                if let Some((_, answer)) = request {
                    answer(None);
                }
            }
            continue;
        };
        line.up.set(true);
        let reader = stream.try_clone().ok();
        let answers = pending.clone();
        let reading = reader.map(|r| {
            std::thread::spawn(move || {
                for text in BufReader::new(r).lines() {
                    let Ok(text) = text else { break };
                    let Ok(v) = serde_json::from_str::<Value>(&text) else {
                        continue;
                    };
                    // Events the compositor sends unasked: the Alt+Tab panel's titles.
                    if v.get("event").and_then(Value::as_str) == Some("alt-tab") {
                        crate::dialogs::alt_tab_event(v.get("payload"));
                        continue;
                    }
                    let Some(id) = v.get("id").and_then(Value::as_u64) else {
                        continue;
                    };
                    if let Some(answer) = answers.lock().unwrap().remove(&id) {
                        answer(v.get("result").cloned());
                    }
                }
            })
        });
        let mut writer = stream;
        let resend: Vec<String> = line
            .latest
            .lock()
            .unwrap()
            .iter()
            .map(|(m, p)| json!({ "method": m, "params": p }).to_string())
            .collect();
        let mut ok = resend.iter().all(|m| writeln!(writer, "{m}").is_ok());
        while ok {
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(Out::Line(text, request)) => {
                    if let Some((id, answer)) = request {
                        pending.lock().unwrap().insert(id, answer);
                    }
                    ok = writeln!(writer, "{text}")
                        .and_then(|_| writer.flush())
                        .is_ok();
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    ok = reading.as_ref().is_some_and(|r| !r.is_finished());
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
        line.up.set(false);
        for (_, answer) in pending.lock().unwrap().drain() {
            answer(None);
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

/// Whether the line is up. Reading it tracks it. `false` when the shell was started some other
/// way: the callers then fall back to what the shell can do on its own.
pub fn available() -> bool {
    line().up.get()
}

/// Sends a message; lost when the line is down.
pub fn send(method: &str, params: Value) {
    let text = json!({ "method": method, "params": params }).to_string();
    let _ = line().out.send(Out::Line(text, None));
}

/// Asks the compositor; the answer (`None` when it fails or the line is down) comes back on the
/// UI thread that asked.
pub fn request(method: &str, params: Value) -> Task<Option<Value>> {
    let (task, done) = Task::pending();
    let line = line();
    let id = {
        let mut next = line.next_id.lock().unwrap();
        *next += 1;
        *next
    };
    let text = json!({ "id": id, "method": method, "params": params }).to_string();
    let answer: Answer = Box::new(move |v| done.complete(v));
    if let Err(e) = line.out.send(Out::Line(text, Some((id, answer)))) {
        let Out::Line(_, Some((_, answer))) = e.0 else {
            return task;
        };
        answer(None);
    }
    task
}

/// Sends a setting the frames are drawn with, and keeps it to send again when the line comes up.
fn send_kept(method: &str, params: Value) {
    {
        let mut latest = line().latest.lock().unwrap();
        latest.retain(|(m, _)| m != method);
        latest.push((method.to_owned(), params.clone()));
    }
    send(method, params);
}

/// Sends the settings the frames are drawn with whenever they change, for as long as the shell
/// runs. Call once, from the root. Each message is a memo of the settings: a change of another
/// setting sends nothing.
pub fn share_settings() {
    share("animations", || {
        let on = animations();
        let speed = settings().with(|s| s.animation_speed);
        json!({ "on": on, "speed": speed })
    });
    share("glass", || {
        let (level, colour, strength) =
            settings().with(|s| (s.glass_blur, s.glass_color.clone(), s.glass_color_strength));
        json!({ "level": level, "colour": colour, "strength": strength })
    });
    share("corners", || {
        let square = settings().with(|s| s.square_corners);
        json!({ "square": square })
    });
    crate::settings::follow_look();
    share("theme", || {
        let name = settings().with(|s| s.theme.clone());
        json!({ "name": name })
    });
    share("programs", crate::programs::compositor_programs);
    share("display", || {
        let choices = settings().with(|s| s.display_choices.clone());
        json!({ "choices": choices })
    });
}

/// Sends `method` with what `params` makes, at once and each time it changes.
fn share(method: &'static str, params: impl Fn() -> Value + 'static) {
    let params = memo(params);
    effect(move || send_kept(method, params.get()));
}
