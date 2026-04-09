---
title: Material 3 Expressive Alignment (v1) — Milestones
status: active
date: 2026-02-18
scope: ecosystem/fret-ui-material3, ecosystem/fret-ui-kit, diag gates
---

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Material Web: https://github.com/material-components/material-web
- Compose Material3: https://github.com/JetBrains/compose-multiplatform-core (see `compose/material3`)
- MUI Material UI: https://github.com/mui/material-ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

# Material 3 Expressive Alignment (v1) — Milestones

This document is a **one-screen milestone board** for the Material 3 Expressive alignment workstream.

Source of truth for detailed TODOs: `docs/workstreams/material3-expressive-alignment-v1/material3-expressive-alignment-v1-todo.md`.
Narrative + layering rules: `docs/workstreams/material3-expressive-alignment-v1/material3-expressive-alignment-v1.md`.

## Definition of done (component-level)

We consider a component “aligned (v1)” when:

1. Token mapping is explicit and stable (`md.comp.*` keys + clear fallback chain).
2. Interaction policy lives in the correct layer (no policy leakage into `crates/*`).
3. Motion outcomes are deterministic under fixed timestep when gated.
4. At least one regression gate exists (unit test and/or `fretboard-dev diag` script).

## Milestones

### M0 — Foundations (tokens + motion + evidence)

Acceptance criteria:

- `ecosystem/fret-ui-material3` has a clear “token source of truth” story (versioned import + mapping).
- Motion drivers are reused from `ecosystem/fret-ui-kit` where appropriate (no duplicate tween engines).
- UI gallery surfaces provide stable `test_id` targets for scripts.
- A small set of diag scripts exists to capture state + motion evidence.

Status: In progress.

### M1 — Switch (toggle) parity baseline

Acceptance criteria:

- Switch chrome transitions match Material Web timing (selected/unselected color crossfade).
- Focus chroming rules match upstream selectors (handle/icons vs track split).
- Basic icon modes are supported and gated (`icons`, `show-only-selected-icon`).

Status: In progress (baseline exists; continue tightening).

Evidence anchors:

- Parity note: `docs/workstreams/material3/material3-switch-handle-parity-note.md`
- Scripts:
  - `tools/diag-scripts/ui-gallery-material3-switch-handle-screenshots.json`
  - `tools/diag-scripts/ui-gallery-material3-switch-chrome-crossfade-timeline-screenshots.json`
  - `tools/diag-scripts/ui-gallery-material3-switch-focus-chroming-screenshots.json`
  - `tools/diag-scripts/ui-gallery-material3-switch-icons-state-matrix-screenshots.json`

### M2 — Text field parity baseline

Acceptance criteria:

- Outlined/filled text fields match token-driven hover/focus/error/disabled outcomes.
- Keyboard routing + focus-visible behavior is gated via diag evidence.

Status: In progress.

### M3 — Core controls set (checkbox/radio/button/slider)

Acceptance criteria:

- Controls are recipe-complete in UI gallery with stable selectors.
- Token + interaction + motion deltas are documented with anchors.
- At least one gate per control family exists.

Status: Not started.

### M4 — Overlay-driven surfaces (menus/dialog/snackbar)

Acceptance criteria:

- Dismiss + focus trap/restore policy is centralized in `fret-ui-kit` primitives.
- Material3 overlay recipes exist in `fret-ui-material3`.
- Overlay correctness is gated via scripts (dismiss + focus restore + placement invariants).

Status: Not started.
