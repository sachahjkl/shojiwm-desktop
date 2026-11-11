//! Where the shell's files are. The pictures and scripts are the QML shell's (`shell/assets`,
//! `shell/gtk-switch.sh`), shared while the shell moves over: `$SHOJIWM_SHELL_DIR` when set,
//! else the installed configuration's `~/.config/shojiwm/shell`.

use std::path::PathBuf;

/// The directory of the shell's files.
pub fn shell_dir() -> PathBuf {
    std::env::var_os("SHOJIWM_SHELL_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| crate::settings::config_home().join("shojiwm").join("shell"))
}

/// A file of the shell, if it is there.
pub fn shell_file(name: &str) -> Option<PathBuf> {
    let path = shell_dir().join(name);
    path.exists().then_some(path)
}

/// A picture of `shell/assets`, as a path (whether it is there or not: a missing picture shows
/// nothing, and the path says what was asked for).
pub fn asset(name: &str) -> PathBuf {
    shell_dir().join("assets").join(name)
}

/// `$XDG_RUNTIME_DIR`, else the temporary directory.
pub fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

/// `$XDG_STATE_HOME/shojiwm-win7`: the notes, the notification history, the run history.
pub fn state_dir() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| crate::settings::home().map(|h| h.join(".local/state")))
        .unwrap_or_else(std::env::temp_dir)
        .join("shojiwm-win7")
}
