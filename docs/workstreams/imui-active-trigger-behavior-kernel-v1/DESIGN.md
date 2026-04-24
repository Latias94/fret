# ImUi Active Trigger Behavior Kernel v1

Status: closed execution lane
Last updated: 2026-04-24

Status note (2026-04-24): this lane closed after switch, menu item, menu trigger, submenu trigger,
and tab trigger response/lifecycle duplication moved into the private active-trigger kernel.
Slider value editing, text focus/edit lifecycle, and disclosure context/double-click cleanup should
start as narrower follow-ons.

## Why This Lane Exists

`imui-item-behavior-kernel-v1` closed after moving full pressable item behavior into a private
kernel for button, checkbox/radio, selectable, and combo trigger controls. The remaining repeated
IMUI behavior is a smaller shape: active-only triggers that need lifecycle, active item, hover,
focus-visible, rect, and enabled-state response population, but do not need long press, drag
threshold tracking, right-click context-menu reporting, or double-click reporting.

This lane owns that smaller private kernel in `fret-ui-kit::imui`.

## Assumptions First

- Area: lane ownership
  - Assumption: this is a narrow follow-on to the closed full pressable item kernel lane.
  - Evidence: `docs/workstreams/imui-item-behavior-kernel-v1/CLOSEOUT_AUDIT_2026-04-24.md`.
  - Confidence: Confident
  - Consequence if wrong: active-only trigger cleanup could blur the full item kernel boundary.

- Area: owning crate
  - Assumption: the implementation owner remains `ecosystem/fret-ui-kit::imui`.
  - Evidence: all duplicated behavior sits in kit IMUI controls; `fret-imui` is only the public
    facade and test proof surface.
  - Confidence: Confident
  - Consequence if wrong: the public facade could gain policy helpers before the private behavior
    model is proven.

- Area: kernel boundary
  - Assumption: popup open/close, roving focus, menubar switching, submenu grace, and tab selection
    side effects stay local to their existing control modules.
  - Evidence: these are policy-heavy paths in `menu_controls.rs`, `menu_family_controls.rs`, and
    `tab_family_controls.rs`.
  - Confidence: Confident
  - Consequence if wrong: the helper would become a generic menu/tab policy engine instead of an
    active-trigger behavior kernel.

- Area: excluded controls
  - Assumption: sliders remain out of this lane because they own value mutation, pointer capture,
    and edit commit semantics.
  - Evidence: `slider_controls.rs` mutates the value model during pointer down/move/key input and
    controls pointer capture.
  - Confidence: Confident
  - Consequence if wrong: value-editing behavior would be hidden behind a trigger helper with the
    wrong abstraction.

## Scope

In scope:

- Add a private active-trigger behavior owner inside `fret-ui-kit::imui`.
- Migrate switch, menu item, menu trigger, submenu trigger, and tab trigger shared response and
  lifecycle wiring.
- Delete replaced family-local active/lifecycle/hover response duplication.
- Keep activation side effects and shortcut semantics local to each family.
- Keep public API, runtime, and ADR-level behavior contracts unchanged unless evidence proves a
  current contract is wrong.

Out of scope:

- Full pressable item behavior already covered by `item_behavior.rs`.
- Slider value editing and pointer capture.
- Popup dismissal, focus restore, submenu grace, menubar roving/switching policy, and tab selection
  policy.
- Public `fret-imui` API widening.
- Runtime mechanism changes in `fret-ui`.

## Target Shape

The private kernel should own:

- clearing stale pointer/key hooks for active-only triggers;
- left-pointer active item lifecycle;
- optional focus request on pointer press;
- hover delay response fields;
- focus-visible response state;
- rect, clicked, changed, lifecycle, and disabled response population.

The private kernel should not own:

- activate handlers;
- key shortcut handlers;
- command dispatch;
- popup/model toggling;
- menu/tab navigation policy;
- value mutation.
