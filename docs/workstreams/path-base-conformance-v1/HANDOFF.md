# Path Base Conformance v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This lane is closed. It was opened as a narrow follow-on to ADR 0080's remaining conformance gap and
closed after adding the base WGPU path conformance gates.

## Useful Gates

No action remains in this lane. Future path work should start as a narrower additive follow-on when
it changes a path contract, backend, or style surface.

```bash
cargo test -p fret-render-wgpu --locked --test path_base_conformance -j 1
cargo test -p fret-render-wgpu --locked --lib renderer::path::tests::path_metrics_bounds_contain_tessellated_vertices -j 1
```

## Guardrails

- Keep code changes scoped to renderer path tests/contract fixes.
- Update ADR 0080 and implementation alignment only after gates prove the behavior.
- If a new renderer behavior bug appears, either fix it in this lane if it is directly ADR 0080
  base path behavior, or split a narrower follow-on.
