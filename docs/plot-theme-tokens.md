# Plot Theme Tokens (`fret-plot`)

`fret-plot` resolves most plot colors via theme tokens when the corresponding `LinePlotStyle` fields
are `None`. This keeps plot styling consistent with the rest of the token-driven theme system (ADR
0102) while keeping the plot style surface small and ergonomic.

## Lookup Order

For each token, resolution follows this precedence:

1. `fret.plot.*` (preferred, framework-owned)
2. `plot.*` (compatibility / third-party themes)
3. Typed theme baseline (`theme.colors.*`) fallback

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

## Example Theme Config Snippet

```json
{
  "name": "My Theme",
  "colors": {
    "fret.plot.background": "#0F1115",
    "fret.plot.grid": "#2A2F3A",
    "fret.plot.crosshair": "#66CCFF",
    "fret.plot.tooltip.background": "#121826",
    "fret.plot.tooltip.border": "#2A2F3A"
  }
}
```

