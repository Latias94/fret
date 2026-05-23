# Editor Canvas Paint Replay Fast Path v1 Milestones

## M1 - Fast Path Implemented

Completed on 2026-05-24.
- Retained row scene fragments store capture bounds.
- Planned no-overlay replay rows derive target origin from retained bounds and current bounds.
- Overlay rows continue through the existing paint-time overlay path.

## M2 - Local Gates Passed

Completed on 2026-05-24.
- Focused code-editor planned replay tests pass.
- `cargo check -p fret-code-editor --tests --features syntax-rust` passes.
- Format, JSON, workstream catalog, and diff whitespace gates pass.

## M3 - Target-Machine Evidence Captured

Completed on 2026-05-24.
- Baseline validation passes.
- Rebuilt attribution validation includes paint-perf counters.
- Artifact verification and closeout pass.
- Parent owner decision is updated from evidence, not expectation.

## Closeout

The lane is closed as of 2026-05-24. Remaining Canvas replay work should move into a new bounded
follow-on with its own repro, gates, and closeout.
