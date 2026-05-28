# Material 3 Checkbox Gallery Diagnostics Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The Material3 component matrix still marked Checkbox as `packet_done_known_follow_ons` even though
the remaining risk was concrete gallery evidence, not a component architecture gap. The promoted
centered-chrome script was also stale: it navigated to the aggregate Material3 gallery page, while
the stable checkbox selectors now live on the dedicated Material3 Checkbox page.

## Truth

- Checkbox recipe owns bool/optional model mapping, tri-state toggle outcomes, semantics, and
  chrome assembly.
- Material foundation owns shared state-layer/ripple and minimum interactive target sizing.
- Gallery diagnostics should prove that the 48px interaction target contains centered visual
  chrome and that tri-state semantics survive standard and expressive gallery states.
- A stale diagnostics target is a test-harness problem, not a recipe, foundation, kit-policy, or
  mechanism problem.

## Boundaries

- No component code should change unless the repaired diagnostics prove behavior drift.
- No `fret-ui-kit` abstraction is justified by this packet.
- No core mechanism change is justified by this packet.

## Reference Axis

- Compose Material3: checkbox semantics, mixed/indeterminate state, minimum interactive component
  size, indication/ripple outcomes.
- Fret Material3 foundation: existing `foundation::indication` and
  `foundation::interactive_size`.
- Fret diagnostics: promoted UI Gallery scripts using stable `test_id` anchors.
