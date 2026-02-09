# Primitives Interaction Semantics Alignment v1 — Combobox (Audit Sheet)

Status: Active (workstream note; not a contract)

This sheet audits `Combobox` against Base UI outcomes (baseline via shadcn/ui v4), mapped onto
Fret’s non-DOM model (overlay layers, semantics tree, runner input dispatch).

Workstream:

- Overview: `docs/workstreams/primitives-interaction-semantics-alignment-v1.md`
- Progress matrix: `docs/workstreams/primitives-interaction-semantics-alignment-v1-matrix.md`

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/combobox.tsx`
  - Note: imports `@base-ui/react` (Base UI is the semantic baseline)
- Upstream Base UI source: `repo-ref/base-ui/packages/*` (exact module depends on the pinned revision)

---

## Current Fret implementation anchors

- Primitive semantics helpers (open/value change gating; reasons mapping):
  - `ecosystem/fret-ui-kit/src/primitives/combobox.rs`
- shadcn recipe (currently owns key semantics): `ecosystem/fret-ui-shadcn/src/combobox.rs`
- Headless helpers already used: `ecosystem/fret-ui-kit/src/headless/mod.rs` (`cmdk_selection`, `cmdk_score`)

Scripted repros / bundles:

- `tools/diag-scripts/ui-gallery-combobox-commit-pixels-changed.json`
- `tools/diag-scripts/ui-gallery-combobox-open-select-focus-restore.json`

---

## Outcome model (what we must preserve)

State:

- `open` (controllable/uncontrolled)
- `query` (input text, drives filtering; not equal to `value`)
- `value` (selected value; `onValueChange` only on real changes)
- `active/highlight` (keyboard + pointer hover)

Reasons:

- open/close reasons (e.g. trigger press, typing, arrow down, outside press, escape, focus out)
- commit reason (enter/item press)
- focus-restore policy keyed by close reason

Invariants (examples):

- `onValueChange` fires only when the value actually changes.
- closing reason drives focus restoration (and must be configurable).
- highlight navigation does not mutate `value` until commit.

---

## Audit checklist (dimension-driven)

### Model

- [ ] `M` Write down the open/close/commit transitions and reasons (Base UI shaped).

### Policy (Trigger / Listbox / Commit)

- [ ] `-` TriggerPolicy: open on trigger press / input focus / typing / arrow down (configurable).
- [ ] `-` ListboxPolicy: highlight navigation + scroll-into-view + typeahead (cmdk-like).
- [ ] `-` SelectionCommitPolicy: enter/click commit; close-on-commit; guard against misclick.

### Focus

- [ ] `M` Define reason-aware focus restore outcomes and make it configurable.

### Dismiss

- [ ] `M` Map Fret dismiss reasons to Base UI shaped reasons (`OutsidePress`, `EscapeKey`, `FocusOut`, …).

### Pointer

- [ ] `-` Decide click-through vs barrier outcomes for the popup (shadcn combobox is Popover + Command).
- [ ] `-` Ensure pointer interactions during close transitions behave as intended (no “stuck modal”).

### Keys

- [ ] `-` Arrow navigation and enter commit are consistent with Base UI outcomes.
- [ ] `-` Escape closes; typing updates query and opens as configured.

### A11y (semantics)

- [ ] `M` Active-descendant semantics are correct (input keeps focus, highlight moves).
- [ ] `M` Listbox/options roles and selected/highlight mapping are correct.

### Placement / size

- [ ] `M` Popup width anchors to trigger; height clamps to available viewport (shadcn v4 style).

### Time

- [ ] `M` Any delays are `Duration` and semantic (not recipe magic numbers).

### Tests / gates

- [ ] `I` Existing recipe tests cover at least: value-change gating and close-transition pointer behavior.
- [x] `I` Add primitive-level unit tests once extracted to `fret-ui-kit::primitives::combobox`.
- [ ] `I` Keep at least one diag script gating a multi-step commit flow.

---

## Notes / next actions

- The key refactor is to move the reusable state-machine helpers out of the shadcn recipe into a
  dedicated primitive module, so other ecosystems (or non-shadcn apps) can share semantics without
  copying recipe code.
