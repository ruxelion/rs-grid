//! Locale strings for rs-grid UI chrome (context menu,
//! search bar, etc.).
//!
//! The `Locale` struct centralises every user-visible string
//! that the grid emits outside of cell data.  Four built-in
//! locales are provided (`en`, `fr`, `de`, `es`); users can
//! construct a custom `Locale { .. }` for any other language.

/// All translatable UI strings used by rs-grid.
///
/// Each field corresponds to a label, shortcut hint, or
/// placeholder rendered by the grid chrome.  The struct
/// implements `Default` (English).
#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    // ── Context menu — clipboard ────────────────────────────
    pub cut: String,
    pub copy: String,
    pub copy_with_headers: String,
    pub paste: String,

    // ── Context menu — column actions ───────────────────────
    pub pin_column: String,
    pub unpin_column: String,
    pub sort_ascending: String,
    pub sort_descending: String,
    pub clear_sort: String,
    pub autosize_this_column: String,
    pub autosize_all_columns: String,

    // ── Keyboard shortcut hints ─────────────────────────────
    pub shortcut_cut: String,
    pub shortcut_copy: String,
    pub shortcut_paste: String,

    // ── Search ──────────────────────────────────────────────
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
        Self {
            cut: "Cut".into(),
            copy: "Copy".into(),
            copy_with_headers: "Copy with headers".into(),
            paste: "Paste".into(),
            pin_column: "Pin Column".into(),
            unpin_column: "Unpin Column".into(),
            sort_ascending: "Sort Ascending".into(),
            sort_descending: "Sort Descending".into(),
            clear_sort: "Clear Sort".into(),
            autosize_this_column: "Autosize This Column".into(),
            autosize_all_columns: "Autosize All Columns".into(),
            shortcut_cut: "Ctrl+X".into(),
            shortcut_copy: "Ctrl+C".into(),
            shortcut_paste: "Ctrl+V".into(),
            search_placeholder: "Find\u{2026}".into(),
        }
    }

    /// French.
    pub fn fr() -> Self {
        Self {
            cut: "Couper".into(),
            copy: "Copier".into(),
            copy_with_headers: "Copier avec en-t\u{ea}tes".into(),
            paste: "Coller".into(),
            pin_column: "\u{c9}pingler la colonne".into(),
            unpin_column: "D\u{e9}s\u{e9}pingler la colonne".into(),
            sort_ascending: "Tri croissant".into(),
            sort_descending: "Tri d\u{e9}croissant".into(),
            clear_sort: "Annuler le tri".into(),
            autosize_this_column: "Ajuster cette colonne".into(),
            autosize_all_columns: "Ajuster toutes les colonnes".into(),
            shortcut_cut: "Ctrl+X".into(),
            shortcut_copy: "Ctrl+C".into(),
            shortcut_paste: "Ctrl+V".into(),
            search_placeholder: "Rechercher\u{2026}".into(),
        }
    }

    /// German.
    pub fn de() -> Self {
        Self {
            cut: "Ausschneiden".into(),
            copy: "Kopieren".into(),
            copy_with_headers: "Mit \u{dc}berschriften kopieren".into(),
            paste: "Einf\u{fc}gen".into(),
            pin_column: "Spalte fixieren".into(),
            unpin_column: "Spalte l\u{f6}sen".into(),
            sort_ascending: "Aufsteigend sortieren".into(),
            sort_descending: "Absteigend sortieren".into(),
            clear_sort: "Sortierung aufheben".into(),
            autosize_this_column: "Diese Spalte anpassen".into(),
            autosize_all_columns: "Alle Spalten anpassen".into(),
            shortcut_cut: "Strg+X".into(),
            shortcut_copy: "Strg+C".into(),
            shortcut_paste: "Strg+V".into(),
            search_placeholder: "Suchen\u{2026}".into(),
        }
    }

    /// Detect the browser language (`navigator.language`) and
    /// return the best matching built-in locale.
    ///
    /// Matches on the primary language subtag (the part before
    /// the first `-`), so `"fr-FR"`, `"fr-CA"`, and `"fr"` all
    /// resolve to `Locale::fr()`.  Falls back to English for
    /// unsupported languages.
    pub fn from_browser() -> Self {
        let lang = web_sys::window()
            .and_then(|w| w.navigator().language())
            .unwrap_or_default();
        Self::from_language_tag(&lang)
    }

    /// Return the built-in locale matching a BCP 47 language
    /// tag (e.g. `"fr-FR"`, `"de"`, `"es-MX"`).
    ///
    /// Only the primary subtag is considered.  Falls back to
    /// English for unknown tags.
    pub fn from_language_tag(tag: &str) -> Self {
        let primary = tag.split('-').next().unwrap_or("en");
        match primary {
            "fr" => Self::fr(),
            "de" => Self::de(),
            "es" => Self::es(),
            _ => Self::en(),
        }
    }

    /// Spanish.
    pub fn es() -> Self {
        Self {
            cut: "Cortar".into(),
            copy: "Copiar".into(),
            copy_with_headers: "Copiar con encabezados".into(),
            paste: "Pegar".into(),
            pin_column: "Fijar columna".into(),
            unpin_column: "Desfijar columna".into(),
            sort_ascending: "Orden ascendente".into(),
            sort_descending: "Orden descendente".into(),
            clear_sort: "Quitar orden".into(),
            autosize_this_column: "Ajustar esta columna".into(),
            autosize_all_columns: "Ajustar todas las columnas".into(),
            shortcut_cut: "Ctrl+X".into(),
            shortcut_copy: "Ctrl+C".into(),
            shortcut_paste: "Ctrl+V".into(),
            search_placeholder: "Buscar\u{2026}".into(),
        }
    }
}
