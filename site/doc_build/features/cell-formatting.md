# Cell Formatting

## Overview

`CellFormat` controls how raw cell values are displayed. It is purely visual —
the underlying data is not modified. Set it on `ColumnDef::format`:

```rust
col.format = Some(CellFormat::Number {
    decimal_places: 2,
    thousands_sep: Some(' '),
    decimal_sep: '.',
});
```

## Format types

### Number

```rust
CellFormat::Number {
    decimal_places: u8,           // digits after decimal
    thousands_sep: Option<char>,  // e.g. Some(' ') or Some(',')
    decimal_sep: char,            // e.g. '.' or ','
}
```

| Raw          | Format                                                          | Output       | Align |
| ------------ | --------------------------------------------------------------- | ------------ | ----- |
| `"1234.567"` | `decimal_places: 2, thousands_sep: Some(' '), decimal_sep: '.'` | `"1 234.57"` | Right |
| `"1234.5"`   | `decimal_places: 2, thousands_sep: Some('.'), decimal_sep: ','` | `"1.234,50"` | Right |

### Percent

```rust
CellFormat::Percent {
    decimal_places: u8,
}
```

Multiplies by 100 and appends `%`:

| Raw      | Output     |
| -------- | ---------- |
| `"0.75"` | `"75.00%"` |

### Currency

```rust
CellFormat::Currency {
    symbol: String,               // e.g. "$" or "€"
    decimal_places: u8,
    thousands_sep: Option<char>,
    symbol_after: bool,           // true → "42.50 €", false → "$42.50"
}
```

| Raw        | symbol\_after | Output         |
| ---------- | ------------- | -------------- |
| `"42.5"`   | `false`       | `"$42.50"`     |
| `"1234.5"` | `true`        | `"1 234.50 €"` |

### Boolean

```rust
CellFormat::Boolean {
    true_label: String,    // e.g. "✓" or "Yes"
    false_label: String,   // e.g. "✗" or "No"
}
```

Truthy values: `"true"`, `"1"`, `"yes"` (case-insensitive). Everything
else is falsy. Displayed centered.

### Image

```rust
CellFormat::Image {
    base_url: Option<String>,  // URL prefix (None = raw is full URL)
    border_radius: f64,        // corner radius in px
    padding: f64,              // padding inside the cell
}
```

Renders the cell value as an image. The final URL is `base_url + raw_value`.

### ImageText

```rust
CellFormat::ImageText {
    base_url: String,      // e.g. "https://flagcdn.com/w40/"
    suffix: String,        // e.g. ".png"
    image_size: f64,       // square image size in px
    border_radius: f64,
    gap: f64,              // gap between image and text
}
```

Raw value format: `"KEY Label"` — first token is the image key (lowercased
for the URL), rest is the display text.

Example: `"FR France"` → image URL `https://flagcdn.com/w40/fr.png` + text `"France"`

### Custom

```rust
CellFormat::Custom(Rc::new(|raw: &str| FormattedCell {
    text: raw.to_uppercase(),
    bold: true,
    color: Some([255, 0, 0, 255]),  // red
    ..Default::default()
}))
```

Full control over text, alignment, bold, and color via a callback.

## FormattedCell output

All formats produce a `FormattedCell`:

```rust
pub struct FormattedCell {
    pub text: String,
    pub align: Option<CellAlign>,  // Left, Center, Right
    pub bold: bool,
    pub color: Option<[u8; 4]>,    // RGBA override
}
```

## Fallback behavior

If the raw value cannot be parsed (e.g. `"abc"` for a Number format), the
raw value is displayed as-is with no formatting applied.
