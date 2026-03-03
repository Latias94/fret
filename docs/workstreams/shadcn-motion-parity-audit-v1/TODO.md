# Shadcn Motion Parity Audit v1 — TODO

Last updated: 2026-03-03.

## P0 (high-signal, low-risk)

- Add a transition driver for NavigationMenu trigger chevron rotation (upstream has
  `transition duration-300 ... rotate-180`).
- Add a minimal deterministic gate for the chevron rotation (`--fixed-frame-delta-ms 16`) to
  prevent regressions.

## P1 (timebase correctness: eliminate 60Hz coupling)

- Refactor shadcn `Spinner` away from `frame_id * speed` and toward a duration-driven rotation.
  - Ensure reduced motion stops requesting frames and snaps/settles deterministically.
- Refactor shadcn `Skeleton` pulse away from `frame_id`-driven `sin()` toward a duration-driven
  pulse.
- Refactor shadcn `InputOtp` caret blink away from `frame_id` modulo toward a 1000ms cycle.

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

