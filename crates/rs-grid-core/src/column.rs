use std::rc::Rc;

// ── cell alignment ──────────────────────────────────────

/// Horizontal alignment override for a formatted cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CellAlign {
    #[default]
    Left,
    Center,
    Right,
}

// ── formatted cell output ───────────────────────────────

/// Output of `format_cell()` — display text plus optional
/// style overrides.
#[derive(Debug, Clone, Default)]
pub struct FormattedCell {
    pub text: String,
    pub align: Option<CellAlign>,
    pub bold: bool,
    /// RGBA colour override. `None` = use theme default.
    pub color: Option<[u8; 4]>,
}

// ── cell format enum ────────────────────────────────────

/// Per-column display format.
///
/// Purely visual — does not affect the underlying data.
/// When `ColumnDef::format` is `Some`, the scene builder
/// pipes the raw string through `format_cell()` before
/// rendering.
pub enum CellFormat {
    /// Numeric: 1234.5 → "1 234.50"
    Number {
        decimal_places: u8,
        /// e.g. `Some(' ')` or `Some(',')`.
        thousands_sep: Option<char>,
        /// e.g. `'.'` or `','`.
        decimal_sep: char,
    },
    /// Percentage: 0.75 → "75.00%"
    Percent { decimal_places: u8 },
    /// Currency: 42.5 → "$42.50"
    Currency {
        symbol: String,
        decimal_places: u8,
        thousands_sep: Option<char>,
        /// `true` → "42.50 €", `false` → "$42.50"
        symbol_after: bool,
    },
    /// Boolean: "true"/"1"/"yes" → true_label, else
    /// false_label.
    Boolean {
        true_label: String,
        false_label: String,
    },
    /// User-provided formatting callback.
    Custom(Rc<dyn Fn(&str) -> FormattedCell>),
    /// Image: cell value is a URL rendered as an image.
    Image {
        /// Optional base URL prefix. Final URL = base_url + raw.
        /// If `None`, raw value is the full URL.
        base_url: Option<String>,
        /// Corner radius in logical pixels (0 = sharp).
        border_radius: f64,
        /// Padding inside the cell around the image.
        padding: f64,
    },
    /// Image + text side by side (like AG Grid's flag
    /// cell renderer).
    ///
    /// Raw value = `"KEY Label"` — first token is the
    /// image key (lowercased for the URL), rest is the
    /// display text.
    ///
    /// Image URL = `{base_url}{key.lowercase()}{suffix}`.
    ImageText {
        /// URL prefix, e.g. `"https://flagcdn.com/w40/"`.
        base_url: String,
        /// URL suffix, e.g. `".png"`.
        suffix: String,
        /// Square image size in logical px.
        image_size: f64,
        /// Corner radius for the image.
        border_radius: f64,
        /// Gap between image and text.
        gap: f64,
    },
}

impl Clone for CellFormat {
    fn clone(&self) -> Self {
        match self {
            Self::Number {
                decimal_places,
                thousands_sep,
                decimal_sep,
            } => Self::Number {
                decimal_places: *decimal_places,
                thousands_sep: *thousands_sep,
                decimal_sep: *decimal_sep,
            },
            Self::Percent { decimal_places } => Self::Percent {
                decimal_places: *decimal_places,
            },
            Self::Currency {
                symbol,
                decimal_places,
                thousands_sep,
                symbol_after,
            } => Self::Currency {
                symbol: symbol.clone(),
                decimal_places: *decimal_places,
                thousands_sep: *thousands_sep,
                symbol_after: *symbol_after,
            },
            Self::Boolean {
                true_label,
                false_label,
            } => Self::Boolean {
                true_label: true_label.clone(),
                false_label: false_label.clone(),
            },
            Self::Custom(f) => Self::Custom(Rc::clone(f)),
            Self::Image {
                base_url,
                border_radius,
                padding,
            } => Self::Image {
                base_url: base_url.clone(),
                border_radius: *border_radius,
                padding: *padding,
            },
            Self::ImageText {
                base_url,
                suffix,
                image_size,
                border_radius,
                gap,
            } => Self::ImageText {
                base_url: base_url.clone(),
                suffix: suffix.clone(),
                image_size: *image_size,
                border_radius: *border_radius,
                gap: *gap,
            },
        }
    }
}

impl std::fmt::Debug for CellFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number {
                decimal_places,
                thousands_sep,
                decimal_sep,
            } => f
                .debug_struct("Number")
                .field("decimal_places", decimal_places)
                .field("thousands_sep", thousands_sep)
                .field("decimal_sep", decimal_sep)
                .finish(),
            Self::Percent { decimal_places } => f
                .debug_struct("Percent")
                .field("decimal_places", decimal_places)
                .finish(),
            Self::Currency {
                symbol,
                decimal_places,
                thousands_sep,
                symbol_after,
            } => f
                .debug_struct("Currency")
                .field("symbol", symbol)
                .field("decimal_places", decimal_places)
                .field("thousands_sep", thousands_sep)
                .field("symbol_after", symbol_after)
                .finish(),
            Self::Boolean {
                true_label,
                false_label,
            } => f
                .debug_struct("Boolean")
                .field("true_label", true_label)
                .field("false_label", false_label)
                .finish(),
            Self::Custom(_) => f.debug_tuple("Custom").field(&"..").finish(),
            Self::Image {
                base_url,
                border_radius,
                padding,
            } => f
                .debug_struct("Image")
                .field("base_url", base_url)
                .field("border_radius", border_radius)
                .field("padding", padding)
                .finish(),
            Self::ImageText {
                base_url,
                suffix,
                image_size,
                border_radius,
                gap,
            } => f
                .debug_struct("ImageText")
                .field("base_url", base_url)
                .field("suffix", suffix)
                .field("image_size", image_size)
                .field("border_radius", border_radius)
                .field("gap", gap)
                .finish(),
        }
    }
}

// ── format_cell ─────────────────────────────────────────

/// Format a raw cell string according to `fmt`.
///
/// Returns the raw value unchanged (as fallback) if
/// parsing fails.
pub fn format_cell(raw: &str, fmt: &CellFormat) -> FormattedCell {
    match fmt {
        CellFormat::Number {
            decimal_places,
            thousands_sep,
            decimal_sep,
        } => match raw.parse::<f64>() {
            Ok(v) => {
                let text = format_number(
                    v,
                    *decimal_places,
                    *thousands_sep,
                    *decimal_sep,
                );
                FormattedCell {
                    text,
                    align: Some(CellAlign::Right),
                    ..Default::default()
                }
            }
            Err(_) => FormattedCell {
                text: raw.to_owned(),
                ..Default::default()
            },
        },
        CellFormat::Percent { decimal_places } => match raw.parse::<f64>() {
            Ok(v) => {
                let pct = v * 100.0;
                let text = format!(
                    "{:.prec$}%",
                    pct,
                    prec = *decimal_places as usize,
                );
                FormattedCell {
                    text,
                    align: Some(CellAlign::Right),
                    ..Default::default()
                }
            }
            Err(_) => FormattedCell {
                text: raw.to_owned(),
                ..Default::default()
            },
        },
        CellFormat::Currency {
            symbol,
            decimal_places,
            thousands_sep,
            symbol_after,
        } => match raw.parse::<f64>() {
            Ok(v) => {
                let num =
                    format_number(v, *decimal_places, *thousands_sep, '.');
                let text = if *symbol_after {
                    format!("{num} {symbol}")
                } else {
                    format!("{symbol}{num}")
                };
                FormattedCell {
                    text,
                    align: Some(CellAlign::Right),
                    ..Default::default()
                }
            }
            Err(_) => FormattedCell {
                text: raw.to_owned(),
                ..Default::default()
            },
        },
        CellFormat::Boolean {
            true_label,
            false_label,
        } => {
            let is_true = matches!(
                raw.trim().to_lowercase().as_str(),
                "true" | "1" | "yes"
            );
            FormattedCell {
                text: if is_true {
                    true_label.clone()
                } else {
                    false_label.clone()
                },
                align: Some(CellAlign::Center),
                ..Default::default()
            }
        }
        CellFormat::Custom(cb) => cb(raw),
        CellFormat::Image { .. } => FormattedCell {
            text: raw.to_owned(),
            ..Default::default()
        },
        CellFormat::ImageText { .. } => {
            // Extract text after the first space (the
            // image key). The scene builder handles the
            // image primitive separately.
            let text = raw
                .find(' ')
                .map(|i| raw[i + 1..].to_owned())
                .unwrap_or_else(|| raw.to_owned());
            FormattedCell {
                text,
                ..Default::default()
            }
        }
    }
}


impl CellFormat {
    /// Returns `true` if this format renders a full-cell
    /// image.
    pub fn is_image(&self) -> bool {
        matches!(self, CellFormat::Image { .. })
    }

    /// Returns `true` if this format renders an image +
    /// text side by side.
    pub fn is_image_text(&self) -> bool {
        matches!(self, CellFormat::ImageText { .. })
    }
}

/// Format a f64 with fixed decimal places, a configurable
/// decimal separator, and optional thousands grouping.
fn format_number(
    value: f64,
    decimal_places: u8,
    thousands_sep: Option<char>,
    decimal_sep: char,
) -> String {
    let prec = decimal_places as usize;
    let raw = format!("{value:.prec$}");

    // Split on '.' (Rust always uses '.' for f64 display).
    let (int_part, dec_part) = match raw.split_once('.') {
        Some((i, d)) => (i, Some(d)),
        None => (raw.as_str(), None),
    };

    // Insert thousands separator into the integer part.
    let int_formatted = if let Some(sep) = thousands_sep {
        let negative = int_part.starts_with('-');
        let digits = if negative { &int_part[1..] } else { int_part };
        let mut result = String::with_capacity(digits.len() + digits.len() / 3);
        for (i, ch) in digits.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(sep);
            }
            result.push(ch);
        }
        let grouped: String = result.chars().rev().collect();
        if negative {
            format!("-{grouped}")
        } else {
            grouped
        }
    } else {
        int_part.to_owned()
    };

    match dec_part {
        Some(d) => {
            format!("{int_formatted}{decimal_sep}{d}")
        }
        None => int_formatted,
    }
}

// ── column definition ───────────────────────────────────

/// Definition of a single grid column.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    /// Unique key used to look up cell values in a row.
    pub key: String,
    /// Display label shown in the column header.
    pub label: String,
    /// Width in logical (CSS) pixels.
    pub width: f64,
    /// Optional display format for cell values.
    pub format: Option<CellFormat>,
}

impl ColumnDef {
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        width: f64,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            width,
            format: None,
        }
    }
}

/// Precomputed left-edge offsets for every column, plus total content width.
#[derive(Debug, Clone, Default)]
pub struct ColumnOffsets {
    /// `offsets[i]` is the x position of the left edge of column `i`.
    pub offsets: Vec<f64>,
    pub total_width: f64,
}

impl ColumnOffsets {
    pub fn compute(columns: &[ColumnDef]) -> Self {
        let mut offsets = Vec::with_capacity(columns.len());
        let mut x = 0.0_f64;
        for col in columns {
            offsets.push(x);
            x += col.width;
        }
        Self {
            offsets,
            total_width: x,
        }
    }

    /// Return the column index whose bounds contain `x`, or `None`.
    pub fn hit_column(&self, x: f64, columns: &[ColumnDef]) -> Option<usize> {
        for (i, &offset) in self.offsets.iter().enumerate() {
            let right = offset + columns[i].width;
            if x >= offset && x < right {
                return Some(i);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cols() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 150.0),
            ColumnDef::new("c", "C", 50.0),
        ]
    }

    #[test]
    fn compute_offsets() {
        let cols = cols();
        let o = ColumnOffsets::compute(&cols);
        assert_eq!(o.offsets, vec![0.0, 100.0, 250.0]);
        assert_eq!(o.total_width, 300.0);
    }

    #[test]
    fn compute_empty() {
        let o = ColumnOffsets::compute(&[]);
        assert!(o.offsets.is_empty());
        assert_eq!(o.total_width, 0.0);
    }

    #[test]
    fn hit_column_first() {
        let cols = cols();
        let o = ColumnOffsets::compute(&cols);
        assert_eq!(o.hit_column(0.0, &cols), Some(0));
        assert_eq!(o.hit_column(99.9, &cols), Some(0));
    }

    #[test]
    fn hit_column_second() {
        let cols = cols();
        let o = ColumnOffsets::compute(&cols);
        assert_eq!(o.hit_column(100.0, &cols), Some(1));
        assert_eq!(o.hit_column(249.9, &cols), Some(1));
    }

    #[test]
    fn hit_column_last() {
        let cols = cols();
        let o = ColumnOffsets::compute(&cols);
        assert_eq!(o.hit_column(250.0, &cols), Some(2));
        assert_eq!(o.hit_column(299.9, &cols), Some(2));
    }

    #[test]
    fn hit_column_out_of_range() {
        let cols = cols();
        let o = ColumnOffsets::compute(&cols);
        assert_eq!(o.hit_column(300.0, &cols), None);
        assert_eq!(o.hit_column(-1.0, &cols), None);
    }

    // ── format_cell tests ──────────────────────────────

    #[test]
    fn format_number_basic() {
        let fmt = CellFormat::Number {
            decimal_places: 2,
            thousands_sep: Some(' '),
            decimal_sep: '.',
        };
        let fc = format_cell("1234.567", &fmt);
        assert_eq!(fc.text, "1 234.57");
        assert_eq!(fc.align, Some(CellAlign::Right));
    }

    #[test]
    fn format_number_no_thousands_sep() {
        let fmt = CellFormat::Number {
            decimal_places: 1,
            thousands_sep: None,
            decimal_sep: '.',
        };
        let fc = format_cell("9876.54", &fmt);
        assert_eq!(fc.text, "9876.5");
    }

    #[test]
    fn format_number_comma_decimal_sep() {
        let fmt = CellFormat::Number {
            decimal_places: 2,
            thousands_sep: Some('.'),
            decimal_sep: ',',
        };
        let fc = format_cell("1234.5", &fmt);
        assert_eq!(fc.text, "1.234,50");
    }

    #[test]
    fn format_number_invalid_fallback() {
        let fmt = CellFormat::Number {
            decimal_places: 2,
            thousands_sep: Some(' '),
            decimal_sep: '.',
        };
        let fc = format_cell("abc", &fmt);
        assert_eq!(fc.text, "abc");
    }

    #[test]
    fn format_number_zero_decimals() {
        let fmt = CellFormat::Number {
            decimal_places: 0,
            thousands_sep: Some(' '),
            decimal_sep: '.',
        };
        let fc = format_cell("1234.9", &fmt);
        assert_eq!(fc.text, "1 235");
    }

    #[test]
    fn format_percent() {
        let fmt = CellFormat::Percent { decimal_places: 2 };
        let fc = format_cell("0.75", &fmt);
        assert_eq!(fc.text, "75.00%");
        assert_eq!(fc.align, Some(CellAlign::Right));
    }

    #[test]
    fn format_percent_invalid() {
        let fmt = CellFormat::Percent { decimal_places: 1 };
        let fc = format_cell("N/A", &fmt);
        assert_eq!(fc.text, "N/A");
    }

    #[test]
    fn format_currency_before() {
        let fmt = CellFormat::Currency {
            symbol: "$".into(),
            decimal_places: 2,
            thousands_sep: Some(','),
            symbol_after: false,
        };
        let fc = format_cell("42.5", &fmt);
        assert_eq!(fc.text, "$42.50");
        assert_eq!(fc.align, Some(CellAlign::Right));
    }

    #[test]
    fn format_currency_after() {
        let fmt = CellFormat::Currency {
            symbol: "\u{20ac}".into(),
            decimal_places: 2,
            thousands_sep: Some(' '),
            symbol_after: true,
        };
        let fc = format_cell("1234.5", &fmt);
        assert_eq!(fc.text, "1 234.50 \u{20ac}");
    }

    #[test]
    fn format_boolean_true() {
        let fmt = CellFormat::Boolean {
            true_label: "\u{2713}".into(),
            false_label: "\u{2717}".into(),
        };
        assert_eq!(format_cell("true", &fmt).text, "\u{2713}");
        assert_eq!(format_cell("1", &fmt).text, "\u{2713}");
        assert_eq!(format_cell("yes", &fmt).text, "\u{2713}");
        assert_eq!(format_cell("TRUE", &fmt).text, "\u{2713}");
    }

    #[test]
    fn format_boolean_false() {
        let fmt = CellFormat::Boolean {
            true_label: "\u{2713}".into(),
            false_label: "\u{2717}".into(),
        };
        assert_eq!(format_cell("false", &fmt).text, "\u{2717}");
        assert_eq!(format_cell("0", &fmt).text, "\u{2717}");
        assert_eq!(format_cell("no", &fmt).text, "\u{2717}");
        assert_eq!(format_cell("", &fmt).text, "\u{2717}");
    }

    #[test]
    fn format_boolean_alignment() {
        let fmt = CellFormat::Boolean {
            true_label: "Y".into(),
            false_label: "N".into(),
        };
        let fc = format_cell("true", &fmt);
        assert_eq!(fc.align, Some(CellAlign::Center));
    }

    #[test]
    fn format_custom() {
        let fmt = CellFormat::Custom(Rc::new(|raw: &str| FormattedCell {
            text: raw.to_uppercase(),
            ..Default::default()
        }));
        let fc = format_cell("hello", &fmt);
        assert_eq!(fc.text, "HELLO");
    }

    #[test]
    fn columndef_format_default_none() {
        let col = ColumnDef::new("a", "A", 100.0);
        assert!(col.format.is_none());
    }

    #[test]
    fn format_number_negative() {
        let fmt = CellFormat::Number {
            decimal_places: 2,
            thousands_sep: Some(','),
            decimal_sep: '.',
        };
        let fc = format_cell("-1234.5", &fmt);
        assert_eq!(fc.text, "-1,234.50");
    }

    #[test]
    fn format_image_returns_raw() {
        let fmt = CellFormat::Image {
            base_url: Some("https://example.com/".into()),
            border_radius: 4.0,
            padding: 2.0,
        };
        let fc = format_cell("photo.png", &fmt);
        assert_eq!(fc.text, "photo.png");
    }

    #[test]
    fn cell_format_is_image() {
        let img = CellFormat::Image {
            base_url: None,
            border_radius: 0.0,
            padding: 0.0,
        };
        assert!(img.is_image());
        let num = CellFormat::Number {
            decimal_places: 2,
            thousands_sep: None,
            decimal_sep: '.',
        };
        assert!(!num.is_image());
    }

    #[test]
    fn format_image_text_extracts_label() {
        let fmt = CellFormat::ImageText {
            base_url: "https://cdn.example.com/".into(),
            suffix: ".png".into(),
            image_size: 20.0,
            border_radius: 2.0,
            gap: 6.0,
        };
        let fc = format_cell("FR France", &fmt);
        assert_eq!(fc.text, "France");
    }

    #[test]
    fn format_image_text_no_space_fallback() {
        let fmt = CellFormat::ImageText {
            base_url: String::new(),
            suffix: String::new(),
            image_size: 16.0,
            border_radius: 0.0,
            gap: 4.0,
        };
        let fc = format_cell("US", &fmt);
        assert_eq!(fc.text, "US");
    }

    #[test]
    fn image_format_clone() {
        let fmt = CellFormat::Image {
            base_url: Some("https://cdn.example.com/".into()),
            border_radius: 8.0,
            padding: 3.0,
        };
        let cloned = fmt.clone();
        assert!(matches!(
            cloned,
            CellFormat::Image {
                border_radius,
                padding,
                ..
            } if border_radius == 8.0 && padding == 3.0
        ));
    }
}
