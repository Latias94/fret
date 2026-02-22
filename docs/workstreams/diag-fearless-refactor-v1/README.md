---
title: Diagnostics Fearless Refactor v1
status: draft
date: 2026-02-22
scope: diagnostics, automation, tooling, refactor
---

# Diagnostics Fearless Refactor v1

This workstream is about making the diagnostics + scripted UI automation stack easier to evolve **without** large rewrites, while
keeping the day-to-day debugging loop fast:

- smaller, more queryable artifacts (avoid “open a 200MB `bundle.json`”),
- more modular implementation (reduce churn in `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`),
- more modular CLI tooling (reduce churn in `crates/fret-diag/src/stats.rs`),
- better support for AI/agentic triage (deterministic evidence + stable indexes).

This workstream is intentionally scoped to refactors and additive sidecars. It must not change the core runtime contracts in
`crates/fret-ui` (mechanism only).

Related living docs:

- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/workstreams/diag-extensibility-and-capabilities-v1/README.md`

## Current state (evidence anchors)

- Diagnostics service + script runner live in `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`.
- Script step handlers are already split into dedicated modules under `ecosystem/fret-bootstrap/src/ui_diagnostics/`.
- Internal script runner state types were extracted into `ecosystem/fret-bootstrap/src/ui_diagnostics/script_types.rs` to reduce churn.
- The per-frame script driver (`UiDiagnosticsService::drive_script_for_window`) was extracted into
  `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`.
- `fret-diag` CLI dispatch remains centralized, but larger subcommands are being extracted into dedicated modules to reduce churn in
  `crates/fret-diag/src/lib.rs`:
  - `crates/fret-diag/src/diag_perf.rs` (extracted `diag perf` command handler)
- `crates/fret-diag/src/stats.rs` remains large, but UI gallery checks have started moving into dedicated submodules under
  `crates/fret-diag/src/stats/`:
  - `crates/fret-diag/src/stats/ui_gallery_markdown_editor.rs`
  - `crates/fret-diag/src/stats/ui_gallery_code_editor.rs`
- `fret-diag` CLI commands treat sidecars as optional accelerators:
  - validate `kind` / `schema_version` / `warmup_frames`,
  - accept `_root/` bundle layouts,
  - regenerate invalid sidecars from adjacent `bundle.json` when possible.
  - Evidence: `crates/fret-diag/src/commands/sidecars.rs`

## Goals

1. **Artifact ergonomics**
   - Keep `bundle.json` reviewable and bounded.
   - Add small sidecars for fast queries (indexing, fingerprints, bloom filters, step markers).
2. **Implementation modularity**
   - Split the monolithic `ui_diagnostics.rs` by responsibility (script engine, bundle dump, index writing, WS bridge).
3. **AI/agent-friendly debugging**
   - Make “what happened” explainable via structured evidence (selector traces, hit-test traces, routing traces).
   - Make “where to look” cheap: stable indexes that allow tools to filter/locate snapshots quickly.
4. **Debt removal (finish the migration)**
   - After extraction steps land, remove transitional forwarders and redundant code paths so the new module boundaries stick.

## Non-goals

- Replacing the canonical protocol types in `crates/fret-diag-protocol`.
- Introducing policy into mechanism crates (`crates/fret-ui`, `crates/fret-core`).
- Changing UI component behavior to satisfy diagnostics (components should expose `test_id`/semantics; runner/tooling adapts).

## Plan

See:

- TODO list: `docs/workstreams/diag-fearless-refactor-v1/todo.md`
- Milestones: `docs/workstreams/diag-fearless-refactor-v1/milestones.md`
- Migration plan: `docs/workstreams/diag-fearless-refactor-v1/migration-plan.md`
- Sidecar details: `docs/workstreams/diag-fearless-refactor-v1/frames-index.md`
- Agent loop: `docs/workstreams/diag-fearless-refactor-v1/agent-loop.md`
