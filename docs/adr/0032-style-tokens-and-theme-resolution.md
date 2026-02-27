# ADR 0032: Style Tokens and Theme Resolution (Typed, Editor-Grade)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

Editor UIs need strong visual consistency across:

- docking chrome, panels, inspectors, menus, tooltips,
- multiple OS windows (tear-off),
- embedded engine viewports with UI overlays.

If we do not decide the **styling contract** early, we risk a late rewrite caused by:

- ad-hoc per-widget colors and spacing,
- inconsistent DPI scaling and density modes,
- inability to support user themes and plugin-provided panels consistently.

In addition, Fret already commits to file-scoped settings and layering (ADR 0014). Themes are a natural
consumer of this infrastructure, but the *styling system* must remain framework-level, while the *theme content*
remains app-owned.

References / inspiration:

- Zed settings UX and strong-typed configuration motivation:
  - https://zed.dev/blog/settings-ui
- Zed theme system (non-normative code anchors):
  - theme registry + loading and theme schema content:
    `repo-ref/zed/crates/theme/src/registry.rs`,
    `repo-ref/zed/crates/theme/src/schema.rs`
- Framework vs app scope boundary:
  - `docs/adr/0027-framework-scope-and-responsibilities.md`
- Settings files and layering:
  - `docs/adr/0014-settings-and-configuration-files.md`

## Decision

Adopt a **typed token-based style system** with explicit theme resolution.

### 1) Styling is expressed as tokens, not ad-hoc values

Define a token set (names TBD) such as:

- colors: `Surface`, `Panel`, `Border`, `TextPrimary`, `TextMuted`, `Accent`, `Danger`, ...
- spacing/radius: `Space1..N`, `Radius1..N`
- typography: `FontBody`, `FontMono`, `FontSizeSm/Md/Lg`, `LineHeight...`
- interaction states: `Hover`, `Active`, `Selected`, `Disabled`, `FocusRing`

Widgets consume tokens; they do not hard-code RGBA/px values.

### 2) Theme resolution is a stable contract

Introduce a runtime service (conceptually `Theme` / `StyleSystem`) that:

- resolves tokens to concrete values for the current window DPI scale and density,
- supports per-window overrides (useful for multi-monitor setups),
- supports user-provided theme files layered by scope (ADR 0014).

Color boundary rule:

- Theme files are authored in sRGB (human-friendly), but the resolved values exposed to UI/layout/scene building
  must be **linear** colors to match the `fret-core` display list contract (ADR 0002 / ADR 0040).
  The sRGB→linear conversion happens at theme resolution time (CPU-side or shader-side), not by passing sRGB
  values through `SceneOp`.

### 3) Theme content remains app-owned

Fret provides:

- token definitions and resolution rules,
- APIs for querying tokens in layout/paint.

The editor app provides:

- default theme values,
- optional user theme files,
- plugin theme extensions (namespaced tokens), if desired.

### 4) Density and scaling are first-class

To avoid “pixel drift” and later refactors:

- define a density mode (`compact`/`comfortable` etc.) as a theme input,
- define rounding/pixel-snapping rules for layout outputs (ties into text quality and SDF AA).

### 5) Theme changes are reactive

Theme changes must:

- invalidate affected UI subtrees deterministically,
- integrate with the effects/redraw pipeline (ADR 0001).

## Consequences

- The framework can support Unity/Godot-like editor UX with consistent visuals and customization.
- Plugin panels can look native without copying style values.
- DPI scaling and density changes become manageable without rewriting widget code.

## Open Questions (To Decide Before Implementation)

### Locked P0 Choices

1) **Token schema**: typed tokens + stable string keys.
   - In Rust APIs: tokens are typed (newtypes/enums) to keep widget code correct and discoverable.
   - In theme files: tokens are addressed by **stable, namespaced string keys** (e.g. `color.panel.background`).
   - Plugin/component ecosystems must namespace keys to avoid collisions (e.g. `plugin.my_tool.*`).

2) **Per-component overrides**: allowed, but only as structured “style props”.
   - Widgets may accept optional overrides for a small, well-defined subset (colors, spacing, typography).
   - Overrides merge as: `component overrides` > `window overrides` > `project theme` > `user theme` > `default theme`.
   - Widgets must not accept arbitrary CSS-like strings.

3) **Color space**: standardized by ADR 0040.
   - Theme colors are authored as sRGB values (theme files).
   - Theme resolution produces linear colors for `SceneOp`.
   - Renderer composites in linear and performs the correct conversion to the surface format (ADR 0040).

Additional locked behavior:

- Theme resolution is **window-aware** (DPI scale factor + density mode are inputs).
- Theme changes produce a single “theme revision” increment that participates in layout/paint caching keys.

## Addendum: Named color compatibility keys (2026-02-26)

Some upstream ecosystems (notably Tailwind/shadcn) use named colors directly (e.g. `text-white`,
`bg-black`). Fret's primary contract remains **semantic tokens** (typed keys + stable namespaced
strings), but we also reserve a small set of **named color** keys for ecosystem alignment.

- Reserved named colors (non-semantic): `white`, `black`
- Widgets should prefer semantic tokens, but may use named colors when the upstream source of truth
  is explicitly a named color (e.g. shadcn destructive uses `text-white`).
