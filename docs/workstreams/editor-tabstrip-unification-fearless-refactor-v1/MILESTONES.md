# Editor TabStrip Unification Fearless Refactor v1 (Milestones)

## M0 — Baseline + parity map

Deliverables:
- A written “Fret vs Zed vs Dockview” parity matrix focused on editor tab UX.
- A list of missing behaviors (ranked by user-impact and implementation risk).
- A written “owner layer map” for each behavior (mechanism vs policy).

Exit criteria:
- All gaps have an owner layer decision (mechanism in `fret-ui-headless` vs policy in ecosystem).

## M1 — Overflow dropdown correctness

Deliverables:
- Consistent overflow membership (viewport + margin) across workspace and docking.
- Overflow dropdown items are deterministic and policy-driven per adapter.
- Selecting an item scrolls the strip so the tab becomes visible.
- Overflow dropdown close affordances (if enabled) do not implicitly activate tabs.

Gates:
- `cargo nextest run -p fret-docking -p fret-workspace`
- Add a `fretboard diag` script that opens overflow dropdown, selects a tab, and records evidence.

## M2 — Drag/drop + end-drop surfaces closure

Deliverables:
- Shared surface classification vocabulary for both implementations.
- End-drop insert index resolves in canonical order (group-local).

Gates:
- Existing docking arbitration scripts remain passing:
  - `tools/diag-scripts/docking/arbitration/`
- Add/extend a workspace diag script for end-drop + overflow surfaces.

## M3 — Editor keyboard/focus semantics (limited)

Deliverables:
- Keyboard navigation invariants documented (what keys we support, where policy lives).
- Focus restore rules for tab close / switch validated in at least one demo/diag script.

Gates:
- Unit tests for headless helpers remain stable.
- Diag evidence bundle recorded for one “keyboard navigation” scenario.
