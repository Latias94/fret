# Editor Canvas Paint Replay Fast Path v1 Milestones

## M1 - Fast Path Implemented

- Retained row scene fragments store capture bounds.
- Planned no-overlay replay rows derive target origin from retained bounds and current bounds.
- Overlay rows continue through the existing paint-time overlay path.

## M2 - Local Gates Passed

- Focused code-editor planned replay tests pass.
- `cargo check -p fret-code-editor --tests --features syntax-rust` passes.
- Format, JSON, workstream catalog, and diff whitespace gates pass.

## M3 - Target-Machine Evidence Captured

- Baseline validation passes.
- Rebuilt attribution validation includes paint-perf counters.
- Artifact verification and closeout pass.
- Parent owner decision is updated from evidence, not expectation.
