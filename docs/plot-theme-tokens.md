# Plot Theme Tokens (`fret-plot`)

`fret-plot` resolves most plot colors via theme tokens when the corresponding `LinePlotStyle` fields
are `None`. This keeps plot styling consistent with the rest of the token-driven theme system (ADR
0102) while keeping the plot style surface small and ergonomic.

## Lookup Order

For each token, resolution follows this precedence:

1. `fret.plot.*` (preferred, framework-owned)
2. `plot.*` (compatibility / third-party themes)
3. Typed theme baseline (`theme.colors.*`) fallback

Notes:

- Color tokens are used when the corresponding `LinePlotStyle` color fields are `None`.
- Metric and palette tokens are applied only when the corresponding `LinePlotStyle` fields are left
  at their default values (so explicit per-canvas styles always win).

## Tokens (Colors)

All tokens below are optional.

- `fret.plot.background` (fallback: `theme.colors.panel_background`)
- `fret.plot.border` (fallback: `theme.colors.panel_border`)
- `fret.plot.axis` (fallback: `theme.colors.panel_border`)
- `fret.plot.grid` (fallback: `Color { a: 0.35, ..theme.colors.panel_border }`)
- `fret.plot.label` (fallback: `theme.colors.text_muted`)
- `fret.plot.crosshair` (fallback: `Color { a: 0.65, ..theme.colors.accent }`)
- `fret.plot.selection.stroke` (fallback: `fret.plot.crosshair`)
- `fret.plot.selection.fill` (fallback: `Color { a: 0.18 * selection_stroke.a, ..selection_stroke }`)
- `fret.plot.tooltip.background` (fallback: `theme.colors.menu_background`)
- `fret.plot.tooltip.border` (fallback: `theme.colors.menu_border`)
- `fret.plot.tooltip.text` (fallback: `theme.colors.text_primary`)

Compatibility keys:

- `plot.background`, `plot.border`, `plot.axis`, `plot.grid`, `plot.label`, `plot.crosshair`
- `plot.selection.stroke`, `plot.selection.fill`
- `plot.tooltip.background`, `plot.tooltip.border`, `plot.tooltip.text`

## Tokens (Series Palette)

If `LinePlotStyle.series_palette` is left at its default value, `fret-plot` will resolve a theme-driven
palette for multi-series plots.

Tokens:

- `fret.plot.palette.0` .. `fret.plot.palette.9`

Compatibility keys:

- `plot.palette.0` .. `plot.palette.9`

## Tokens (Metrics)

These tokens are interpreted as pixel values (the `ThemeConfig.metrics` map stores `f32`).

- `fret.plot.border_width` (fallback: `LinePlotStyle::default().border_width`)
- `fret.plot.padding` (fallback: `LinePlotStyle::default().padding`)
- `fret.plot.axis_gap` (fallback: `LinePlotStyle::default().axis_gap`)
- `fret.plot.stroke_width` (fallback: `LinePlotStyle::default().stroke_width`)
- `fret.plot.hover_threshold` (fallback: `LinePlotStyle::default().hover_threshold`)

Compatibility keys:

- `plot.border_width`, `plot.padding`, `plot.axis_gap`, `plot.stroke_width`, `plot.hover_threshold`

## Example Theme Config Snippet

```json
{
  "name": "My Theme",
  "colors": {
    "fret.plot.background": "#0F1115",
    "fret.plot.grid": "#2A2F3A",
    "fret.plot.crosshair": "#66CCFF",
    "fret.plot.tooltip.background": "#121826",
    "fret.plot.tooltip.border": "#2A2F3A",
    "fret.plot.palette.0": "#66CCFF",
    "fret.plot.palette.1": "#FF7A8A"
  }
}
```
