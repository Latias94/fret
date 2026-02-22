---
title: Diagnostics Fearless Refactor v1 (TODO)
status: draft
date: 2026-02-22
scope: diagnostics, automation, tooling, refactor
---

# Diagnostics Fearless Refactor v1 (TODO)

## M1: Make the monolith smaller (safe mechanical moves)

- [x] Extract internal script runner state types into `ecosystem/fret-bootstrap/src/ui_diagnostics/script_types.rs`.
- [x] Move script evidence + trace helpers into `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`.
- [x] Extract `drive_script_for_window` finalization into `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`.
- [x] Extract the per-frame driver (`UiDiagnosticsService::drive_script_for_window`) out of
      `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` into `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`.
  - Keep the public entrypoint signature stable.
- [ ] Define a stable “module boundary” inside `ecosystem/fret-bootstrap/src/ui_diagnostics/`:
  - script execution / state / step handlers,
  - bundle dumping + sidecar writers,
  - DevTools WS bridge wiring.
- [x] Keep a regression gate: `cargo check -p fret-ui-gallery` after each extraction step.

## M1b: Make `fret-diag` stats less monolithic (mechanical moves)

- [x] Move UI gallery markdown-editor checks out of `crates/fret-diag/src/stats.rs` into
      `crates/fret-diag/src/stats/ui_gallery_markdown_editor.rs`.
- [ ] Move UI gallery code-editor checks out of `crates/fret-diag/src/stats.rs` into
      `crates/fret-diag/src/stats/ui_gallery_code_editor.rs`.

## M2: Shrink + index artifacts (sidecars over monolithic JSON)

- [ ] Define the “minimum useful bundle” contract (what must be in `bundle.json` vs what can be in sidecars).
- [x] Add query-friendly indexes (sidecars) for tools/agents (implemented in `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_dump.rs`):
  - `bundle.index.json` (snapshot selectors, semantics fingerprints, test-id bloom),
  - `bundle.meta.json` (bundle-level counters + uniqueness summaries),
  - `test_ids.index.json` (test-id catalog / lookup),
  - `script.result.json` + `bundle.index.json.script` (script step → snapshot mapping for fast evidence lookup, script dumps only).
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
