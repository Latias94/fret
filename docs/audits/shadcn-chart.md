# shadcn/ui chart audit (New York v4)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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

## Accessibility (`accessibilityLayer`)

The upstream docs recommend enabling `accessibilityLayer` to add keyboard access and screen reader
support for Recharts-driven charts.

In Fret, the closest portable outcome is exposed via an opt-in accessibility layer on the native
chart canvas surface:

- Pass: `fret-chart::ChartCanvas` can be made focusable (Tab/click) via `set_accessibility_layer(true)`.
- Pass: While focused, `ArrowLeft/ArrowRight/ArrowUp/ArrowDown` navigate between data points (by
  `data_index` and series), driving the engine hover state.
- Pass: Semantics `value` is populated from the tooltip formatter so screen readers can announce
  the current point context (category + series values) without requiring DOM nodes.

Evidence anchors:

- Keyboard + semantics: `ecosystem/fret-chart/src/retained/canvas.rs`
- Gallery gate: `tools/diag-scripts/ui-gallery-chart-accessibility-layer-keyboard.json`

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

Constrained viewport coverage (current):

- `chart-tooltip-default.vp375x320` / `chart-tooltip-advanced.vp375x320` are gated (panel size + row geometry).
- `chart-*-legend.vp375x320` are gated to validate wrapping height outcomes under narrow widths.
- `chart-*-interactive.hover-mid-vp1440x240` are gated to validate tooltip/cursor/active-marker geometry under tight height.
