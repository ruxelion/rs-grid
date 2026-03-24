# CSS Variables Reference

## How it works

rs-grid reads CSS custom properties from `:root` at mount time via
`theme_from_css_vars()`. Each variable maps to a field on the `Theme` struct.
If a variable is absent or unparseable, the light theme default is used.

## Supported color formats

- `#rrggbb` / `#rrggbbaa`
- `#rgb` / `#rgba`
- `rgb(r, g, b)`
- `rgba(r, g, b, a)` — `a` is a 0–1 float

## Color variables

| CSS Variable                 | Theme field        | Light default            | Dark default             | Description                |
| ---------------------------- | ------------------ | ------------------------ | ------------------------ | -------------------------- |
| `--rs-grid-bg`               | `bg`               | `#ffffff`                | `rgb(26,27,38)`          | Cell background            |
| `--rs-grid-header-bg`        | `header_bg`        | `rgb(248,249,250)`       | `rgb(36,40,59)`          | Header background          |
| `--rs-grid-header-text`      | `header_text`      | `rgb(24,29,31)`          | `rgb(169,177,214)`       | Header text color          |
| `--rs-grid-cell-text`        | `cell_text`        | `rgb(24,29,31)`          | `rgb(192,202,245)`       | Cell text color            |
| `--rs-grid-grid-line`        | `grid_line`        | `rgb(224,224,224)`       | `rgb(42,47,69)`          | Cell border color          |
| `--rs-grid-header-border`    | `header_border`    | `rgb(186,191,199)`       | `rgb(61,68,102)`         | Header bottom border       |
| `--rs-grid-selection-fill`   | `selection_fill`   | `rgba(31,119,220,0.18)`  | `rgba(122,162,255,0.2)`  | Selection rectangle fill   |
| `--rs-grid-selection-border` | `selection_border` | `rgba(31,119,220,0.82)`  | `rgba(122,162,255,0.8)`  | Selection rectangle border |
| `--rs-grid-scrollbar-track`  | `scrollbar_track`  | `rgb(241,241,241)`       | `rgb(31,35,53)`          | Scrollbar track background |
| `--rs-grid-scrollbar-thumb`  | `scrollbar_thumb`  | `rgba(100,100,110,0.63)` | `rgba(169,177,214,0.4)`  | Scrollbar thumb color      |
| `--rs-grid-row-alt-bg`       | `row_alt_bg`       | `rgb(252,252,253)`       | `rgb(30,32,48)`          | Alternating row background |
| `--rs-grid-row-hover-bg`     | `row_hover_bg`     | `rgba(0,0,0,0.04)`       | `rgba(255,255,255,0.04)` | Row hover overlay          |
| `--rs-grid-search-highlight` | `search_highlight` | `rgba(255,213,0,0.31)`   | `rgba(255,213,0,0.31)`   | Search match highlight     |
| `--rs-grid-search-current`   | `search_current`   | `rgba(255,165,0,0.55)`   | `rgba(255,165,0,0.55)`   | Current search match       |
| `--rs-grid-skeleton-fg`      | `skeleton_fg`      | `rgba(200,200,200,0.39)` | `rgba(60,65,90,0.39)`    | Skeleton loading bar color |

## Size & typography variables

| CSS Variable                 | Theme field        | Default | Type | Description                          |
| ---------------------------- | ------------------ | ------- | ---- | ------------------------------------ |
| `--rs-grid-font-size`        | `font_size`        | `14px`  | px   | Cell text font size                  |
| `--rs-grid-header-font-size` | `header_font_size` | `12px`  | px   | Header text font size                |
| `--rs-grid-header-font-bold` | `header_font_bold` | `true`  | bool | Header bold (`true`/`false`/`1`/`0`) |
| `--rs-grid-cell-padding`     | `cell_padding`     | `10px`  | px   | Horizontal cell padding              |
| `--rs-grid-scrollbar-width`  | `scrollbar_width`  | `14px`  | px   | Scrollbar track + thumb width        |
| `--rs-grid-scrollbar-radius` | `scrollbar_radius` | `4px`   | px   | Scrollbar thumb corner radius        |

## Example: custom dark theme

```css
:root {
  --rs-grid-bg: #1e1e2e;
  --rs-grid-header-bg: #313244;
  --rs-grid-header-text: #cdd6f4;
  --rs-grid-cell-text: #cdd6f4;
  --rs-grid-grid-line: #45475a;
  --rs-grid-header-border: #585b70;
  --rs-grid-selection-fill: rgba(137, 180, 250, 0.2);
  --rs-grid-selection-border: rgba(137, 180, 250, 0.8);
  --rs-grid-scrollbar-track: #1e1e2e;
  --rs-grid-scrollbar-thumb: rgba(166, 173, 200, 0.4);
  --rs-grid-row-alt-bg: #181825;
  --rs-grid-row-hover-bg: rgba(255, 255, 255, 0.04);
  --rs-grid-font-size: 13px;
  --rs-grid-header-font-size: 12px;
}
```

:::note
CSS variables are read **once** at mount time. Changing them dynamically
requires re-mounting the grid or passing an updated `Theme` struct
programmatically via a Leptos signal.
:::
