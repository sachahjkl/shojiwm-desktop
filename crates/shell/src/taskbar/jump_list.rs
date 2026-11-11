//! The jump list of a taskbar button, on a right click (QML `JumpList`). From the top: the files
//! pinned to it and the files the program opened last (from `recently-used.xbel`), each with
//! its pin; its tasks, which are the actions of its `.desktop` file; then the program itself,
//! the pin to the taskbar, and "Close window".

use super::Bar;
use crate::apps::{
    WindowGroup, file_icon, is_taskbar_pinned, launch, open_with, pinned_files_for,
    recent_unpinned, set_file_pinned, set_taskbar_pinned,
};
use crate::ui::{MenuLine, bar_menu_options, popup_menu};
use anti_webui::prelude::*;

/// The jump list of the button `key`, open while the bar says so. Put it in the button, whose
/// box is `anchor`.
pub(super) fn jump_list(
    bar: Bar,
    key: String,
    group: Memo<Option<WindowGroup>>,
    anchor: NodeRef,
) -> Show {
    let open = bar.jump_list;
    popup_menu(
        move || open.with(|j| j.as_deref() == Some(key.as_str())),
        move || bar.jump_list.set(None),
        untrack(|| bar_menu_options(anchor)),
        move || {
            untrack(|| group.get())
                .map(|g| lines(&g))
                .unwrap_or_default()
        },
    )
}

fn lines(group: &WindowGroup) -> Vec<MenuLine> {
    let mut lines = Vec::new();
    let mut files_or_tasks = false;
    if let Some(entry) = &group.entry {
        let pinned = pinned_files_for(entry);
        if !pinned.is_empty() {
            lines.push(MenuLine::heading("Pinned"));
        }
        for file in pinned {
            let (open, unpin) = (
                (entry.clone(), file.path.clone()),
                (entry.clone(), file.path.clone()),
            );
            lines.push(
                MenuLine::item(file.name.clone(), move || open_with(&open.0, &open.1))
                    .icon(file_icon(&file.name))
                    .pin(true, move || set_file_pinned(&unpin.0, &unpin.1, false)),
            );
        }
        let recent = recent_unpinned(entry);
        if !recent.is_empty() {
            lines.push(MenuLine::heading("Recent"));
            files_or_tasks = true;
        }
        for file in recent {
            let (open, pin) = (
                (entry.clone(), file.path.clone()),
                (entry.clone(), file.path.clone()),
            );
            lines.push(
                MenuLine::item(file.name.clone(), move || open_with(&open.0, &open.1))
                    .icon(file_icon(&file.name))
                    .pin(false, move || set_file_pinned(&pin.0, &pin.1, true)),
            );
        }
        if !entry.actions.is_empty() {
            lines.push(MenuLine::heading("Tasks"));
            files_or_tasks = true;
        }
        for action in &entry.actions {
            let run = action.clone();
            lines.push(
                MenuLine::item(action.name.clone(), move || {
                    let _ = run.launch();
                })
                .icon(action.icon.clone().unwrap_or_default()),
            );
        }
    }
    if files_or_tasks {
        lines.push(MenuLine::separator());
    }
    let open_label = if group.name.is_empty() {
        "Open".to_owned()
    } else {
        group.name.clone()
    };
    let start = group.entry.clone();
    lines.push(
        MenuLine::item(open_label, move || {
            if let Some(entry) = &start {
                launch(entry);
            }
        })
        .icon(group.icon.clone())
        .enabled(group.entry.is_some()),
    );
    if let Some(entry) = &group.entry {
        let pinned = is_taskbar_pinned(entry);
        let label = if pinned {
            "Unpin this program from taskbar"
        } else {
            "Pin this program to taskbar"
        };
        let entry = entry.clone();
        lines.push(MenuLine::item(label, move || {
            set_taskbar_pinned(&entry, !pinned)
        }));
    }
    if !group.windows.is_empty() {
        lines.push(MenuLine::separator());
        let label = if group.windows.len() > 1 {
            "Close all windows"
        } else {
            "Close window"
        };
        let windows = group.windows.clone();
        lines.push(MenuLine::item(label, move || {
            for w in &windows {
                let _ = w.close();
            }
        }));
    }
    lines
}
