# Shadcn Motion Parity Audit v1

Last updated: 2026-03-03.

## Goal

Maintain an explicit, reviewable tracker for **motion / animation parity** when aligning
`ecosystem/fret-ui-shadcn` to upstream shadcn/ui v4.

This workstream focuses on **outcome parity** (timing, interruption rules, reduced-motion behavior,
hit-testing implications), not API parity (DOM/CSS/Framer Motion).

## Scope

In-scope parity gaps:

- Discrete state changes that should be smooth (CSS `transition-*` style outcomes).
- Continuous animations (`animate-spin`, `animate-pulse`, caret blink, etc.).
- Time-based gates that are currently frame-count based (e.g. `delayMs`-like behavior).
- Reduced-motion behavior (should settle, stop requesting frames, and remain deterministic).

Out of scope (v1):

- Reproducing upstream implementation details 1:1 (Framer Motion variants, CSS keyframes).
- Pixel-perfect parity without a behavioral gate.
- Non-shadcn extras unless they block shadcn parity (e.g. marquee blocks).

## Sources of truth

Primary (shadcn v4):

- Upstream docs: `repo-ref/ui/apps/v4/content/docs/components/*.mdx`
- Upstream component sources (New York v4): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/*.tsx`

Secondary (headless behavior references):

- Radix primitives: `repo-ref/primitives/`
- Base UI: `repo-ref/base-ui/`

Fret implementation:

- shadcn recipes: `ecosystem/fret-ui-shadcn/src/*.rs`
- kit policy + motion drivers: `ecosystem/fret-ui-kit/src/declarative/*`
- mechanism surfaces (layout/hit-test/semantics): `crates/*`

Animation inspiration (not a spec):

- Animata: `repo-ref/animata/`

## Motion taxonomy (how we decide which driver to use)

**Discrete transitions** (toggle `open/closed`, `expanded/collapsed`, `rotate-180`, etc.)

- Model as a **transition** with a clear start/end target.
- Prefer `ecosystem/fret-ui-kit::declarative::transition::*` (tokenized durations/easing, reduced
  motion aware).

**Continuous motion** (spinner rotation, skeleton pulse, caret blink)

- Model as **duration-driven** motion, not `frame_id`-driven.
- Prefer `ecosystem/fret-ui-kit::declarative::motion::*` / `motion_value::*` so timing is stable
  across:
  - fixed-delta diag gates (`--fixed-frame-delta-ms 16`)
  - variable frame rate (60/120/144Hz)
  - headless tests (no wall-time snapshots)

## Parity rules of thumb (custom renderer constraints)

- Treat upstream motion as a **UX spec**, not an API.
- Prefer kit drivers over ad-hoc per-component math.
- Keep timings tunable via theme tokens (durations/easings), not hard-coded numbers.
- Be explicit about transforms:
  - Use `RenderTransform` when hit-testing should move with visuals.
  - Use `VisualTransform` for paint-only motion.
- Lock motion-sensitive changes with a deterministic gate:
  - `fretboard diag run ... --fixed-frame-delta-ms 16` and a screenshot or pixel-delta check, and/or
  - a focused unit test that checks monotonic progression + reduced motion settling.

## Deliverables

- A living parity table: `docs/workstreams/shadcn-motion-parity-audit-v1/PARITY_MATRIX.md`
- Workstream plan: `docs/workstreams/shadcn-motion-parity-audit-v1/TODO.md`
- Milestones (review checkpoints): `docs/workstreams/shadcn-motion-parity-audit-v1/MILESTONES.md`

