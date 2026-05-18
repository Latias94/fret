# WGPU Text Measure Dead Branch Prune v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Dead Branches Deleted

Exit criteria:

- `crates/fret-render-wgpu/src/text/measure.rs` no longer contains `#[cfg(any())]`.
- `TextSystem::measure` and `measure_attributed` remain thin facades over
  `TextMeasureCaches`.

Status: Complete on 2026-05-18.

## M1 - Measurement Behavior Verified

Exit criteria:

- `fret-render-wgpu` test targets compile.
- Plain and attributed measure/prepare parity tests pass under fractional scale factors.

Status: Complete on 2026-05-18.

## M2 - Workstream Closed

Exit criteria:

- Workstream catalog and JSON validation pass.
- Closeout note records the active measurement owner and the deleted stale duplicate.

Status: Complete on 2026-05-18.
