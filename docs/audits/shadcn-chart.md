# shadcn/ui v4 Audit — Chart

This audit tracks parity for the shadcn/ui v4 `chart` surface (new-york-v4) against `repo-ref/ui`.

Unlike most shadcn components, `chart` is a thin wrapper over an upstream **chart engine** (Recharts).
To reach 1:1 parity, we need to be explicit about which parts are “wrapper UI” vs “chart rendering”.

## Upstream references (source of truth)

- Wrapper API + tooltip/legend implementations:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/chart.tsx`
- Example gallery (golden sources):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/charts/*.tsx`
  - Registry index: `repo-ref/ui/apps/v4/registry/new-york-v4/charts/_registry.ts`
- Goldens (expected outcomes):
  - `goldens/shadcn-web/v4/new-york-v4/chart-*.json` (76 variants today)

## Fret implementation status

Current:

- Theme: `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs` derives a small palette from `chart-*` tokens
  (useful for non-chart components too).
- Wrapper UI: `ecosystem/fret-ui-shadcn/src/chart.rs` exists and currently implements the tooltip panel chrome/layout
  (`ChartTooltipContent`) plus a small data model (`ChartTooltipItem`, `ChartTooltipIndicator`).
- Golden gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` includes initial `chart-tooltip-*` panel geometry
  gates (min width + padding/border + line-height outcomes).
- No chart rendering backend is wired for shadcn parity yet (axes/series/tooltip hit-testing, etc).

As a result, most of the `chart-*` golden family is still ungated; only tooltip-panel pages are referenced today.

## Parity scope (what “1:1” means here)

`chart` parity is two layers:

1. **Wrapper UI parity (shadcn-owned)**
   - `ChartContainer` layout (`aspect-video`, `text-xs`) and the “Recharts CSS overrides” mapping.
   - `ChartTooltipContent` geometry + chrome + indicator taxonomy (`dot`/`line`/`dashed`), label rules,
     typography (`font-mono tabular-nums` values), and min width (`min-w-[8rem]`).
   - `ChartLegendContent` layout, icon rules, and typography.

2. **Chart engine parity (Recharts-owned)**
   - Series rendering (bar/line/area/pie/radar/radial), grids, axes/ticks, and active markers/cursors.
   - Tooltip anchoring behavior (defaultIndex, cursor overlays, series-specific payload).
   - Event model (hover/press/drag) and accessibility metadata.

Wrapper UI parity is implementable directly in Fret.
Chart engine parity requires a dedicated chart subsystem (or a strict adapter to an existing one).

## Recommended implementation plan (phased)

P0 (unlock golden gating breadth):

- Implement shadcn `chart` wrapper UI components in `ecosystem/fret-ui-shadcn/src/chart.rs`:
  - `ChartContainer` (layout + palette binding surface).
  - `ChartTooltipContent` (including indicator variants and label rules). (In progress: panel geometry is gated.)
  - `ChartLegendContent`.
- Add initial “wrapper-only” goldens for `chart-tooltip-*` pages:
  - Focus on tooltip/legend panel geometry + chrome.
  - Avoid asserting SVG path geometry until the chart engine exists.

P1 (engine contracts + first real charts):

- Define a minimal chart contract (scales, axes, series, hover model) in `fret-ui-kit` (or a dedicated crate).
- Implement rendering + hit-testing in `fret-render` and hook it up via declarative elements.
- Gate a small set of “engine-critical” pages first (`chart-bar-default`, `chart-line-default`, `chart-area-default`).

P2 (full breadth + interactive variants):

- Expand to the remaining chart families (pie/radar/radial) and interactive variants.
- Add constrained viewport and DPI variants only once core geometry is stable.

## Validation / tracking

- Coverage snapshot:
  - `pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -AsMarkdown -FilterMissingPrefix chart`
- When `chart` implementation starts landing, update:
  - `docs/audits/shadcn-new-york-v4-coverage.md` (coverage snapshot)
  - `docs/audits/shadcn-new-york-v4-alignment.md` (alignment notes)
  - `docs/shadcn-declarative-progress.md` (`chart` row + audit status)
