# shadcn/ui chart audit (New York v4)

This document tracks parity work for shadcn/ui **Chart** surfaces (and related legend/tooltip/axis behavior).

## Status

- **Scope:** Not audited yet.
- **Breadth coverage:** Included in `docs/audits/shadcn-new-york-v4-coverage.md`.
- **Depth checklist:** Tracked in `docs/audits/shadcn-new-york-v4-depth-checklist.md`.

## What “1:1 parity” means here

At minimum, chart parity should cover:

- Layout primitives: plot area insets, axis label sizing, tick alignment, legend layout.
- Tooltip/overlay: placement and collision, viewport constraints, pointer tracking, and visible animations.
- Styling: color tokens, typography, grid/axis stroke widths, radii, and opacity.
- Data contracts: series ordering, stacked/grouped behavior, and default variants.

## Next actions (proposed)

1. Identify the upstream reference implementation(s) in `repo-ref/ui` used for chart demos in `v4/new-york-v4`.
2. Add **open-mode** goldens for tooltip/legend overlays (including constrained viewport variants).
3. Add **high-signal** Rust gates: tooltip rects, plot insets, legend wrapping, and axis tick count/spacing.
4. Add evidence anchors (tests + key functions) in this file as work lands.
