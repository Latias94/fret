# WGPU Conformance Harness v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This lane is closed. The shared helper extraction and first path-related test migration are done.

## Next Action

No action remains in this lane.

```bash
cargo test -p fret-render-wgpu --locked --test path_base_conformance --test path_stroke_style_v2_conformance --test path_paint_conformance --test path_material_paint_conformance -j 1
```

## Guardrails

- Keep the helper in `tests/support/`; do not expose it as production API.
- Preserve each test's assertions and no-adapter skip behavior.
- Do not migrate unrelated effect/text/clip tests unless this lane is explicitly widened.
