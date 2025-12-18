# ADR 0032: Style Tokens and Theme Resolution (Typed, Editor-Grade)

Status: Proposed

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

1) **Token schema**:
   - do we encode tokens as Rust enums only, or also support stable string keys for theme files?
2) **Per-component overrides**:
   - do widgets accept “style props” that override tokens, and how are they merged?
3) **Color spaces**:
   - sRGB vs linear blending assumptions and where conversions happen (ties into renderer).

