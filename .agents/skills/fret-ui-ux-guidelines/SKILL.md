---
name: fret-ui-ux-guidelines
description: Build beautiful, editor-grade apps with Fret by applying shadcn-style visual hierarchy, spacing rhythm, and interaction affordances (keyboard-first, focus-visible, predictable layering). Use when designing new screens, refactoring layouts, or turning a “works” UI into a cohesive “looks good” UI.
---

# Fret UI/UX guidelines (make it look good)

## When to use

Use this skill when:

- You’re designing a new screen (settings, dashboard, inspector, workspace shell).
- The UI “works” but feels messy, inconsistent, or visually noisy.
- You want editor-grade usability: keyboard-first, focus-visible, predictable layering.

This skill is about **app-level UI composition** and **visual hierarchy**. For theme/tokens, use:

- `fret-design-system-styles` (baseline preset + `ThemeConfig` overrides)

For concrete component/app recipes, use:

- `fret-shadcn-app-recipes`

## Inputs to collect (ask the user)

1. **Surface type**: editor workspace / settings / data-heavy / content viewer
2. **Scheme**: light / dark
3. **Density**: compact / default / comfortable (editor UIs usually want compact)
4. **Vibe keywords**: minimal / soft / hud / neubrutal / glass overlays / high-contrast
5. **Primary modality**: keyboard-first / mouse-first / mixed

If the user has no preference: default to **dark + compact editor** and iterate.

## Smallest starting point (one command)

- `cargo run -p fretboard -- dev native --bin workspace_shell_demo`

## Quick start (golden path)

- Prefer `use fret_ui_shadcn::prelude::*;` for UI code to stay on the shadcn-aligned path.
- Apply a baseline shadcn preset early (e.g., New York v4) and express “look” via **token overrides**.
- Use tokens (`Space`, `Radius`, `MetricRef`, `ColorRef`) instead of sprinkling `Px(...)` literals.
- Keep one scroll root per pane; avoid nested scroll unless you are deliberately composing an inner scroll (tables, listboxes).

## Quick style picks (works today)

These are intentionally **editor-focused** styles shipped in `fret-design-system-styles`:

- `editor-compact`: default for editor shells (dense lists/tables/inspector).
- `editor-comfortable`: forms-heavy apps where readability beats density.
- `editor-soft`: friendlier, more rounded “soft UI” look (diffuse elevation).
- `editor-neubrutal`: flat + bold outlines (high contrast, playful).
- `editor-hud`: dark sci‑fi HUD with neon accent (developer tools / monitoring).
- `overlays-glass`: apply selectively to overlays (dialogs/popovers), not globally.

Use the generator to get a `ThemeConfig` patch:

```bash
# From the `fret-design-system-styles` skill directory:
python scripts/stylegen.py --suggest "dark compact hud editor"
python scripts/stylegen.py --style editor-compact > theme_overrides.json
```

## Workflow

1. Pick the target surface type and density (editor shells usually want compact).
2. Apply a baseline theme preset + small token overrides (avoid per-component magic numbers).
3. Compose screens using a small set of repeatable patterns (workspace shell, inspector, toolbars).
4. Add stable `test_id` to interactive affordances and keep one repro script for any tricky interaction.

## Definition of done (what to leave behind)

- The target surface has a clear hierarchy (3 surface layers) and consistent spacing rhythm (tokened, not magic numbers).
- Density and focus visuals are coherent across the surface (control heights, ring width/offset).
- Interactive affordances are keyboard-first (focus-visible everywhere; no hover-only essential actions).
- Tricky interactions (menus/select/combobox/docking) have stable `test_id` and at least one repro gate (diag script or invariant test).
- Any style changes are expressed as `ThemeConfig` overrides or shared recipes (not per-component drift).

## Visual hierarchy playbook

Aim for **3 surface layers** (consistent across the app):

1. App background (`background`)
2. Panel surfaces (`card` / `muted`)
3. Elevated overlays (`popover` / `dialog`) + strong focus ring (`ring`)

Rules:

- Use **spacing** first, borders second, shadows last. Don’t stack heavy borders + heavy shadows everywhere.
- Keep separators subtle; prefer grouping via padding + section titles.
- Ensure **selected** state and **focus** state are distinct:
  - selection: `accent` / `muted`
  - focus: `ring` (focus-visible only)

## Spacing rhythm (editor-grade)

Pick a density profile and stick to it.

- Controls in the same region should share height (inputs/buttons/selects).
- Lists/tables should use a consistent row height (no per-screen tweaks).
- Use a small set of gaps/paddings (e.g. “tight”, “normal”, “section”) mapped to tokens.

Tip: if the UI feels “messy”, **reduce the number of distinct spacing values** before touching colors.

## Navigation and affordances (the “feels good” part)

- Keyboard-first defaults:
  - Every clickable thing needs a focus-visible style.
  - Menus/select/combobox should be navigable without moving focus unexpectedly (active-descendant where needed).
- Hover is additive, not required. Don’t hide core actions behind hover-only affordances.
- Empty/loading/error states are first-class:
  - empty: explain what to do next (CTA)
  - loading: skeletons for layout stability
  - error: inline message + retry action

## Common “beautiful editor” layout patterns

### 1) Workspace shell

- Top: `Menubar` (commands as source of truth)
- Left: `Sidebar` (nav + search + sections)
- Center: primary viewport (canvas/text/code)
- Right: inspector (property list)
- Bottom: optional “console/log” panel

Use docking only when users need to rearrange panes:

- `fret-docking-and-viewports` + `app-docking-workspace` recipe

### 2) Inspector panel

- Group properties into sections with short headers.
- Use consistent label column width (or a single-column “stacked” layout in narrow panes).
- Avoid long forms in modals; prefer side sheets for deep settings.

### 3) Toolbars

- Group actions (primary vs secondary) and separate groups with a subtle separator.
- Prefer icons + tooltips for dense editor controls, but keep at least the primary actions labeled somewhere.

## Common pitfalls

- Many different radii/shadow styles across screens.
- 7+ different padding values on one page.
- Nested scroll surfaces “fighting” each other.
- Relying on hover-only UI for essential actions.
- Inconsistent focus ring thickness/offset across controls.

## Evidence anchors

- Theme/tokens: `fret-design-system-styles`
- Layout/styling primitives: `fret-layout-and-style`
- Overlays/focus: `fret-overlays-and-focus`
- Recipes + parity: `fret-shadcn-app-recipes`, `fret-shadcn-source-alignment`

## Related skills

- `fret-design-system-styles` (pick a baseline + generate `ThemeConfig` patches)
- `fret-shadcn-app-recipes` (concrete component composition patterns)
