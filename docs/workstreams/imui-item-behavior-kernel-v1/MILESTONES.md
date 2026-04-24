# ImUi Item Behavior Kernel v1 Milestones

Status: closed execution plan
Last updated: 2026-04-24

## M0 - Baseline Audit

Status: complete on 2026-04-24.

Goal: prove the duplication problem and freeze the private kernel shape before editing behavior.

Exit criteria:

- Current behavior owners are listed by control family.
- The kernel input/output vocabulary is written down.
- Deleted/replaced helper candidates are named.
- First-slice gates are confirmed runnable or explicitly marked blocked.

## M1 - Button-Family Kernel Slice

Status: complete on 2026-04-24.

Goal: land the smallest behavior kernel that removes real duplication from button-like controls.

Exit criteria:

- Button-like controls route common enabled/hover/active/pressed/response behavior through the
  private kernel.
- Removed code is gone rather than preserved as fallback.
- Focused `fret-ui-kit` button/adapter/response tests pass.
- `cargo nextest run -p fret-imui --no-fail-fast` passes or any failure is proven unrelated.

## M2 - Second-Family Proof

Status: complete on 2026-04-24 for checkbox/radio, selectable, and combo trigger; switch
intentionally remains outside this kernel shape.

Goal: prove the kernel is not button-specific.

Exit criteria:

- One second family uses the same kernel without moving family-specific visual/layout policy into
  the kernel.
- Tests cover the behavior shared by both families.
- `imui_interaction_showcase_demo` and `imui_editor_proof_demo` still build.

## M3 - Scope Boundary Verdict

Status: complete on 2026-04-24.

Goal: decide whether the remaining pressure still belongs here.

Exit criteria:

- Remaining item-like families are either migrated or explicitly routed to narrower lanes.
- Any public API or hard-contract pressure is either rejected or captured in ADR/alignment work.
- `tools/check_layering.py`, `git diff --check`, and `tools/check_workstream_catalog.py` pass.
- A closeout note states whether future work should continue here, start a follow-on, or stay
  closed.

Verdict:

- Selectable rows and combo triggers migrated into the private full pressable item kernel.
- Switch, menu item, menu trigger, submenu trigger, tab trigger, and slider behavior are routed to
  narrower future lanes because they are active-only, menu/tab policy-heavy, or value-editing
  paths rather than the same full item behavior.
- The lane is closed by `CLOSEOUT_AUDIT_2026-04-24.md`.
