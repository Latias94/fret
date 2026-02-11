# Primitives Interaction Semantics Alignment v1 — Sheet (Audit Sheet)

Status: Active (workstream note; not a contract)

Baseline: shadcn/ui v4 `Sheet` outcomes (Radix Dialog-shaped semantics + side placement + motion).

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sheet.tsx`
- Upstream Radix primitive (Dialog): `repo-ref/primitives/packages/react/dialog/src/*`

---

## Current Fret implementation anchors

- Primitive/mechanism substrate (barrier + focus/dismiss routing):
  - `ecosystem/fret-ui-kit/src/primitives/dialog.rs`
- shadcn recipe (side placement + motion + open-change callbacks):
  - `ecosystem/fret-ui-shadcn/src/sheet.rs`

Key implementation anchors (motion + open-change semantics):

- Open change callbacks with “changed” vs “completed” split:
  - `ecosystem/fret-ui-shadcn/src/sheet.rs` (`sheet_open_change_events`)
- Side placement and inherited side provider:
  - `ecosystem/fret-ui-shadcn/src/sheet.rs` (`SheetSide`, `with_sheet_side_provider`)
- Dialog mechanism surface used by Sheet:
  - `ecosystem/fret-ui-shadcn/src/sheet.rs` (`use fret_ui_kit::primitives::dialog as radix_dialog`)
- Reason-aware close auto-focus guard + default-close dismissal wrapper (Dialog substrate):
  - `ecosystem/fret-ui-kit/src/primitives/dialog.rs` (`DialogCloseAutoFocusGuardPolicy`, `dialog_close_auto_focus_guard_hooks`)
  - `ecosystem/fret-ui-shadcn/src/sheet.rs` (wires guard hooks into `modal_dialog_request_with_options_and_dismiss_handler`)

Related tests/gates:

- Shared dialog gates:
  - `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`
  - `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`
- Scripted repros:
  - `tools/diag-scripts/ui-gallery-sheet-escape-focus-restore.json` (gate: escape close + focus restore)

---

## Outcome model (what we must preserve)

State:

- `open`
- `present` / `animating` (presence-aware close completion)
- `side` (Left/Right/Top/Bottom) and derived placement

Reasons:

- open: trigger press / programmatic
- close: escape / barrier outside press / close button / programmatic

Invariants:

- Close completion events are presence-aware (no early “completed” while animating).
- Barrier hit-testing blocks underlay interaction when modal.
- Focus restore is stable and reason-aware (escape vs barrier vs programmatic).
- Side placement clamps correctly near viewport edges.

---

## Audit checklist (dimension-driven)

- [ ] `M` Document open/close reasons and “changed vs completed” callback contract.
- [ ] `M/I` Document side placement/clamp outcomes (and map to popper/collision primitives).
- [x] `G` Add a minimal diag gate: open → escape close → focus restore (and optionally barrier click).
