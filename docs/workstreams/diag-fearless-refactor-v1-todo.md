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
  - [x] Extract active-window heartbeat writer helper:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`
  - [x] Extract progress writer + pending cross-window drag cancel helper:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`
  - [x] Extract per-step evidence scoping + step-state reset helpers:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`
  - [x] Extract window/cursor/mouse/insets effect-only steps:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs`
  - [x] Extract non-window “effect-only” steps (reset/wait/clipboard/open-inject):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs`
  - [x] Extract capture steps (bundle + screenshot export requests):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs`
  - [x] Extract keyboard/text injection steps (press/type/ime):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_input.rs`
  - [x] Extract selector-driven text input step (`type_text_into`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_input.rs`
  - [x] Extract menu selection step (`menu_select`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_menu.rs`
  - [x] Extract scroll-into-view step (`scroll_into_view`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_scroll.rs`
  - [x] Extract ensure-visible step (`ensure_visible`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_visibility.rs`
  - [x] Extract wheel step (`wheel`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer.rs`
  - [x] Extract click step (`click`) with window handoff + fail-fast behavior:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer.rs`
  - [x] Extract move pointer step (`move_pointer`) with fail-fast behavior:
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer.rs`
- [x] Extract stable click steps (`click_stable`, `click_selectable_text_span_stable`):
  - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer.rs`
- [x] Extract assert step (`assert`):
  - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_assert.rs`
- [x] Extract pointer session steps (`pointer_down`, `pointer_move`, `pointer_up`):
  - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer_session.rs`
  - [x] Extract drag steps (`drag_pointer`, `drag_pointer_until`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_drag.rs`
  - [x] Extract drag-to step (`drag_to`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_drag.rs`
  - [x] Extract pointer sweep step (`move_pointer_sweep`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_pointer_sweep.rs`
  - [x] Extract wait step (`wait_bounds_stable`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs`
  - [x] Extract menu path step (`menu_select_path`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_menu.rs`
  - [x] Extract slider step (`set_slider_value`):
    - [x] `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_slider.rs`
- [ ] Extract “inspect/pick state machine” into a module (keep UI/UX policy out of `fret-ui`).
- [ ] Keep DevTools WS wiring isolated (already split; ensure minimal coupling).

## Bundle size & AI loops (Plan 1)

- [ ] Document recommended defaults for AI loops (env presets) and keep them consistent across tooling and runtime.
- [ ] Ensure `bundle.index.json` / `test_ids.index.json` generation is part of the “pack/repro” happy path (where appropriate).
  - [x] Runtime writes `bundle.index.json` + `bundle.meta.json` on dump (native filesystem).
  - [ ] Tooling consumes `bundle.index.json` for fast queries/slicing.
  - [ ] Generate `test_ids.index.json` (or equivalent) as a first-class sidecar.
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
