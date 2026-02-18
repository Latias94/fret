# Material 3 Expressive Alignment v1 — TODO Tracker

Status: Active (workstream tracker)

This document tracks cross-cutting TODOs for:

- `docs/workstreams/material3-expressive-alignment-v1.md`
- `docs/workstreams/material3-expressive-alignment-v1-milestones.md`

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `M3X-{area}-{nnn}`

When completing an item, prefer leaving 1–3 evidence anchors:

- upstream reference (file + symbol, or doc)
- in-tree owner (file + key symbol)
- regression gate (unit test and/or `tools/diag-scripts/*.json`)

## M0 — Guardrails + evidence discipline

- [x] M3X-guard-001 Establish bounded diag workflow (no grepping `bundle.json`).
  - Evidence:
    - `docs/ui-diagnostics-and-scripted-tests.md`
    - `apps/fretboard` (`diag meta/query/slice`)

- [ ] M3X-guard-002 Define the “always-run” gate set for Material3 work (fast inner-loop).
  - Candidate:
    - `cargo nextest run -p fret-ui-material3 -E <switch|textfield subset>`
    - `cargo run -p fretboard -- diag script lint tools/diag-scripts/ui-gallery-material3-*.json`

- [ ] M3X-guard-003 Decide how we version upstream references (Material Web token versions, Compose versions).
  - Goal: keep token source drift explainable (not “mystery numbers”).

## M1 — Switch (toggle) parity tightening

- [x] M3X-switch-001 Crossfade selected/unselected chrome during toggle (67ms linear).
  - Evidence:
    - `ecosystem/fret-ui-material3/src/switch.rs`
    - `tools/diag-scripts/ui-gallery-material3-switch-chrome-crossfade-timeline-screenshots.json`

- [x] M3X-switch-002 Split focus chroming selectors (handle/icons focus-within vs unselected track focus-visible).
  - Evidence:
    - `ecosystem/fret-ui-material3/src/switch.rs`
    - `tools/diag-scripts/ui-gallery-material3-switch-focus-chroming-screenshots.json`

- [~] M3X-switch-003 Decide + implement the switch “overshoot” position motion source of truth (Material Web vs Compose).
  - Material Web: handle-container margin transition with overshoot bezier.
  - Compose: motion scheme + springy thumb offset/size.
  - Decision (v1): match Material Web’s handle-container overshoot for position; keep size/pressed behavior iterative.
  - Gate: fixed-timestep timeline screenshots (`fretboard diag run --fixed-frame-delta-ms 16`).

- [x] M3X-switch-004 Align focus state-layer policy (focus ring only; no focus state layer).
  - Rationale: Material Web switch ripple/state layer only expresses hover/pressed; focus is via focus ring + chroming.
  - Evidence:
    - `ecosystem/fret-ui-material3/src/switch.rs`
    - `tools/diag-scripts/ui-gallery-material3-switch-focus-visible-screenshots.json`

## M2 — Text field token/state parity

- [x] M3X-textfield-001 Honor outlined hover tokens (track/outline + container hover).
  - Evidence:
    - `ecosystem/fret-ui-material3/src/text_field.rs`
    - `tools/diag-scripts/*text-field*` (if present)

- [ ] M3X-textfield-002 Audit focused/error/disabled token fallbacks vs upstream.
  - Gate: headless goldens for state matrix + one diag script for focus routing (mouse vs keyboard).

## M3 — Buttons + IconButton (recipe + motion)

- [ ] M3X-button-001 Define the Material3 button taxonomy in `fret-ui-material3` (scopes + names).
- [ ] M3X-button-002 Tokenize density/height/padding and lock a baseline in UI gallery.
- [ ] M3X-button-003 Motion + interaction: pressed, hover, focus ring, disabled (gate with scripts).

## M4 — Overlay-driven components (menus, dialogs, snackbars)

- [ ] M3X-overlay-001 Identify overlay policy ownership boundaries (dismiss rules, focus trap/restore).
  - Owner: `ecosystem/fret-ui-kit` (policy), `ecosystem/fret-ui-material3` (recipes).
  - Gate: `fretboard diag` scripts for dismiss + focus restore invariants.

- [ ] M3X-overlay-002 Material3 menu surface: anchor, offset, collision/flip, keyboard navigation.
- [ ] M3X-overlay-003 Dialog: focus trap + escape/outside click policy + scroll locking (as applicable).
- [ ] M3X-overlay-004 Snackbar: queueing policy + action button semantics.
