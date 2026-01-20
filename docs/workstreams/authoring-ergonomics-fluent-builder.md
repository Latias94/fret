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

- For v1, `ui()` should be the “single obvious way” for shadcn authoring. `Styled<T>` is useful as a tiny helper, but
  it risks splitting the ecosystem into two competing patterns.

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

- `ChromeRefinement` exposes `rounded(Radius)` and a few shorthands; per-corner controls are not first-class in the
  `ui()` chain.

### 4.4 “Node constructor” gap (`div().h_flex()` feel)

gpui-component’s authoring loop starts from a single constructor (`div()`), then refines.

Fret today:

- The patch chain (`ui()`) assumes you already have a component/type to patch.
- For layout-only authoring, we already have component-layer helpers like
  `fret-ui-kit::declarative::stack::{hstack, vstack}` (`ecosystem/fret-ui-kit/src/declarative/stack.rs`), but they
  are not integrated with the `ui()` patch chain (authors still pass `LayoutRefinement` explicitly).

---

## 5. Suggested v1 Direction (Aligned with Fret Design)

Recommendation: keep the existing `UiPatch` model, but provide two complementary “golden path” entry points:

1) **Patch existing components** (already present): `component.ui().px_2().w_full().into_element(cx)`
2) **Build layout nodes fluently** (partially present): `stack::hstack/vstack` exist, but we still want an
   integrated path where layout nodes can be patched via `ui()` (or an equivalent builder) without “props struct +
   `LayoutRefinement` ceremony”.

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
| `h_flex()` / `v_flex()` | start a flex layout | `stack::hstack/vstack` (component-layer), or `cx.row/column` (runtime) | Integrate layout constructors with `ui()` patching (or provide a patchable wrapper) |
| `paddings(Edges)` | batch edge edits | no single helper | Add `UiBuilder::paddings(Edges4<...>)` (token-aware + px-friendly) |
| `margins(Edges)` | batch edge edits | no single helper | Add `UiBuilder::margins(Edges4<...>)` (token-aware + px-friendly, supports `auto`) |
| `debug_*()` | debug borders | ad-hoc per component | Add builder debug helpers (debug-only gated) |
| `focused_border(cx)` | focus ring/border | component-local focus ring logic | Add a `ChromeRefinement` preset in kit |
| `popover_style(cx)` | common popover skin | component-local popover recipes | Add a preset in `fret-ui-shadcn` (policy layer) |
| `corner_radii(Corners)` | per-corner radii | not first-class | Add per-corner radius refinement in kit |

---

## 7. Next Steps

Execute the TODO tracker in small, reviewable slices:

- Start with **helpers** (edges, debug helpers, per-corner radii).
- Then add **layout constructors** (`ui::h_flex` / `ui::v_flex`) to reduce “props struct noise”.
- Keep `docs/shadcn-declarative-progress.md` updated when the authoring surface changes.
