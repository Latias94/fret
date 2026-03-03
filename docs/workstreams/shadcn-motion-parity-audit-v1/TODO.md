# Shadcn Motion Parity Audit v1 — TODO

Last updated: 2026-03-03.

## P0 (high-signal, low-risk)

- Landed: unit test gate for NavigationMenu trigger chevron rotation transition.
- Optional: add a fixed-delta diag screenshot pair (`--fixed-frame-delta-ms 16`) for visual review.
- Landed: `Combobox` popup placement + presence (fade/zoom/side slide) + unit test gate.
- Landed: `Switch` thumb slides between states (duration-driven tween) + unit test gate.
- Landed: `Progress` indicator translate animates on value changes (duration-driven tween) + unit test gate.

## P1 (timebase correctness: eliminate 60Hz coupling)

- Landed: `Spinner` / `Skeleton` / `InputOtp` continuous animations are duration-driven.
- Landed: `AvatarFallback` delay is duration-driven (`delay_ms` matches Radix `delayMs`).
- Landed: shadcn extras `Marquee` uses a duration-driven timebase (no `frame_id` delta coupling).
- Landed: kit `drive_transition_*` does not advance multiple times per frame (prevents call-count-driven transitions).
- Gap: `Sonner` / `Toast` should animate enter/exit (fade + slide), not snap.
- Gap: common primitives should ease hover/focus style changes (`transition-*` parity), not snap (Button/Badge/Toggle/Input/Textarea/TabsTrigger/Table rows/ScrollArea thumb; requires reusable style transition helpers).

## P2 (API + token cleanup)

- Replace “per-frame” speed knobs (`radians per frame`, `px per frame`) with “per-second” or
  “duration” semantics where public APIs exist.
- Introduce/standardize token keys for motion durations and easings used by shadcn recipes.
- Audit which continuous animations should be `VisualTransform` vs `RenderTransform`.

## P3 (gates + documentation hygiene)

- For each motion change, add at least one:
  - unit test (reduced motion + progression invariants), and/or
  - diag script gate (fixed delta; stable `test_id`).
- Landed: unit test gates exist for NavigationMenu chevron, Spinner, Skeleton pulse, and InputOtp caret blink.
- Keep `PARITY_MATRIX.md` updated with evidence anchors (file paths + tests/scripts).
