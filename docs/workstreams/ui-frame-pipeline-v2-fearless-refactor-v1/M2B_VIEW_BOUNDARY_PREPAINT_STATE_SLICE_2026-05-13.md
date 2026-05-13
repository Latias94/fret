# M2B ViewBoundary Prepaint State Slice - 2026-05-13

Status: landed

## Truth

The runtime now has a minimal internal `ViewBoundaryState` store, and typed prepaint outputs are no
longer owned by `Node`.

## Artifacts

- `crates/fret-ui/src/tree/view_boundary.rs`
  - `BoundaryId`
  - `ViewBoundaryState`
  - `BoundaryPrepaintState`
  - boundary layout dependency metadata derived from current `ViewCacheFlags`
- `crates/fret-ui/src/tree/node_storage.rs`
  - removes `Node::prepaint_outputs`
- `crates/fret-ui/src/tree/ui_tree_invalidation.rs`
  - `begin_prepaint_outputs_for_node(...)`, `set_prepaint_output(...)`, and typed readers now use
    boundary prepaint state
- `ecosystem/fret-bootstrap/src/ui_diagnostics/cache_root_diagnostics.rs`
  - `debug.cache_roots[].boundary.prepaint_owner` now reports
    `view_boundary_prepaint_state` when the runtime boundary state exists
- `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_types.rs`
  - adds `debug.boundaries[]` as the first top-level boundary diagnostics surface
- `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_impl.rs`
  - builds `debug.boundaries[]` from direct `UiTree::debug_boundary_stats()` enumeration and joins
    cache-root outcomes when the same boundary is also a cache root

## Wiring

- `UiTree` owns `view_boundaries: SecondaryMap<NodeId, ViewBoundaryState>`.
- Prepaint creates or refreshes boundary state through `ensure_view_boundary_state(...)` before
  writing typed outputs.
- `CanvasPrepaintCx` and `CanvasPainter` keep their existing typed-output API, but the storage below
  that API is now boundary-owned.
- Node removal calls `remove_view_boundary_state(...)`, so prepaint outputs cannot outlive the node
  that owns the boundary.
- `should_reuse_view_cache_node(...)` now reads the contained-relayout decision through boundary
  layout dependency metadata instead of directly rechecking only raw view-cache flags.
- `debug.boundaries[]` is built from direct `ViewBoundaryState` diagnostics enumeration, so the
  bundle has a first-class boundary list. Existing cache-root diagnostics remain compatible and are
  joined back into matching boundaries as outcome fields.

## Proof

Correctness gates:

```bash
cargo nextest run -p fret-ui tree::tests::prepaint::prepaint_output_is_owned_by_view_boundary_state_and_removed_with_node tree::tests::prepaint::prepaint_output_store_is_keyed_by_cache_root_prepaint_key declarative::tests::canvas::canvas_prepaint_output_is_visible_to_canvas_paint --no-fail-fast
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics cache_root_boundary boundary_diagnostics_are_built_from_boundary_stats_with_cache_root_outcomes --no-fail-fast
cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-bootstrap --features syntax-rust
python3 tools/check_layering.py
```

Observed result:

- `fret-ui` focused nextest: 3 passed, 928 skipped.
- `fret-bootstrap` boundary diagnostics nextest: 4 passed, 97 skipped.
- `cargo check`: passed.
- `tools/check_layering.py`: passed.

## Residual Risk

- This slice makes prepaint output state boundary-owned, but it is still a typed-output carrier. It
  is not the final boundary-owned `SceneFragment` store.
- `debug.boundaries[]` is now directly enumerated from `ViewBoundaryState`, but its final
  dirty/build/layout/prepaint/paint ownership model is still transitional while cache-root outcome
  fields are joined in for compatibility.
- Layout containment now has a minimal boundary dependency metadata owner, but broader dirty-set and
  solve-set migration from `dirty_cache_roots` is still pending.
- No new perf claim is made by this slice. The 20-30% p95/max closeout target still depends on the
  final scene-fragment store and a fresh `ui-code-editor-resize-probes` run.
