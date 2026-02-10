# Primitives Interaction Semantics Alignment v1 — Milestones

Status: Active (workstream note; not a contract)

This workstream aligns `Select` and `Combobox` semantics by porting upstream outcomes (Radix + Base
UI), expressed as explicit state machines and composable policies in `ecosystem/fret-ui-kit`.

Reference: `docs/workstreams/primitives-interaction-semantics-alignment-v1.md`

---

## M0 — Audit + model (docs-first)

Definition of done:

- The desired outcome/state-machine model is written down for Select + Combobox.
- Policy split is agreed (TriggerPolicy / ListboxPolicy / SelectionCommitPolicy).
- Each “knob” is named and mapped to the correct layer (`fret-ui` vs `fret-ui-kit` vs shadcn recipe).

Evidence:

- `docs/workstreams/primitives-interaction-semantics-alignment-v1.md`
- `docs/workstreams/primitives-interaction-semantics-alignment-v1-todo.md`

---

## M1 — Select policy parameterization (Radix default)

Goal: make existing behavior explicit and configurable without changing defaults.

Definition of done:

- `fret-ui-kit::primitives::select` exposes an explicit config surface for:
  - `pointer_up_guard` (Radix default)
  - `mouse_up_selection_gate` (optional)
  - `cancel_open_on_mouseup_outside` (optional; default off)
- Unit tests gate the invariants of each knob.

---

## M2 — Combobox primitive extraction (Base UI baseline)

Goal: move the core state machine out of the shadcn recipe.

Definition of done:

- New module: `ecosystem/fret-ui-kit/src/primitives/combobox.rs`.
- shadcn recipe uses the primitive module for:
  - open-change reasons,
  - value-change gating,
  - close reason → focus restore policy hook (even if default is “Base UI-like”).
- At least a minimal set of unit tests exist at the primitives layer.

---

## M3 — Regression gates (scripts + goldens)

Definition of done:

- At least 1 `fretboard diag` script per primitive for a multi-step scenario:
  - Select: open → pointer-up suppression → commit selection.
  - Combobox: type → navigate highlight → commit → close + focus restore.
- When layout/style outcomes are sensitive, add/extend shadcn-web golden parity tests for the demo.

---

## M4 — Expand the template to other primitives

Definition of done:

- The same policy split template is applied to at least one additional overlay/listbox-like family
  (e.g. DropdownMenu or NavigationMenu) without duplicating state machines in recipes.

