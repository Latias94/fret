---
title: Diag Fearless Refactor v1 (TODO)
status: draft
date: 2026-02-21
scope: diagnostics, automation, bundle-schema, refactor
---

# Diag Fearless Refactor v1 (TODO)

This file tracks tasks for `docs/workstreams/diag-fearless-refactor-v1.md`.

## Runtime modularization (reduce `ui_diagnostics.rs` blast radius)

- [x] Extract filesystem triggers into a dedicated module:
  - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`
- [x] Extract “bundle writer” responsibilities (schema selection, semantics-mode application, JSON writing) into a module:
  - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_dump.rs`
- [ ] Extract “script runner” responsibilities (step state machine + evidence capture) into a module:
  - [x] Extract pending-script start/bootstrap helper:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`
  - [x] Extract single-active-script migration helper:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`
  - [x] Extract keepalive/heartbeat helper (when current window has no active script):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`
- [ ] Extract “inspect/pick state machine” into a module (keep UI/UX policy out of `fret-ui`).
- [ ] Keep DevTools WS wiring isolated (already split; ensure minimal coupling).

## Bundle size & AI loops (Plan 1)

- [ ] Document recommended defaults for AI loops (env presets) and keep them consistent across tooling and runtime.
- [ ] Ensure `bundle.index.json` / `test_ids.index.json` generation is part of the “pack/repro” happy path (where appropriate).
- [ ] Add a short “AI-first” recipe to `docs/ui-diagnostics-and-scripted-tests.md` that links to:
  - `diag meta`, `diag index`, `diag query`, `diag slice`, `diag ai-packet`.

## Schema migration hygiene

- [ ] Decide the migration policy for schema v1 → v2:
  - target dates for flipping defaults for manual dumps,
  - how long v1 stays supported by tooling,
  - criteria for deprecating v1-only fields.
- [ ] Add one regression guard that prevents re-introducing forked protocol types in the runtime runner.

## Plan 2 (defer until Plan 1 is solid)

- [ ] Prototype manifest-first chunked bundle layout (snapshots/logs/semantics split).
- [ ] Add a compatibility materializer to emit `bundle.json` from the manifest.
- [ ] Add packing/hashing conventions and a `diag pack` integration path.
