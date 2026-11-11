//! greetd's side of the log-on screen: its IPC protocol over the socket in `$GREETD_SOCK`. Each
//! message is a JSON object after its length in 4 bytes (native byte order). The greeter sends
//! a request and reads one response to it, in order:
//!
//! - `create_session { username }`, then `post_auth_message_response { response }` to each
//!   `auth_message` until `success` (or an `error`);
//! - `start_session { cmd, env }`: greetd starts the session once the greeter has left;
//! - `cancel_session`, after a failure.
//!
//! The socket is read and written on a thread of its own, since each response can take
//! seconds (PAM waits after a wrong password).

use anti_webui::prelude::*;
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::mpsc::{Sender, channel};

/// What the greeter asks greetd.
pub enum Request {
    CreateSession { username: String },
    /// The answer to an `auth_message`: what was typed for a question, nothing for
    /// information.
    Respond(Option<String>),
    StartSession { cmd: Vec<String>, env: Vec<String> },
    CancelSession,
}

/// What kind of message PAM sends through greetd.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthKind {
    /// A question whose answer shows as typed.
    Visible,
    /// A question whose answer is hidden: a password.
    Secret,
    Info,
    Error,
}

/// What greetd answers.
pub enum Response {
    Success,
    AuthMessage { kind: AuthKind, text: String },
    /// `auth`: the password or the answer was wrong; otherwise something else went wrong.
    Error { auth: bool, description: String },
}

impl Request {
    fn to_json(&self) -> Value {
        match self {
            Request::CreateSession { username } => {
                json!({ "type": "create_session", "username": username })
            }
            Request::Respond(response) => {
                json!({ "type": "post_auth_message_response", "response": response })
            }
            Request::StartSession { cmd, env } => {
                json!({ "type": "start_session", "cmd": cmd, "env": env })
            }
            Request::CancelSession => json!({ "type": "cancel_session" }),
        }
    }
}

fn parse(value: &Value) -> Result<Response, String> {
    let field = |key: &str| value.get(key).and_then(Value::as_str).unwrap_or_default();
    match field("type") {
        "success" => Ok(Response::Success),
        "error" => Ok(Response::Error {
            auth: field("error_type") == "auth_error",
            description: field("description").to_owned(),
        }),
        "auth_message" => {
            let kind = match field("auth_message_type") {
                "visible" => AuthKind::Visible,
                "secret" => AuthKind::Secret,
                "info" => AuthKind::Info,
                _ => AuthKind::Error,
            };
            Ok(Response::AuthMessage {
                kind,
                text: field("auth_message").to_owned(),
            })
        }
        other => Err(format!("greetd said something unknown: {other:?}")),
    }
}

/// Sends a request and reads its response.
fn exchange(stream: &mut UnixStream, request: &Request) -> Result<Response, String> {
    let mut body = serde_json::to_vec(&request.to_json()).map_err(|e| e.to_string())?;
    let length = u32::try_from(body.len()).map_err(|_| "a request too long".to_owned())?;
    let written = stream
        .write_all(&length.to_ne_bytes())
        .and_then(|()| stream.write_all(&body));
    // A response holds a password: it does not stay in memory.
    body.iter_mut().for_each(|b| *b = 0);
    written.map_err(|e| format!("cannot write to greetd: {e}"))?;
    let mut length = [0u8; 4];
    stream
        .read_exact(&mut length)
        .map_err(|e| format!("cannot read from greetd: {e}"))?;
    let mut answer = vec![0u8; u32::from_ne_bytes(length) as usize];
    stream
        .read_exact(&mut answer)
        .map_err(|e| format!("cannot read from greetd: {e}"))?;
    let value: Value = serde_json::from_slice(&answer).map_err(|e| e.to_string())?;
    parse(&value)
}

/// The line to greetd. Requests go in order; each response comes to the function given to
/// [`Greetd::connect`], on the UI thread that connected.
pub struct Greetd {
    requests: Sender<Request>,
}

impl Greetd {
    /// Connects to `$GREETD_SOCK`. Call it from a root: the responses come there.
    pub fn connect(on_response: impl Fn(Response) + 'static) -> Result<Greetd, String> {
        let path = std::env::var_os("GREETD_SOCK").ok_or("no $GREETD_SOCK: not run by greetd")?;
        let mut stream =
            UnixStream::connect(&path).map_err(|e| format!("cannot reach greetd: {e}"))?;
        let to_ui = ui_channel(on_response);
        let (requests, received) = channel::<Request>();
        std::thread::Builder::new()
            .name("shojiwm-shell greetd".into())
            .spawn(move || {
                for request in received {
                    let response = exchange(&mut stream, &request).unwrap_or_else(|e| {
                        Response::Error {
                            auth: false,
                            description: e,
                        }
                    });
                    to_ui.send(response);
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Greetd { requests })
    }

    pub fn send(&self, request: Request) {
        let _ = self.requests.send(request);
    }
}
