//! Fuzzy matching for the searches (the launcher, the Start menu): what is typed matches a name
//! when its letters come in order in it, not only side by side. "fx" finds "Firefox". The best
//! matches come first: letters side by side, at the start of words, and at the start of the
//! name. Each match says which letters of the name it used, for the rows to highlight them.
//!
//! The scoring is nucleo's (the matcher of the Helix editor), with its fzf-like rules.

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// A thing that matched: its score (higher is better) and the indices, in characters, of the
/// letters of its name that the typed text used, in order.
pub struct Matched<T> {
    pub item: T,
    pub score: u32,
    pub marks: Vec<usize>,
}

/// The items whose name (`name_of`) matches `typed`, best first; ties keep the items' order.
/// Nothing typed matches nothing.
pub fn rank<T>(
    typed: &str,
    items: impl IntoIterator<Item = T>,
    name_of: impl Fn(&T) -> String,
) -> Vec<Matched<T>> {
    let typed = typed.trim();
    if typed.is_empty() {
        return Vec::new();
    }
    let pattern = Pattern::parse(typed, CaseMatching::Ignore, Normalization::Smart);
    let mut matcher = Matcher::new(Config::DEFAULT);
    let mut buffer = Vec::new();
    let mut found: Vec<Matched<T>> = items
        .into_iter()
        .filter_map(|item| {
            let name = name_of(&item);
            let mut marks: Vec<u32> = Vec::new();
            let score =
                pattern.indices(Utf32Str::new(&name, &mut buffer), &mut matcher, &mut marks)?;
            marks.sort_unstable();
            marks.dedup();
            Some(Matched {
                item,
                score,
                marks: marks.into_iter().map(|i| i as usize).collect(),
            })
        })
        .collect();
    // Stable: equal scores keep the order the items came in.
    found.sort_by_key(|m| std::cmp::Reverse(m.score));
    found
}

/// A regular expression for `fd` that finds the names `typed` would match: its letters in order,
/// anything between them. The ranking is then [`rank`]'s.
pub fn file_pattern(typed: &str) -> String {
    typed
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| {
            let mut s = String::new();
            if "\\.+*?()|[]{}^$#&-~".contains(c) {
                s.push('\\');
            }
            s.push(c);
            s
        })
        .collect::<Vec<_>>()
        .join(".*")
}
