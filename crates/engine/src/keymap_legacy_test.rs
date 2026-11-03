#![allow(dead_code)]
use super::{Binding, Context, KEYS, Platform};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub shift: bool,
    pub alt: bool,
    pub cmd: bool,
    pub ctrl: bool,
}

impl Mods {
    pub fn is_none(self) -> bool {
        self == Mods::default()
    }
}

/// How far a text editing key moves or deletes on `platform`, as `(word, line)` for
/// [`crate::textedit::key`]. macOS: Alt by word, Cmd to the line's ends (Up/Down: the text's).
/// Windows and Linux: Ctrl by word, Ctrl+Home/End to the text's ends.
pub fn text_motion(platform: Platform, key: &str, m: Mods) -> (bool, bool) {
    match platform {
        Platform::Mac => (m.alt, m.cmd),
        Platform::Other if matches!(key, "Home" | "End") => (false, m.cmd || m.ctrl),
        Platform::Other => (m.cmd || m.ctrl, false),
    }
}

/// A key name as canonical (`=` for `Equals`, `ArrowLeft` for `Left`, `Delete` for `Backspace`),
/// and whether the name implies Shift (`?` is Shift+`/`). `None` for names that aren't keys.
pub fn normalize(name: &str) -> Option<(&str, bool)> {
    let shifted = |k: &'static str| Some((k, true));
    let plain = |k: &'static str| Some((k, false));
    match name {
        "" => None,
        "Plus" | "+" => shifted("="),
        "Questionmark" | "?" => shifted("/"),
        "<" => shifted(","),
        ">" => shifted("."),
        "OpenCurlyBracket" | "{" => shifted("["),
        "CloseCurlyBracket" | "}" => shifted("]"),
        "Pipe" | "|" => shifted("\\"),
        "Exclamationmark" | "!" => shifted("1"),
        "Colon" | ":" => shifted(";"),
        "\"" => shifted("'"),
        "_" => shifted("-"),
        "Equals" => plain("="),
        "Minus" => plain("-"),
        "OpenBracket" => plain("["),
        "CloseBracket" => plain("]"),
        "Backslash" => plain("\\"),
        "Quote" => plain("'"),
        "Comma" => plain(","),
        "Period" => plain("."),
        "Slash" => plain("/"),
        "Semicolon" => plain(";"),
        "Backtick" => plain("`"),
        "Backspace" | "Delete" | "Del" => plain("Delete"),
        "Return" | "Enter" => plain("Enter"),
        "Esc" | "Escape" => plain("Escape"),
        "Left" | "ArrowLeft" => plain("ArrowLeft"),
        "Right" | "ArrowRight" => plain("ArrowRight"),
        "Up" | "ArrowUp" => plain("ArrowUp"),
        "Down" | "ArrowDown" => plain("ArrowDown"),
        n if n.chars().count() == 1 => Some((n, false)),
        n if NAMED.iter().any(|k| k.eq_ignore_ascii_case(n)) => Some((n, false)),
        _ => None,
    }
}

const NAMED: &[&str] =
    &["Tab", "Space", "PageUp", "PageDown", "Home", "End", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "Menu"];

/// Parse a binding (`Shift+Cmd+K`) into its modifiers and canonical key.
pub fn parse(spec: &str) -> Option<(Mods, &str)> {
    let mut m = Mods::default();
    let mut key = None;
    for part in spec.split('+') {
        match part {
            "Shift" => m.shift = true,
            "Cmd" => m.cmd = true,
            "Alt" | "Option" => m.alt = true,
            "Ctrl" => m.ctrl = true,
            k => {
                if key.is_some() {
                    return None;
                }
                let (k, shift) = normalize(k)?;
                m.shift |= shift;
                key = Some(k);
            }
        }
    }
    key.map(|k| (m, k))
}

fn same_mods(platform: Platform, want: Mods, got: Mods) -> bool {
    match platform {
        Platform::Mac => want == got,
        // Cmd and Ctrl are one key here.
        Platform::Other => want.shift == got.shift && want.alt == got.alt && (want.cmd || want.ctrl) == (got.cmd || got.ctrl),
    }
}

/// Whether a binding fires for `key` with these modifiers on `platform`.
pub fn matches(spec: &str, platform: Platform, key: &str, mods: Mods) -> bool {
    let Some((key, shift)) = normalize(key) else { return false };
    let mods = Mods { shift: mods.shift || shift, ..mods };
    parse(spec).is_some_and(|(want, k)| k.eq_ignore_ascii_case(key) && same_mods(platform, want, mods))
}

/// The binding `key` with these modifiers fires in `context` on `platform`.
pub fn lookup(platform: Platform, context: Context, key: &str, mods: Mods) -> Option<&'static Binding> {
    KEYS.iter().filter(|b| b.live_in(context)).find(|b| b.keys(platform).iter().any(|s| matches(s, platform, key, mods)))
}

/// A command's keys on `platform` (all its bindings, the menu's first).
pub fn bindings(id: &str, platform: Platform) -> impl Iterator<Item = &'static str> {
    KEYS.iter().filter(move |b| b.id == id).flat_map(move |b| b.keys(platform).iter().copied())
}

/// The key a menu shows for a command on `platform`.
pub fn primary(id: &str, platform: Platform) -> Option<&'static str> {
    bindings(id, platform).next()
}

/// A binding as the platform writes it: `⇧⌘K` on macOS, `Ctrl+Shift+K` elsewhere.
pub fn display(spec: &str, platform: Platform) -> String {
    if spec.is_empty() {
        return String::new();
    }
    let mut m = Mods::default();
    let mut key = "";
    for part in spec.split('+') {
        match part {
            "Shift" => m.shift = true,
            "Cmd" => m.cmd = true,
            "Alt" | "Option" => m.alt = true,
            "Ctrl" => m.ctrl = true,
            k => key = k,
        }
    }
    // The zoom-in key reads as `+` (with Shift it types one).
    if key == "=" {
        key = "+";
        m.shift = false;
    }
    let mac = platform == Platform::Mac;
    let key = match (key, mac) {
        ("Delete", true) => "⌫",
        ("Enter", true) => "↵",
        ("Tab", true) => "⇥",
        ("PageUp", true) => "⇞",
        ("PageDown", true) => "⇟",
        ("Delete", false) => "Backspace",
        ("PageUp", false) => "PgUp",
        ("PageDown", false) => "PgDn",
        ("Escape", _) => "Esc",
        ("ArrowLeft", _) => "←",
        ("ArrowRight", _) => "→",
        ("ArrowUp", _) => "↑",
        ("ArrowDown", _) => "↓",
        (k, _) => k,
    };
    let mut out = String::new();
    if mac {
        for (on, glyph) in [(m.ctrl, "⌃"), (m.alt, "⌥"), (m.shift, "⇧"), (m.cmd, "⌘")] {
            if on {
                out.push_str(glyph);
            }
        }
        out.push_str(key);
    } else {
        for (on, name) in [(m.ctrl || m.cmd, "Ctrl"), (m.alt, "Alt"), (m.shift, "Shift"), (!key.is_empty(), key)] {
            if on {
                if !out.is_empty() {
                    out.push('+');
                }
                out.push_str(name);
            }
        }
    }
    out
}
