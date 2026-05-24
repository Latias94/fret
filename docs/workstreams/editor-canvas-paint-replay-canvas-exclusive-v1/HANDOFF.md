# Editor Canvas Paint Replay Canvas Exclusive v1 Handoff

Date: 2026-05-24

## Current State

Active. ECPR-CX-010 is complete: Canvas `on_paint` callback time is now surfaced separately as
`paint.canvas_on_paint` / `paint_canvas_on_paint_time_us`. The remaining owner is still Canvas
exclusive / `paint.widget` overhead outside row setup.

## Next Action

Use the new counter in the next source-backed probe, then decide whether the residual is generic
widget traversal, Canvas callback work, or code-editor replay bookkeeping.

## Validation

Local evidence already on hand:

- r65 fast-path closeout audit
- r65 baseline validation
- r65 rebuilt attribution validation
- r65 closeout
- `cargo fmt -p fret-ui -p fret-bootstrap -p fret-diag --check`
- `cargo check -p fret-ui --tests`
- `cargo check -p fret-bootstrap`
- `cargo check -p fret-diag`
- `cargo nextest run -p fret-diag full_registered_perf_key_registry_covers_consumed_debug_stats_fields registered_perf_key_units_match_names trace_exported_perf_key_registry_contains_core_timeline_keys --no-fail-fast`

## Cautions

- Do not reopen the closed row-setup, resource-touch, or fast-path lanes.
- Do not change checked-in baselines from this lane.
- Keep the first slice source-backed and bounded.
