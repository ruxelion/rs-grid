# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.18](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.17...rs-grid-web-v0.1.18) - 2026-07-27

### Other

- updated the following local packages: rs-grid-scene, rs-grid-render-canvas

## [0.1.17](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.16...rs-grid-web-v0.1.17) - 2026-07-22

### Other

- Add filter functionality and localization support

## [0.1.16](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.15...rs-grid-web-v0.1.16) - 2026-07-09

### Added

- per-row visibility predicate for cell buttons ([#64](https://github.com/ruxelion/rs-grid/pull/64))
- implement row-number gutter width management and server-side page fetching

## [0.1.15](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.14...rs-grid-web-v0.1.15) - 2026-07-08

### Added

- implement ExtendRowChecked command for shift+click row selection

### Fixed

- adjust cell position calculation to include checkbox column width

## [0.1.14](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.13...rs-grid-web-v0.1.14) - 2026-07-08

### Added

- add support for invalid cell background and border theming

## [0.1.13](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.12...rs-grid-web-v0.1.13) - 2026-07-07

### Added

- add row-selection checkbox column with tri-state functionality
- text cell editor switches between a single-line `<input>` and a wrapping/multiline `<textarea>` based on content, with `Alt+Enter` for manual line breaks and live-resizing height as lines are added or removed
- `flash_cells_error` renders a distinct error flash (`Theme::flash_error_fill`) on cells a `CutSelection`/Ctrl+X copied but couldn't clear (locked or failing validation), instead of looking identical to a fully-successful cut

### Fixed

- *(rs-grid-web)* SetSort and ClearSort never refetch server-side pages
- prevent paste event when an edit input is active

## [0.1.12](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.11...rs-grid-web-v0.1.12) - 2026-07-04

### Added

- add row-number gutter width adjustment and related tests

## [0.1.11](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.10...rs-grid-web-v0.1.11) - 2026-07-03

### Added

- clamp cell content and row rendering to sticky header boundaries

## [0.1.10](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.9...rs-grid-web-v0.1.10) - 2026-07-03

### Other

- updated the following local packages: rs-grid-core, rs-grid-scene, rs-grid-render-canvas

## [0.1.9](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.8...rs-grid-web-v0.1.9) - 2026-07-02

### Added

- *(tooltip)* implement at-rest validation tooltip for invalid cells

## [0.1.8](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.7...rs-grid-web-v0.1.8) - 2026-07-02

### Fixed

- *(canvas)* optimize canvas resizing to prevent visual flash during drag-resize

## [0.1.7](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.6...rs-grid-web-v0.1.7) - 2026-07-02

### Added

- add per-cell editability with editable predicates

## [0.1.6](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.5...rs-grid-web-v0.1.6) - 2026-07-01

### Added

- *(validation)* add live validation state callback and improve validation UI integration
- *(validation)* add declarative validation rules and live feedback for edits

## [0.1.5](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.4...rs-grid-web-v0.1.5) - 2026-06-21

### Added

- add scene generation and retrieval tools

## [0.1.4](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.3...rs-grid-web-v0.1.4) - 2026-06-20

### Added

- add localStorage helpers and persistable layout snapshot for examples

## [0.1.3](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.2...rs-grid-web-v0.1.3) - 2026-06-17

### Added

- implement value-driven progress bar with customizable styles and rendering

### Other

- backfill v0.1.2 changelog entries for PR #40 fixes

## [0.1.2](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.1...rs-grid-web-v0.1.2) - 2026-06-15

### Fixed

- `show_edit_input`: `None` editor no longer opens a text overlay; dispatches `CancelEdit` instead

### Other

- updated the following local packages: rs-grid-core, rs-grid-scene, rs-grid-render-canvas

## [0.1.1](https://github.com/ruxelion/rs-grid/compare/rs-grid-web-v0.1.0...rs-grid-web-v0.1.1) - 2026-06-15

### Added

- implement release-plz for automated versioning and changelog generation

### Other

- simplify conditional statements using `&&` for clarity
- reorder import statements for consistency across multiple files
- Update documentation and code references for Rust 2024 edition; add AGENTS.md files for new crates
