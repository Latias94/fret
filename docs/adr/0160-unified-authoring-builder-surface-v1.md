# ADR 0160: Unified Authoring Builder Surface (v1)

- Status: Proposed
- Date: 2026-01-16

## Context

Fret’s long-term UI direction is a GPUI-style “build an element tree every frame” runtime substrate (ADR 0028),
with ecosystem crates (`ecosystem/*`) providing policy-heavy components and recipes (ADR 0066).

Today, our *mechanisms* for style and layout patches are already strong:

- `ChromeRefinement` (control chrome: padding/radius/border/colors),
- `LayoutRefinement` (layout-affecting patches: margin/position/size/flex/grid-ish primitives),
- token-first sizing/density primitives (`Space`, `Radius`, `MetricRef`),
- a declarative-only `fret-ui-shadcn` surface with per-component `refine_style(...)` / `refine_layout(...)` patterns.

However, the *authoring experience* is still fragmented:

- Application code mixes several styles:
  - component-specific `refine_style/refine_layout`,
  - direct construction of runtime props (`ContainerProps`, `LayoutStyle`) with `Theme::global(...)`,
  - `StyledExt` exists, but is not consistently usable across shadcn components (missing trait integration).
- This fragmentation reduces code density, makes UI harder to scan, and slows iteration.
- For a general-purpose UI framework, the authoring surface must be:
  - **consistent** (one way to do common things),
  - **discoverable** (autocomplete tells you the vocabulary),
  - **composable** (works uniformly across shadcn components and non-shadcn building blocks),
  - **mechanism/policy correct** (no policy leaking into `crates/*`).

This ADR locks a *single*, ecosystem-owned “golden path” authoring interface: a unified builder surface that
applies typed style/layout patches consistently across components.

## Goals

1. Provide one primary authoring pattern for ecosystem UI:
   - “component/value → builder chain → `into_element(cx)`”.
2. Make common UI overrides (spacing/size/radius/border/colors) highly discoverable and consistent across:
   - `fret-ui-shadcn` components,
   - `fret-ui-kit` primitives/recipes.
3. Preserve kernel boundaries:
   - no policy surfaces added to `crates/fret-ui` (ADR 0066),
   - no new renderer contracts required.
4. Keep token-first semantics:
   - users express intent via typed tokens, not ad-hoc numeric px everywhere (ADR 0032 / ADR 0056).

## Non-goals

- Introducing a new runtime styling system (CSS, class strings, selector engine).
- Changing the stable runtime contract surface (`crates/fret-ui`) (ADR 0066).
- Mandating proc-macros; this ADR is achievable with plain Rust builder APIs.
- Replacing shadcn taxonomy; `fret-ui-shadcn` remains the “names + recipes” layer.

## Decision

### 1) Introduce a unified “UI builder” for ecosystem authoring

We introduce an ecosystem-owned builder API (in `fret-ui-kit`, re-exported by `fret-ui-shadcn`) with:

- a single entry point, e.g. `component.ui()` (name bikesheddable),
- a single terminal operation, `into_element(cx)`.

Conceptually:

```rust
Button::new("Open")
    .ui()
    .px_3()
    .py_2()
    .w_full()
    .rounded_md()
    .into_element(cx)
```

This builder is a *patch aggregator*; it does not render by itself. Rendering still happens in the component’s
`into_element(cx)` (ADR 0039 / ADR 0028).

### 1a) Patch-only roots must still allow a single `ui().into_element(cx, ...)` terminal

Some shadcn components are intentionally *patch-only* at the root level: their `into_element` needs additional
arguments such as trigger/content closures or data callbacks.

To keep the authoring surface consistent (one entry point, one terminal), we standardize on *builder extension traits*
exported by `fret-ui-shadcn`:

- `component.ui().into_element(cx, ...)` is always available for public shadcn components,
- when extra arguments are required, the builder’s `into_element` mirrors the component’s `into_element` signature.

Examples (illustrative):

```rust
Popover::new(open)
    .ui()
    .into_element(cx, |cx| Button::new("Trigger").into_element(cx), |cx| cx.text("Panel"));

DataTable::new()
    .ui()
    .into_element(cx, data, data_revision, state, columns, row_key, col_key, cell);
```

Implementation note:

- these extension traits live in `ecosystem/fret-ui-shadcn/src/ui_builder_ext/*` and are re-exported in the
  `fret_ui_shadcn::prelude` so applications can `use fret_ui_shadcn::prelude::*;` and always get the terminal method.

### 2) Unify patch vocabulary under one “UI patch” type

We define a single patch structure, conceptually:

- `chrome: ChromeRefinement`
- `layout: LayoutRefinement`
- “cross-cutting authoring fields” that are universally useful in apps/tests/diagnostics:
  - `test_id`
  - `a11y_label` *(optional; exact shape depends on semantics authoring)*
  - `debug_name` *(optional; used for diagnostics exports / inspector UI when enabled)*
  - `view_cache` hints *(optional; see below)*

Normative merge rule:

- patch composition is “last writer wins” per field, but *structural patches merge*:
  - `ChromeRefinement::merge(...)` and `LayoutRefinement::merge(...)` semantics remain the source of truth.

### 3) One fluent method set (Tailwind-ish, typed)

The unified builder provides a small, stable method vocabulary aligned with existing tokens:

- spacing: `p_*`, `px_*`, `py_*`, `m_*`, `gap_*` (where applicable),
- sizing: `w_full`, `h_full`, `min_h_*`, `max_w_*`, `w_px(MetricRef)`, ...
- chrome: `rounded_*`, `border_*`, `bg(ColorRef)`, `text_color(ColorRef)`, ...

Rules:

- methods must resolve to typed refinements (`ChromeRefinement` / `LayoutRefinement`);
  they must not secretly mutate runtime `LayoutStyle` directly.
- where “gap” is a container concept (row/column/stack), it is part of the container’s builder surface
  (see `Stack` wrappers below).

### 4) Trait integration: shadcn components must opt into the unified builder

All public `fret-ui-shadcn` components must implement a single trait that allows `ui()` to exist, for example:

- `UiExt` (extension trait): provides `fn ui(self) -> UiBuilder<Self>`.

This is the key to preventing multiple authoring styles from coexisting indefinitely.

### 5) Provide builder-friendly wrappers for common non-shadcn building blocks

To avoid “shadcn-only ergonomics”, `fret-ui-kit` should provide builder-friendly wrappers for:

- basic layout containers (stack/row/column),
- basic boxes/panels (a `Box`/`Div`-like container that wraps `cx.container(...)`),

and these wrappers must share the same builder vocabulary and patch types.

### 6) Optional: expose view-cache boundaries through the builder (ecosystem-only)

If view caching is enabled at the runtime level, authors frequently want a *composition-friendly* boundary.

This ADR allows (but does not require) the builder to expose a mechanism-only wrapper:

- `.view_cache()` / `.view_cache_contained()` / `.view_cache_uncontained()`

which expands to a declarative `ViewCache` wrapper element at render time.

Important rule:

- the builder only exposes the mechanism; policy (“when should I cache?”) remains a recipe/app decision.

### 7) Precedence and correctness rules (normative)

1. Component defaults remain authoritative unless overridden:
   - user patches *merge over* component defaults.
2. Patch application must be explicit and local:
   - retained widgets must not silently accept `LayoutRefinement` (consistent with the warning in `LayoutRefinement` docs).
3. The builder must not capture `Theme` eagerly:
   - token resolution happens at `into_element(cx)` time using `Theme::global(&*cx.app)` (keeps behavior consistent under theme changes).
4. The builder must not introduce new runtime contracts:
   - it is a pure ecosystem authoring surface.

## Implementation Plan (non-binding)

### Phase 0: Narrow-slice MVP (prove the UX)

1. Add `UiBuilder<T>` + `UiPatch` in `ecosystem/fret-ui-kit`.
2. Implement `UiExt::ui()` for a small set of high-frequency shadcn components:
   - `Button`, `Input`, `Popover`, `Tooltip`.
3. Provide one or two builder-friendly containers in `fret-ui-kit`:
   - `Box` (container with chrome+layout),
   - `HStack`/`VStack` wrappers or builder-friendly equivalents.
4. Update one representative demo screen (components gallery) to use the new API and keep it as the “golden path” example.

Acceptance (authoring ergonomics):

- Equivalent UI structure uses significantly fewer “style plumbing” lines (target: 30–50% reduction in a representative screen).
- Autocomplete shows a single consistent style vocabulary regardless of which component is used.

### Phase 1: Expand to the full shadcn surface

- All public shadcn components implement the builder entrypoint.
- Patch-only root elements also provide `ui().into_element(cx, ...)` via builder extension traits (see 1a).
- Existing `refine_style/refine_layout` remain temporarily but are deprecated once the builder is stable.

### Phase 2: Integration and tooling polish

- Add optional `debug_name/test_id` helpers to improve diagnostics exports (ADR 0036 / ADR 0159).
- If desired, add a small “authoring lint” (clippy-like) or tests ensuring new shadcn components do not ship without builder support.

## Alternatives Considered

### A) Keep per-component `refine_style/refine_layout` only

Rejected: it preserves fragmentation, encourages ad-hoc naming drift, and makes “one golden path” hard to teach.

### B) Use `StyledExt` as-is and only implement `RefineStyle` for shadcn components

Rejected as the end-state: `StyledExt` currently models only `ChromeRefinement`, while authoring also needs
layout-affecting patches (`LayoutRefinement`) and cross-cutting fields. It can be used as an internal building block,
but the public authoring surface should converge on a single builder.

### C) Add a CSS-like class string system

Rejected: it conflicts with token-first typed design goals and introduces a parsing/validation surface that is hard to
stabilize early for a Rust-native framework.

### D) Require patch-only roots to use `build()` before `into_element(...)`

Rejected: it introduces an API cliff where some components use `ui().into_element(cx)` while others require
`ui().build().into_element(cx, ...)`. This reduces code density and hurts discoverability (autocomplete does not show
the terminal method on the primary builder type).

## References

- Declarative element model: `docs/adr/0028-declarative-elements-and-element-state.md`
- Component authoring layers: `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
- Runtime vs component boundary: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Token-first theme semantics: `docs/adr/0032-style-tokens-and-theme-resolution.md`, `docs/adr/0056-component-size-and-density-system.md`
- Ecosystem conventions: `docs/adr/0148-component-ecosystem-authoring-conventions-v1.md`, `docs/adr/0154-ecosystem-crate-taxonomy-glue-and-ui-kit-split-v1.md`
- GPUI-style ergonomics reference: `repo-ref/gpui-component/crates/ui/src/styled.rs`
- Coverage tracker: `docs/shadcn-declarative-progress.md`
