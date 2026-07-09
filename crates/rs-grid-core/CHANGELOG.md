# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/ruxelion/rs-grid/compare/rs-grid-core-v0.3.1...rs-grid-core-v0.4.0) - 2026-07-09

### Added

- per-row visibility predicate for cell buttons ([#64](https://github.com/ruxelion/rs-grid/pull/64))
- implement row-number gutter width management and server-side page fetching

## [0.3.1](https://github.com/ruxelion/rs-grid/compare/rs-grid-core-v0.3.0...rs-grid-core-v0.3.1) - 2026-07-08

### Added

- implement ExtendRowChecked command for shift+click row selection

## [0.3.0](https://github.com/ruxelion/rs-grid/compare/rs-grid-core-v0.2.5...rs-grid-core-v0.3.0) - 2026-07-07

### Added

- add row-selection checkbox column with tri-state functionality
- `CutSelection` reports skipped cells (locked or failing validation) via a new `CommandOutput::CutApplied { text, skipped }`, distinguishing "cut succeeded" from "copied but couldn't clear"

## [0.2.5](https://github.com/ruxelion/rs-grid/compare/rs-grid-core-v0.2.4...rs-grid-core-v0.2.5) - 2026-07-04

### Added

- add row-number gutter width adjustment and related tests

## [0.2.4](https://github.com/ruxelion/rs-grid/compare/rs-grid-core-v0.2.3...rs-grid-core-v0.2.4) - 2026-07-03

### Added

- add per-cell decoration support with CellDecorator for visual annotations

## [0.2.3](https://github.com/ruxelion/rs-grid/compare/rs-grid-core-v0.2.2...rs-grid-core-v0.2.3) - 2026-07-02

### Added

- implement ClearCells command to clear selected cells without clipboard interaction
- *(clipboard)* enhance CutSelection to skip invalid cells during clearing

## [0.2.2](https://github.com/ruxelion/rs-grid/compare/rs-grid-core-v0.2.1...rs-grid-core-v0.2.2) - 2026-07-02

### Added

- *(validation)* enhance cell validation during paste operations and add visual indicators for invalid cells

## [0.2.1](https://github.com/ruxelion/rs-grid/compare/rs-grid-core-v0.2.0...rs-grid-core-v0.2.1) - 2026-07-02

### Added

- add per-cell editability with editable predicates

## [0.2.0](https://github.com/ruxelion/rs-grid/compare/rs-grid-core-v0.1.4...rs-grid-core-v0.2.0) - 2026-07-01

### Added

- *(validation)* add declarative validation rules and live feedback for edits

## [0.1.4](https://github.com/ruxelion/rs-grid/compare/rs-grid-core-v0.1.3...rs-grid-core-v0.1.4) - 2026-06-21

### Added

- add scene generation and retrieval tools
- disable default libtest bench harness for lib targets in Cargo.toml

## [0.1.3](https://github.com/ruxelion/rs-grid/compare/rs-grid-core-v0.1.2...rs-grid-core-v0.1.3) - 2026-06-17

### Added

- implement value-driven progress bar with customizable styles and rendering

### Other

- backfill v0.1.2 changelog entries for PR #40 fixes

## [0.1.2](https://github.com/ruxelion/rs-grid/compare/rs-grid-core-v0.1.1...rs-grid-core-v0.1.2) - 2026-06-15

### Added

- add `GridModelBuilder::editable(bool)` builder method, symmetric with `selectable()` and `column_reorderable()`

### Other

- update criterion version to 0.8 in Cargo.toml and add new dependencies in Cargo.lock

## [0.1.1](https://github.com/ruxelion/rs-grid/compare/rs-grid-core-v0.1.0...rs-grid-core-v0.1.1) - 2026-06-15

### Added

- implement release-plz for automated versioning and changelog generation

### Fixed

- update edition to 2024 in Cargo.toml and refactor variable names for clarity

### Other

- simplify conditional statements using `&&` for clarity
- reorder import statements for consistency across multiple files
- Update documentation and code references for Rust 2024 edition; add AGENTS.md files for new crates
