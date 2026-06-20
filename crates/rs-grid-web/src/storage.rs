//! Thin `localStorage` helpers for persisting small pieces of demo state
//! (such as a column layout) by string key.
//!
//! Every function degrades gracefully to a no-op / `None` when storage is
//! unavailable — private browsing, storage disabled, or a non-browser host —
//! rather than panicking. Pair with `example_common::layout::LayoutSnapshot`
//! to persist and restore a grid's column layout.

use web_sys::Storage;

/// Resolve the window's `localStorage`, if available.
fn local_storage() -> Option<Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

/// Read a string value by key. Returns `None` if storage is unavailable or
/// the key is absent.
pub fn get_item(key: &str) -> Option<String> {
    local_storage()?.get_item(key).ok().flatten()
}

/// Write a string value by key. Returns `true` on success, `false` if storage
/// is unavailable or the write was rejected (e.g. quota exceeded).
pub fn set_item(key: &str, value: &str) -> bool {
    local_storage().is_some_and(|ls| ls.set_item(key, value).is_ok())
}

/// Remove a value by key. Returns `true` on success, `false` if storage is
/// unavailable or the removal failed.
pub fn remove_item(key: &str) -> bool {
    local_storage().is_some_and(|ls| ls.remove_item(key).is_ok())
}
