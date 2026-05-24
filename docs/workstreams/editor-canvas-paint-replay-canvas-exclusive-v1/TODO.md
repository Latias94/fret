# Editor Canvas Paint Replay Canvas Exclusive v1 TODO

## Tasks

- [x] ECPR-CX-010: Establish the source-backed attribution boundary for the residual Canvas exclusive /
  paint-widget tail.
  - Result:
    `paint_canvas_on_paint_time` / `paint_canvas_on_paint_time_us` now measure Canvas `on_paint`
    callback time separately from generic widget paint time and are carried through `fret-ui`,
    `fret-bootstrap`, and `fret-diag`.
  - Scope:
    `crates/fret-ui/src/tree/paint/node.rs`,
    `crates/fret-ui/src/canvas.rs`,
    `ecosystem/fret-code-editor/src/editor/mod.rs`,
    `ecosystem/fret-code-editor/src/editor/paint/mod.rs`,
    `ecosystem/fret-code-editor/src/editor/paint/scene.rs`.
  - Expected result:
    the next lane can tell whether the remaining owner is generic widget traversal, Canvas callback
    work, or code-editor replay bookkeeping.
  - Validation:
    - `rg -n "debug_paint_widget_exclusive|debug_paint_widget_hotspots|CanvasPainter|on_paint" crates/fret-ui ecosystem/fret-code-editor -g "*.rs"`
    - `cargo fmt -p fret-ui -p fret-bootstrap -p fret-diag --check`
    - `cargo check -p fret-ui --tests`
    - `cargo check -p fret-bootstrap`
    - `cargo check -p fret-code-editor --tests --features syntax-rust`
    - `cargo check -p fret-diag`
    - `cargo nextest run -p fret-diag full_registered_perf_key_registry_covers_consumed_debug_stats_fields registered_perf_key_units_match_names trace_exported_perf_key_registry_contains_core_timeline_keys --no-fail-fast`
  - Notes:
    source audit complete; keep the next slice source-backed and bounded.

- [ ] ECPR-CX-020: Land the smallest bounded fix or diagnostics split once the owner boundary is proven.
  - Expected result:
    the remaining Canvas exclusive / paint-widget cost is either reduced or proven to live in a
    different layer with source-backed evidence.
  - Validation:
    target-machine r66 baseline/attrib validation, artifact verification, and closeout.
  - Notes:
    use the new `paint.canvas_on_paint` counter to compare widget traversal, Canvas callback work,
    and code-editor replay bookkeeping before widening scope.

## Current Decision

The fast-path lane is closed, but the r65 closeout still selects `owner=canvas-paint-replay`. The
source-backed boundary is now explicit: `paint.canvas_on_paint` isolates Canvas callback time from
generic `paint.widget` time. Use the new counter to decide whether the residual belongs to widget
traversal, Canvas callback work, or code-editor replay bookkeeping before widening scope.
