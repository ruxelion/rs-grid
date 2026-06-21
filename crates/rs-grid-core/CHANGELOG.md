# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
