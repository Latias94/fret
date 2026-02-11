# ADR 0131: `fret-chart` Theme Tokens and Style Resolution (ECharts-Inspired)

Status: Accepted (P1)

## Context

`delinea` is a headless engine. Visual output is produced by `fret-chart`, which:

- lays out the chart (grid + axes),
- maps input into headless actions,
- translates marks into `SceneOp` drawing.

Today, `fret-chart` uses a hard-coded `ChartStyle::default()`. This is good for early demos but becomes a
liability for:

- theme portability (dark/light, brand themes),
- ecosystem consistency (aligning with `Theme` and ADR 0050 / ADR 0101),
- long-term extensibility (tooltips, legends, selection overlays).

ECharts separates a stable option model from theme packs; we want a similar “theme-driven defaults” stance:

- a default style derived from the app theme,
- opt-in override knobs for custom looks.

## Relationship to Other ADRs

- ADR 0050: theme config schema and baseline tokens.
- ADR 0101: semantic theme keys and extensible token registry.
- ADR 0190: delinea headless chart engine.
- ADR 0202: dataset storage and indices (for performance; style must not interfere).

## Decision

### 1) `ChartStyle` is resolved from `Theme` by default

`ChartCanvas` (the retained widget) derives its `ChartStyle` from the global `Theme`:

- style recomputes only when `theme.revision()` changes,
- the resolution is deterministic and allocation-light,
- UI redraw scheduling is unchanged (style changes still require a redraw).

### 2) Token naming: `chart.*` namespace with semantic fallback

We define an extensible token namespace for chart styling:

**Colors**

- `chart.background`
- `chart.axis.line`
- `chart.axis.tick`
- `chart.axis.label`
- `chart.crosshair`
- Series palette (order-based, ECharts-like):
  - `chart.palette.0` .. `chart.palette.9`
  - Fallback: shadcn/new-york extended tokens `chart-1` .. `chart-5`
- `chart.tooltip.background`
- `chart.tooltip.border`
- `chart.tooltip.text`
- `chart.legend.background`
- `chart.legend.border`
- `chart.legend.text`
- `chart.selection.fill`
- `chart.selection.stroke`
- VisualMap (continuous controller; v1):
  - `chart.visualmap.track`
  - `chart.visualmap.range.fill`
  - `chart.visualmap.range.stroke`
  - `chart.visualmap.handle`

**Metrics** (pixel-based; keys may optionally live under `metric.*`)

- `metric.chart.stroke.width`
- `metric.chart.padding`
- `metric.chart.axis.band.x`
- `metric.chart.axis.band.y`
- `metric.chart.axis.line.width`
- `metric.chart.axis.tick.length`
- `metric.chart.visualmap.band.x`
- `metric.chart.visualmap.pad`
- `metric.chart.visualmap.item.gap`
- `metric.chart.visualmap.corner_radius`
- `metric.chart.scatter.point_radius`
- `metric.chart.hover.point_size`
- `metric.chart.tooltip.padding.x`
- `metric.chart.tooltip.padding.y`
- `metric.chart.tooltip.corner_radius`
- `metric.chart.legend.padding.x`
- `metric.chart.legend.padding.y`
- `metric.chart.legend.corner_radius`
- `metric.chart.legend.item.gap`
- `metric.chart.legend.swatch.size`
- `metric.chart.legend.swatch.gap`
- `metric.chart.selection.stroke.width`

Resolution order:

1. Look up the explicit chart token (e.g. `chart.axis.label`).
2. If missing, fall back to the repo’s semantic tokens (e.g. `foreground`, `muted-foreground`, `border`,
   `card`, `popover`, `primary`).
3. For metrics, fall back to baseline `metric.*` keys (e.g. `metric.padding.sm`, `metric.radius.sm`,
   `metric.font.size`) and/or constants when no reasonable baseline exists.

This keeps defaults working in all themes while giving theme authors a stable surface for chart-specific
polish.

### 3) Explicit style override remains supported

Callers may override styling by setting a fixed `ChartStyle` on `ChartCanvas`. When a fixed style is used:

- theme-driven updates do not overwrite the caller’s style,
- the widget remains self-contained and deterministic.

## Consequences

- Chart visuals become consistent with the rest of Fret UI under different themes.
- We avoid spreading hard-coded RGBA values across the ecosystem.
- Theme authors gain a stable extension surface without needing to modify core theme structs.

Trade-offs:

- Chart-specific token defaults may be incomplete initially; fallbacks must remain stable.
- Some tokens (e.g. legend spacing) do not map cleanly onto existing baseline metrics and may remain
  “chart-only”.

## Follow-ups

P0:

- Implement `ChartStyle::from_theme(&Theme)` with the resolution order above.
- Update `fret-chart` docs to mention the token namespace.

P1:

- Add per-series palette support (theme-driven color cycles aligned with ECharts palettes).
- Add formatter hooks for tooltip/axis labels (locale/time zone; see ADR 0201).

## References

- Theme baseline: `docs/adr/0050-theme-config-schema-and-baseline-tokens.md`
- Theme registry: `docs/adr/0101-semantic-theme-keys-and-extensible-token-registry.md`
- ECharts theme packs: https://echarts.apache.org/en/theme-builder.html (concept reference)

## Evidence

- Style resolution: `ecosystem/fret-chart/src/retained/style.rs` (`ChartStyle::from_theme`)
- Style application: `ecosystem/fret-chart/src/retained/canvas.rs` (`ChartCanvas::sync_style_from_theme`)
- Series palette usage: `ecosystem/fret-chart/src/retained/canvas.rs` (`ChartCanvas::series_color`)
