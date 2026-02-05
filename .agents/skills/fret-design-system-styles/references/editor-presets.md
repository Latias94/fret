# Editor style presets (starter set)

These are **starter presets** that map “style intent” → concrete token overrides.

Use them as small patches on top of:

- `fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(...)`, or
- an app-owned `ThemeConfig` JSON.

## Preset: Compact editor (dense)

Intent:

- reduce vertical height of common controls
- tighten list/table rows
- keep focus ring visible but subtle

Suggested baseline:

- `new-york-v4` + `Zinc` + `Dark` (developer-tool vibe)

Example `theme_overrides.json`:

```json
{
  "name": "compact-editor-overrides",
  "metrics": {
    "metric.padding.sm": 7.0,
    "metric.padding.md": 9.0,

    "component.size.md.input.h": 32.0,
    "component.size.md.button.h": 32.0,
    "component.size.sm.button.h": 28.0,
    "component.size.lg.button.h": 36.0,
    "component.size.md.icon_button.size": 32.0,

    "component.table.row_min_h": 32.0,
    "component.list.row_height": 24.0,

    "component.ring.width": 2.0,
    "component.ring.offset": 0.0
  }
}
```

## Preset: Comfortable editor (spacious)

Intent:

- more whitespace for long sessions
- larger hit targets
- slightly rounder surfaces

Example `theme_overrides.json`:

```json
{
  "name": "comfortable-editor-overrides",
  "metrics": {
    "metric.padding.sm": 10.0,
    "metric.padding.md": 12.0,
    "metric.radius.lg": 12.0,

    "component.size.md.input.h": 40.0,
    "component.size.md.button.h": 40.0,
    "component.size.sm.button.h": 36.0,
    "component.size.lg.button.h": 44.0,
    "component.size.md.icon_button.size": 40.0,

    "component.table.row_min_h": 40.0,
    "component.list.row_height": 28.0,

    "component.ring.width": 3.0,
    "component.ring.offset": 0.0
  }
}
```

## Preset: Glass panels (selective)

Intent:

- use glassmorphism for a few “special” surfaces (command palette, overlays, inspector popovers)
- keep the base theme simple; glass is an effect recipe, not a global background

Notes:

- Use `fret-ui-kit` glass recipe tokens (see `docs/effects-authoring.md`).
- Prefer scoped usage (`EffectLayer` / glass recipe wrappers), not “glass everywhere”.

Minimal overrides (example):

```json
{
  "name": "glass-overrides",
  "colors": {
    "component.glass.tint": "#FFFFFF22",
    "component.glass.border": "#FFFFFF26"
  },
  "metrics": {
    "component.glass.blur_radius_px": 18.0,
    "component.glass.blur_downsample": 2.0
  }
}
```
