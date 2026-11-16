//! Waiting on the disk without polling: inotify, on a worker thread.
//!
//! A followed file is watched through its folder, not by itself: editors and atomic writers
//! (the shell's own settings, GTK's `recently-used.xbel`) write a new file and rename it over the
//! old one, and a watch on the old inode would see nothing after the first rename. While a folder
//! on the way is missing, its nearest existing parent is watched for it, and the watches are set
//! again when it appears.
//!
//! A [`Watch`] only says that something may have changed: the caller looks (a time, a read, a
//! connect). An event can then be told twice, or come from a name that only looks like the
//! file's; that costs a `stat`, never a wrong value.

use std::collections::HashSet;
use std::ffi::{CString, OsStr, OsString};
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const MASK: u32 = libc::IN_CREATE
    | libc::IN_DELETE
    | libc::IN_MOVED_FROM
    | libc::IN_MOVED_TO
    | libc::IN_CLOSE_WRITE
    | libc::IN_ATTRIB
    | libc::IN_DELETE_SELF
    | libc::IN_MOVE_SELF;

/// Events after which the watches must be set again: a watched folder went away.
const GONE: u32 = libc::IN_DELETE_SELF | libc::IN_MOVE_SELF | libc::IN_IGNORED;

fn owned(paths: &[impl AsRef<Path>]) -> Vec<PathBuf> {
    paths.iter().map(|p| p.as_ref().to_path_buf()).collect()
}

pub struct Watch {
    fd: OwnedFd,
    /// Folders whose every change counts.
    folders: Vec<PathBuf>,
    /// Files whose changes count.
    files: Vec<PathBuf>,
    /// The watches on `folders` themselves: any name there counts.
    whole: HashSet<i32>,
    /// The names that count in the other watches: the files' names, and the name of each folder
    /// on the way that does not exist yet.
    names: HashSet<OsString>,
}

impl Watch {
    /// Follows every change of the entries of each folder (not below them).
    pub fn folders(folders: &[impl AsRef<Path>]) -> io::Result<Self> {
        Self::new(owned(folders), Vec::new())
    }

    /// Follows each file, including its creation, removal and replacement by a rename.
    pub fn files(files: &[impl AsRef<Path>]) -> io::Result<Self> {
        Self::new(Vec::new(), owned(files))
    }

    fn new(folders: Vec<PathBuf>, files: Vec<PathBuf>) -> io::Result<Self> {
        // SAFETY: a plain call; the descriptor is owned at once.
        let fd = unsafe { libc::inotify_init1(libc::IN_CLOEXEC | libc::IN_NONBLOCK) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let mut watch = Watch {
            // SAFETY: `fd` is a fresh descriptor nothing else owns.
            fd: unsafe { OwnedFd::from_raw_fd(fd) },
            folders,
            files,
            whole: HashSet::new(),
            names: HashSet::new(),
        };
        watch.arm();
        Ok(watch)
    }

    fn add(&self, dir: &Path) -> Option<i32> {
        let path = CString::new(dir.as_os_str().as_bytes()).ok()?;
        // SAFETY: `path` outlives the call; the descriptor is ours.
        let wd = unsafe { libc::inotify_add_watch(self.fd.as_raw_fd(), path.as_ptr(), MASK) };
        (wd >= 0).then_some(wd)
    }

    /// Watches `dir`, or, while it is missing, its nearest existing parent for the name on the way.
    fn add_folder(&mut self, dir: &Path) -> Option<i32> {
        if let Some(wd) = self.add(dir) {
            return Some(wd);
        }
        let mut child = dir;
        for parent in dir.ancestors().skip(1) {
            if let Some(name) = child.file_name() {
                self.names.insert(name.to_owned());
            }
            if self.add(parent).is_some() {
                return None;
            }
            child = parent;
        }
        None
    }

    /// Sets the watches. Adding a watch on a folder watched already gives its descriptor again.
    fn arm(&mut self) {
        for folder in self.folders.clone() {
            if let Some(wd) = self.add_folder(&folder) {
                self.whole.insert(wd);
            }
        }
        for file in self.files.clone() {
            if let Some(name) = file.file_name() {
                self.names.insert(name.to_owned());
            }
            if let Some(dir) = file.parent() {
                self.add_folder(dir);
            }
        }
    }

    /// Waits until something followed may have changed (`true`) or `timeout` passes (`false`;
    /// `None` waits without bound). Changes that come together are told once.
    pub fn wait(&mut self, timeout: Option<Duration>) -> io::Result<bool> {
        let deadline = timeout.map(|t| Instant::now() + t);
        loop {
            let ms: i32 = match deadline {
                None => -1,
                Some(deadline) => {
                    let left = deadline.saturating_duration_since(Instant::now());
                    // Rounded up, so that a wait of less than a millisecond does not spin.
                    i32::try_from(left.as_micros().div_ceil(1000)).unwrap_or(i32::MAX)
                }
            };
            let mut poll = libc::pollfd {
                fd: self.fd.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            // SAFETY: one valid pollfd for the length of the call.
            let ready = unsafe { libc::poll(&mut poll, 1, ms) };
            if ready < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(error);
            }
            if ready == 0 {
                return Ok(false);
            }
            if self.read_events()? {
                return Ok(true);
            }
        }
    }

    /// Reads what is queued; whether any of it counts.
    fn read_events(&mut self) -> io::Result<bool> {
        // Aligned for the headers the kernel writes into it.
        #[repr(C, align(4))]
        struct Buffer([u8; 4096]);
        let mut buf = Buffer([0; 4096]);
        let mut counts = false;
        let mut rearm = false;
        loop {
            // SAFETY: `buf` outlives the call and its length is given.
            let n =
                unsafe { libc::read(self.fd.as_raw_fd(), buf.0.as_mut_ptr().cast(), buf.0.len()) };
            if n < 0 {
                let error = io::Error::last_os_error();
                match error.kind() {
                    io::ErrorKind::WouldBlock => break,
                    io::ErrorKind::Interrupted => continue,
                    _ => return Err(error),
                }
            }
            if n == 0 {
                break;
            }
            let bytes = &buf.0[..n as usize];
            let header = std::mem::size_of::<libc::inotify_event>();
            let mut at = 0;
            while at + header <= bytes.len() {
                // SAFETY: the kernel wrote a whole event header at `at`.
                let event: libc::inotify_event =
                    unsafe { std::ptr::read_unaligned(bytes[at..].as_ptr().cast()) };
                let end = (at + header + event.len as usize).min(bytes.len());
                let raw = &bytes[at + header..end];
                let name = OsStr::from_bytes(
                    &raw[..raw.iter().position(|&b| b == 0).unwrap_or(raw.len())],
                );
                at = end;
                if event.mask & libc::IN_Q_OVERFLOW != 0 {
                    // Events were lost: anything may have changed.
                    counts = true;
                    rearm = true;
                    continue;
                }
                if event.mask & GONE != 0 {
                    self.whole.remove(&event.wd);
                    counts = true;
                    rearm = true;
                    continue;
                }
                if self.whole.contains(&event.wd) {
                    counts = true;
                }
                if !name.is_empty() && self.names.contains(name) {
                    counts = true;
                    // A missing folder on the way may have come: watch it now.
                    rearm |= event.mask & (libc::IN_CREATE | libc::IN_MOVED_TO) != 0;
                }
            }
        }
        if rearm {
            self.arm();
        }
        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("shell-watch-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Waits up to a bound for a change to be told.
    fn told(watch: &mut Watch) -> bool {
        watch.wait(Some(Duration::from_secs(5))).unwrap()
    }

    #[test]
    fn follows_a_file_replaced_by_a_rename() {
        let dir = scratch("rename");
        let file = dir.join("settings.json");
        std::fs::write(&file, "1").unwrap();
        let mut watch = Watch::files(&[&file]).unwrap();
        for text in ["2", "3"] {
            let partial = dir.join("settings.json.partial");
            std::fs::write(&partial, text).unwrap();
            std::fs::rename(&partial, &file).unwrap();
            assert!(told(&mut watch));
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ignores_other_names_and_times_out() {
        let dir = scratch("other");
        let file = dir.join("a");
        let mut watch = Watch::files(&[&file]).unwrap();
        std::fs::write(dir.join("b"), "x").unwrap();
        assert!(!watch.wait(Some(Duration::from_millis(50))).unwrap());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn follows_a_file_whose_folder_comes_later() {
        let dir = scratch("later");
        let file = dir.join("one/two/file");
        let mut watch = Watch::files(&[&file]).unwrap();
        std::fs::create_dir(dir.join("one")).unwrap();
        assert!(told(&mut watch));
        std::fs::create_dir(dir.join("one/two")).unwrap();
        assert!(told(&mut watch));
        std::fs::write(&file, "x").unwrap();
        assert!(told(&mut watch));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
