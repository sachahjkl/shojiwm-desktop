//! Work off the interface thread: a disk read, a process, a parse. The work runs on a worker and
//! its result comes back on the interface thread.

use anti_webui::prelude::*;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

/// Runs `work` on a worker, then `done` with its result on the interface thread, under the owner
/// that is current now; nothing runs if that owner is gone by then.
pub fn off_thread<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
    done: impl FnOnce(T) + 'static,
) {
    off_thread_in(Owner::current(), work, done);
}

/// [`off_thread`], `done` running under `keep` (none: under no owner).
///
/// The task lives in a root of its own, so that a task started again and again (a timer, a
/// rescan) leaves nothing behind. The root goes after `done`, on the next turn: the task marks
/// itself done once its callback has returned, and a root disposed inside the callback would take
/// that mark with it.
pub fn off_thread_in<T: Send + 'static>(
    keep: Option<Owner>,
    work: impl FnOnce() -> T + Send + 'static,
    done: impl FnOnce(T) + 'static,
) {
    let slot: Rc<Cell<Option<Owner>>> = Rc::default();
    let own = slot.clone();
    let ((), root) = create_root(move || {
        task(work).then(move |value| {
            match keep {
                Some(keep) if keep.is_alive() => keep.with(|| done(value)),
                Some(_) => {}
                None => done(value),
            }
            if let Some(root) = own.take() {
                set_timeout(Duration::ZERO, move || root.dispose());
            }
        });
    });
    slot.set(Some(root));
}

/// Runs `program` with `args` for as long as it lives, and calls `on_line` on the interface thread
/// with each line it prints. It is started and read on a thread of its own: starting a process is
/// a system call. It ends at its next line once the shell is gone (its pipe is closed then).
pub fn watch(program: &str, args: &[&str], on_line: impl Fn(String) + 'static) {
    let to_ui = ui_channel(on_line);
    let program = program.to_owned();
    let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    let name: String = format!("watch {program}").chars().take(15).collect();
    let _ = std::thread::Builder::new().name(name).spawn(move || {
        use std::io::BufRead;
        let mut command = std::process::Command::new(&program);
        command
            .args(&args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null());
        let child = command.spawn();
        let Ok(mut child) = child else {
            return;
        };
        if let Some(out) = child.stdout.take() {
            for line in std::io::BufReader::new(out).lines() {
                let Ok(line) = line else {
                    break;
                };
                to_ui.send(line);
            }
        }
        let _ = child.wait();
    });
}

/// Calls `on_change` on the interface thread after something is made, removed, renamed or written
/// in the folder `dir` (not below it). The folder is watched with inotify on a thread of its own,
/// without a process; changes that come together are told once.
pub fn watch_folder(dir: &std::path::Path, on_change: impl Fn() + 'static) {
    let to_ui = ui_channel(move |()| on_change());
    let dir = dir.to_owned();
    let _ = std::thread::Builder::new()
        .name("watch folder".into())
        .spawn(move || {
            use std::os::unix::ffi::OsStrExt;
            let Ok(path) = std::ffi::CString::new(dir.as_os_str().as_bytes()) else {
                return;
            };
            // SAFETY: plain calls on a descriptor this thread owns; `buf` outlives each read.
            unsafe {
                let fd = libc::inotify_init1(libc::IN_CLOEXEC);
                if fd < 0 {
                    return;
                }
                let mask = libc::IN_CREATE
                    | libc::IN_DELETE
                    | libc::IN_MOVED_FROM
                    | libc::IN_MOVED_TO
                    | libc::IN_CLOSE_WRITE
                    | libc::IN_ATTRIB
                    | libc::IN_DELETE_SELF
                    | libc::IN_MOVE_SELF;
                if libc::inotify_add_watch(fd, path.as_ptr(), mask) < 0 {
                    libc::close(fd);
                    return;
                }
                let mut buf = [0u8; 4096];
                loop {
                    let n = libc::read(fd, buf.as_mut_ptr().cast(), buf.len());
                    if n <= 0 {
                        break;
                    }
                    to_ui.send(());
                }
                libc::close(fd);
            }
        });
}
