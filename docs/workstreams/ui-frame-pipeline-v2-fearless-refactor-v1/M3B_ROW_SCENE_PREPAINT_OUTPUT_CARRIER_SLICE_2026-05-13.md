# M3B Row Scene Prepaint Output Carrier Slice - 2026-05-13

Status: landed; the transitional row-scene replay plan now lives in node-scoped canvas prepaint
output instead of `CodeEditorState`.

Status note (2026-05-14): this slice is now partially superseded by
`M3C_BOUNDARY_SCENE_FRAGMENT_CARRIER_SLICE_2026-05-14.md`. The replay-plan carrier no longer uses
generic canvas prepaint output; it now flows through `ViewBoundaryState::scene_fragment`.

## Scope

This slice keeps the code-editor replay-plan story transitional, but changes the carrier:

- `crates/fret-ui`'s `PrepaintOutputs` now exposes typed set/get/mut helpers through canvas
  prepaint and paint.
- `CanvasPrepaintCx` and `CanvasPainter` both read/write the same node-scoped prepaint output slot.
- `ecosystem/fret-code-editor` no longer stores `RowSceneReplayPlan` on `CodeEditorState`;
  prepaint produces the plan and writes it into the canvas output slot, paint consumes it from
  `CanvasPainter`.
- `RowSceneReplayPlan.visible_window` and the old editor-state helpers stay deleted.

This is still transitional: the carrier is node-scoped prepaint output, not the final
`ViewBoundary` scene-fragment owner.

## Implementation

Main changes:

- `crates/fret-ui/src/tree/node_storage.rs`
  - `PrepaintOutputs::set_box(...)`, `get_any(...)`, and `get_any_mut(...)` now support object-safe
    prepaint output storage.
- `crates/fret-ui/src/tree/ui_tree_invalidation.rs`
  - added typed tree helpers for boxed prepaint output writes and dynamic access.
- `crates/fret-ui/src/canvas.rs`
  - `CanvasPrepaintCx::set_output(...)`, `output(...)`, and `output_mut(...)` now forward to the
    node-scoped prepaint output slot.
  - `CanvasPainter::prepaint_output(...)` and `prepaint_output_mut(...)` now consume the same slot
    during paint.
- `ecosystem/fret-code-editor/src/editor/mod.rs`
  - the windowed-rows prepaint hook now writes the replay plan into canvas prepaint output.
- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
  - paint now consumes `RowSceneReplayPlan` from `CanvasPainter::prepaint_output_mut(...)`.
- `ecosystem/fret-code-editor/src/editor/state.rs`
  - removed `row_scene_replay_plan` from `CodeEditorState`,
  - removed `RowSceneReplayPlan.visible_window`,
  - removed the old reset/push/take helpers.
- `crates/fret-ui/src/declarative/tests/canvas.rs`
  - added regression coverage that a stable frame preserves the previous canvas prepaint output and
    that a changed prepaint key resets it.

## Evidence

Focused correctness gate:

- `cargo nextest run -p fret-ui declarative::tests::canvas::canvas_prepaint_output_is_visible_to_canvas_paint --no-fail-fast`

Perf gate:

```bash
cargo run -p fretboard-dev --release -- diag perf ui-code-editor-resize-probes \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --sort time \
  --top 15 \
  --json \
  --dir target/fret-diag-code-editor-resize-probes-canvas-prepaint-output-20260513 \
  --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Perf output directory:

- `target/fret-diag-code-editor-resize-probes-canvas-prepaint-output-20260513`

Worst bundle:

- `target/fret-diag-code-editor-resize-probes-canvas-prepaint-output-20260513/1778685213875/bundle.schema2.json`

Observed result from that run:

- time p50/p95/max: total `1103/1576/1576us`, layout `35/344/344us`,
  prepaint `251/360/360us`, paint `659/877/877us`
- hot p50/p95: `layout.engine_solve=0/133us`, `paint.widget=445/661us`,
  `paint.text_prepare=10/13us`
- `code_editor.paint_perf` p50/p95 total: `175/403us`
- `code_editor.paint_perf.us_row_text` p50/p95: `0/5us`
- planned/used replay entries still matched `2090/2090`
- row scene replay hit rate remained `99%`
- renderer prepare/encode/upload counters stayed at zero

Worst-bundle attribution:

- `target/release/fretboard-dev diag stats target/fret-diag-code-editor-resize-probes-canvas-prepaint-output-20260513/1778685213875/bundle.schema2.json --sort time --top 15`
- time p50/p95: total `1103/1576us`, layout `35/344us`, prepaint `251/360us`,
  paint `659/877us`
- hot p50/p95: `layout.engine_solve=0/133us`, `paint.widget=445/661us`,
  `paint.text_prepare=10/13us`
- `code_editor.paint_perf` sum planned/used replay entries: `2090/2090`
- `code_editor.paint_perf` p50/p95 `us_row_scene_prepaint_plan`: `55/77us`
- `code_editor.paint_perf` p50/p95 `us_row_text`: `0/5us`

## Deletion Audit

What changed:

- `CodeEditorState.row_scene_replay_plan` is gone.
- the replay-plan carrier is no longer editor-state-owned; it is now a node-scoped prepaint output
  slot.
- paint reads the carrier from the node's prepaint output rather than from editor-local storage.

What is still transitional:

- the carrier is still `PrepaintOutputs`, not the final `ViewBoundary` fragment store.
- diagnostics still speak in code-editor paint-perf terms rather than canonical boundary fragment
  terms.
- the final scene-fragment owner and boundary dependency metadata are still pending.

Follow-up deletion/narrowing target:

- move the replay plan carrier from node-scoped prepaint output into boundary-owned state,
- then delete the transitional output path once the boundary fragment store owns validation and
  replay.
