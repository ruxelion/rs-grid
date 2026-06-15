# Changelog

rs-grid uses **independent per-crate versioning**. Each publishable crate keeps
its own changelog, generated automatically by
[release-plz](https://release-plz.dev/) from
[Conventional Commits](https://www.conventionalcommits.org/), following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> While the project is pre-1.0, the public API may change between minor versions.

## Per-crate changelogs

| Crate | Changelog |
| --- | --- |
| `rs-grid-core` | [crates/rs-grid-core/CHANGELOG.md](crates/rs-grid-core/CHANGELOG.md) |
| `rs-grid-scene` | [crates/rs-grid-scene/CHANGELOG.md](crates/rs-grid-scene/CHANGELOG.md) |
| `rs-grid-render-canvas` | [crates/rs-grid-render-canvas/CHANGELOG.md](crates/rs-grid-render-canvas/CHANGELOG.md) |
| `rs-grid-web` | [crates/rs-grid-web/CHANGELOG.md](crates/rs-grid-web/CHANGELOG.md) |
| `rs-grid-leptos` | [crates/rs-grid-leptos/CHANGELOG.md](crates/rs-grid-leptos/CHANGELOG.md) |
| `rs-grid-dioxus` | [crates/rs-grid-dioxus/CHANGELOG.md](crates/rs-grid-dioxus/CHANGELOG.md) |
| `rs-grid-yew` | [crates/rs-grid-yew/CHANGELOG.md](crates/rs-grid-yew/CHANGELOG.md) |
| `rs-grid-icons` | [crates/rs-grid-icons/CHANGELOG.md](crates/rs-grid-icons/CHANGELOG.md) |

The per-crate files are created on the first release-plz run. Until then, the
only released milestone is the initial public, open-source release preparation
(`0.1.0`).
