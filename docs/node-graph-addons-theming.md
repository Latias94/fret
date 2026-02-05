# Node graph theming & token plumbing (contract)

This document defines the stable, **policy-light** theming contract for `ecosystem/fret-node`.

`fret-node` does not ship a component library. Theming here means **explicit, bounded token
bundles** (`NodeGraphStyle`, `NodeGraphBackgroundStyle`) that embedding apps can store in a B-layer
store and apply to the canvas/overlays without reaching into widget internals.

## Who owns what

### `Theme` (host UI)

`fret-ui` provides the app-wide `Theme` surface (colors/metrics/typography). It is global and
shared across the UI tree.

### `NodeGraphStyle` (node editor tokens)

`NodeGraphStyle` is the node editor’s explicit token bundle:

- background/grid tokens,
- node/chrome sizing tokens,
- overlay sizing/margins tokens (minimap/controls),
- render culling + zoom clamps.

It can be constructed from the host theme:

- `NodeGraphStyle::from_theme(theme)` (snapshot at the time you call it),
- `NodeGraphStyle::from_snapshot(theme_snapshot)` (more explicit for widget/host code).

### `NodeGraphBackgroundStyle` (background-only overrides)

`NodeGraphBackgroundStyle` is a **bounded sub-bundle** for background/grid tokens only. It is
intended for per-editor customization (e.g. “dots vs lines”, spacing, colors) without touching
interaction logic or derived geometry.

## Precedence rules

### 1) Explicit style wins

If you pass a full style with `NodeGraphCanvas::with_style(style)`, that configuration is used as
is (and `colorMode` is disabled).

### 2) `colorMode` owns the base palette

If you pass `NodeGraphCanvas::with_color_mode(mode)`, the canvas will sync its base `NodeGraphStyle`
from the current theme snapshot:

- `System`: tracks theme revision (for live theme switches),
- `Light` / `Dark`: forces the corresponding XyFlow-like palettes.

This sync clears paint caches and can invalidate style-derived render artifacts.

### 3) Background overrides are additive

If you also pass `NodeGraphCanvas::with_background_style(background)`, the background tokens are
applied **after** any `colorMode` sync. This keeps background customization stable even when the
base style is updated from the theme.

Background updates are **paint-only**: they must not rebuild derived geometry (conformance gate).

## Recommended patterns

### Per-editor background variant (store-driven)

Keep a background token bundle in your B-layer store and apply it when building the editor UI:

```rust
let mut style = NodeGraphStyle::from_theme(Theme::global(app));
let background = store.read(|s| s.node_graph_background); // NodeGraphBackgroundStyle
style = style.with_background_style(background);

let canvas = NodeGraphCanvas::new(graph, view).with_style(style);
```

If you prefer `colorMode` tracking, use `with_color_mode(...)` and still apply
`with_background_style(...)` for per-editor background overrides.

Evidence (demo):

- `apps/fret-examples/src/node_graph_demo.rs`

## Conformance gates

- Background updates do not rebuild derived geometry:
  `ecosystem/fret-node/src/ui/canvas/widget/tests/background_style_conformance.rs`

## Accessibility note

If you want `aria-activedescendant`-style semantics for focused nodes/ports/edges, mount the
semantics-only children under the canvas (in this exact order):

- `NodeGraphA11yFocusedPort`
- `NodeGraphA11yFocusedEdge`
- `NodeGraphA11yFocusedNode`

The canvas will set `SemanticsNode.active_descendant` to one of these children based on its
internal focus state.

Conformance:

- `ecosystem/fret-node/src/ui/canvas/widget/tests/a11y_active_descendant_conformance.rs`
