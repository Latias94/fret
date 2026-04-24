# ImUi Active Trigger Behavior Kernel v1 Milestones

Status: closed execution plan
Last updated: 2026-04-24

## M0 - Boundary Freeze

Status: complete on 2026-04-24.

Exit criteria:

- Full pressable item behavior remains owned by `item_behavior.rs`.
- Active-only trigger behavior is named separately.
- Slider value editing and menu/tab policy are explicitly out of scope.

## M1 - Active Trigger Kernel Slice

Status: complete on 2026-04-24.

Exit criteria:

- Switch, menu item, menu trigger, submenu trigger, and tab trigger share the private active-trigger
  behavior kernel.
- Activation side effects and shortcut semantics stay local to each control family.
- Replaced duplicate pointer/lifecycle/hover response blocks are deleted.

## M2 - Closeout Or Follow-On

Status: complete on 2026-04-24.

Exit criteria:

- Focused and full IMUI gates pass.
- The workstream catalog is updated.
- A closeout note records whether any remaining cleanup should be slider-specific, menu/tab-policy
  specific, or public API work.

Verdict:

- Switch, menu item, menu trigger, submenu trigger, and tab trigger were migrated.
- Slider, text, and disclosure work should start as narrower follow-ons if needed.
- The lane is closed by `CLOSEOUT_AUDIT_2026-04-24.md`.
