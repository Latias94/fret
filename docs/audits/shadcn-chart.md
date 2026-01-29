# shadcn/ui Chart (new-york-v4) Audit (Fret)

This audit tracks alignment for the shadcn/ui v4 **Chart** surface in the `new-york-v4` preset.

## Scope and baseline

- Upstream baseline:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/chart.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/blocks/chart-*.tsx`
- Fret implementation:
  - `ecosystem/fret-ui-shadcn/src/chart.rs`
  - Theme tokens: `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs` (`chart-*` palette)
- Tests / goldens:
  - shadcn-web goldens: `goldens/shadcn-web/v4/new-york-v4/chart-*.json`
  - gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (chart section)

## Current alignment posture

Fret is not implementing Recharts; instead we are aligning the **shadcn “chart scaffolding + tooltip/legend UI”**
and validating geometry outcomes against shadcn-web goldens.

What is currently gated (high level):

- Chart scaffold geometry (container + plot area bounds) for representative families.
- Tooltip panel chrome + internal row layout for tooltip variants.
- Scripted “hover-mid” snapshots that gate tooltip panel size, cursor geometry, and active marker (dot) geometry for interactive pages.

This is sufficient to catch many regressions in:

- spacing tokens (`px/py`, gaps, line-height outcomes),
- tooltip/legend wrapper layout,
- cursor/active marker placement for the scripted scenarios we currently export.

## Known gaps / risks

The following are expected sources of drift and need explicit depth gates before we claim 1:1 parity:

- Axis rendering parity (tick placement, text metrics, label truncation).
- Stacked/normalized series math (area/bar stacking and “expand” variants).
- Hit-testing / interaction model (what is considered “nearest” on hover, keyboard focus affordances).
- Mobile/viewport variants beyond the current minimal set (avoid exploding the matrix until the geometry is stable).

## Recommended next steps (P0 → P1)

P0 (maximize signal per test):

1. Add a small set of **stable geometry anchors** per family (e.g. cursor line rect, active dot rect, tooltip bounds)
   for `chart-*-interactive*` pages, across the existing scripted scenarios.
2. Gate **axis tick count + tick bounding boxes** for `chart-*-axes` pages (keep tolerant to sub-pixel rounding).

P1 (broader confidence, still controlled):

1. Introduce a limited DPI/font-metrics sweep (1–2 DPIs, 1 “weird metrics” font) for tooltip/legend/axis-heavy pages.
2. Expand stacking/math gates once the baseline implementation stops churn (avoid fragile tests while refactoring).
