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

Status note (2026-04-01): complete for v1. Footprint parity and alpha-budget sanity are both
covered, and the mechanism layer now has a deterministic softness-profile gate that proves the
portable shadow gets darker toward the edge while staying within the recipe-owned alpha budget under
layer compositing.

Exit criteria:

- At least one deterministic softness-oriented gate exists beyond footprint-only evidence.

## M3 - Renderer review evidence

Status note (2026-04-01): complete. Representative elevated surfaces now have a curated screenshot
suite so renderer-level shadow review is reproducible instead of ad hoc, and `todo_demo` now also
has a focused screenshot repro for app-local composition review.

Exit criteria:

- A named diag suite exists for representative shadow surfaces.
- The suite includes at least one non-overlay surface and one overlay/toast surface.
- A focused app-local repro exists when composition amplifies the renderer fallback.

## M4 - Automated renderer parity

Status note (2026-04-01): still open. Screenshot evidence now exists, but there is no automated
renderer-level pixel/readback gate for perceptual shadow softness yet.

Exit criteria:

- At least one renderer-level screenshot-diff or readback gate exists for representative shadow
  surfaces.
