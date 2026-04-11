# M0 Baseline Audit — 2026-04-11

Status: closed baseline note

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/device-shell-adaptive-facade-promotion-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/device-shell-strategy-surface-v1/TARGET_INTERFACE_STATE.md`

## Assumptions-first baseline

### 1) This must be a new follow-on, not a reopened closed lane

- Area: lane state
- Assumption: wrapper growth is explicitly deferred by both previously closed device-shell lanes,
  so the work should start in a new narrow folder.
- Evidence:
  - `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
  - `docs/workstreams/device-shell-adaptive-facade-promotion-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- Confidence: Confident
- Consequence if wrong: the repo would blur a new wrapper question back into already-closed owner
  split or facade-promotion history.

### 2) `Combobox` is the only real recipe-owned device-shell wrapper today

- Area: current public surface
- Assumption: no other `fret-ui-shadcn` surface currently exposes public `device_shell_*` wrapper
  APIs above the shared helper.
- Evidence:
  - `ecosystem/fret-ui-shadcn/src/combobox.rs`
  - `rg -n "pub fn .*device_shell" ecosystem/fret-ui-shadcn/src`
- Confidence: Confident
- Consequence if wrong: the lane would be under-auditing an already-shipped second wrapper.

### 3) `Date Picker` and `Breadcrumb` are still app-local helper proofs, not wrapper candidates

- Area: gallery proof surfaces
- Assumption: the shipped device-shell helper consumers remain explicit app/gallery compositions
  that should stay reviewable at the call site instead of being wrapped immediately.
- Evidence:
  - `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
  - `docs/workstreams/device-shell-strategy-surface-v1/M3_SECOND_CONSUMER_PROOF_2026-04-11.md`
- Confidence: Confident
- Consequence if wrong: the lane would miss a wrapper candidate that already repeats enough policy
  to deserve recipe ownership.

### 4) `Dialog` and `Sidebar` remain explicit non-wrapper boundaries

- Area: boundary preservation
- Assumption: `Dialog` vs `Drawer` and `SidebarProvider::is_mobile(...)` are still intentionally
  excluded from wrapper growth.
- Evidence:
  - `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
  - `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
  - `docs/workstreams/device-shell-strategy-surface-v1/TARGET_INTERFACE_STATE.md`
- Confidence: Confident
- Consequence if wrong: this lane could accidentally widen docs/proof or app-shell surfaces into
  the wrong owner layer.

### 5) The smallest landable slice is owner alignment of the existing wrapper, not a new public API

- Area: execution plan
- Assumption: the next useful step is to make `Combobox` delegate shell classification to the
  shared helper owner and add a source gate, without inventing another wrapper.
- Evidence:
  - `ecosystem/fret-ui-kit/src/adaptive.rs`
  - `ecosystem/fret-ui-shadcn/src/combobox.rs`
  - `docs/workstreams/device-shell-strategy-surface-v1/TARGET_INTERFACE_STATE.md`
- Confidence: Likely
- Consequence if wrong: the lane could spend effort on internal cleanup that does not actually
  reduce drift or clarify the wrapper boundary.

## Immediate implication

This lane does not need a second wrapper or a broader API audit to start.

It only needs to answer one narrow question:

- should the repo keep `Combobox` as the sole current recipe-owned device-shell wrapper exemplar
  while leaving other surfaces on the shared helper or explicit proof lanes?

The baseline answer is yes, and the smallest landing slice is:

1. align the existing `Combobox` wrapper internals with the shared helper owner,
2. add a focused wrapper-vs-helper source gate,
3. and close the lane unless a second stable wrapper candidate appears.
