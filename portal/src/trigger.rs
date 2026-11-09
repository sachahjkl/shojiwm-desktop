//! Shortcut triggers: from the portal's `preferred_trigger` to a ShojiWM key
//! binding.
//!
//! The XDG shortcuts specification writes a trigger as modifiers and one
//! keysym name joined by `+`: `CTRL+SHIFT+a`, `LOGO+F1`, `ALT+space`. Some
//! programs send the GTK form instead: `<Control><Alt>v`. Both become
//! ShojiWM's form, `Ctrl+Alt+Shift+Super+key`, with the modifiers in this
//! order, so one key gets one binding.

/// A trigger in ShojiWM's form, or `None` when it names no key.
pub fn parse(text: &str) -> Option<String> {
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut logo = false;
    let mut key = None;

    for token in tokens(text) {
        match token.to_ascii_lowercase().as_str() {
            "ctrl" | "control" | "primary" => ctrl = true,
            "alt" | "mod1" | "option" => alt = true,
            "shift" => shift = true,
            "logo" | "super" | "meta" | "mod4" | "win" | "cmd" | "command" => logo = true,
            // Num Lock is a state, not a modifier a shortcut can need.
            "num" | "numlock" => {}
            _ if key.is_some() => return None,
            _ => key = Some(token),
        }
    }

    let key = key?;
    if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    // One letter is the lowercase keysym: `a`, not `A` (which is Shift+a).
    let key = if key.len() == 1 {
        key.to_ascii_lowercase()
    } else {
        key.to_string()
    };

    let mut parts = Vec::new();
    for (on, name) in [
        (ctrl, "Ctrl"),
        (alt, "Alt"),
        (shift, "Shift"),
        (logo, "Super"),
    ] {
        if on {
            parts.push(name.to_string());
        }
    }
    parts.push(key);
    Some(parts.join("+"))
}

/// What the user reads for a trigger: `Alt+Space`, `Ctrl+Alt+V`.
pub fn describe(trigger: &str) -> String {
    trigger
        .split('+')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("+")
}

/// The parts of a trigger, in either form.
fn tokens(text: &str) -> Vec<&str> {
    let text = text.trim();
    if text.starts_with('<') {
        // `<Control><Alt>v`: each modifier in angle brackets, then the key.
        let mut out = Vec::new();
        let mut rest = text;
        while let Some(stripped) = rest.strip_prefix('<') {
            let Some(end) = stripped.find('>') else {
                break;
            };
            out.push(&stripped[..end]);
            rest = &stripped[end + 1..];
        }
        if !rest.is_empty() {
            out.push(rest);
        }
        out
    } else {
        text.split('+')
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specification_form() {
        assert_eq!(parse("ALT+space").as_deref(), Some("Alt+space"));
        assert_eq!(parse("CTRL+SHIFT+a").as_deref(), Some("Ctrl+Shift+a"));
        assert_eq!(parse("LOGO+F1").as_deref(), Some("Super+F1"));
        assert_eq!(
            parse("SHIFT+CTRL+ALT+v").as_deref(),
            Some("Ctrl+Alt+Shift+v")
        );
        assert_eq!(parse("NUM+CTRL+x").as_deref(), Some("Ctrl+x"));
    }

    #[test]
    fn gtk_form() {
        assert_eq!(parse("<Control><Alt>v").as_deref(), Some("Ctrl+Alt+v"));
        assert_eq!(parse("<Super>Return").as_deref(), Some("Super+Return"));
    }

    #[test]
    fn other_spellings() {
        assert_eq!(parse("option+space").as_deref(), Some("Alt+space"));
        assert_eq!(parse("Ctrl+A").as_deref(), Some("Ctrl+a"));
        assert_eq!(parse("f9").as_deref(), Some("f9"));
    }

    #[test]
    fn no_key() {
        assert_eq!(parse(""), None);
        assert_eq!(parse("CTRL+ALT"), None);
        assert_eq!(parse("CTRL+a+b"), None);
        assert_eq!(parse("CTRL+a b"), None);
    }

    #[test]
    fn description() {
        assert_eq!(describe("Alt+space"), "Alt+Space");
        assert_eq!(describe("Ctrl+Alt+v"), "Ctrl+Alt+V");
    }
}
