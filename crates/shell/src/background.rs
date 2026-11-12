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
