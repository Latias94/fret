# Material3 Expressive Alignment v1

Status: Active (workstream note; not a contract)

This workstream aligns Fret's Material 3 component ecosystem to upstream behaviors and tokens (including
Material 3 “Expressive”), with regression protection (tests + diag scripts + evidence bundles).

Trackers:

- Plan: `docs/workstreams/material3-expressive-alignment-v1-refactor-plan.md`
- TODO: `docs/workstreams/material3-expressive-alignment-v1-todo.md`
- Milestones: `docs/workstreams/material3-expressive-alignment-v1-milestones.md`

Upstream references (local snapshots under `repo-ref/`):

- MUI Material UI: `repo-ref/material-ui/`
- Compose Material3: `repo-ref/compose-multiplatform-core/compose/material3/`
- Material Web (tokens + CSS implementation detail): `repo-ref/material-web/`
- Base UI (headless patterns): `repo-ref/base-ui/`
- Radix primitives (for interaction policy comparisons): `repo-ref/primitives/`

---

## Goals

1. Match Material 3 behavior outcomes (states, motion, density, semantics).
2. Keep layering clean:
   - mechanism/contract work stays in `crates/*`,
   - policy/state machines live in `ecosystem/fret-ui-kit`,
   - Material recipes + token mapping live in `ecosystem/fret-ui-material3`.
3. Make parity reviewable:
   - record 1–3 evidence anchors per change (upstream file/symbol, in-tree owner path/symbol, gate path).
4. Prevent regressions:
   - unit tests for deterministic token/state logic,
   - headless suites + goldens for stable token-to-style mapping,
   - `fretboard diag` scripts (stable `test_id`) for interaction + motion.

---

## Non-goals

- Recreating DOM/Compose APIs 1:1; we only port behavior outcomes.
- Leaking component policy into `crates/fret-ui` unless it is a true mechanism/contract.
- Locking a “final” Material component API surface; recipes may evolve as long as behavior stays aligned.

---

## Source-of-truth ordering

When references disagree, prefer:

1. Material 3 guidelines/spec intent (when unambiguous),
2. Compose Material3 (GPU/toolkit style state machines + semantics),
3. MUI Material UI (web interaction quirks, defaults, density),
4. Material Web implementation detail (tokens + CSS fallback chains),
5. Base UI / Radix as headless pattern references (accessibility + composition).

---

## Fearless refactor plan

The detailed “fearless refactor” workflow for this workstream lives in:
`docs/workstreams/material3-expressive-alignment-v1-refactor-plan.md`.

---

## Regression gates and evidence

Preferred gate stack:

- Unit tests near token/policy owners.
- Headless suites with explicit golden update workflow (`FRET_UPDATE_GOLDENS=1` only when changes are intended).
- `tools/diag-scripts/*.json` (schema v2) that:
  - navigates via stable `test_id`,
  - captures at least one `capture_bundle`,
  - uses screenshots only when they add signal.

Diagnostics hygiene:

- Do not grep or print `bundle.json` directly (it is large).
- Use bounded helpers:
  - `cargo run -p fretboard -- diag meta <bundle_dir|bundle.json> --json`
  - `cargo run -p fretboard -- diag query test-id <bundle_dir|bundle.json> <pattern> --mode contains --top 50`
  - `cargo run -p fretboard -- diag slice <bundle_dir|bundle.json> --test-id <test_id>`

---

## Component inventory (initial)

This list is intentionally incomplete; expand it as we land parity work:

- Toggle/Switch
- Slider
- TextField (filled/outlined)
- Checkbox / Radio
- Buttons (text/filled/tonal/outlined)
- Chips (assist/filter/input/suggestion)
- FAB / Extended FAB
- Tabs
- Menus / Select
- Dialog / Bottom sheet (if applicable)
- Navigation components

Each component change must declare:

- upstream reference(s),
- owning layer(s),
- gate(s) added/updated,
- evidence bundle (if interaction/motion changed).
