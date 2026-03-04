# Shadcn Motion Parity Audit v1 - TODO

Last updated: 2026-03-04.

## P0 (high-signal, low-risk)

- Landed: deterministic unit test gate for Accordion content measured-height expand/collapse (`animate-accordion-{down,up}` outcome).
- Landed: unit test gate for NavigationMenu trigger chevron rotation transition.
- Optional: add a fixed-delta diag screenshot pair (`--fixed-frame-delta-ms 16`) for visual review.
- Landed: `Combobox` popup placement + presence (fade/zoom/side slide) + unit test gate.
- Landed: `Switch` thumb slides between states (duration-driven tween) + unit test gate.
- Landed: `Progress` indicator translate animates on value changes (duration-driven tween) + unit test gate.
- Landed: popper overlays without explicit `duration-*` default to ~150ms (tw-animate-css default).
- Landed: `NavigationMenu` content open/close duration is ~200ms (upstream `duration-200`).
- Landed: `ContextMenu` / `HoverCard` popper overlays use scale+fade+side slide presence (~150ms) and have close-transition click-through gates.
- Landed: `DropdownMenu` presence fades/zooms/slides and has a unit test gate.

## P1 (timebase correctness: eliminate 60Hz coupling)

- Landed: `Spinner` / `Skeleton` / `InputOtp` continuous animations are duration-driven.
- Landed: `AvatarFallback` delay is duration-driven (`delay_ms` matches Radix `delayMs`).
- Landed: shadcn extras `Marquee` uses a duration-driven timebase (no `frame_id` delta coupling).
- Landed: kit `drive_transition_*` does not advance multiple times per frame (prevents call-count-driven transitions).
- Landed: `Button` hover background transition eases (~150ms, Tailwind default) and has a unit test gate.
- Landed: `Button` focus ring alpha transitions ease in/out (~150ms, Tailwind default) and have a unit test gate.
- Landed: `Toggle` hover background transition eases (~150ms, Tailwind default) and has a unit test gate.
- Landed: `Toggle` focus ring transitions ease in/out and have a unit test gate.
- Landed: `Input` / `Textarea` focus ring + border transitions ease (~150ms, Tailwind default) and have unit test gates.
- Landed: `BreadcrumbLink` hover foreground uses `transition-colors` semantics (~150ms) and has a unit test gate.
- Landed: `TableRow` hover background uses `transition-colors` semantics (~150ms) and has a unit test gate.
- Landed: `NativeSelect` focus ring + border transitions ease (~150ms, Tailwind default) and has a unit test gate.
- Landed: `SelectTrigger` focus ring + border transitions ease (~150ms, Tailwind default) and has a unit test gate.
- Landed: `Checkbox` / `RadioGroupItem` focus ring transitions ease (~150ms, Tailwind default) and have unit test gates.
- Landed: `Slider` thumb hover/focus ring transitions ease (~150ms, Tailwind default) and has a unit test gate.
- Landed: `Switch` track background/border/ring style transitions ease (~150ms, Tailwind default) and have a unit test gate.
- Landed: `InputOtp` slot border + ring transitions ease (`transition-all`) and have a unit test gate.
- Landed: `ScrollArea` viewport focus ring eases (`transition-[color,box-shadow]`) and has a unit test gate.
- Landed: `SidebarGroupLabel` collapses with `margin+opacity` (shadcn `-mt-8 opacity-0`) and has a unit test gate.
- Landed: `InputGroup` wrapper ring/border transitions ease (~150ms, Tailwind default) and have a unit test gate.
- Landed: `Combobox` trigger border/ring transitions ease (~150ms, Tailwind default) and have a unit test gate.
- Landed: `Badge` focus-visible border/ring transitions ease (~150ms, Tailwind default) and have a unit test gate.
- Landed: `TabsTrigger` focus ring transitions ease in/out (~150ms, Tailwind default) and have a unit test gate.
- Landed: `Item` hover background transition eases (~100ms, `duration-100`) and has a unit test gate.
- Gap: audit `Drawer` parity vs Vaul (define concrete drag + inertia outcomes; add a fixed-delta diag + a unit test for settle).
- Not audited: audit `Carousel` parity vs Embla (define concrete drag/scroll + snap settle outcomes; add fixed-delta diag + a unit test for settle invariants).

## P2 (API + token cleanup)

- Replace "per-frame" speed knobs (`radians per frame`, `px per frame`) with "per-second" or "duration" semantics where public APIs exist.
- Introduce/standardize token keys for motion durations and easings used by shadcn recipes.
- Audit which continuous animations should be `VisualTransform` vs `RenderTransform`.

## P3 (gates + documentation hygiene)

- For each motion change, add at least one:
  - unit test (reduced motion + progression invariants), and/or
  - diag script gate (fixed delta; stable `test_id`).
- Landed: unit test gates exist for NavigationMenu chevron, Spinner, Skeleton pulse, and InputOtp caret blink.
- Keep `PARITY_MATRIX.md` updated with evidence anchors (file paths + tests/scripts).
