//! The link to the compositor: the shell socket of the Windows 7
//! configuration, which speaks ShojiWM's newline-delimited JSON.
//!
//! The service sends the triggers to bind, again after each reconnection:
//!
//!   {"id":1,"method":"shortcuts","params":{"triggers":["Alt+space"]}}
//!
//! and reads the presses and releases back:
//!
//!   {"event":"shortcut","payload":{"trigger":"Alt+space","state":"pressed"}}

use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::net::unix::OwnedWriteHalf;
use tokio::sync::{mpsc, watch};

/// A shortcut pressed or released.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub trigger: String,
    pub pressed: bool,
}

/// The socket of the configuration: `SHOJIWM_DESKTOP_SOCKET`, else the path
/// the configuration makes from `XDG_RUNTIME_DIR` and `WAYLAND_DISPLAY`.
pub fn socket_path() -> PathBuf {
    if let Some(path) = std::env::var_os("SHOJIWM_DESKTOP_SOCKET") {
        return path.into();
    }
    let runtime = std::env::var_os("XDG_RUNTIME_DIR").unwrap_or_else(|| "/tmp".into());
    let display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".into());
    PathBuf::from(runtime).join(format!("shojiwm-{display}-desktop.sock"))
}

/// Keep the compositor bound to the current triggers, and pass its events
/// on. The compositor can restart or reload its configuration: the link
/// then connects again and sends the triggers again.
pub async fn run(
    path: PathBuf,
    mut triggers: watch::Receiver<Vec<String>>,
    events: mpsc::UnboundedSender<Event>,
) {
    let mut connected_before = false;
    loop {
        match UnixStream::connect(&path).await {
            Ok(stream) => {
                tracing::info!(path = %path.display(), "connected to the compositor");
                connected_before = true;
                match serve(stream, &mut triggers, &events).await {
                    Ok(()) => return,
                    Err(error) => tracing::info!(%error, "lost the compositor"),
                }
            }
            Err(error) if !connected_before => {
                tracing::debug!(%error, path = %path.display(), "no compositor yet");
            }
            Err(_) => {}
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// One connection, until it fails. `Ok` only when the service stops.
async fn serve(
    stream: UnixStream,
    triggers: &mut watch::Receiver<Vec<String>>,
    events: &mpsc::UnboundedSender<Event>,
) -> std::io::Result<()> {
    let (read, mut write) = stream.into_split();
    let mut lines = BufReader::new(read).lines();
    let mut id = 0u64;

    let current = triggers.borrow_and_update().clone();
    send(&mut write, &mut id, &current).await?;

    loop {
        tokio::select! {
            changed = triggers.changed() => {
                if changed.is_err() {
                    return Ok(());
                }
                let current = triggers.borrow_and_update().clone();
                send(&mut write, &mut id, &current).await?;
            }
            line = lines.next_line() => {
                let Some(line) = line? else {
                    return Err(std::io::ErrorKind::UnexpectedEof.into());
                };
                if let Some(event) = parse_event(&line) {
                    let _ = events.send(event);
                }
            }
        }
    }
}

async fn send(
    write: &mut OwnedWriteHalf,
    id: &mut u64,
    triggers: &[String],
) -> std::io::Result<()> {
    *id += 1;
    let message = serde_json::json!({
        "id": *id,
        "method": "shortcuts",
        "params": { "triggers": triggers },
    });
    tracing::info!(?triggers, "binding");
    let mut line = message.to_string();
    line.push('\n');
    write.write_all(line.as_bytes()).await
}

#[derive(Deserialize)]
struct Message {
    event: Option<String>,
    payload: Option<Payload>,
}

#[derive(Deserialize)]
struct Payload {
    trigger: String,
    state: String,
}

/// A `shortcut` event; answers and other events give `None`.
fn parse_event(line: &str) -> Option<Event> {
    let message: Message = serde_json::from_str(line).ok()?;
    if message.event.as_deref() != Some("shortcut") {
        return None;
    }
    let payload = message.payload?;
    let pressed = match payload.state.as_str() {
        "pressed" => true,
        "released" => false,
        _ => return None,
    };
    Some(Event {
        trigger: payload.trigger,
        pressed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events() {
        assert_eq!(
            parse_event(
                r#"{"event":"shortcut","payload":{"trigger":"Alt+space","state":"pressed"}}"#
            ),
            Some(Event {
                trigger: "Alt+space".into(),
                pressed: true
            })
        );
        assert_eq!(
            parse_event(
                r#"{"event":"shortcut","payload":{"trigger":"Alt+space","state":"released"}}"#
            )
            .map(|event| event.pressed),
            Some(false)
        );
        assert_eq!(parse_event(r#"{"id":1,"result":["Alt+space"]}"#), None);
        assert_eq!(parse_event(r#"{"event":"workspace","payload":{}}"#), None);
    }
}
