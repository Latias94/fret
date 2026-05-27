# Node graph theming & token plumbing (contract)

This document defines the stable, **policy-light** theming direction for `ecosystem/fret-node`.

`fret-node` does not ship a component library. Theming here means **explicit, bounded token
bundles** (`NodeGraphStyle`, `NodeGraphBackgroundStyle`) consumed by the declarative node graph
surface and overlays without exposing implementation-local canvas state.

## Who owns what

### `Theme` (host UI)

`fret-ui` provides the app-wide `Theme` surface (colors/metrics/typography). It is global and
shared across the UI tree.

### `NodeGraphStyle` (node editor tokens)

`NodeGraphStyle` is the node editor’s explicit token bundle. v2 splits it into two planes:

- `style.paint`: paint-only chrome (colors, dashes, shadows, overlay sizing tokens).
- `style.geometry`: geometry-affecting metrics (layout, hit-testing, measurement).

This split exists to keep geometry caches stable: paint-only changes should not rebuild derived
geometry or hit-testing indexes.

It can be constructed from the host theme:

- `NodeGraphStyle::from_theme(theme)` (snapshot at the time you call it),
- `NodeGraphStyle::from_snapshot(theme_snapshot)` (more explicit for widget/host code).

### `NodeGraphBackgroundStyle` (background-only overrides)

`NodeGraphBackgroundStyle` is a **bounded sub-bundle** for background/grid tokens only. It is
intended for per-editor customization (e.g. “dots vs lines”, spacing, colors) without touching
interaction logic or derived geometry.

## Precedence rules

### 1) Declarative surface style source

The current public declarative surface derives `NodeGraphStyle` from the host `Theme` snapshot.
First-class per-surface style injection is intentionally deferred until the binding/controller API
has a stable policy for theme tracking, invalidation, and geometry-cache ownership.

### 2) `colorMode` owns the base palette

`NodeGraphColorMode` remains the palette vocabulary for style construction:

- `System`: tracks theme revision (for live theme switches),
- `Light` / `Dark`: forces the corresponding XyFlow-like palettes.

Derived geometry / spatial index invalidation is **gated by the geometry fingerprint**
(paint-only palette changes must not rebuild geometry).

### 3) Background overrides are additive

`NodeGraphBackgroundStyle` remains a bounded sub-bundle that can be applied to a `NodeGraphStyle`
value with `NodeGraphStyle::with_background_style(...)`. Public per-surface background injection is
deferred with the broader style injection API.

Background updates are **paint-only**: they must not rebuild derived geometry (conformance gate).

## Recommended patterns

### Per-editor background variant (store-driven)

Keep a background token bundle in your B-layer store, derive a `NodeGraphStyle`, and pass it only
through explicit style/preset seams once those seams are promoted:

```rust
let mut style = NodeGraphStyle::from_theme(Theme::global(app));
let background = store.read(|s| s.node_graph_background); // NodeGraphBackgroundStyle
style = style.with_background_style(background);

let _style = style;
```

Until then, use the app `Theme`, paint override providers, and `NodeGraphSkin`/preset seams for
supported UI customization.

Evidence (demo):

- `apps/fret-examples/src/node_graph_demo.rs`

## Conformance gates

- Paint-only cache keys include style paint tokens while derived geometry keys use geometry tokens:
  `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
- Declarative surface conformance:
  `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

## Accessibility note

If you want `aria-activedescendant`-style semantics for focused nodes/ports/edges, keep the derived
internals store (`NodeGraphInternalsStore`) attached to the declarative surface and expose semantics
through declarative children rather than implementation-local canvas widgets.

Conformance:

- `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
