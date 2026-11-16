# The taskbar's thumbnails

Code: `crates/shell/src/taskbar/thumbnails.rs`, `taskbar/buttons.rs`, `taskbar/mod.rs`; the tab
preview (`tab_preview.rs`) shares the live picture.

## Purpose

The pointer resting on a button with windows shows a card per window: its title, a close button
and a live picture. The pointer can cross from the button to the cards and click them.

## Design

- **A layer of their own**, not a popup of the bar: a popup takes a grab, and the cards must take
  clicks with no press on the button first.
- **The bar owns when they go.** `Bar` keeps two facts: the button with windows the pointer is on
  (`button_inside`) and whether the pointer is in the thumbnails (`preview_inside`). One effect in
  the bar's own layer arms a timer of `HIDE_THUMBNAILS` while both are off, and its next run
  cancels the timer. No polling.
- **Thumbnails already open go over to another button at once**; the first open waits
  `thumb_delay`.
- **One list of groups per bar** (`Bar::groups`) for the row of buttons and the thumbnails. A
  card finds its window by id in the few windows of its group.
- **One live picture** (`thumbnails::live_picture`): the capture at the screen's scale, taken again
  every `LIVE_EVERY`, fitted in its box. The thumbnails and the tab preview each keep their own
  delay and placement.

## Subtleties

- **A timer lives in the window it was set in.** A timer set from the thumbnails' layer died when
  the layer closed, so the hide timer is set in the bar's layer only.
- **A layer that closes calls no hover handler**: `preview_inside` and the window highlight are
  reset when the layer closes.
- **A button only clears `button_inside` when it holds its own key**: the next button may have
  set its key first.
- **The tooltip of a button names a pinned program that is not running.** With an empty label
  `shell_tooltip` opens no popup: an empty one stood between the button and the thumbnails.

## Refused

- **Re-arming the timer while the pointer is in the thumbnails**: polling, and the timer was owned
  by whichever layer armed it.
- **A map of every window by id, kept for the session**: rebuilt on each window change with no
  thumbnails shown.
