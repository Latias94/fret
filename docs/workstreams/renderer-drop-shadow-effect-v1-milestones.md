# Renderer Effect: Drop Shadow v1 — Milestones

Status: Draft (workstream tracker)

Tracking files:

- `docs/workstreams/renderer-drop-shadow-effect-v1.md`
- `docs/workstreams/renderer-drop-shadow-effect-v1-todo.md`
- `docs/workstreams/renderer-drop-shadow-effect-v1-milestones.md`

## M0 — ADR lock + bounded parameters

Exit criteria:

- ADR defines the v1 surface (single layer, bounded blur, solid color).
- Degradation is deterministic and explicitly testable.

## M1 — wgpu implementation + portability validation

Exit criteria:

- Step is implemented in the filter-content pipeline.
- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu` is green.

## M2 — Conformance + perf gates

Exit criteria:

- GPU readback conformance exists.
- Perf gate exists with a checked-in baseline.

