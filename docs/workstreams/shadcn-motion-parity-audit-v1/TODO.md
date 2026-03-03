# Shadcn Motion Parity Audit v1 — TODO

Last updated: 2026-03-03.

## P0 (high-signal, low-risk)

- Add a minimal deterministic gate for NavigationMenu trigger chevron rotation
  (`--fixed-frame-delta-ms 16`) to prevent regressions.

## P1 (timebase correctness: eliminate 60Hz coupling)

- Landed: `Spinner` / `Skeleton` / `InputOtp` continuous animations are duration-driven.
- Landed: `AvatarFallback` delay is duration-driven (`delay_ms` matches Radix `delayMs`).
- Landed: shadcn extras `Marquee` uses a duration-driven timebase (no `frame_id` delta coupling).

## P2 (API + token cleanup)

- Replace “per-frame” speed knobs (`radians per frame`, `px per frame`) with “per-second” or
  “duration” semantics where public APIs exist.
- Introduce/standardize token keys for motion durations and easings used by shadcn recipes.
- Audit which continuous animations should be `VisualTransform` vs `RenderTransform`.

## P3 (gates + documentation hygiene)

- For each motion change, add at least one:
  - unit test (reduced motion + progression invariants), and/or
  - diag script gate (fixed delta; stable `test_id`).
- Keep `PARITY_MATRIX.md` updated with evidence anchors (file paths + tests/scripts).
