# Shadcn Motion Parity Audit v1 — TODO

Last updated: 2026-03-04.

## P0 (high-signal, low-risk)

- Landed: unit test gate for NavigationMenu trigger chevron rotation transition.
- Optional: add a fixed-delta diag screenshot pair (`--fixed-frame-delta-ms 16`) for visual review.
- Landed: `Combobox` popup placement + presence (fade/zoom/side slide) + unit test gate.
- Landed: `Switch` thumb slides between states (duration-driven tween) + unit test gate.
- Landed: `Progress` indicator translate animates on value changes (duration-driven tween) + unit test gate.
- Landed: popper overlays without explicit `duration-*` default to ~150ms (tw-animate-css default).
- Landed: `NavigationMenu` content open/close duration is ~200ms (upstream `duration-200`).

## P1 (timebase correctness: eliminate 60Hz coupling)

- Landed: `Spinner` / `Skeleton` / `InputOtp` continuous animations are duration-driven.
- Landed: `AvatarFallback` delay is duration-driven (`delay_ms` matches Radix `delayMs`).
- Landed: shadcn extras `Marquee` uses a duration-driven timebase (no `frame_id` delta coupling).
- Landed: kit `drive_transition_*` does not advance multiple times per frame (prevents call-count-driven transitions).
- Landed: `Button` hover background transition eases (~150ms, Tailwind default) and has a unit test gate.
- Landed: `Toggle` hover background transition eases (~150ms, Tailwind default) and has a unit test gate.
- Landed: `Toggle` focus ring transitions ease in/out and have a unit test gate.
- Landed: `Input` / `Textarea` focus ring + border transitions ease (~150ms, Tailwind default) and have unit test gates.
- Landed: `NativeSelect` focus ring + border transitions ease (~150ms, Tailwind default) and has a unit test gate.
- Landed: `SelectTrigger` focus ring + border transitions ease (~150ms, Tailwind default) and has a unit test gate.
- Landed: `Checkbox` / `RadioGroupItem` focus ring transitions ease (~150ms, Tailwind default) and have unit test gates.
- Landed: `Slider` thumb hover/focus ring transitions ease (~150ms, Tailwind default) and has a unit test gate.
- Landed: `TabsTrigger` focus ring transitions ease in/out (~150ms, Tailwind default) and have a unit test gate.
- Landed: `Item` hover background transition eases (~100ms, `duration-100`) and has a unit test gate.
- Gap: common primitives should ease hover/focus style changes (`transition-*` parity), not snap. Suggested ordering:
  - P1: Badge / BreadcrumbLink / Table rows / ScrollArea viewport + scrollbar.
  - P1: Add a gate for Accordion content measured-height motion (`animate-accordion-{down,up}` outcome).
- Gap: audit `Drawer` parity vs Vaul (define concrete drag + inertia outcomes; add a fixed-delta diag + a unit test for settle).

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
