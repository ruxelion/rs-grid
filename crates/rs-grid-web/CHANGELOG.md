# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
