//! Locale strings for rs-grid UI chrome (context menu,
//! search bar, etc.).
//!
//! Each supported language lives in its own `.toml` file as
//! a flat `key = "value"` list.  Files are embedded at
//! compile time via `include_str!` — no runtime I/O.
//!
//! To add a new language:
//! 1. Copy `en.toml` → `xx.toml` and translate the values
//! 2. Add a `pub fn xx()` constructor below
//! 3. Add a `"xx"` branch in `from_language_tag`

/// All translatable UI strings used by rs-grid.
///
/// Each field corresponds to a label, shortcut hint, or
/// placeholder rendered by the grid chrome.  The struct
/// implements `Default` (English).
///
/// Four built-in locales are provided via [`Locale::en`],
/// [`Locale::fr`], [`Locale::de`], [`Locale::es`].
/// Users can also construct a custom `Locale { .. }` for
/// any other language.
#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    // ── Context menu — clipboard ────────────────────────
    pub cut: String,
    pub copy: String,
    pub copy_with_headers: String,
    pub paste: String,

    // ── Context menu — column actions ───────────────────
    pub pin_column: String,
    pub unpin_column: String,
    pub sort_ascending: String,
    pub sort_descending: String,
    pub clear_sort: String,
    pub autosize_this_column: String,
    pub autosize_all_columns: String,

    // ── Keyboard shortcut hints ─────────────────────────
    pub shortcut_cut: String,
    pub shortcut_copy: String,
    pub shortcut_paste: String,

    // ── Search ──────────────────────────────────────────
    pub search_placeholder: String,
}

impl Default for Locale {
    fn default() -> Self {
        Self::en()
    }
}

impl Locale {
    /// English (default).
    pub fn en() -> Self {
        parse_toml(include_str!("en.toml"))
    }

    /// French.
    pub fn fr() -> Self {
        parse_toml(include_str!("fr.toml"))
    }

    /// German.
    pub fn de() -> Self {
        parse_toml(include_str!("de.toml"))
    }

    /// Spanish.
    pub fn es() -> Self {
        parse_toml(include_str!("es.toml"))
    }

    /// Italian.
    pub fn it() -> Self {
        parse_toml(include_str!("it.toml"))
    }

    /// Portuguese.
    pub fn pt() -> Self {
        parse_toml(include_str!("pt.toml"))
    }

    /// Dutch.
    pub fn nl() -> Self {
        parse_toml(include_str!("nl.toml"))
    }

    /// Polish.
    pub fn pl() -> Self {
        parse_toml(include_str!("pl.toml"))
    }

    /// Turkish.
    pub fn tr() -> Self {
        parse_toml(include_str!("tr.toml"))
    }

    /// Russian.
    pub fn ru() -> Self {
        parse_toml(include_str!("ru.toml"))
    }

    /// Ukrainian.
    pub fn uk() -> Self {
        parse_toml(include_str!("uk.toml"))
    }

    /// Arabic.
    pub fn ar() -> Self {
        parse_toml(include_str!("ar.toml"))
    }

    /// Japanese.
    pub fn ja() -> Self {
        parse_toml(include_str!("ja.toml"))
    }

    /// Chinese (Simplified).
    pub fn zh() -> Self {
        parse_toml(include_str!("zh.toml"))
    }

    /// Korean.
    pub fn ko() -> Self {
        parse_toml(include_str!("ko.toml"))
    }

    /// Detect the browser language (`navigator.language`)
    /// and return the best matching built-in locale.
    ///
    /// Matches on the primary language subtag (the part
    /// before the first `-`), so `"fr-FR"`, `"fr-CA"`, and
    /// `"fr"` all resolve to [`Locale::fr`].  Falls back to
    /// English for unsupported languages.
    pub fn from_browser() -> Self {
        let lang = web_sys::window()
            .and_then(|w| w.navigator().language())
            .unwrap_or_default();
        Self::from_language_tag(&lang)
    }

    /// Return the built-in locale matching a BCP 47
    /// language tag (e.g. `"fr-FR"`, `"de"`, `"es-MX"`).
    ///
    /// Only the primary subtag is considered.  Falls back
    /// to English for unknown tags.
    pub fn from_language_tag(tag: &str) -> Self {
        let primary = tag.split('-').next().unwrap_or("en");
        match primary {
            "fr" => Self::fr(),
            "de" => Self::de(),
            "es" => Self::es(),
            "it" => Self::it(),
            "pt" => Self::pt(),
            "nl" => Self::nl(),
            "pl" => Self::pl(),
            "tr" => Self::tr(),
            "ru" => Self::ru(),
            "uk" => Self::uk(),
            "ar" => Self::ar(),
            "ja" => Self::ja(),
            "zh" => Self::zh(),
            "ko" => Self::ko(),
            _ => Self::en(),
        }
    }
}

// ── Minimal TOML parser ─────────────────────────────────
//
// Handles only flat `key = "value"` pairs.  Blank lines
// and `#` comments are skipped.  No dependency needed.

fn parse_toml(src: &str) -> Locale {
    let en = Locale {
        cut: String::new(),
        copy: String::new(),
        copy_with_headers: String::new(),
        paste: String::new(),
        pin_column: String::new(),
        unpin_column: String::new(),
        sort_ascending: String::new(),
        sort_descending: String::new(),
        clear_sort: String::new(),
        autosize_this_column: String::new(),
        autosize_all_columns: String::new(),
        shortcut_cut: String::new(),
        shortcut_copy: String::new(),
        shortcut_paste: String::new(),
        search_placeholder: String::new(),
    };
    let mut loc = en;

    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, rest)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let val = parse_toml_string(rest.trim());

        match key {
            "cut" => loc.cut = val,
            "copy" => loc.copy = val,
            "copy_with_headers" => {
                loc.copy_with_headers = val;
            }
            "paste" => loc.paste = val,
            "pin_column" => loc.pin_column = val,
            "unpin_column" => loc.unpin_column = val,
            "sort_ascending" => loc.sort_ascending = val,
            "sort_descending" => {
                loc.sort_descending = val;
            }
            "clear_sort" => loc.clear_sort = val,
            "autosize_this_column" => {
                loc.autosize_this_column = val;
            }
            "autosize_all_columns" => {
                loc.autosize_all_columns = val;
            }
            "shortcut_cut" => loc.shortcut_cut = val,
            "shortcut_copy" => loc.shortcut_copy = val,
            "shortcut_paste" => loc.shortcut_paste = val,
            "search_placeholder" => {
                loc.search_placeholder = val;
            }
            _ => {} // unknown keys silently ignored
        }
    }
    loc
}

/// Strip surrounding `"` quotes from a TOML basic string.
fn parse_toml_string(s: &str) -> String {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_en_toml() {
        let loc = parse_toml(include_str!("en.toml"));
        assert_eq!(loc.cut, "Cut");
        assert_eq!(loc.copy, "Copy");
        assert_eq!(loc.copy_with_headers, "Copy with headers");
        assert_eq!(loc.paste, "Paste");
        assert_eq!(loc.shortcut_cut, "Ctrl+X");
        assert_eq!(loc.search_placeholder, "Find\u{2026}");
    }

    #[test]
    fn parse_fr_toml() {
        let loc = parse_toml(include_str!("fr.toml"));
        assert_eq!(loc.cut, "Couper");
        assert_eq!(loc.search_placeholder, "Rechercher\u{2026}");
    }

    #[test]
    fn parse_all_keys_populated() {
        for (name, src) in [
            ("en", include_str!("en.toml")),
            ("fr", include_str!("fr.toml")),
            ("de", include_str!("de.toml")),
            ("es", include_str!("es.toml")),
            ("it", include_str!("it.toml")),
            ("pt", include_str!("pt.toml")),
            ("nl", include_str!("nl.toml")),
            ("pl", include_str!("pl.toml")),
            ("tr", include_str!("tr.toml")),
            ("ru", include_str!("ru.toml")),
            ("uk", include_str!("uk.toml")),
            ("ar", include_str!("ar.toml")),
            ("ja", include_str!("ja.toml")),
            ("zh", include_str!("zh.toml")),
            ("ko", include_str!("ko.toml")),
        ] {
            let loc = parse_toml(src);
            assert!(!loc.cut.is_empty(), "{name}: cut is empty");
            assert!(!loc.copy.is_empty(), "{name}: copy is empty");
            assert!(
                !loc.search_placeholder.is_empty(),
                "{name}: search_placeholder is empty"
            );
        }
    }

    #[test]
    fn from_language_tag_matches() {
        assert_eq!(Locale::from_language_tag("fr-FR").cut, "Couper");
        assert_eq!(Locale::from_language_tag("de").cut, "Ausschneiden");
        assert_eq!(Locale::from_language_tag("ja").cut, "切り取り");
        // Unknown tag falls back to English
        assert_eq!(Locale::from_language_tag("xx").cut, "Cut");
    }
}
