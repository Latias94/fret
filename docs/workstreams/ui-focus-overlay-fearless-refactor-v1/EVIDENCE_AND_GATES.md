# Evidence and Gates

## Minimum gates (local)

- `cargo fmt`
- Targeted regression tests (Phase A/B invariants):
  - `cargo nextest run -p fret-ui --lib outside_press_branch_containment_uses_child_edges_not_parent_pointers`
  - `cargo nextest run -p fret-ui --lib dismissible_outside_press_prevent_default_keeps_focus`
  - `cargo nextest run -p fret-ui --lib dismissible_outside_press_without_prevent_default_clears_focus`
- Full suite (when practical): `cargo nextest run -p fret-ui`
- `python3 tools/check_layering.py`

## Existing regression coverage (anchors)

- Outside press routing: `crates/fret-ui/src/tree/tests/outside_press.rs`
- Escape dismissal: `crates/fret-ui/src/tree/tests/escape_dismiss.rs`
- Focus scope trap: `crates/fret-ui/src/tree/tests/focus_scope.rs`
- Declarative dismissible interactions: `crates/fret-ui/src/declarative/tests/interactions/dismissible.rs`

## Phase C anchors (snapshot PR0)

- Snapshot types + builder: `crates/fret-ui/src/tree/dispatch_snapshot.rs`
- Debug entrypoint (no behavior change): `crates/fret-ui/src/tree/ui_tree_debug/query.rs` (`debug_dispatch_snapshot`)
- Snapshot parity report: `crates/fret-ui/src/tree/ui_tree_debug/query.rs` (`debug_dispatch_snapshot_parity`)
- Outside-press routed via snapshot (PR2): `crates/fret-ui/src/tree/ui_tree_outside_press.rs` and
  `crates/fret-ui/src/tree/dispatch/window.rs`

## New artifacts (Phase A/B)

- Unit test: stale parent pointers do not break outside-press branch exclusion.
- Unit tests: outside-press default focus clearing vs `prevent_default` suppression.
