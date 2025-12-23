# ADR 0050: Theme Config Schema and Baseline Tokens (P0)

Status: Accepted
Scope: Framework-level styling contract (Fret); theme content remains app-owned

## Context

ADR 0032 establishes that Fret must adopt a typed token-based styling system with explicit theme
resolution rules. To avoid a late-stage styling rewrite, we need a concrete P0 baseline:

- a minimal token set that covers the editor surfaces we already ship (dock chrome, lists/trees,
  menus, text inputs),
- a theme config schema that can be loaded from files and layered,
- stable naming that can evolve without breaking widget code.

We also want to stay compatible with the ecosystem patterns proven by:

- **Godot**: themes are layered and cached via a theme context; controls query themed items rather
  than hard-coding values.
- **gpui-component**: an ergonomic `cx.theme()` API and JSON theme configs using dotted keys.

## Decision

### 1) Baseline token set (P0)

Fret defines a small set of typed tokens, grouped by function:

**Core surfaces**

- `color.surface.background`
- `color.panel.background`
- `color.panel.border`

**Text**

- `color.text.primary`
- `color.text.muted`
- `color.text.disabled`

**Selection / states**

- `color.accent`
- `color.selection.background`
- `color.hover.background`
- `color.focus.ring`

**Menus**

- `color.menu.background`
- `color.menu.border`
- `color.menu.item.hover`
- `color.menu.item.selected`

**Lists / trees**

- `color.list.background`
- `color.list.border`
- `color.list.row.hover`
- `color.list.row.selected`

**Scrollbars**

- `color.scrollbar.track`
- `color.scrollbar.thumb`
- `color.scrollbar.thumb.hover`

**Viewport overlays (editor)**

- `color.viewport.selection.fill`
- `color.viewport.selection.stroke`
- `color.viewport.marker`
- `color.viewport.drag_line.pan`
- `color.viewport.drag_line.orbit`
- `color.viewport.gizmo.x`
- `color.viewport.gizmo.y`
- `color.viewport.gizmo.handle.background`
- `color.viewport.gizmo.handle.border`
- `color.viewport.rotate_gizmo`

**Metrics**

- `metric.radius.sm`, `metric.radius.md`, `metric.radius.lg`
- `metric.padding.sm`, `metric.padding.md`
- `metric.scrollbar.width`

These tokens are intentionally minimal. Additional tokens may be added as new widgets/components
appear, but existing token names should remain stable.

### 2) Theme config schema (JSON, dotted keys)

Theme files are authored in sRGB hex and map into the token set using stable dotted keys.

Top-level fields:

- `name` (string)
- `author` (string, optional)
- `url` (string, optional)

Token values:

- `colors`: a flat object where keys are dotted token keys and values are hex strings:
  - `#RRGGBB` or `#RRGGBBAA`
- `metrics`: a flat object where keys are dotted metric keys and values are numbers (pixels):
  - e.g. `6`, `8`, `10`

Example:

```json
{
  "name": "HardHacker-inspired Dark",
  "author": "HardHackerLabs (palette reference)",
  "url": "https://github.com/hardhackerlabs/themes",
  "colors": {
    "color.surface.background": "#282433",
    "color.text.primary": "#EEE9FC",
    "color.selection.background": "#3F3951",
    "color.accent": "#E965A5"
  },
  "metrics": {
    "metric.radius.md": 8,
    "metric.scrollbar.width": 10
  }
}
```

### 3) Color space and resolution output

- Theme colors in files are authored in **sRGB**.
- The resolved values exposed to widgets and `SceneOp` are **linear** `fret_core::Color`.
- The sRGB→linear conversion happens during theme application (CPU-side), consistent with ADR 0040.

### 4) Layering rules (P0)

Theme resolution composes layers in the following precedence order:

1. component overrides (explicit style parameters)
2. window overrides (future work)
3. project theme (future work)
4. user theme (future work)
5. default theme (built-in)

P0 implementation may start with (5) + optional single file override, but the precedence order is
considered part of the contract.

### 5) Reactive updates

The theme service must expose a monotonic `theme_revision` that changes when theme values change.
Widgets may cache resolved styles keyed by `theme_revision`.

### 5.1) Extension tokens (namespaced dotted keys)

In addition to the baseline typed tokens above, the theme system stores **all parsed theme entries**
in a key/value map so that component libraries can consume namespaced tokens without changing `fret-ui`.

- Unknown keys are allowed and preserved (e.g. `component.button.*`, `engine.node_graph.*`).
- Components may query by string key (best effort) and fall back when missing.
- Extensions must be namespaced to avoid collisions.

This aligns with the “typed API for core + string keys for extensibility” model used by editor-grade
frameworks and keeps `fret-ui` reusable for third-party component ecosystems.

### 6) Boundary: theme vs renderer/viewport clear

The theme system is responsible for **UI chrome** (panels, lists, menus, text inputs) and should
not implicitly change engine rendering output.

- UI surfaces should paint an explicit background (e.g. `color.surface.background`) so the UI does
  not visually depend on the runner `clear_color`.
- Viewport clear colors and scene backgrounds are **engine/editor settings** (or scene/environment
  state), not theme tokens. A future editor may expose a preference for viewport background, but it
  should not be coupled to UI themes by default.

## Consequences

- Widgets stop hard-coding RGBA/px values and become theme-driven early.
- We can add more themes later (including importing palettes like HardHacker) without rewriting
  widget code.
- Docking chrome and editor surfaces can be made consistent across platforms.

## Gap vs Godot Theme Items (Guidance)

Godot’s editor theme system goes beyond colors/metrics. It also themes:

- icons (tinted folder icons, toolbar icons),
- fonts and font sizes per control class,
- per-control styleboxes (margins/padding, borders, hover/pressed variants),
- numeric constants (separators, grabber sizes).

Fret’s P0 baseline intentionally starts with `colors` + `metrics`. As we approach “Unity/Godot
comfort”, we should expand tokens in the same direction:

- add typography tokens (`font.body`, `font.mono`, sizes, line heights),
- add per-surface chrome tokens (dock tab active/inactive, separators),
- add icon tint tokens and an icon registry boundary (renderer-owned atlas),
- add “stylebox-like” presets as structured tokens (padding/border/radius/shadow),
  rather than stringly-typed CSS.

## References

- ADR 0032: `docs/adr/0032-style-tokens-and-theme-resolution.md`
- gpui-component theme system (schema + registry + `cx.theme()`):
  - `repo-ref/gpui-component/crates/ui/src/theme/mod.rs`
  - `repo-ref/gpui-component/crates/ui/src/theme/schema.rs`
- Godot theme context and editor theme generation:
  - `repo-ref/godot/scene/theme/theme_db.h`
  - `repo-ref/godot/editor/themes/theme_classic.cpp`
  - `repo-ref/godot/editor/themes/editor_theme_manager.cpp`
- HardHacker palette reference (Apache-2.0):
  - https://github.com/hardhackerlabs/themes
