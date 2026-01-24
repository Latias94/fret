# Authoring Ergonomics: Fluent Builder Coverage Audit (v1)

Status: Draft (workstream note; not an ADR)

This document audits the current fluent authoring surface in Fret (ecosystem layer) and outlines a v1 plan to close the
ergonomics gap vs Zed/GPUI and `gpui-component` without violating Fret’s layering rules.

Tracking:

- TODO tracker: `docs/workstreams/authoring-ergonomics-fluent-builder-todo.md`
- Shadcn declarative progress: `docs/shadcn-declarative-progress.md` (component parity + `ui()` coverage table)
- Authoring-model ADR (macros planned, not implemented): `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
- Unified builder ADR (ecosystem-only): `docs/adr/0175-unified-authoring-builder-surface-v1.md`

---

## 1. Scope and Non-Goals

In scope (v1):

- Improve *authoring density* (fewer lines / less boilerplate) for shadcn-ish UI composition.
- Expand the fluent surface around the existing `UiPatch` model (chrome + layout) in ecosystem.
- Provide a clear mapping from “GPUI-style chains” to “Fret-style chains”, including what is intentionally different.

Non-goals (v1):

- Do not move policy into `crates/fret-ui` (mechanisms/contracts only).
- Do not require proc-macro derive to be productive (derive is valuable, but should not block the “golden path”).
- Do not chase 1:1 API parity with GPUI if it conflicts with Fret’s element taxonomy (e.g. `div().h_flex()` vs `cx.flex(...)`).

---

## 2. Current Fret Surfaces (What Exists Today)

### 2.1 Unified patch chain (`ui()` in ecosystem)

Primary surface:

- `ecosystem/fret-ui-kit/src/ui_builder.rs`
  - `UiPatch { chrome: ChromeRefinement, layout: LayoutRefinement }`
  - `UiBuilder<T>` provides `style_*` + `layout_*` chains and merges patches.
  - Types opt in via `UiPatchTarget`, `UiSupportsChrome`, `UiSupportsLayout`.

Where it plugs in:

- `ecosystem/fret-ui-shadcn/src/ui_ext/*` (component opt-in glue)

### 2.2 Minimal `Styled<T>` chain (chrome-only)

- `ecosystem/fret-ui-kit/src/styled.rs`
  - Provides a small `Styled<T>::px_2().rounded_md().finish()` chain for types implementing `RefineStyle`.
  - This is currently a *subset* of what `UiBuilder` can express, and it duplicates concepts (`ChromeRefinement` merging).

Observation:

- Decision (v1): keep `Styled<T>` intentionally tiny and chrome-only.
  - Purpose: an escape hatch for types that can accept **only** `ChromeRefinement` but are not `UiPatchTarget`
    (or where `LayoutRefinement` would be a no-op / misleading).
  - Golden path: `ui()` is the single recommended authoring chain for shadcn recipes.
  - Export strategy: `fret-ui-shadcn::prelude` does not re-export `.styled()` to avoid suggesting it as the default.

---

## 3. GPUI / gpui-component Reference (What We’re Comparing Against)

Reference surface (non-normative, used for ergonomics benchmarking):

- `repo-ref/gpui-component/crates/ui/src/styled.rs`
  - `h_flex()`, `v_flex()` convenience constructors
  - `StyledExt` methods such as `paddings(...)`, `margins(...)`, debug borders, `popover_style(...)`, `corner_radii(...)`

Key difference to keep in mind:

- GPUI’s `Styled` chain is attached to a “DOM-like element builder” (`div()`), while Fret often chooses explicit element
  kinds (`Container`, `Flex`, `Row`, `ScrollArea`, etc.) at construction time.

---

## 4. Coverage Audit (Delta That Impacts Authoring Feel)

### 4.1 “Convenience presets” gap

gpui-component provides opinionated presets that compress common recipes:

- `popover_style(cx)`
- `focused_border(cx)`
- `debug_*()` helpers

Fret today:

- These are mostly implemented as ad-hoc recipe code per component (good for layering), but not exported as a
  discoverable fluent chain on the authoring path.

### 4.2 “Edges helpers” gap

gpui-component exposes `paddings(Edges)` / `margins(Edges)` which is a big ergonomics win for “copy geometry from spec”.

Fret today:

- `UiBuilder` supports `paddings/margins/insets` via a token-aware 4-edge type (`Edges4`).
  - This avoids the “repeat `pt/pr/pb/pl`” pattern when porting exact geometry from specs/goldens.
  - `fret_core::Edges` remains Px-only (`crates/fret-core/src/geometry.rs`) and is not used for authoring.

### 4.3 “Per-corner radii” gap

gpui-component supports `corner_radii(Corners)` directly.

Fret today:

- Per-corner radii are supported via `Corners4` + `ChromeRefinement::corner_radii(...)` and `rounded_tl/tr/br/bl`,
  and are available on the `ui()` chain.

### 4.4 “Node constructor” gap (`div().h_flex()` feel)

gpui-component’s authoring loop starts from a single constructor (`div()`), then refines.

Fret today:

- The patch chain (`ui()`) assumes you already have a component/type to patch.
- Layout constructors are available via `fret-ui-kit::ui::{h_flex, v_flex}` which return a patchable builder (plus
  gap/alignment shorthands), and can be re-exported by shadcn prelude for app code.
- `fret-ui-kit::ui::stack` exists for overlay/layer composition (patchable builder).

---

## 5. Suggested v1 Direction (Aligned with Fret Design)

Recommendation: keep the existing `UiPatch` model, but provide two complementary “golden path” entry points:

1) **Patch existing components** (already present): `component.ui().px_2().w_full().into_element(cx)`
2) **Build layout nodes fluently** (present): `ui::h_flex/v_flex` provide a patchable flex constructor path that can
   participate in the same authoring vocabulary as components.

This keeps the layering clean:

- `crates/fret-ui`: no policy; no tailwind-ish API.
- `fret-ui-kit`: owns the “tailwind-ish” token scale and fluent ergonomics.
- `fret-ui-shadcn`: owns shadcn taxonomy + presets built on top of `fret-ui-kit`.

---

## 6. Mapping Table: gpui-component → Fret

This is not a 1:1 parity target; it is a “what should feel equally easy” checklist.

| gpui-component | Intent | Fret today | v1 action |
| --- | --- | --- | --- |
| `refine_style(&StyleRefinement)` | apply a patch | `ui().style(ChromeRefinement)` / `ui().layout(LayoutRefinement)` | Keep; add more shorthands |
| `h_flex()` / `v_flex()` | start a flex layout | `fret-ui-kit::ui::h_flex/v_flex` (patchable builder) | Done |
| `paddings(Edges)` | batch edge edits | `UiBuilder::paddings(Edges4<...>)` (token-aware + px-friendly) | Done |
| `margins(Edges)` | batch edge edits | `UiBuilder::margins(Edges4<...>)` (token-aware + px-friendly, supports `auto`) | Done |
| `debug_*()` | debug borders | `UiBuilder::debug_border_*` (debug-only gated) | Done (different palette) |
| `focused_border(cx)` | focus border | `UiBuilder::focused_border()` | Done |
| `popover_style(cx)` | common popover skin | `UiBuilder::popover_style()` (shadcn policy preset) | Done |
| `corner_radii(Corners)` | per-corner radii | `UiBuilder::corner_radii(Corners4<...>)` | Done |

---

## 7. Next Steps

Execute the TODO tracker in small, reviewable slices:

- Start with **helpers** (edges, debug helpers, per-corner radii).
- Then expand the **layout constructors** surface (e.g. `ui::stack`, `ui::grid`, plus more shorthands).
- Keep `docs/shadcn-declarative-progress.md` updated when the authoring surface changes.

---

## 8. What “Full Coverage” Should Mean (v1)

Avoid measuring parity by “number of fluent methods”.

Instead, v1 “coverage” is:

1) **A single, boring golden path** for 90% of shadcn-style authoring:
   - patch components via `ui()`
   - author layout nodes via `fret-ui-kit::ui::*` constructors returning `UiBuilder<...>`
2) **Token-aligned geometry without repetition**:
   - `Edges4` / `Corners4` batch edits
   - common shorthands (`p_2`, `mx_2`, `rounded_md`, `shadow_sm`, etc.)
3) **Layer-correct presets**:
   - kit-level “mechanical” presets (debug borders, focused border)
   - shadcn-level “policy” presets (popover/dialog/menu surfaces)

This keeps the surface cohesive without turning `fret-ui-kit` into a general-purpose Tailwind port.

---

## 9. Proposed v1 Backlog (Highest ROI)

These are the remaining items that most directly improve authoring density for editor-ish UI:

1) **More shadcn surface presets** (as needed):
   - keep these in `fret-ui-shadcn` (policy layer)
   - expose them as discoverable helpers, not ad-hoc per component
2) **A patchable container constructor** (layout node, not a component):
   - `ui::container(cx, |cx| ...) -> UiBuilder<...>` as the “default box”
   - lets authors write layered barriers/underlays without dropping to raw `cx.container(...)`
3) **Text authoring v1** (minimal, but ergonomic):
   - `ui::text(...)` / `ui::label(...)` constructors with a small, typed text refinement surface
   - keep the scope narrow: size/weight/color + a default shadcn-aligned line-height
