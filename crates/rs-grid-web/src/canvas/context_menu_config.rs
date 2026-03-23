/// Identifies a built-in action that the grid knows how to
/// execute from the context menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinAction {
    /// Cut the selection to clipboard.
    Cut,
    /// Copy the selection to clipboard.
    Copy,
    /// Copy with column header labels.
    CopyWithHeaders,
    /// Paste from clipboard.
    Paste,
    /// Pin (freeze) the clicked column.
    PinColumn,
    /// Unpin the clicked column.
    UnpinColumn,
}

/// A single item in the context menu.
#[derive(Debug, Clone)]
pub enum ContextMenuItem {
    /// A built-in action with optional label/icon/shortcut
    /// overrides.
    Builtin {
        /// Which built-in action to trigger.
        action: BuiltinAction,
        /// Override the default label. `None` = built-in default.
        label: Option<String>,
        /// Override the default SVG icon HTML.
        /// `None` = built-in default.
        icon: Option<String>,
        /// Override the shortcut hint text.
        /// `None` = built-in default.
        shortcut: Option<String>,
    },
    /// A visual separator line.
    Separator,
}

impl ContextMenuItem {
    /// Create a Cut item with default label/icon/shortcut.
    pub fn cut() -> Self {
        Self::Builtin {
            action: BuiltinAction::Cut,
            label: None,
            icon: None,
            shortcut: None,
        }
    }

    /// Create a Copy item with default label/icon/shortcut.
    pub fn copy() -> Self {
        Self::Builtin {
            action: BuiltinAction::Copy,
            label: None,
            icon: None,
            shortcut: None,
        }
    }

    /// Create a Copy With Headers item.
    pub fn copy_with_headers() -> Self {
        Self::Builtin {
            action: BuiltinAction::CopyWithHeaders,
            label: None,
            icon: None,
            shortcut: None,
        }
    }

    /// Create a Paste item with default label/icon/shortcut.
    pub fn paste() -> Self {
        Self::Builtin {
            action: BuiltinAction::Paste,
            label: None,
            icon: None,
            shortcut: None,
        }
    }

    /// Create a Pin Column item.
    pub fn pin_column() -> Self {
        Self::Builtin {
            action: BuiltinAction::PinColumn,
            label: None,
            icon: None,
            shortcut: None,
        }
    }

    /// Create an Unpin Column item.
    pub fn unpin_column() -> Self {
        Self::Builtin {
            action: BuiltinAction::UnpinColumn,
            label: None,
            icon: None,
            shortcut: None,
        }
    }

    /// Create a visual separator line.
    pub fn separator() -> Self {
        Self::Separator
    }

    /// Override the label for a `Builtin` item.
    /// No-op on `Separator`.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        if let Self::Builtin {
            label: ref mut l, ..
        } = self
        {
            *l = Some(label.into());
        }
        self
    }

    /// Override the SVG icon for a `Builtin` item.
    /// No-op on `Separator`.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        if let Self::Builtin {
            icon: ref mut i, ..
        } = self
        {
            *i = Some(icon.into());
        }
        self
    }

    /// Override the shortcut hint for a `Builtin` item.
    /// No-op on `Separator`.
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        if let Self::Builtin {
            shortcut: ref mut s,
            ..
        } = self
        {
            *s = Some(shortcut.into());
        }
        self
    }
}

/// Configuration for context menus.
///
/// When a field is `None`, the grid uses its default built-in
/// menu items.
#[derive(Default)]
pub struct ContextMenuConfig {
    /// Items shown on right-click in the cell/row area.
    /// Default: Cut, Copy, Copy with Headers, ---, Paste.
    pub cell_items: Option<Vec<ContextMenuItem>>,
    /// Items shown on right-click on a column header.
    /// Default: Pin / Unpin Column.
    pub col_header_items: Option<Vec<ContextMenuItem>>,
}
