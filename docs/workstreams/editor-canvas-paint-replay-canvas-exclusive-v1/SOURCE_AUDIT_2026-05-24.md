# Editor Canvas Paint Replay Canvas Exclusive v1 Source Audit

Date: 2026-05-24

## Scope

Inspect the remaining Canvas exclusive / `paint.widget` tail after the r65 fast-path closeout.

## Findings

- `crates/fret-ui/src/tree/paint/node.rs` measures widget paint with
  `debug_paint_widget_exclusive_resume()` / `debug_paint_widget_exclusive_pause()`. The exclusive
  timer wraps `widget.paint(&mut cx)` directly, so the owner boundary is the widget paint call
  itself, not a post-processing phase.
- The same node path records `debug_paint_widget_hotspots` after the widget returns. That record
  uses `element_record_for_node(...)` and optional debug-path lookup, so hotspot attribution is
  already a second-order diagnostic layer rather than the primary owner.
- `crates/fret-ui/src/canvas.rs` exposes `CanvasPainter` as the paint-side API for retained canvas
  resources, frame id, redraw requests, hosted-resource touches, and scene access. Any residual
  Canvas owner will surface through this API or through the host widget that invokes it.
- `ecosystem/fret-code-editor/src/editor/mod.rs` wires the code-editor row surface through
  `windowed_rows_surface_with_pointer_region(...)` and passes a `CanvasPainter` into
  `paint::paint_row(...)`. The code-editor itself is not the outer `paint.widget` timer; it is the
  Canvas callback body running inside that timer.
- The r65 fast-path lane already removed no-overlay planned replay row-setup work. The remaining
  owner is therefore outside the row-setup fast path and must be split either as generic widget
  traversal, Canvas callback work, or attribution bookkeeping around that callback.

## Initial Inference

The next bounded slice should start as a source-backed attribution split for the remaining
Canvas exclusive / `paint.widget` tail. If the audit proves the cost is still inside the code-editor
Canvas callback, the following mechanism slice can target the callback body; otherwise the lane
should land a diagnostics split in `fret-ui`.

## Evidence

- `crates/fret-ui/src/tree/paint/node.rs`
- `crates/fret-ui/src/canvas.rs`
- `ecosystem/fret-code-editor/src/editor/mod.rs`
- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
