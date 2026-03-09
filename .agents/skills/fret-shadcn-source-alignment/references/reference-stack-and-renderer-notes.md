# Reference stack and renderer notes

Use this note when the hard part is choosing the right source of truth or translating DOM/CSS assumptions into Fret’s GPU-first renderer.

## 1) Pick the upstream reference stack explicitly

Use the right source for the right kind of parity:

- **APG**: keyboard navigation and composite widget semantics
- **Radix**: overlays, portal/presence, dismissal/focus nuances, and interaction outcomes
- **Floating UI**: placement, flip, shift, arrow geometry
- **cmdk**: command palette behavior details

See `docs/reference-stack-ui-behavior.md` for the repo’s priority order and boundary mapping.

## 2) Motion parity in a custom renderer

shadcn/Radix sources are DOM-first. Fret is a GPU-first custom renderer, so match **outcomes**, not APIs.

Rules of thumb:

- treat upstream motion as a UX spec, not an implementation to port 1:1
- prefer ecosystem motion drivers over ad-hoc per-component math
- keep motion tunable via theme tokens
- choose hit-testing semantics explicitly:
  - `RenderTransform` when hit-testing should move with visuals
  - `VisualTransform` when the change is paint-only
- lock motion-sensitive changes with a deterministic diag gate (`--fixed-frame-delta-ms 16`)

## 3) Renderer parity (CSS/Tailwind → GPU-first self-rendering)

Rule of thumb:

- recipe/style choice → `ecosystem/fret-ui-shadcn`
- reusable style vocabulary or shaping behavior → `ecosystem/fret-ui-kit`
- new draw primitive or cross-backend correctness → scene contract + renderer(s), then expose via `crates/fret-ui`

Common CSS → Fret translations:

- rounded corners → prefer first-class rounded-rect ops
- border/ring → distinguish layout-affecting border vs paint-only ring
- box-shadow/elevation → decide whether the outcome needs a real blur or a cheaper approximation
- transform animations → pick `RenderTransform` vs `VisualTransform` intentionally
- `overflow: hidden` + rounded clipping → gate scroll/overlay interactions where clipping affects behavior

## 4) When to add a render primitive

Add or extend a scene op only if at least one is true:

- the upstream outcome cannot be reached with existing ops without quality/perf regression
- multiple shadcn/Radix components need the same capability
- the behavior must be identical across backends

If you add a primitive, require:

- a contract-level invariant test, and
- at least one consumer-level usage anchor (recipe + diag script or parity case)

## 5) Semantic conflict hazards

The most expensive parity regressions often come from multiple “truths” fighting each other.

Rules:

- choose exactly one driver per responsive decision: viewport **or** container
- if both are needed, expose an explicit recipe-level knob and gate both modes
- in overlays, prefer viewport queries unless the container region is proven stable
- prefer app-owned theme metadata for dark/light styling decisions
- use hysteresis around thresholds to prevent resize flicker

## 6) Useful anchors

- `docs/reference-stack-ui-behavior.md`
- `docs/adr/0231-container-queries-and-frame-lagged-layout-queries-v1.md`
- `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
- `docs/adr/0061-focus-rings-and-focus-visible.md`
- `crates/fret-ui/src/element.rs`
- `crates/fret-ui/src/tree/hit_test.rs`
