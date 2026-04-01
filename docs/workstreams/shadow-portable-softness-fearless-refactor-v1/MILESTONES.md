# Shadow Portable Softness (Fearless Refactor v1) — Milestones

Status: In Progress

Last updated: 2026-04-01

Related:

- Design: `docs/workstreams/shadow-portable-softness-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/shadow-portable-softness-fearless-refactor-v1/TODO.md`

## M0 - Root-cause freeze

Status note (2026-04-01): complete. Shared shadcn control shadow lanes are now sufficiently gated
that the remaining subjective "hard shadow" report is traced to the portable painter, not preset
geometry drift.

Exit criteria:

- Shared `shadow-xs` surfaces have direct footprint evidence.
- The remaining issue is classified as painter fidelity.

## M1 - Portable alpha-budget closure

Status note (2026-04-01): complete for the initial fix. The portable painter now normalizes
per-step alpha, and mechanism tests assert that multi-step blur no longer inflates total shadow
opacity.

Exit criteria:

- Multi-step portable shadow alpha is normalized.
- Zero-blur behavior is preserved.
- Mechanism tests cover both outcomes.

## M2 - Pixel-profile confidence

Status note (2026-04-01): still open. Footprint parity and alpha-budget sanity are now both
covered, but a stronger perceptual/CSS softness gate is still missing.

Exit criteria:

- At least one deterministic softness-oriented gate exists beyond footprint-only evidence.
