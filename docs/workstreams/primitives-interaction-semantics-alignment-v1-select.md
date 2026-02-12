# Primitives Interaction Semantics Alignment v1 — Select (Audit Sheet)

Status: Active (workstream note; not a contract)

This sheet audits the `Select` family against Radix outcomes (baseline) with optional Base UI
anti-misclick policies, mapped onto Fret’s non-DOM model (overlay layers, semantics tree, runner
input dispatch).

Workstream:

- Overview: `docs/workstreams/primitives-interaction-semantics-alignment-v1.md`
- Progress matrix: `docs/workstreams/primitives-interaction-semantics-alignment-v1-matrix.md`

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/select.tsx`
- Upstream Radix primitive: `repo-ref/primitives/packages/react/select/src/*`

---

## Current Fret implementation anchors

- Primitive/policy: `ecosystem/fret-ui-kit/src/primitives/select.rs`
- shadcn recipe: `ecosystem/fret-ui-shadcn/src/select.rs`
  - Explicit knobs: `SelectMousePolicies`, `SelectMouseUpSelectionGatePolicy`

Scripted repros / bundles (timeline-style):

- `tools/diag-scripts/ui-gallery-shadcn-select-commit.json`
- `tools/diag-scripts/ui-gallery-select-commit-and-label-update.json`
- `tools/diag-scripts/ui-gallery-select-keyboard-commit-apple.json`
- `tools/diag-scripts/ui-gallery-select-typeahead-commit-banana.json`
- `tools/diag-scripts/ui-gallery-select-disabled-item-no-commit.json`
- `tools/diag-scripts/ui-gallery-select-roving-skips-disabled-orange.json`
- `tools/diag-scripts/ui-gallery-select-wheel-scroll.json`
- `tools/diag-scripts/ui-gallery-select-wheel-up-from-bottom.json`

---

## Outcome model (what we must preserve)

State:

- `open` (controllable/uncontrolled)
- `value` (typically recipe-owned; primitive provides helpers)
- `active/highlight` (roving focus / active descendant depending on surface)
- “mouse-open guard armed” (to suppress the matching `pointerup`)

Key reasons:

- open reason (e.g. trigger press, open keys)
- dismiss reason (escape/outside press/focus outside/scroll)
- commit reason (enter/item press/programmatic)
- optional “cancel open” reason (Base UI style)

Invariants (examples):

- Do not accidentally commit an item on the same click that opened the popup.
- Pointer vs keyboard/touch have distinct open/commit paths.
- Outside interactions are blocked when Select is modal (Radix-like outcome), and configurable
  otherwise.
- Typeahead and roving/highlight do not unexpectedly mutate selection without an explicit commit.

---

## Audit checklist (dimension-driven)

Mark each item with one of:

- `-` not audited
- `M` modeled
- `I` implemented
- `G` gated (tests/scripts prevent regressions)

### Model

- [ ] `M` Document open/close/commit reasons and the allowed transitions.
- [ ] `M` Identify which behaviors are “Radix default” vs “Base UI optional knobs”.

### Policy (Trigger / Listbox / Commit)

- [ ] `I` TriggerPolicy: mouse/touch/keyboard open behavior (including “open keys”).
- [ ] `I` ListboxPolicy: highlight navigation + typeahead + scroll-into-view.
- [ ] `I` SelectionCommitPolicy: enter/click commit + close-on-commit semantics.
- [ ] `I` Make anti-misclick knobs explicit (instead of ad-hoc recipe wiring):
  - [x] `I` `pointer_up_guard` (Radix default)
  - [x] `I` `mouse_up_selection_gate` (optional)
  - [ ] `-` `cancel_open_on_mouseup_outside` (optional)

### Focus

- [ ] `I` Initial focus target on open (selected item vs content).
- [ ] `I` Focus restore on close is reason-aware and respects `preventDefault`-style outcomes.

### Dismiss

- [ ] `I` Escape closes (with reason).
- [ ] `I` Outside press closes with correct “click-through vs barrier” behavior.
- [ ] `I` Focus-out closes only when intended (overlay focus routing).

### Pointer

- [ ] `I` Mouse `pointerdown` open does not cause “same click” selection commit (guard).
- [ ] `I` Touch opens on click-like up (no drag open).
- [ ] `-` Base UI cancel-open (mouseup outside after mousedown-open) is possible as an opt-in.

### Keys

- [ ] `I` Arrow nav and open keys match Radix outcomes.
- [ ] `I` Typeahead while open highlights correctly; while closed is explicitly chosen.

### A11y (semantics)

- [ ] `I` Trigger semantics: `role=ComboBox`, expanded/controls wired.
- [ ] `I` Content semantics: `role=ListBox`; options expose selected/disabled/highlight state.
- [ ] `I` Active-descendant/roving mapping is correct for AccessKit.

### Placement / size

- [ ] `I` Item-aligned positioning matches Radix (including scroll button mount sequencing).
- [ ] `G` Tight viewport clamping (max height/width) is gated against shadcn-web goldens when relevant.

### Time

- [ ] `M` Any delays/gates are `Duration` and reflect a named semantic (not “magic ms”).

### Tests / gates

- [ ] `G` Unit tests gate pointer-up suppression and key/pointer transitions.
- [ ] `G` At least one diag script gates open → commit without misclick regressions.

---

## Notes / next actions

- The primitive already contains significant Radix-shaped behavior and tests; the main gap is making
  Base UI style “stronger anti-misclick” behaviors explicit as opt-in policies, not recipe hacks.
