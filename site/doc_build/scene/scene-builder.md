# Scene Builder

## Overview

`SceneBuilder` is the bridge between the data model (`GridState`) and the
rendering layer. It produces a `SceneFrame` — an immutable, ordered list
of drawing primitives.

## Usage

```rust
use rs_grid_scene::{SceneBuilder, Theme};

let builder = SceneBuilder::new(dpr, theme);
let frame = builder.build(&grid_state, column_drag_hint);
```

### Parameters

| Parameter          | Type                      | Description                              |
| ------------------ | ------------------------- | ---------------------------------------- |
| `dpr`              | `f64`                     | Device pixel ratio (e.g. 2.0 for Retina) |
| `theme`            | `Theme`                   | Visual properties for rendering          |
| `grid_state`       | `&GridState`              | Current grid state                       |
| `column_drag_hint` | `Option<&ColumnDragHint>` | Visual feedback during column drag       |

## SceneFrame

```rust
pub struct SceneFrame {
    primitives: Vec<ScenePrimitive>,
}
```

`SceneFrame` is produced fresh each frame — there is no mutable state
between frames. This makes the rendering pipeline pure and predictable.

## Draw order

Primitives are ordered **back-to-front**:

1. Background fills (cell backgrounds, alternating rows)
2. Grid lines
3. Cell text and images
4. Selection overlays
5. Headers (sticky, drawn on top)
6. Scrollbars (topmost layer)

The renderer draws them in sequence — later primitives paint over earlier ones.

## What the builder computes

For each frame, the builder:

1. Determines visible rows from `ViewportState`
2. Iterates visible columns (with pinned columns first)
3. Emits `Rect` primitives for cell backgrounds
4. Emits `Line` primitives for grid lines
5. Formats cell values via `CellFormat` and emits `Text` primitives
6. Emits `Image` primitives for image-format cells
7. Draws selection highlight (`Rect` with semi-transparent fill)
8. Draws search match highlights
9. Draws hover row overlay
10. Draws sticky header row
11. Draws scrollbars (arrows, track, thumb)
12. Draws column drag feedback if active

## Logical coordinates

All coordinates in `SceneFrame` are **logical pixels** (CSS pixels).
The renderer is responsible for applying the DPR:

```
canvas.width = viewport_width × dpr
canvas.height = viewport_height × dpr
context.scale(dpr, dpr)
```

## ColumnDragHint

When a user drags a column to reorder it, a `ColumnDragHint` provides
visual feedback:

```rust
pub struct ColumnDragHint {
    pub from_idx: usize,   // column being dragged
    pub current_x: f64,    // current mouse x position
}
```

The builder renders a semi-transparent ghost of the dragged column
at the current mouse position.
