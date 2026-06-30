# M3C Boundary Scene Fragment Carrier Slice - 2026-05-14

Status: landed in working tree

## Truth

The code-editor row-scene replay plan no longer travels through the generic typed prepaint-output
carrier. It is now carried through boundary-owned scene-fragment state on `ViewBoundaryState`.

This is still a vertical slice, not final closeout: the code-editor still owns row-scene validation
policy and diagnostics, while `fret-ui` owns the mechanism that stores and exposes the fragment
state for prepaint-to-paint handoff.

## Artifacts

- `crates/fret-ui/src/tree/view_boundary.rs`
  - adds `BoundarySceneFragmentState` beside `BoundaryPrepaintState`.
  - keeps the fragment store keyed by the same prepaint key as the boundary prepaint state.
- `crates/fret-ui/src/tree/ui_tree_invalidation.rs`
  - adds typed and object-safe scene-fragment accessors on `UiTree`.
- `crates/fret-ui/src/tree/prepaint/interaction.rs`
  - begins scene-fragment state when a widget prepaint pass begins.
- `crates/fret-ui/src/canvas.rs`
  - adds `CanvasSceneFragment<T>` with scene ops, hosted resource side indexes, local bounds, and
    scene origin.
  - exposes `CanvasPrepaintCx::set_scene_fragment(...)` and
    `CanvasPainter::scene_fragment_mut(...)`.
  - exposes `set_scene_fragment_debug(...)` for carriers that can report fragment entry counts.
- `ecosystem/fret-bootstrap/src/ui_diagnostics/cache_root_diagnostics.rs`
  - adds `debug.boundaries[].scene_fragment_owner`, `scene_fragment_slots`, and
    `scene_fragment_entries`.
  - the U7 convergence slice extends those boundary diagnostics with `scene_fragment_chunks` and
    `scene_fragment_fingerprint` so retained chunks can be tracked before renderer chunk encoding
    reuse replaces the flat `Scene` bridge.
- `ecosystem/fret-code-editor/src/editor/state.rs`
  - splits row-scene replay payload from the runtime fragment carrier.
- `ecosystem/fret-code-editor/src/editor/mod.rs`
  - writes the row-scene replay plan into the canvas boundary scene-fragment slot.
- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
  - consumes `RowSceneReplayPlan` from `CanvasPainter::scene_fragment_mut(...)`.
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
  - stores row-scene ops/resources/origin/bounds as `CanvasSceneFragment<RowSceneFragmentPayload>`.

## Proof

Correctness gates:

```bash
cargo nextest run -p fret-ui declarative::tests::canvas::canvas_scene_fragment_is_boundary_owned_and_keyed_by_prepaint_key --no-fail-fast
cargo nextest run -p fret-code-editor --features syntax-rust prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint --no-fail-fast
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics cache_root_boundary boundary_diagnostics_are_built_from_boundary_stats_with_cache_root_outcomes --no-fail-fast
cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor --features syntax-rust
```

Observed result:

- `fret-ui` canvas scene-fragment nextest: 1 passed, 931 skipped.
- `fret-code-editor` row-scene replay nextest: 1 passed, 128 skipped.
- `fret-bootstrap` boundary diagnostics nextest: 4 passed, 97 skipped.
- `cargo check`: passed.

## Deletion Audit

Deleted/narrowed by this slice:

- The code-editor row-scene replay plan no longer uses `CanvasPrepaintCx::set_output(...)`.
- Paint no longer consumes `RowSceneReplayPlan` through `CanvasPainter::prepaint_output_mut(...)`.
- Replayable row-scene entries now have an explicit fragment-shaped carrier with scene ops,
  hosted resources, local bounds, and origin.
- `debug.boundaries[]` can now report scene-fragment owner, slot count, and fragment entry count
  for debug-aware carriers.

Still transitional:

- Generic prepaint outputs remain for non-fragment prepaint data and existing canvas tests.
- Row-scene validation and counters still live in `ecosystem/fret-code-editor`.
- `debug.boundaries[]` does not yet report fragment hit/reject reasons directly.
- A final perf run is still required before the workstream can claim the 20-30% p95/max closeout
  target.

## Next

- Add boundary fragment diagnostics for planned/used/rejected fragment entries.
- Decide whether code-editor `rows_scene_prepaint_*` counters can be merged into boundary
  diagnostics after the perf gate covers the fragment path.
- Run `ui-code-editor-resize-probes` and worst-bundle `diag stats` for the boundary-fragment path.
