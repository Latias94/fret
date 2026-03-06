# shadcn/ui v4 Audit - Chart

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This document tracks parity work for shadcn/ui chart surfaces (and related tooltip/legend/axis behavior).

## Status

- Scope: Partially audited (tooltip + legend layout/chrome, chart-facing recipe surface).
- Breadth coverage: Included in `docs/audits/shadcn-new-york-v4-coverage.md`.
- Depth checklist: Tracked in `docs/audits/shadcn-new-york-v4-depth-checklist.md`.

## Fret implementation anchors

- Component code: `ecosystem/fret-ui-shadcn/src/chart.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/chart.rs`
- Copyable usage snippet: `apps/fret-ui-gallery/src/ui/snippets/chart/usage.rs`
- Geometry gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_chart_tooltip.rs`
- Interactive hover gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_chart_hover_mid.rs`
- Baseline chart DOM invariants: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_chart.rs`

## Authoring surface

- Pass: `ChartConfig` + `ChartConfigItem` already cover the upstream config-map authoring model.
- Pass: `ChartContainer` provides the expected chart-scoped wrapper/context surface.
- Pass: `ChartTooltip` / `ChartTooltipContent` and `ChartLegend` / `ChartLegendContent` already cover the important shadcn recipe outcomes for tooltip/legend authoring.
- Note: Because Fret's actual chart engine integration lives below this recipe layer, this surface does not need a generic `compose()` builder; the main docs gap was a concise minimal `Usage` example.

## What 1:1 parity means here

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

- Pass: `fret-chart::ChartCanvas` can be made focusable via `set_accessibility_layer(true)`.
- Pass: While focused, arrow keys navigate between data points and drive the engine hover state.
- Pass: Semantics `value` is populated from the tooltip formatter so screen readers can announce the
  current point context without requiring DOM nodes.

Evidence anchors:

- Keyboard + semantics: `ecosystem/fret-chart/src/retained/canvas.rs`
- Gallery gate: `tools/diag-scripts/ui-gallery-chart-accessibility-layer-keyboard.json`

## Conclusion

- Result: This component does not currently indicate a missing mechanism-layer gap in the shadcn-facing API surface.
- Result: The main missing piece for docs parity was a concise gallery `Usage` example for `ChartContainer` + tooltip/legend recipes.
- Result: Follow-up work should focus on concrete engine wiring and interactive chart regressions rather than adding more authoring surface.

## Validation

- `cargo check -p fret-ui-gallery --message-format short`
- `ecosystem/fret-ui-shadcn/tests/web_vs_fret_chart_tooltip.rs`
- `ecosystem/fret-ui-shadcn/tests/web_vs_fret_chart_hover_mid.rs`
- `ecosystem/fret-ui-shadcn/tests/web_vs_fret_chart.rs`
