---
title: Diagnostics Fearless Refactor v1 (TODO)
status: draft
date: 2026-02-22
scope: diagnostics, automation, tooling, refactor
---

# Diagnostics Fearless Refactor v1 (TODO)

## M1: Make the monolith smaller (safe mechanical moves)

- [ ] Extract script engine core out of `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` into
      `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`.
  - Keep the public entrypoint signature stable.
  - Keep internal state types in `ecosystem/fret-bootstrap/src/ui_diagnostics/script_types.rs`.
- [ ] Define a stable “module boundary” inside `ecosystem/fret-bootstrap/src/ui_diagnostics/`:
  - script execution / state / step handlers,
  - bundle dumping + sidecar writers,
  - DevTools WS bridge wiring.
- [ ] Add a small regression gate: `cargo check -p fret-ui-gallery` after each extraction step.

## M2: Shrink + index artifacts (sidecars over monolithic JSON)

- [ ] Define the “minimum useful bundle” contract (what must be in `bundle.json` vs what can be in sidecars).
- [ ] Add query-friendly indexes (sidecars) for tools/agents:
  - snapshot selectors (`frame_id`, `unix_ms`),
  - test-id presence indexes (probabilistic is OK),
  - script step → snapshot mapping for fast evidence lookup.
- [ ] Make sidecars forward-compatible:
  - versioned schema,
  - additive-only evolution,
  - documented failure behavior when sidecars are missing.

## M3: Tooling + AI loop

- [ ] Define CLI “agent presets” (commands + env vars) for repeatable triage.
- [ ] Prefer structured evidence diffs over screenshot diffs where possible.
- [ ] Document a recommended script authoring style for stability (selectors first, bounded waits).

## Evidence anchors (keep updated as implementation changes)

- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_types.rs`

