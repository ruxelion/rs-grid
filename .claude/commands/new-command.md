Add a new `GridCommand` variant to rs-grid-core, following the repeatable
pattern so only the genuine business logic needs judgement — everything else is
a deterministic checklist.

**Argument**: a short description of the command, e.g.
`/new-command toggle a column's visibility`.

Work through these steps, in `crates/rs-grid-core` unless noted:

1. **Classify the version impact first** (AGENTS.md → Versioning): a new public
   command variant on the `#[non_exhaustive]` enum is a **minor** bump. State it.
2. **Variant** — add the variant to `enum GridCommand` in
   `src/commands.rs`, with doc-comment and fields (remember: row indices are
   `u64`, column indices `usize`). Add it to the right category in the
   doc-comment table at the top of the enum.
3. **Handle it in `apply`** — add a match arm in
   `GridState::apply(GridCommand)` (`src/state.rs`). This is the only place
   `GridState` is mutated; never mutate fields directly elsewhere. Return the
   appropriate `CommandOutput`.
4. **Undo/redo** — if the command mutates data (not just selection/scroll),
   record it on the undo history so `Undo`/`Redo` reverse it. Mirror how a
   sibling mutating command (e.g. `PasteAt`) does it.
5. **Clipboard** — only if the command interacts with copy/cut/paste.
6. **Callbacks** — if it changes cell data or column layout, make sure the
   relevant `rs-grid-web` callback fires (`set_on_change` /
   `set_on_columns_changed`); see `rs-grid-web/AGENTS.md` → Public callbacks.
7. **Test** — add an inline `#[cfg(test)]` test in the module asserting the
   state transition. Run `/test`.
8. **Docs** — update `commands.rs` category table and any affected `AGENTS.md`
   per the documentation-sync rule.

Only steps 3–4 (the actual semantics + undo) need real thought; the rest is
mechanical. Do the mechanical parts exactly, and reason carefully about the
mutation + undo.
