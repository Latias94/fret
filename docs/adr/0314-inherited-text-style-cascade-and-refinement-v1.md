# ADR 0314: Inherited Text Style Cascade and Refinement (v1)

Status: Accepted
Date: 2026-03-07

## Context

Fret currently has a narrow inherited styling mechanism for foreground color:

- `ForegroundScope` in `crates/fret-ui/src/element.rs`
- `current_color` authoring helpers in `ecosystem/fret-ui-kit/src/declarative/current_color.rs`

That surface solves `currentColor`-style icon/text color inheritance, but it does **not** solve the
broader problem of text typography inheritance.

Today, typography defaults come from one of two places:

1. a leaf text node's explicit `TextStyle`, or
2. the theme default resolved in `crates/fret-ui/src/text/props.rs`.

This leaves a gap for subtree-scoped typography outcomes that are common in component ecosystems:

- description/body text inside alert/dialog/sheet/popover/card/field surfaces,
- AI/content components that want a subtree-local prose/control typography default,
- direct compound-children APIs where text nodes are authored by descendants but should inherit the
  parent surface's typography contract.

The current result is predictable drift:

- components re-implement the same `font.size` / `font.line_height` lookups ad hoc,
- typography inheritance is approximated by recursively patching descendants,
- direct children composition becomes harder to reason about,
- later migration to a real inherited text-style model becomes more expensive.

This is a hard-to-change runtime contract because inherited typography affects:

- layout measurement,
- text shaping cache keys,
- authoring ergonomics,
- component layering between `crates/*` and `ecosystem/*`.

## External references

### GPUI / Zed

GPUI already models cascading text style as an explicit runtime concept:

- container style stores `text: TextStyleRefinement`:
  - `repo-ref/zed/crates/gpui/src/style.rs`
- authoring APIs like `.text_color()`, `.font_weight()`, and `.text_size()` refine subtree text:
  - `repo-ref/zed/crates/gpui/src/styled.rs`
- the window maintains a `text_style_stack` and composes it on demand:
  - `repo-ref/zed/crates/gpui/src/window.rs`
- text leaves consume the composed `window.text_style()`:
  - `repo-ref/zed/crates/gpui/src/elements/text.rs`

Fret does **not** adopt GPUI's full style system in this ADR. The reference is used to justify the
contract shape: inherited text style must be a first-class runtime concept, not a per-component
patching convention.

## Decision

### 1) Introduce a portable `TextStyleRefinement` contract

Fret will add a partial, mergeable text-style refinement surface in the portable text contract.

This refinement is for **subtree defaults**, not per-span rich text styling.

Expected fields (v1 baseline) are the subset needed for common UI typography inheritance:

- font family / font id override,
- size,
- weight,
- slant,
- line height / line height em,
- letter spacing,
- vertical placement,
- leading distribution,
- optional base color,
- wrap / overflow / align only if we explicitly decide they are safe to cascade for passive text.

Implementation detail: the exact Rust type layout is not part of this ADR so long as it remains a
mergeable, portable refinement surface owned by the text contract.

### 2) Inherited text style is a mechanism-level runtime concept

`crates/fret-ui` will provide a runtime carrier for inherited text-style refinements.

This carrier may be represented internally by:

- a dedicated `TextStyleScope` element,
- traversal-owned inherited state,
- or another equivalent mechanism.

What is normative is the **outcome**, not the internal carrier:

- descendants can inherit subtree-local text style defaults,
- inherited text style participates in measurement and paint,
- authors do not need to recursively patch descendant text nodes to achieve inherited typography.

### 3) Resolution priority is explicit leaf > inherited refinement > theme default

For passive text leaves, the final resolved text style is computed in this order:

1. explicit leaf-level overrides,
2. inherited subtree text-style refinement,
3. theme default text style.

This preserves the escape hatch for component-owned text while making inherited typography a stable,
composable default.

### 4) v1 consumer set is intentionally narrow

The inherited text-style contract applies in v1 to passive text surfaces only:

- `Text`
- `StyledText`
- `SelectableText`

Out of scope for v1:

- `TextInput`
- `TextArea`
- editor/code-editing surfaces
- document/prose layout engines that own their own shaping rules

Those surfaces may adopt the refinement model later, but they are excluded from the initial
contract to keep measurement and IME behavior stable.

### 5) Policy and presets remain in the ecosystem layer

`crates/fret-ui` owns the mechanism and merge semantics.

`ecosystem/fret-ui-kit` owns:

- typography presets,
- authoring helpers,
- semantic policy like “description text”, “label text”, “control copy”, or “muted content”.

`ecosystem/fret-ui-shadcn` / `ecosystem/fret-ui-ai` own migration of component recipes.

This ADR does **not** move shadcn/material/AI typography policy into `crates/fret-ui`.

### 6) Existing foreground inheritance remains a separate concern

`ForegroundScope` and inherited foreground stay valid in v1.

Text-style cascade is additive, not a replacement for foreground inheritance. A future cleanup may
bridge the two more tightly, but this ADR does not require a unified style-context object.

### 7) Compatibility is staged

Existing component-local patches may remain temporarily during migration.

The workstream must remove redundant per-component inheritance workarounds once:

- inherited text-style mechanism exists,
- key component families have migrated,
- and regression gates are in place.

## Consequences

### Positive

- Direct compound-children APIs become easier to implement without subtree rewriting.
- Repeated description/body typography logic can move out of individual components.
- Fret aligns with proven GPUI/Zed-style subtree text refinement without copying the entire style
  system.
- Future component ecosystems can express typography intent at the correct layer.

### Costs / risks

- Measurement and text cache plumbing must change; this is not a paint-only feature.
- We must define clear precedence between explicit leaf style and inherited refinement.
- Poorly scoped rollout could accidentally affect inputs/editors if v1 boundaries are not enforced.

## Implementation plan (tracked)

Primary workstream:

- `docs/workstreams/text-style-cascade-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/text-style-cascade-fearless-refactor-v1/TODO.md`
- `docs/workstreams/text-style-cascade-fearless-refactor-v1/MILESTONES.md`

Recommended sequence:

1. Add `TextStyleRefinement` to the portable text contract.
2. Add inherited text-style runtime propagation in `crates/fret-ui`.
3. Migrate passive text leaves to consume the inherited refinement.
4. Add `fret-ui-kit` helpers/presets for subtree-local typography defaults.
5. Migrate high-value component families (description/body/title-adjacent surfaces).
6. Remove temporary component-local recursive patching and duplicate metric lookups where the new
   mechanism makes them redundant.

## Initial evidence anchors

### Current Fret anchors

- Foreground-only inherited style contract:
  - `crates/fret-ui/src/element.rs`
  - `ecosystem/fret-ui-kit/src/declarative/current_color.rs`
- Theme-default text style resolution:
  - `crates/fret-ui/src/text/props.rs`
- Example of current component-local workaround / pressure point:
  - `ecosystem/fret-ui-ai/src/elements/confirmation.rs`

### GPUI / Zed anchors

- `repo-ref/zed/crates/gpui/src/style.rs`
- `repo-ref/zed/crates/gpui/src/styled.rs`
- `repo-ref/zed/crates/gpui/src/window.rs`
- `repo-ref/zed/crates/gpui/src/elements/text.rs`
