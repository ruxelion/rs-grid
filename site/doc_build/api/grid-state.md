# GridState API

## Definition

```rust
pub struct GridState {
    pub model: GridModel,
    pub viewport: ViewportState,
    pub selection: SelectionState,
    pub hovered_row: Option<u64>,
    pub sort: Option<SortState>,
    pub edit: Option<EditCell>,
    pub search: SearchState,
    history: UndoHistory,  // private
}
```

## Fields

| Field         | Type                | Description                                                 |
| ------------- | ------------------- | ----------------------------------------------------------- |
| `model`       | `GridModel`         | Columns, data source, sizing, sort/filter state             |
| `viewport`    | `ViewportState`     | Scroll position (`scroll_x`, `scroll_y`), canvas dimensions |
| `selection`   | `SelectionState`    | Anchor/focus selection, clipboard buffer                    |
| `hovered_row` | `Option<u64>`       | Row under the mouse cursor                                  |
| `sort`        | `Option<SortState>` | Active sort column and direction                            |
| `edit`        | `Option<EditCell>`  | Cell currently being edited                                 |
| `search`      | `SearchState`       | Active search query, matches, current index                 |

## Constructor

```rust
pub fn new(
    model: GridModel,
    viewport_width: f64,
    viewport_height: f64,
) -> Self
```

Creates a `GridState` with the given model and initial viewport dimensions.
All other fields start at their defaults (no selection, no sort, no edit, etc.).

## Methods

### apply

```rust
pub fn apply(&mut self, cmd: GridCommand) -> CommandOutput
```

The **only** way to mutate `GridState`. Applies a command and returns output.

See [GridCommand API](/api/grid-command.md) for all available commands.

## Related types

### SortState

```rust
pub struct SortState {
    pub col_key: String,
    pub dir: SortDir,
}

pub enum SortDir { Asc, Desc }
```

### EditCell

```rust
pub struct EditCell {
    pub row: u64,
    pub col_key: String,
    pub original_value: Option<String>,
}
```

### SearchState

```rust
pub struct SearchState {
    pub query: String,
    pub matches: Vec<CellCoord>,
    pub current: usize,
}
```
