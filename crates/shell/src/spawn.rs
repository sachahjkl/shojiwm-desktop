//! The one way the shell starts a program the user works in: an application, a terminal, the
//! file manager, a file in its program, a line of the Run dialog.
//!
//! Each program runs in a systemd scope of its own, in uwsm's slice for graphical applications,
//! at a lower priority than the desktop. The shell's own helpers (dconf, busctl, gio monitor,
//! systemctl, ...) do not come here: `crate::programs::run_helper` starts those.
//!
//! A fork is a system call: no program is started on the UI thread. Each start goes to one
//! worker thread, the spawner ([`on_spawner`]), which runs the starts in the order they came.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A start for the spawner to do. It gives the child it started, for the spawner to reap.
type Job = Box<dyn FnOnce() -> Option<Child> + Send>;

/// Runs `job` on the spawner thread, after the jobs sent before it. Nothing waits for it. Use it
/// for each `Command::spawn` and each look at the disk that comes before one. The job gives the
/// child it started: the spawner collects its exit, so that no zombie stays.
pub fn on_spawner(job: impl FnOnce() -> Option<Child> + Send + 'static) {
    static SPAWNER: OnceLock<Sender<Job>> = OnceLock::new();
    let sender = SPAWNER.get_or_init(|| {
        let (sender, jobs) = channel::<Job>();
        std::thread::Builder::new()
            .name("spawner".into())
            .spawn(move || spawner(jobs))
            .expect("the spawner thread starts");
        sender
    });
    let _ = sender.send(Box::new(job));
}

/// How often the spawner looks for children that ended, while some still run.
const REAP: Duration = Duration::from_secs(2);

fn spawner(jobs: Receiver<Job>) {
    let mut children: Vec<Child> = Vec::new();
    loop {
        let job = if children.is_empty() {
            jobs.recv().map_err(|_| RecvTimeoutError::Disconnected)
        } else {
            jobs.recv_timeout(REAP)
        };
        match job {
            Ok(job) => children.extend(job()),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        children.retain_mut(|child| matches!(child.try_wait(), Ok(None)));
    }
}

/// Starts `program` on the spawner, with no input and its output dropped. Nothing waits for it.
pub fn spawn_detached(program: &str, args: Vec<String>) {
    let program = program.to_owned();
    on_spawner(move || {
        Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok()
    });
}

/// The nice value of a program started from the desktop. The compositor and the shell stay at
/// nice 0, so they get the CPU first: a build in a terminal no longer makes the taskbar, the
/// menus and the windows stutter.
const APP_NICE: &str = "5";

/// uwsm's slice for graphical applications. Each program gets its own scope in it, so a build in
/// a terminal is accounted to that terminal, not to the compositor's unit.
const APP_SLICE: &str = "app-graphical.slice";

/// The longest program name in a unit name; systemd allows 255 characters for the whole name.
const NAME_LIMIT: usize = 64;

/// Starts `argv` in a scope of its own, apart from the shell: in a new session, with the shell's
/// environment, in `cwd` (else the shell's folder). Nothing waits for it: the spawner starts it.
pub fn start_program(argv: &[String], cwd: Option<&Path>) {
    let Some(first) = argv.first() else {
        return;
    };
    let mut line = vec!["-f".to_owned()];
    line.extend(scope_prefix(&program_name(argv, first)));
    line.extend(argv.iter().cloned());
    let cwd: Option<PathBuf> = cwd.map(Path::to_path_buf);
    on_spawner(move || {
        let mut command = Command::new("setsid");
        command
            .args(line)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(dir) = cwd {
            command.current_dir(dir);
        }
        command.spawn().ok()
    });
}

/// What runs a program named `name` in a scope of its own, at the programs' priority: the
/// command line goes after it. An application of the menu goes through it too
/// (`Application::launch_in`), which keeps its activation token.
pub fn scope_prefix(name: &str) -> Vec<String> {
    [
        "nice".to_owned(),
        "-n".to_owned(),
        APP_NICE.to_owned(),
        "systemd-run".to_owned(),
        "--user".to_owned(),
        "--scope".to_owned(),
        format!("--slice={APP_SLICE}"),
        "--collect".to_owned(),
        "--quiet".to_owned(),
        format!("--unit=app-{}-{}", unit_word(name), suffix()),
        "--".to_owned(),
    ]
    .into()
}

/// Starts a command line, run by `sh`, as [`start_program`] does. Nothing for an empty line.
pub fn start_command_line(line: &str, cwd: Option<&Path>) {
    if line.trim().is_empty() {
        return;
    }
    start_program(&["sh".to_owned(), "-c".to_owned(), line.to_owned()], cwd);
}

/// The program's name for its unit: the file name of the first word, or of the command line's
/// first word for `sh -c`, after any `VAR=value` and `exec`.
fn program_name(argv: &[String], first: &str) -> String {
    let word = match argv {
        [sh, flag, line, ..] if sh == "sh" && flag == "-c" => line
            .split_whitespace()
            .find(|w| !w.contains('=') && *w != "exec")
            .unwrap_or("sh"),
        _ => first,
    };
    let word = word.trim_matches(|c| c == '\'' || c == '"');
    Path::new(word)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "program".to_owned())
}

/// A word for a unit name: systemd allows `[a-zA-Z0-9:_.-]`; any other byte becomes `\xNN`, as
/// `systemd-escape` writes it. A leading `.` is escaped too.
fn unit_word(name: &str) -> String {
    let mut out = String::new();
    for (i, byte) in name.bytes().enumerate() {
        let allowed = byte.is_ascii_alphanumeric()
            || matches!(byte, b':' | b'_' | b'-')
            || (byte == b'.' && i > 0);
        let piece = if allowed {
            (byte as char).to_string()
        } else {
            format!("\\x{byte:02x}")
        };
        if out.len() + piece.len() > NAME_LIMIT {
            break;
        }
        out.push_str(&piece);
    }
    out
}

/// What makes each unit name unique: the shell's process id, the time and a count.
fn suffix() -> String {
    static COUNT: AtomicU32 = AtomicU32::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let count = COUNT.fetch_add(1, Ordering::Relaxed);
    format!("{}-{nanos}-{count}", std::process::id())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(list: &[&str]) -> Vec<String> {
        list.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn names_the_program_of_a_command_line() {
        let argv = words(&["sh", "-c", "GDK_BACKEND=wayland exec /usr/bin/kitty -1"]);
        assert_eq!(program_name(&argv, "sh"), "kitty");
        let argv = words(&["xdg-open", "/tmp/a b"]);
        assert_eq!(program_name(&argv, "xdg-open"), "xdg-open");
    }

    #[test]
    fn escapes_what_a_unit_name_does_not_allow() {
        assert_eq!(unit_word("org.kde.dolphin"), "org.kde.dolphin");
        assert_eq!(unit_word(".hidden"), "\\x2ehidden");
        assert_eq!(unit_word("my app"), "my\\x20app");
    }
}
