//! xdg-desktop-portal-shojiwm-win7: the GlobalShortcuts portal for the
//! Windows 7 configuration of ShojiWM.
//!
//! Programs bind shortcuts through xdg-desktop-portal, which calls this
//! service. The service asks the configuration to bind the keys, over its
//! shell socket, and turns the presses and releases it reports into the
//! portal's `Activated` and `Deactivated` signals.

mod compositor;
mod shortcuts;
mod trigger;

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::ObjectPath;

use shortcuts::{GlobalShortcuts, Shared};

const BUS_NAME: &str = "org.freedesktop.impl.portal.desktop.shojiwm-win7";
const OBJECT_PATH: &str = "/org/freedesktop/portal/desktop";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let (shared, triggers) = Shared::new();
    let connection = zbus::connection::Builder::session()?
        .name(BUS_NAME)?
        .serve_at(
            OBJECT_PATH,
            GlobalShortcuts {
                shared: shared.clone(),
            },
        )?
        .build()
        .await?;
    tracing::info!(bus = BUS_NAME, "serving GlobalShortcuts");

    let (events, mut received) = mpsc::unbounded_channel();
    let path = compositor::socket_path();
    tokio::spawn(compositor::run(path, triggers, events));

    let emitter = SignalEmitter::new(&connection, OBJECT_PATH)?;
    while let Some(event) = received.recv().await {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|time| time.as_millis() as u64)
            .unwrap_or_default();
        for (session, id) in shared.holders(&event.trigger) {
            let Ok(session) = ObjectPath::try_from(session.as_str()) else {
                continue;
            };
            tracing::info!(%session, %id, pressed = event.pressed, "shortcut");
            let sent = if event.pressed {
                GlobalShortcuts::activated(&emitter, session, &id, timestamp, HashMap::new()).await
            } else {
                GlobalShortcuts::deactivated(&emitter, session, &id, timestamp, HashMap::new())
                    .await
            };
            if let Err(error) = sent {
                tracing::warn!(%error, "could not send the signal");
            }
        }
    }
    Ok(())
}
