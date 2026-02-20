# Control Chrome Normalization Audit v1 — Milestones

Status: Active

This file is the milestone tracker for:
`docs/workstreams/control-chrome-normalization-audit-v1.md`

---

## M0 — Contract baseline (unit-level)

Exit criteria:

- `control_chrome_*` is documented as the canonical composition helper for controls with:
  “outer pressable drives hit box; inner chrome paints background/corners/shadow”.
- Unit tests lock the sizing invariant matrix (see TODO file):
  - pressable `flex.grow > 0` ⇒ chrome `Fill` (already present)
  - pressable `w/h = Fill` ⇒ chrome `Fill` (both axes independently)
  - pressable `w/h = Px(_)` ⇒ chrome `Fill`, and min/max shrink math is correct
  - min/max interactions with chrome padding + border are covered

Deliverables:

- Expanded test matrix in `ecosystem/fret-ui-kit/src/declarative/chrome.rs`

---

## M1 — Repository audit (classification + risk)

Exit criteria:

- A repo-wide audit table exists and is kept current:
  `docs/workstreams/control-chrome-normalization-audit-v1-todo.md`
- Every `pressable_with_id_props` callsite that paints a background/border/radius in a child node
  is classified as one of:
  - Uses `control_chrome_*` (preferred)
  - Replicates equivalent normalization explicitly (acceptable, but should be justified)
  - At risk (needs migration)

Deliverables:

- Initial audit table populated with highest-impact components first (Tabs, ButtonGroups,
  Menu triggers/items, Dialog triggers, etc.).

---

## M2 — High-impact migrations (ecosystem)

Exit criteria:

- “At risk” callsites for high-visibility primitives/components are migrated to `control_chrome_*`
  or an equivalent shared helper.
- No known instances remain where outer pressable can stretch while inner chrome stays shrink-wrapped.

Deliverables:

- Targeted refactors in `ecosystem/fret-ui-shadcn`, `ecosystem/fret-ui-material3`, and top-level
  apps that use ad-hoc pressable+chrome patterns.

---

## M3 — Component-level regression gates (diagnostics)

Exit criteria:

- Add 2–3 diagnostic gates that catch “hit/visual separation” regressions in realistic component
  compositions:
  - Tabs triggers in a stretched layout
  - ButtonGroup/ToggleGroup with flex-fill items
  - Dialog trigger(s) inside flex layouts

Deliverables:

- A small `fretboard diag` script (or equivalent) with:
  - screenshot evidence and/or bounds assertions
  - stable `test_id` anchors

---

## M4 — Prevention (make the safe path the default)

Exit criteria (choose one, or both if feasible):

- Introduce a lightweight lint/check or scripted audit that flags new “at risk” patterns.
- Improve docs for component authors so new controls naturally adopt `control_chrome_*`.

Deliverables:

- Documentation updates and/or a simple `rg`-based check in `tools/` (if appropriate).

