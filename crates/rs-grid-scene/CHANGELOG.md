# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2](https://github.com/ruxelion/rs-grid/compare/rs-grid-scene-v0.2.1...rs-grid-scene-v0.2.2) - 2026-07-03

### Added

- add linear interpolation method for Color struct
- add per-cell decoration support with CellDecorator for visual annotations

### Other

- Refactor code structure for improved readability and maintainability

## [0.2.1](https://github.com/ruxelion/rs-grid/compare/rs-grid-scene-v0.2.0...rs-grid-scene-v0.2.1) - 2026-07-02

### Added

- implement ClearCells command to clear selected cells without clipboard interaction

## [0.2.0](https://github.com/ruxelion/rs-grid/compare/rs-grid-scene-v0.1.6...rs-grid-scene-v0.2.0) - 2026-07-02

### Added

- *(validation)* enhance cell validation during paste operations and add visual indicators for invalid cells

## [0.1.6](https://github.com/ruxelion/rs-grid/compare/rs-grid-scene-v0.1.5...rs-grid-scene-v0.1.6) - 2026-07-02

### Added

- add per-cell editability with editable predicates

## [0.1.5](https://github.com/ruxelion/rs-grid/compare/rs-grid-scene-v0.1.4...rs-grid-scene-v0.1.5) - 2026-07-01

### Other

- updated the following local packages: rs-grid-core

## [0.1.4](https://github.com/ruxelion/rs-grid/compare/rs-grid-scene-v0.1.3...rs-grid-scene-v0.1.4) - 2026-06-21

### Added

- add scene generation and retrieval tools
- disable default libtest bench harness for lib targets in Cargo.toml

## [0.1.3](https://github.com/ruxelion/rs-grid/compare/rs-grid-scene-v0.1.2...rs-grid-scene-v0.1.3) - 2026-06-17

### Added

- implement value-driven progress bar with customizable styles and rendering

## [0.1.2](https://github.com/ruxelion/rs-grid/compare/rs-grid-scene-v0.1.1...rs-grid-scene-v0.1.2) - 2026-06-15

### Other

- update criterion version to 0.8 in Cargo.toml and add new dependencies in Cargo.lock

## [0.1.1](https://github.com/ruxelion/rs-grid/compare/rs-grid-scene-v0.1.0...rs-grid-scene-v0.1.1) - 2026-06-15

### Added

- implement release-plz for automated versioning and changelog generation

### Other

- simplify conditional statements using `&&` for clarity
- reorder import statements for consistency across multiple files
- Update documentation and code references for Rust 2024 edition; add AGENTS.md files for new crates
