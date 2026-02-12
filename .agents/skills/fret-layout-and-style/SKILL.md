---
name: fret-layout-and-style
description: Layout and styling in Fret (`fret-ui-kit` token-driven `LayoutRefinement` / `ChromeRefinement` and the `UiBuilder` fluent surface). Use when implementing layouts (flex/stack/scroll), applying tokens (Space/Radius/MetricRef/ColorRef), or debugging overflow/clipping/focus-ring issues.
---

# Fret layout and style

Fret aims for a **Tailwind-like mental model** without a CSS parser: layout/styling is expressed as
typed patches (`LayoutRefinement`, `ChromeRefinement`) and applied via the ecosystem `UiBuilder`
surface (`ui()`).

## When to use

Use this skill when:

- Implementing layouts (flex/stack/scroll) or composing complex panes (toolbars, inspectors, shells).
- Debugging overflow/clipping/focus ring issues (things disappear or hit-testing feels wrong).
- Applying token-driven styling (`Space`, `Radius`, `MetricRef`, `ColorRef`) consistently.

## Inputs to collect (ask the user)

Ask these before touching layout/styling (most bugs are “wrong overflow root” or “wrong patch layer”):

- Target surface: toolbar/inspector/panel shell/list/table/overlay content?
- Problem class: sizing/layout (LayoutRefinement) vs visual chrome (ChromeRefinement)?
- Overflow intent: should the control clip content, or should focus rings remain visible?
- Constraints: scroll root ownership (which container scrolls) and max size constraints?
- Token policy: is this a one-off tweak or should it become a token override?

Defaults if unclear:

- Use `UiBuilder` patches with tokens, keep pressable/root overflow visible, and clip only inside chrome when needed.

## Quick start

**Key concepts:**

- `LayoutRefinement`: size/position/margin/inset/flex/overflow (declarative-only)
- `ChromeRefinement`: padding/radius/border/shadow/colors (not layout-affecting)
- `Space` / `Radius` / `MetricRef` / `ColorRef`: token-driven values (theme-resolved)
- `UiBuilder`: `value.ui().px_3().w_full().rounded_md().into_element(cx)` (ADR 0145)

### A “card” container with padding + border + radius

```rust
use fret_ui_kit::prelude::*;

pub fn card<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::container(cx, |cx| {
        vec![ui::text(cx, "Card content").into_element(cx)]
    })
    .p_4()
    .rounded_md()
    .border_1()
    .bg(ColorRef::Token { key: "card", fallback: ColorFallback::ThemePanelBackground })
    .into_element(cx)
}
```

### Horizontal layout (row) with gaps and alignment

```rust
use fret_ui_kit::prelude::*;

pub fn toolbar<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::h_flex(cx, |cx| {
        vec![
            ui::text(cx, "Left").into_element(cx),
            ui::container(cx, |_| Vec::new()).w_full().into_element(cx), // spacer
            ui::text(cx, "Right").into_element(cx),
        ]
    })
    .gap_metric(MetricRef::space(Space::N2))
    .items_center()
    .px_3()
    .py_2()
    .into_element(cx)
}
```

## Workflow (recommended checklist)

1. Prefer `ui()`/`UiBuilder` surfaces in ecosystem components (patchable and consistent).
2. Use tokens first (spacing/radius/colors) before reaching for raw `Px(...)` literals.
3. Keep overflow responsibilities explicit:
   - Don’t clip focus rings accidentally.
   - Clip only on the chrome container when needed.
4. When something “does nothing”, verify you’re on declarative-only surfaces (refinements do not apply to retained widgets).

## Definition of done (what to leave behind)

- Layout vs chrome responsibilities are explicit (no “mystery no-op” refinements).
- Spacing/radius/colors use tokens (`Space`/`Radius`/`MetricRef`/`ColorRef`) unless there is a clear exception.
- Overflow/clipping is correct (focus rings not accidentally clipped; hit-testing matches visuals).
- If you changed a reusable pattern, it’s expressed as a recipe/helper (not repeated magic numbers).
- If this fixes a regression, there is a small repro artifact (unit test or `tools/diag-scripts/*.json` with `test_id`).

## Practical rules (prevents common regressions)

### 1) Refinements are declarative-only

`LayoutRefinement` / `ChromeRefinement` are **not** silently accepted by retained widgets.
If something “looks like Tailwind but does nothing”, it is probably applied to the wrong layer.

Reference: `ecosystem/fret-ui-kit/src/style/{layout.rs,chrome.rs}`.

### 2) Overflow/clipping: don’t clip focus rings by accident

Overflow is a **paint + hit-test contract**. The recommended structure for controls:

- `Pressable (Overflow::Visible)` → `Chrome container (Overflow::Clip + rounded)` → content

Use the helper:

- `fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props`

Reference: `docs/component-authoring-contracts.md` (“Overflow and clipping”, ADR 0086).

### 3) Use `*_build` when iterators borrow `&mut cx`

If children are built from iterators that capture `&mut cx`, prefer:

- `ui::h_flex_build(cx, |cx, out| { ... })`
- `ui::container_build(cx, |cx, out| { ... })`

This avoids borrow-checker pitfalls while keeping identity stable via `cx.keyed(...)`.

## Evidence anchors (where to look)

- Tailwind-like layout semantics: `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
- Tokens + theme resolution: `docs/adr/0032-style-tokens-and-theme-resolution.md`, `docs/adr/0050-theme-config-schema-and-baseline-tokens.md`
- Component authoring checklist: `docs/component-authoring-contracts.md`
- Code entry points:
  - `ecosystem/fret-ui-kit/src/ui_builder.rs` (`UiBuilder`)
  - `ecosystem/fret-ui-kit/src/style/layout.rs` (`LayoutRefinement`)
  - `ecosystem/fret-ui-kit/src/style/chrome.rs` (`ChromeRefinement`)

## Common pitfalls

- Applying refinements to the wrong layer (no-op styling).
- Clipping at the pressable/root level and losing focus rings.
- Mixing many raw pixel values instead of converging on a token scale.

## Related skills

- `fret-component-authoring` (identity/state/invalidation)
- `fret-design-system-styles` (baseline theme + token overrides)
- `fret-ui-ux-guidelines` (app-level hierarchy and composition)
