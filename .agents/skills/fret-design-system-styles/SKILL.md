---
name: fret-design-system-styles
description: Apply a consistent visual style to Fret apps by selecting a baseline theme (shadcn New York v4 preset) and expressing design decisions as token overrides (`ThemeConfig`). Use when re-skinning an app, defining density/radius/shadows/rings, or translating “style keywords” into concrete token changes.
---

# Fret design system styles (theme-first)

This skill is **about visual style application** (theme + density + elevation + focus visuals), not general UX guardrails.

Fret’s design-system surface (`fret-ui-shadcn`) is **token-driven**. The fastest path to a cohesive look is:

1) pick a baseline (shadcn preset),
2) apply small, intentional `ThemeConfig` overrides (density/radius/shadow/ring),
3) keep per-component magic numbers to a minimum.

## When to use

Use this skill when:

- Picking a baseline theme and making the app look cohesive quickly.
- Re-skinning an app (density/radius/shadows/rings) without touching every component.
- Translating “style keywords” into concrete token changes.

## Inputs to collect (ask the user)

- **Scheme**: `light` / `dark`
- **Vibe keywords**: e.g. `minimal`, `soft`, `neubrutal`, `glass`, `hud`, `high-contrast`
- **Density**: `compact` / `default` / `comfortable`
- **Accent**: primary color intent (brand vs neutral)
- **Target surface**: editor workspace, settings forms, dashboards, etc.

Defaults if unclear:

- `dark` + `compact` editor style, then adjust ring + radius.

## Quick start

1. Pick a baseline preset: `apply_shadcn_new_york_v4(...)`.
2. Decide 1–2 axes to change first (usually density + ring).
3. Apply a small `ThemeConfig` patch and validate in the UI gallery/demo you care about.

## Workflow

Use the rest of this doc in order:

1. Collect user-facing inputs (scheme, density, vibe keywords, target surface).
2. Apply baseline preset.
3. Apply token overrides for density/radius/shadows/ring.
4. Only then consider component-level overrides (and keep them rare).

## Definition of done (what to leave behind)

- Baseline preset is explicit (`apply_shadcn_new_york_v4(...)`) and checked into the app setup.
- A small, reviewable `ThemeConfig` override exists (JSON patch), scoped to 1–2 axes at a time.
- The change is validated in the smallest target UI surface (UI gallery page/demo).
- Per-component magic numbers are minimized; deviations are expressed as token overrides or documented style overrides.

## Baseline (recommended)

Use shadcn “new-york-v4” preset as the baseline so components start from a known vocabulary.

```rust
use fret_ui_shadcn::shadcn_themes::{apply_shadcn_new_york_v4, ShadcnBaseColor, ShadcnColorScheme};

apply_shadcn_new_york_v4(app, ShadcnBaseColor::Zinc, ShadcnColorScheme::Dark);
```

Then layer your overrides:

```rust
let cfg = fret_ui::ThemeConfig::from_slice(include_bytes!("theme_overrides.json"))?;
fret_ui::Theme::with_global_mut(app, |theme| theme.apply_config(&cfg));
```

## Style axes (what you should change)

Use **small token edits** that affect many components at once.

### 1) Density (editor “compactness”)

Primary knobs:

- `metric.padding.sm`, `metric.padding.md` (feeds `Space::*` fallbacks)
- `component.size.*` (shadcn control heights/padding)
- `component.table.row_min_h`, `component.list.row_height` (list/table density)

### 2) Roundness (radius)

- `metric.radius.sm|md|lg` (baseline)
- optional: `component.radius.sm|md|lg` (component override layer)

### 3) Elevation (shadows)

- `shadow` (color)
- `component.shadow.{xs,sm,md,lg,xl}.*` (offset/spread/softness)

### 4) Focus visuals (ring)

- `ring`, `ring-offset-background`
- `component.ring.width`, `component.ring.offset`

## Presets (copy/paste starting points)

See:

- `references/editor-presets.md`
- `references/token-groups.md`
- `references/style_catalog.json` (small style catalog; used by the generator script)

## Generator (recommended)

This skill ships a tiny, dependency-free generator:

```bash
python scripts/stylegen.py --list
python scripts/stylegen.py --suggest "compact dense editor"
python scripts/stylegen.py --style editor-compact > theme_overrides.json
```

Notes:

- The generator prints the suggested baseline preset to **stderr** and the JSON overrides to **stdout**.
- The emitted JSON is a `ThemeConfig` patch intended to be layered *on top of* `apply_shadcn_new_york_v4(...)`.

## Practical rules (prevents “drift”)

- Prefer `use fret_ui_shadcn::prelude::*;` in app UI code.
- Prefer `Space` / `Radius` / `MetricRef` / `ColorRef` and theme tokens over `Px(…)` literals.
- If you need per-component deviations, expose them via `*Style` overrides (see `docs/shadcn-style-override-patterns.md`).

## Evidence anchors (where to look)

- Theme preset + seeded metrics: `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`
- Token semantics and resolution:
  - `docs/adr/0032-style-tokens-and-theme-resolution.md`
  - `docs/adr/0050-theme-config-schema-and-baseline-tokens.md`
  - `docs/shadcn-style-token-conventions.md`
- Shadows and elevation:
  - `docs/adr/0060-shadows-and-elevation.md`
- Minimal app-side override example:
  - `docs/effects-authoring.md` (“Minimal app-side token override example”)

## Common pitfalls

- Tweaking per-component magic numbers instead of applying a token-level override.
- Changing many token axes at once (makes it hard to diagnose “what caused the vibe change”).
- Letting spacing/radius drift across screens (pick a density and stick to it).

## Related skills

- `fret-ui-ux-guidelines` (app-level hierarchy and composition)
- `fret-layout-and-style` (how tokens apply via `UiBuilder` patches)
