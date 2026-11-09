//! `org.freedesktop.impl.portal.GlobalShortcuts`, version 1.
//!
//! A program opens a session, binds its shortcuts with a preferred trigger
//! each, and gets `Activated` and `Deactivated` when the keys go down and
//! up. A shortcut takes its preferred trigger as it is: there is no dialog
//! to choose another. The shortcuts last as long as their session.

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use tokio::sync::watch;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{ObjectPath, OwnedValue, Value};

use crate::trigger;

/// One bound shortcut of a session.
#[derive(Debug, Clone)]
struct Shortcut {
    id: String,
    description: String,
    /// In ShojiWM's form; `None` when the program gave no usable trigger.
    trigger: Option<String>,
}

/// The sessions, shared by the D-Bus objects and the compositor link.
pub struct Shared {
    sessions: Mutex<HashMap<String, Vec<Shortcut>>>,
    /// Every trigger bound by some session, for the compositor link.
    triggers: watch::Sender<Vec<String>>,
}

impl Shared {
    pub fn new() -> (Arc<Self>, watch::Receiver<Vec<String>>) {
        let (triggers, receiver) = watch::channel(Vec::new());
        let shared = Arc::new(Self {
            sessions: Mutex::new(HashMap::new()),
            triggers,
        });
        (shared, receiver)
    }

    /// The sessions and shortcut ids a trigger belongs to.
    pub fn holders(&self, trigger: &str) -> Vec<(String, String)> {
        let sessions = self.sessions.lock().unwrap();
        let mut holders = Vec::new();
        for (session, shortcuts) in sessions.iter() {
            for shortcut in shortcuts {
                if shortcut.trigger.as_deref() == Some(trigger) {
                    holders.push((session.clone(), shortcut.id.clone()));
                }
            }
        }
        holders
    }

    fn publish(&self, sessions: &HashMap<String, Vec<Shortcut>>) {
        let all: BTreeSet<String> = sessions
            .values()
            .flatten()
            .filter_map(|shortcut| shortcut.trigger.clone())
            .collect();
        self.triggers.send_replace(all.into_iter().collect());
    }

    fn close(&self, session: &str) {
        let mut sessions = self.sessions.lock().unwrap();
        if sessions.remove(session).is_some() {
            self.publish(&sessions);
        }
    }
}

/// The `shortcuts` result: `a(sa{sv})`.
fn describe(shortcuts: &[Shortcut]) -> OwnedValue {
    let list: Vec<(String, HashMap<String, Value<'static>>)> = shortcuts
        .iter()
        .map(|shortcut| {
            let mut details = HashMap::new();
            details.insert(
                "description".to_string(),
                Value::from(shortcut.description.clone()),
            );
            details.insert(
                "trigger_description".to_string(),
                Value::from(
                    shortcut
                        .trigger
                        .as_deref()
                        .map(trigger::describe)
                        .unwrap_or_default(),
                ),
            );
            (shortcut.id.clone(), details)
        })
        .collect();
    OwnedValue::try_from(Value::from(list)).expect("a list of strings has no file descriptor")
}

fn text(options: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
    match options.get(key).map(|value| &**value) {
        Some(Value::Str(text)) => Some(text.to_string()),
        _ => None,
    }
}

pub struct GlobalShortcuts {
    pub shared: Arc<Shared>,
}

#[zbus::interface(name = "org.freedesktop.impl.portal.GlobalShortcuts")]
impl GlobalShortcuts {
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        1
    }

    async fn create_session(
        &self,
        _handle: ObjectPath<'_>,
        session_handle: ObjectPath<'_>,
        app_id: String,
        _options: HashMap<String, OwnedValue>,
        #[zbus(object_server)] server: &zbus::ObjectServer,
    ) -> zbus::fdo::Result<(u32, HashMap<String, OwnedValue>)> {
        tracing::info!(%session_handle, %app_id, "CreateSession");
        {
            let mut sessions = self.shared.sessions.lock().unwrap();
            if sessions.contains_key(session_handle.as_str()) {
                return Err(zbus::fdo::Error::InvalidArgs(
                    "session already exists".into(),
                ));
            }
            sessions.insert(session_handle.to_string(), Vec::new());
        }
        let session = Session {
            path: session_handle.to_string(),
            shared: self.shared.clone(),
        };
        if let Err(error) = server.at(&session_handle, session).await {
            self.shared.close(session_handle.as_str());
            return Err(zbus::fdo::Error::Failed(format!(
                "export the session: {error}"
            )));
        }
        Ok((0, HashMap::new()))
    }

    async fn bind_shortcuts(
        &self,
        _handle: ObjectPath<'_>,
        session_handle: ObjectPath<'_>,
        shortcuts: Vec<(String, HashMap<String, OwnedValue>)>,
        _parent_window: String,
        _options: HashMap<String, OwnedValue>,
    ) -> zbus::fdo::Result<(u32, HashMap<String, OwnedValue>)> {
        let bound: Vec<Shortcut> = shortcuts
            .iter()
            .map(|(id, options)| Shortcut {
                id: id.clone(),
                description: text(options, "description").unwrap_or_default(),
                trigger: text(options, "preferred_trigger")
                    .and_then(|wanted| trigger::parse(&wanted)),
            })
            .collect();
        for shortcut in &bound {
            tracing::info!(%session_handle, id = %shortcut.id, trigger = ?shortcut.trigger, "BindShortcuts");
        }
        {
            let mut sessions = self.shared.sessions.lock().unwrap();
            let Some(entry) = sessions.get_mut(session_handle.as_str()) else {
                return Err(zbus::fdo::Error::InvalidArgs("no such session".into()));
            };
            *entry = bound.clone();
            self.shared.publish(&sessions);
        }
        let mut results = HashMap::new();
        results.insert("shortcuts".to_string(), describe(&bound));
        Ok((0, results))
    }

    async fn list_shortcuts(
        &self,
        _handle: ObjectPath<'_>,
        session_handle: ObjectPath<'_>,
    ) -> zbus::fdo::Result<(u32, HashMap<String, OwnedValue>)> {
        let sessions = self.shared.sessions.lock().unwrap();
        let Some(shortcuts) = sessions.get(session_handle.as_str()) else {
            return Err(zbus::fdo::Error::InvalidArgs("no such session".into()));
        };
        let mut results = HashMap::new();
        results.insert("shortcuts".to_string(), describe(shortcuts));
        Ok((0, results))
    }

    #[zbus(signal)]
    pub async fn activated(
        emitter: &SignalEmitter<'_>,
        session_handle: ObjectPath<'_>,
        shortcut_id: &str,
        timestamp: u64,
        options: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn deactivated(
        emitter: &SignalEmitter<'_>,
        session_handle: ObjectPath<'_>,
        shortcut_id: &str,
        timestamp: u64,
        options: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<()>;
}

/// `org.freedesktop.impl.portal.Session` at a session's path: the portal
/// frontend closes it when the program closes the session or goes away.
struct Session {
    path: String,
    shared: Arc<Shared>,
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Session")]
impl Session {
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        1
    }

    async fn close(
        &self,
        #[zbus(object_server)] server: &zbus::ObjectServer,
    ) -> zbus::fdo::Result<()> {
        tracing::info!(session = %self.path, "Close");
        self.shared.close(&self.path);
        if let Ok(path) = ObjectPath::try_from(self.path.clone()) {
            let _ = server.remove::<Self, _>(&path).await;
        }
        Ok(())
    }
}
