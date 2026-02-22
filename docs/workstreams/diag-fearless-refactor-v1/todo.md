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
- [x] Move UI gallery code-editor checks out of `crates/fret-diag/src/stats.rs` into
      `crates/fret-diag/src/stats/ui_gallery_code_editor.rs`.

## M2: Shrink + index artifacts (sidecars over monolithic JSON)

- [x] Define the “minimum useful bundle” contract (what must be in `bundle.json` vs what can be in sidecars).
  - `docs/workstreams/diag-fearless-refactor-v1/minimum-useful-bundle.md`
- [x] Add query-friendly indexes (sidecars) for tools/agents (implemented in `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_dump.rs`):
  - `bundle.index.json` (snapshot selectors, semantics fingerprints, test-id bloom),
  - `bundle.meta.json` (bundle-level counters + uniqueness summaries),
  - `test_ids.index.json` (test-id catalog / lookup),
  - `script.result.json` + `bundle.index.json.script` (script step → snapshot mapping for fast evidence lookup, script dumps only).
- [x] Make sidecars forward-compatible:
  - versioned schema (`kind` + `schema_version`),
  - additive-only evolution,
  - documented failure behavior when sidecars are missing,
  - CLI-side validation + on-demand regeneration when invalid.
  - `docs/workstreams/diag-fearless-refactor-v1/sidecar-schema-policy.md`
  - Evidence anchors:
    - `crates/fret-diag/src/commands/sidecars.rs`
    - `crates/fret-diag/src/commands/query.rs`
    - `crates/fret-diag/src/commands/index.rs`
    - `crates/fret-diag/src/commands/artifacts.rs`
    - `crates/fret-diag/src/lib.rs`

## M2b: Sidecar access is fully centralized (finish the sweep)

- [x] Migrate remaining commands to the shared sidecar helpers (start with `crates/fret-diag/src/commands/slice.rs`).
- [x] Add a small “doctor” command that reports missing/invalid sidecars and suggests the exact regen command.
  - Evidence: `crates/fret-diag/src/commands/doctor.rs`
  - Related: `fretboard diag test-ids-index <bundle>` (explicit generator for `test_ids.index.json`).
  - Bonus: `diag ai-packet` now writes `doctor.json` into the packet for agent-friendly preflight.
  - Agent ergonomics: `diag doctor --fix` can materialize `bundle.json` from manifest chunks (when present) and regenerate common sidecars.

## M3: Tooling + AI loop

- [ ] Define CLI “agent presets” (commands + env vars) for repeatable triage.
- [ ] Prefer structured evidence diffs over screenshot diffs where possible.
- [ ] Document a recommended script authoring style for stability (selectors first, bounded waits).

## M4: Remove debt (finish migration, delete redundant code)

Goal: after the mechanical moves land, **remove the temporary shims** and delete redundant/legacy code paths so the
diagnostics stack stays easy to evolve.

- [ ] Tighten module boundaries in `ecosystem/fret-bootstrap/src/ui_diagnostics/` (no backsliding into the monolith):
  - Script execution / state / step handlers live under `script_engine.rs` + `script_steps_*`.
  - Bundle dumping + sidecar writers live under `bundle_dump.rs` / `bundle_index.rs`.
  - DevTools WS wiring lives under `ui_diagnostics_devtools_ws.rs`.
- [ ] Delete transitional glue after migration:
  - remove duplicated per-step dispatch code from the old location(s),
  - avoid “forwarder wrappers” that exist only because of historical file layout.
- [ ] Remove redundant semantics traversal helpers in gates:
  - prefer `crate::json_bundle::SemanticsResolver` + shared helpers (no `debug.semantics.nodes` re-greps).
- [ ] Reduce “stats mega-module” churn permanently:
  - keep `crates/fret-diag/src/stats.rs` as a small index/exports surface,
  - large check families stay in `crates/fret-diag/src/stats/*.rs`.
- [ ] Audit and remove dead/legacy code paths once consumers have migrated:
  - legacy env knobs that are no longer used,
  - legacy schema compatibility layers that are no longer needed for in-tree workflows.

## Evidence anchors (keep updated as implementation changes)

- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_types.rs`
- `crates/fret-diag/src/stats.rs`
- `crates/fret-diag/src/stats/ui_gallery_markdown_editor.rs`
- `crates/fret-diag/src/stats/ui_gallery_code_editor.rs`
