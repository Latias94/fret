---
title: Material 3 Expressive Alignment (v1)
status: active
date: 2026-02-18
scope: ecosystem/fret-ui-material3, ecosystem/fret-ui-kit, tools/diag-scripts
---

## Upstream references (non-normative)

This workstream references optional local snapshots under `repo-ref/` for convenience.
Upstream sources:

- Material Web: https://github.com/material-components/material-web
- MUI Material UI: https://github.com/mui/material-ui
- Compose Material3: https://github.com/JetBrains/compose-multiplatform-core (see `compose/material3`)
- MUI Base UI: https://github.com/mui/base-ui
- Radix Primitives (for interaction policy comparisons only): https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

# Material 3 Expressive Alignment (v1) — Workstream

This workstream tracks aligning Fret’s Material recipe layer to **Material 3 (Expressive)** outcomes.
The goal is to enable building editor-grade and app-grade UIs with Material behavior without leaking
policy into `crates/*`.

Key constraints:

- `crates/*` stays mechanism/contract only (focus, semantics, layout, overlays, rendering boundaries).
- Material policy and recipes live in `ecosystem/*`:
  - tokens + recipes: `ecosystem/fret-ui-material3`
  - headless policy primitives + motion drivers: `ecosystem/fret-ui-kit`
- Diagnostics is the enforcement tool:
  - scripts: `tools/diag-scripts/*.json`
  - evidence packs: `fretboard diag run --pack`

Milestone board (one-screen): `docs/workstreams/material3-expressive-alignment-v1-milestones.md`.
TODO board (detailed): `docs/workstreams/material3-expressive-alignment-v1-todo.md`.

## What “alignment” means (definition of done)

For each component we consider aligned when we have:

1. A clear upstream source-of-truth for the mismatch class:
   - tokens/layout/density → Material + MUI + Compose (pick one primary)
   - interaction policy → Compose + Base UI (avoid DOM-only assumptions)
   - motion → Compose motion scheme + Material Web durations/easing where relevant
2. The implementation lives in the correct layer (mechanism vs policy vs recipe).
3. A regression gate:
   - unit test for deterministic geometry/state invariants, and/or
   - `fretboard diag` script with stable `test_id` selectors (+ fixed timestep when motion matters).
4. 1–3 evidence anchors: upstream file(s), in-tree owner symbol(s), and gate(s).

## Repo mapping (where changes should land)

- Mechanisms (rare): `crates/fret-ui`, `crates/fret-runtime`, `crates/fret-render`
- Policy primitives (common): `ecosystem/fret-ui-kit`
  - dismiss/focus-restore, roving focus, keyboard routing, motion drivers
- Material recipe layer (default): `ecosystem/fret-ui-material3`
  - token keys (`md.comp.*`) and token fallbacks
  - component recipes (composition + styling)
  - stable `test_id` surfaces for gates

## Current implementation anchors

- Switch recipe: `ecosystem/fret-ui-material3/src/switch.rs`
- Switch tokens: `ecosystem/fret-ui-material3/src/tokens/switch.rs`
- Token source mapping (Material Web v30): `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`
- Text field recipe: `ecosystem/fret-ui-material3/src/text_field.rs`
- UI gallery previews: `apps/fret-ui-gallery/src/ui/previews/material3/`

## Evidence packs and scripts (start here)

- Switch baseline screenshots + bundle:
  - `tools/diag-scripts/ui-gallery-material3-switch-handle-screenshots.json`
- Switch chrome crossfade timeline (fixed timestep recommended):
  - `tools/diag-scripts/ui-gallery-material3-switch-chrome-crossfade-timeline-screenshots.json`
- Switch focus chroming screenshots:
  - `tools/diag-scripts/ui-gallery-material3-switch-focus-chroming-screenshots.json`
- Switch focus-visible screenshots:
  - `tools/diag-scripts/ui-gallery-material3-switch-focus-visible-screenshots.json`
- Switch handle overshoot timeline:
  - `tools/diag-scripts/ui-gallery-material3-switch-handle-overshoot-timeline-screenshots.json`
- Switch icon motion timeline:
  - `tools/diag-scripts/ui-gallery-material3-switch-icon-motion-timeline-screenshots.json`
- Text field hover screenshots:
  - `tools/diag-scripts/ui-gallery-material3-text-field-outlined-hover-screenshots.json`

## Component coverage (v1 target set)

This workstream is intentionally incremental. The initial target set focuses on controls that
exercise token/state/motion layering boundaries:

- Switch (toggle) / Checkbox / Radio
- Buttons (filled/outlined/tonal/text) + IconButton
- Text fields (filled/outlined) + supporting/error states
- Slider
- Menus (incl. focus + dismiss policy) and Select/Combobox-like recipes
- Dialog + Snackbar (overlay behavior, focus trap/restore, dismiss policy)

Detailed sequencing and ownership lives in the TODO + milestones docs.
