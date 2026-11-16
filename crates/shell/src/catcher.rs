//! The click-catcher of each screen: one transparent layer over the screen that closes what is
//! open there (the Start menu, the launcher, the popover of hidden icons) on a press outside it.
//!
//! What can be open registers a [`claim`]: whether it is open on a screen, how much of the
//! screen the catcher must take for it, and how to close it. The catcher of a screen takes the
//! union of the claims open there, and a press on it closes them all. See
//! `src/md/features/click-catcher.md`.

use crate::theme::{bar_at_top, theme};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// How much of the screen the catcher takes while a claim is open. The larger wins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reach {
    /// The screen but the bar: a press on the bar reaches the bar, which closes its own
    /// popovers, and a drop on the bar is the bar's.
    OutsideBar,
    /// The whole screen, the bar too.
    Screen,
}

type IsOpen = Rc<dyn Fn(&str) -> bool>;
type Close = Rc<dyn Fn()>;

struct Claim {
    id: u64,
    open: IsOpen,
    reach: Reach,
    close: Close,
}

thread_local! {
    static CLAIMS: RefCell<Vec<Claim>> = const { RefCell::new(Vec::new()) };
    static NEXT_ID: Cell<u64> = const { Cell::new(0) };
    /// Bumped when a claim comes or goes: the closures are not signals, the catchers read this.
    static CHANGED: Cell<Option<Signal<u64>>> = const { Cell::new(None) };
}

fn changed() -> Signal<u64> {
    CHANGED.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (s, _) = create_root(|| signal(0_u64));
            cell.set(Some(s));
            s
        })
    })
}

/// Registers something the catchers close, for as long as the owner it is called under lives.
/// `open(screen)` says whether it is open on the screen of that connector name; reading it
/// tracks what it reads. `close` closes it.
pub fn claim(open: impl Fn(&str) -> bool + 'static, reach: Reach, close: impl Fn() + 'static) {
    let id = NEXT_ID.with(|n| n.replace(n.get() + 1));
    CLAIMS.with(|c| {
        c.borrow_mut().push(Claim {
            id,
            open: Rc::new(open),
            reach,
            close: Rc::new(close),
        })
    });
    let bump = changed();
    bump.set(bump.get_untracked() + 1);
    on_cleanup(move || {
        CLAIMS.with(|c| c.borrow_mut().retain(|claim| claim.id != id));
        bump.set(bump.get_untracked() + 1);
    });
}

/// The claims open on a screen. Reading it tracks them.
fn open_on(screen: &str) -> Vec<(Reach, Close)> {
    let _ = changed().get();
    // Copied out first: a claim's `open` runs without the list borrowed.
    let claims: Vec<(IsOpen, Reach, Close)> = CLAIMS.with(|c| {
        c.borrow()
            .iter()
            .map(|claim| (claim.open.clone(), claim.reach, claim.close.clone()))
            .collect()
    });
    claims
        .into_iter()
        .filter(|(open, _, _)| open(screen))
        .map(|(_, reach, close)| (reach, close))
        .collect()
}

/// Closes everything open on a screen.
fn close_on(screen: &str) {
    for (_, close) in untrack(|| open_on(screen)) {
        close();
    }
}

/// Opens the catcher of a screen, for as long as the owner it is called under lives. Open it
/// after the bar, so that it stands over it, and before the layers it closes, so that it stands
/// under them.
///
/// It stays mapped with an empty input region while nothing is open: unmapping two layers in one
/// frame once ended the shell's Wayland connection. It asks for no blur: there is nothing of it
/// to see.
pub fn open(output: &Output) {
    let name = output.name.clone();
    let layer = open_layer(
        LayerOptions::new()
            .layer(Layer::Top)
            .anchor(Anchor::ALL)
            .exclusive(Exclusive::Ignore)
            .keyboard(Keyboard::None)
            .input_region(InputRegion::Empty)
            .output(output)
            .namespace("no_blur"),
        {
            let name = name.clone();
            move || div().size_full().on_pointer_down(move |_| close_on(&name))
        },
    );
    let output = output.clone();
    let region = memo(move || {
        let Some(reach) = open_on(&name).into_iter().map(|(reach, _)| reach).max() else {
            return InputRegion::Empty;
        };
        match reach {
            Reach::Screen => InputRegion::Full,
            Reach::OutsideBar => {
                // The screen as it is now: a mode or a scale change moves the bar's edge.
                let size = output
                    .current()
                    .map_or(output.logical_size, |o| o.logical_size);
                InputRegion::Rects(vec![outside_bar(
                    size.width,
                    size.height,
                    theme().taskbar_height as i32,
                    bar_at_top(),
                )])
            }
        }
    });
    effect(move || layer.set_input_region(region.get()));
}

/// The part of a screen `width` by `height` that a bar `bar` high, at its top or its bottom,
/// leaves.
fn outside_bar(width: i32, height: i32, bar: i32, at_top: bool) -> RectI {
    let bar = bar.clamp(0, height);
    RectI::new(0, if at_top { bar } else { 0 }, width, height - bar)
}

#[cfg(test)]
mod tests {
    use super::outside_bar;
    use anti_webui::prelude::RectI;

    #[test]
    fn the_catcher_leaves_the_bar() {
        assert_eq!(
            outside_bar(1920, 1080, 40, false),
            RectI::new(0, 0, 1920, 1040)
        );
        assert_eq!(
            outside_bar(1920, 1080, 40, true),
            RectI::new(0, 40, 1920, 1040)
        );
    }
}
