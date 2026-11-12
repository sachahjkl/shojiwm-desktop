//! The suggestions under the Run dialog's field, as the text is typed: the history, the
//! installed programs, the commands of `PATH`, and the files and folders of a path.
//!
//! Everything here is plain data and functions: the dialog reads the sources in a task when it
//! opens ([`sources`]), then [`rank`] or [`path_suggestions`] give the rows, in a task per query.

use crate::fuzzy;
use anti_webui::shell::prelude::*;
use std::cmp::Reverse;
use std::collections::{BTreeSet, HashSet};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// The most suggestions the list shows.
pub const SHOWN: usize = 8;
/// The score added to the last command of the history; each older one gets `RECENT_STEP` less.
const RECENT_BONUS: u32 = 200;
const RECENT_STEP: u32 = 6;

/// What a suggestion is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Recent,
    Application,
    Command,
    Folder,
    File,
}

impl Kind {
    /// The muted word at the end of its row.
    pub fn word(self) -> &'static str {
        match self {
            Kind::Recent => "Recent",
            Kind::Application => "Application",
            Kind::Command => "Command",
            Kind::Folder => "Folder",
            Kind::File => "File",
        }
    }
}

/// A row of the list: what it shows (`label`, with the letters at `marks` highlighted), what
/// goes in the field (`text`), and the program to start for an application.
#[derive(Clone, PartialEq, Debug)]
pub struct Suggestion {
    pub kind: Kind,
    pub label: String,
    pub text: String,
    pub marks: Vec<usize>,
    pub app: Option<Application>,
}

impl Suggestion {
    /// A suggestion without an application, whose label is its text.
    pub fn plain(kind: Kind, text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            kind,
            label: text.clone(),
            text,
            marks: Vec::new(),
            app: None,
        }
    }
}

fn starts_with_ignoring_case(text: &str, start: &str) -> bool {
    text.to_lowercase().starts_with(&start.to_lowercase())
}

/// The candidates that match `query`, best first, at most [`SHOWN`], each text once. The
/// candidates come in the order history, applications, commands: a tie keeps that order.
/// Labels that start with the query come first, then the fuzzy matches; the history gets a
/// bonus, more for the most recent command. It goes over every candidate: run it in a task.
pub fn rank<'a>(
    query: &str,
    candidates: impl IntoIterator<Item = &'a Suggestion>,
) -> Vec<Suggestion> {
    let query = query.trim();
    let mut recent_seen = 0u32;
    let with_bonus: Vec<(&Suggestion, u32)> = candidates
        .into_iter()
        .map(|s| {
            let bonus = if s.kind == Kind::Recent {
                let bonus = RECENT_BONUS.saturating_sub(recent_seen * RECENT_STEP);
                recent_seen += 1;
                bonus
            } else {
                0
            };
            (s, bonus)
        })
        .collect();
    let mut found = fuzzy::rank(query, with_bonus, |(s, _)| s.label.clone());
    found.sort_by_key(|m| {
        let (s, bonus) = m.item;
        let prefix =
            starts_with_ignoring_case(&s.label, query) || starts_with_ignoring_case(&s.text, query);
        (!prefix, Reverse(m.score + bonus), s.label.chars().count())
    });
    let mut seen: HashSet<&str> = HashSet::new();
    let mut out = Vec::new();
    for m in found {
        let (s, _) = m.item;
        if !seen.insert(s.text.as_str()) {
            continue;
        }
        let mut s = s.clone();
        s.marks = m.marks;
        out.push(s);
        if out.len() == SHOWN {
            break;
        }
    }
    out
}

/// The commands of the history, last first, as suggestions.
pub fn history_suggestions(history: &[String]) -> Vec<Suggestion> {
    history
        .iter()
        .map(|entry| Suggestion::plain(Kind::Recent, entry.clone()))
        .collect()
}

/// What the dialog suggests besides its history: the applications, each with the command its
/// `Exec` line runs, then the commands of `PATH` (`commands`, sorted). Build it in a task.
pub fn sources<'a>(
    programs: impl IntoIterator<Item = (&'a Application, &'a str)>,
    commands: &[String],
) -> Vec<Suggestion> {
    let on_path = |name: &str| commands.binary_search_by(|c| c.as_str().cmp(name)).is_ok();
    let mut out: Vec<Suggestion> = programs
        .into_iter()
        .map(|(app, exec)| Suggestion {
            kind: Kind::Application,
            label: app.name.clone(),
            text: app_command(exec, on_path),
            marks: Vec::new(),
            app: Some(app.clone()),
        })
        .collect();
    out.extend(
        commands
            .iter()
            .map(|name| Suggestion::plain(Kind::Command, name.clone())),
    );
    out
}

/// The first suggestion whose text continues `query` (whatever the case), for the field to
/// complete inline.
pub fn inline(query: &str, suggestions: &[Suggestion]) -> Option<String> {
    if query.is_empty() {
        return None;
    }
    let length = query.chars().count();
    suggestions
        .iter()
        .find(|s| s.text.chars().count() > length && starts_with_ignoring_case(&s.text, query))
        .map(|s| s.text.clone())
}

// ----- the commands of PATH ------------------------------------------------------------------------

/// A file name of `PATH` that is a command to suggest: not hidden, not a Nix wrapper.
pub fn keeps_command(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('.') && !name.ends_with("-wrapped")
}

/// The folders of `PATH`.
pub fn path_dirs() -> Vec<PathBuf> {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default()
}

/// The names of the executable files in `dirs` (a link counts for what it points to), sorted,
/// each once. It reads the disk: run it in a task.
pub fn scan_commands(dirs: &[PathBuf]) -> Vec<String> {
    let mut names = BTreeSet::new();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !keeps_command(&name) || names.contains(&name) {
                continue;
            }
            let executable = std::fs::metadata(entry.path())
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0);
            if executable {
                names.insert(name);
            }
        }
    }
    names.into_iter().collect()
}

/// What an application's `Exec` line runs, for the field: the line without its field codes
/// (`%u`, `%F`, ...), and its first word shortened to its file name when that is a command of
/// `PATH` (`/nix/store/…/bin/ghostty` gives `ghostty`).
pub fn app_command(exec: &str, on_path: impl Fn(&str) -> bool) -> String {
    let mut words: Vec<String> = exec
        .split_whitespace()
        .filter(|w| !(w.len() == 2 && w.starts_with('%') && *w != "%%"))
        .map(|w| w.replace("%%", "%"))
        .collect();
    if let Some(first) = words.first_mut() {
        let bare = first.trim_matches('"');
        if bare.starts_with('/')
            && let Some(name) = bare.rsplit('/').next()
            && on_path(name)
        {
            *first = name.to_owned();
        }
    }
    words.join(" ")
}

// ----- paths -----------------------------------------------------------------------------------------

/// A name in a folder, and whether it is a folder (a link counts for what it points to).
#[derive(Clone, PartialEq, Debug)]
pub struct Entry {
    pub name: String,
    pub folder: bool,
}

/// The folder part of a path, as typed, and the start of the name in it: `~/Doc` gives
/// (`~/`, `Doc`), `~` gives (`~/`, ``). `None` for a text that is not a path: one that does
/// not start with `/`, `~/`, `./` or `../`.
pub fn split_path(text: &str) -> Option<(&str, &str)> {
    if text == "~" {
        return Some(("~/", ""));
    }
    let path = ["/", "~/", "./", "../"].iter().any(|p| text.starts_with(p));
    if !path {
        return None;
    }
    let cut = text.rfind('/')? + 1;
    Some((&text[..cut], &text[cut..]))
}

/// The folder a typed folder part names: `~/` in `home`, `./` and `../` from `home` (where
/// the dialog runs its commands).
pub fn resolve_dir(dir: &str, home: &Path) -> PathBuf {
    if let Some(rest) = dir.strip_prefix("~/") {
        home.join(rest)
    } else if dir.starts_with('/') {
        PathBuf::from(dir)
    } else {
        home.join(dir)
    }
}

/// The names in `dir`, sorted. It reads the disk: run it in a task.
pub fn list_dir(dir: &Path) -> Vec<Entry> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<Entry> = entries
        .flatten()
        .map(|e| Entry {
            name: e.file_name().to_string_lossy().into_owned(),
            folder: std::fs::metadata(e.path()).is_ok_and(|m| m.is_dir()),
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// The suggestions of a path: the names of `entries` that match `start`, after `dir` as
/// typed, folders ending with `/`. Hidden names only when `start` begins with a dot. Nothing
/// typed after the folder: its folders, then its files, by name.
pub fn path_suggestions(dir: &str, start: &str, entries: &[Entry]) -> Vec<Suggestion> {
    let hidden = start.starts_with('.');
    let visible = entries
        .iter()
        .filter(|e| hidden || !e.name.starts_with('.'));
    let make = |e: &Entry| {
        let kind = if e.folder { Kind::Folder } else { Kind::File };
        let slash = if e.folder { "/" } else { "" };
        Suggestion::plain(kind, format!("{dir}{}{slash}", e.name))
    };
    if start.is_empty() {
        let mut list: Vec<&Entry> = visible.collect();
        list.sort_by_key(|e| !e.folder);
        return list.into_iter().take(SHOWN).map(make).collect();
    }
    let offset = dir.chars().count();
    let mut found = fuzzy::rank(start, visible, |e| e.name.clone());
    found.sort_by_key(|m| {
        (
            !starts_with_ignoring_case(&m.item.name, start),
            Reverse(m.score),
            m.item.name.chars().count(),
        )
    });
    found
        .into_iter()
        .take(SHOWN)
        .map(|m| {
            let mut s = make(m.item);
            s.marks = m.marks.iter().map(|i| i + offset).collect();
            s
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(list: &[Suggestion]) -> Vec<&str> {
        list.iter().map(|s| s.text.as_str()).collect()
    }

    #[test]
    fn keeps_plain_commands_only() {
        assert!(keeps_command("ghostty"));
        assert!(keeps_command("git-upload-pack"));
        assert!(!keeps_command(".ghostty-wrapped"));
        assert!(!keeps_command("ghostty-wrapped"));
        assert!(!keeps_command(".hidden"));
        assert!(!keeps_command(""));
    }

    #[test]
    fn scans_executable_files_once() {
        let root = std::env::temp_dir().join(format!("run-completion-{}", std::process::id()));
        let (a, b) = (root.join("a"), root.join("b"));
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        let write = |path: PathBuf, mode: u32| {
            std::fs::write(&path, "").unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
        };
        write(a.join("ghostty"), 0o755);
        write(a.join(".ghostty-wrapped"), 0o755);
        write(a.join("notes.txt"), 0o644);
        write(b.join("ghostty"), 0o755);
        write(b.join("vim"), 0o755);
        std::fs::create_dir_all(b.join("folder")).unwrap();
        std::os::unix::fs::symlink(b.join("vim"), b.join("vi")).unwrap();
        std::os::unix::fs::symlink(b.join("missing"), b.join("broken")).unwrap();
        let found = scan_commands(&[a, b, root.join("absent")]);
        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(found, vec!["ghostty", "vi", "vim"]);
    }

    #[test]
    fn splits_paths() {
        assert_eq!(split_path("~/Doc"), Some(("~/", "Doc")));
        assert_eq!(split_path("~"), Some(("~/", "")));
        assert_eq!(split_path("/usr/bi"), Some(("/usr/", "bi")));
        assert_eq!(split_path("/"), Some(("/", "")));
        assert_eq!(split_path("./build/"), Some(("./build/", "")));
        assert_eq!(split_path("../x"), Some(("../", "x")));
        assert_eq!(split_path("ghostty"), None);
        assert_eq!(split_path("~user"), None);
        assert_eq!(split_path("vim ~/notes"), None);
    }

    #[test]
    fn resolves_folders_from_home() {
        let home = Path::new("/home/me");
        assert_eq!(resolve_dir("~/", home), PathBuf::from("/home/me/"));
        assert_eq!(resolve_dir("~/Doc/", home), PathBuf::from("/home/me/Doc/"));
        assert_eq!(resolve_dir("/usr/", home), PathBuf::from("/usr/"));
        assert_eq!(resolve_dir("./a/", home), PathBuf::from("/home/me/./a/"));
    }

    #[test]
    fn suggests_names_of_a_folder() {
        let entries = vec![
            Entry {
                name: ".config".into(),
                folder: true,
            },
            Entry {
                name: "Documents".into(),
                folder: true,
            },
            Entry {
                name: "doc.txt".into(),
                folder: false,
            },
            Entry {
                name: "Downloads".into(),
                folder: true,
            },
            Entry {
                name: "notes".into(),
                folder: false,
            },
        ];
        let all = path_suggestions("~/", "", &entries);
        assert_eq!(
            texts(&all),
            ["~/Documents/", "~/Downloads/", "~/doc.txt", "~/notes"]
        );
        let doc = path_suggestions("~/", "doc", &entries);
        assert_eq!(texts(&doc), ["~/doc.txt", "~/Documents/"]);
        assert_eq!(doc[1].kind, Kind::Folder);
        assert_eq!(doc[1].marks, vec![2, 3, 4]);
        let hidden = path_suggestions("~/", ".c", &entries);
        assert_eq!(texts(&hidden), ["~/.config/"]);
    }

    #[test]
    fn ranks_prefixes_first_and_recent_higher() {
        let candidates = vec![
            Suggestion::plain(Kind::Recent, "ghostty --title work"),
            Suggestion::plain(Kind::Command, "light-ghost"),
            Suggestion::plain(Kind::Command, "ghostty"),
            Suggestion::plain(Kind::Command, "gh"),
            Suggestion::plain(Kind::Command, "vim"),
        ];
        let found = rank("gh", &candidates);
        assert_eq!(
            texts(&found),
            ["ghostty --title work", "gh", "ghostty", "light-ghost"]
        );
        assert_eq!(found[1].marks, vec![0, 1]);
    }

    #[test]
    fn ranks_each_text_once_and_at_most_shown() {
        let mut candidates = vec![Suggestion::plain(Kind::Recent, "ghostty")];
        candidates.push(Suggestion::plain(Kind::Command, "ghostty"));
        candidates.extend((0..20).map(|i| Suggestion::plain(Kind::Command, format!("g{i}"))));
        let found = rank("g", &candidates);
        assert_eq!(found.len(), SHOWN);
        assert_eq!(found[0].kind, Kind::Recent);
        assert_eq!(found.iter().filter(|s| s.text == "ghostty").count(), 1);
    }

    #[test]
    fn the_history_comes_first_and_ranks_higher() {
        let history = history_suggestions(&["vim notes".to_owned()]);
        let others = sources(std::iter::empty(), &["vi".to_owned(), "vim".to_owned()]);
        assert_eq!(texts(&others), ["vi", "vim"]);
        assert_eq!(others[0].kind, Kind::Command);
        let found = rank("vi", history.iter().chain(&others));
        assert_eq!(found[0].kind, Kind::Recent);
        assert_eq!(texts(&found), ["vim notes", "vi", "vim"]);
    }

    #[test]
    fn completes_inline_from_the_first_that_continues() {
        let list = vec![
            Suggestion::plain(Kind::Command, "light-ghost"),
            Suggestion::plain(Kind::Command, "Ghostty"),
        ];
        assert_eq!(inline("gho", &list).as_deref(), Some("Ghostty"));
        assert_eq!(inline("ghostty", &list), None);
        assert_eq!(inline("", &list), None);
    }

    #[test]
    fn commands_of_applications() {
        let on_path = |name: &str| name == "ghostty";
        assert_eq!(
            app_command(
                "/nix/store/abc-ghostty/bin/ghostty --gtk-single-instance=true",
                on_path
            ),
            "ghostty --gtk-single-instance=true"
        );
        assert_eq!(app_command("firefox %u", on_path), "firefox");
        assert_eq!(app_command("/opt/app/run %F", on_path), "/opt/app/run");
        assert_eq!(app_command("printf 100%%", on_path), "printf 100%");
    }
}
