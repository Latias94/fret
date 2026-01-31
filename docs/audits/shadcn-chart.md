# shadcn/ui chart audit (New York v4)

This document tracks parity work for shadcn/ui **Chart** surfaces (and related legend/tooltip/axis behavior).

## Status

- **Scope:** Partially audited (tooltip + legend layout/chrome).
- **Breadth coverage:** Included in `docs/audits/shadcn-new-york-v4-coverage.md`.
- **Depth checklist:** Tracked in `docs/audits/shadcn-new-york-v4-depth-checklist.md`.

Evidence anchors:

- Tooltip + legend geometry gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_chart_tooltip.rs`
- Hover-mid (interactive) tooltip + cursor gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_chart_hover_mid.rs`
- Baseline chart DOM invariants (web-only): `ecosystem/fret-ui-shadcn/tests/web_vs_fret_chart.rs`

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

Recommended P0 depth extensions:

- Gate the **interactive** tooltip surfaces that depend on cursor position (`*.hover-mid` pages):
  - cursor rect geometry (e.g. `recharts-tooltip-cursor`)
  - active marker geometry (e.g. `recharts-active-dot`)
- Add a constrained viewport variant for at least one tooltip-heavy page to validate overflow behavior.
