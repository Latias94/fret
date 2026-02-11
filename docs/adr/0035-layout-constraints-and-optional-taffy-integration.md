# ADR 0035: Layout Constraints and Optional Taffy Integration (Hybrid Editor-Friendly Layout)

Status: Accepted

## Context

Editor UIs combine:

- highly controlled layouts (docking, splitters, scroll regions, toolbars),
- flexible layouts (property panels, settings forms),
- text measurement and baseline alignment,
- DPI scaling and pixel snapping.

If a project commits too early to a single monolithic layout engine, it risks:

- poor fit for docking/editor chrome,
- large rewrites when adding Flex/Grid,
- unclear measurement contracts (especially for text and viewports).

Fret already defines a core layout contract where containers write child bounds explicitly (ADR 0005).
We want to retain this editor-friendly predictability while enabling Flex/Grid where appropriate.

References:

- Layout bounds as source of truth:
  - `docs/adr/0005-retained-ui-tree.md`
- Text measurement boundary (metrics):
  - `docs/adr/0006-text-system.md`
- Text implementation strategy considerations:
  - `docs/adr/0029-text-pipeline-and-atlas-strategy.md`

## Decision

### 1) Keep explicit layout as the primary contract

The authoritative contract remains:

- containers assign child bounds via `layout_in(child, rect)`,
- the UI runtime stores bounds for hit-testing and paint (ADR 0005).

This remains stable regardless of whether the UI execution model is retained widgets or declarative elements (ADR 0028).

### 2) Adopt a hybrid approach: taffy as an internal algorithm for specific containers

`taffy` (or any layout engine) is used as an implementation detail for specific containers:

- `Flex` container,
- `Grid` container,
- possibly a `Form`/`InspectorLayout` helper.

Docking, splitters, and scroll views remain custom editor-friendly containers.

This avoids forcing all widgets into a single constraints model while still enabling powerful layouts.

#### “Two trees” concern: how to sync `UiTree` and `taffy`

It is normal for a layout engine (like `taffy`) to maintain its own internal node graph. The contract for Fret is:

- `UiTree` (or the future element runtime substrate) remains the **source of truth** for:
  - identity (focus/capture targets),
  - event routing and hit-testing,
  - final child bounds used for paint.
- `taffy` is an **internal algorithm** used by specific containers (`Flex`/`Grid`) to compute sizes and positions.

Recommended implementation strategy (P0):

- A `Flex`/`Grid` container owns a `taffy::TaffyTree` subtree and a mapping of child identity → `taffy::NodeId`.
  - retained widgets: child identity can be `NodeId`,
  - declarative elements: child identity should be `GlobalElementId` (ADR 0028).
- On layout:
  - update/create `taffy` nodes for the current set of children (by stable identity),
  - run `taffy` layout with constraints,
  - write computed bounds back into the UI runtime via `layout_in(child, rect)` so hit-test/paint use the same bounds.

This avoids needing to “synchronize two global trees”: only the container doing Flex/Grid participates in `taffy`.

Future evolution (compatible with this boundary):

- A window-scoped layout engine may own a canonical Taffy tree for declarative flow layout and treat
  docking-defined viewports as independent layout roots (ADR 0114). This generalizes the same
  “taffy as an internal algorithm” principle, while enabling incremental updates and multi-viewport
  integration without per-container islands.

Dependency rule (P0):

- Layout engine crates (including `taffy`) must not be required by `fret-core`.
- Layout engines may be used by `fret-ui` and/or `fret-components-*` as implementation details (see ADR 0037).

### 3) Define a stable measurement interface for leaf nodes

Any engine-driven layout algorithm needs a way to measure leaves:

- text: through `TextMetrics` (ADR 0006 / ADR 0029),
- images/icons: known intrinsic size or resource metadata,
- viewports: explicit preferred size and/or “fill available” semantics.

The UI runtime must expose a measurement hook that is independent from a specific layout engine.

Note: ADR 0113 refines this measurement surface by introducing an explicit `AvailableSpace`
(Definite/MinContent/MaxContent) model and a non-reentrant intrinsic measurement path.

Baseline default (P0):

- Baselines are required only for text-centric primitives (labels, text fields, inline value editors).
- Non-text widgets are not required to provide a baseline.
- The measurement interface must allow returning an optional baseline offset for text primitives.

Constraints default (P0):

- All widgets must support standardized `min_size` / `max_size` constraints (even if some widgets ignore them initially).
- Containers must respect these constraints when computing child bounds.

### 4) Pixel snapping and rounding rules are explicit

To avoid “1px drift” and AA inconsistencies across DPI:

- define rounding rules for layout outputs (logical → physical conversion),
- define how snapping interacts with borders and SDF AA (ADR 0030),
- define how snapping interacts with text positioning (ADR 0029).

Locked P0 snapping rules:

- **Core coordinate space remains logical pixels** (ADR 0017). Hit-testing and layout operate in logical space.
- **Renderer snaps axis-aligned paint rects** to device pixels for shape primitives:
  - `snap(x) = round(x * scale) / scale`
  - `snap_rect`: apply `snap` independently to each edge (min/max).
- **Clip/scissor uses outward rounding** to avoid cutting pixels:
  - `min = floor(min * scale) / scale`, `max = ceil(max * scale) / scale`
- **Hairline**: for 1-device-pixel separators, widgets should use `hairline = 1.0 / scale` logical pixels rather than `1.0`.
  This keeps dividers crisp across fractional scaling.

### 5) Layout caching and invalidation are compatible with both models

Whether the runtime is retained or declarative:

- layout results can be cached by stable identity + constraints,
- invalidation remains explicit (ADR 0005),
- container-level caching must not violate event routing correctness.

Performance default (P0):

- Flex/Grid containers must be able to cache layout results keyed by:
  - stable identity,
  - constraints,
  - style token inputs (ADR 0032),
  - text metrics revisions (ADR 0006 / ADR 0029),
  - DPI scale.

## Consequences

- Docking and editor chrome remain predictable and controllable.
- Flex/Grid can be introduced without refactoring the entire UI runtime.
- Text and viewport measurement remain cleanly integrated via stable boundaries.

## Open Questions (To Decide Before Implementation)

### Locked P0 Choices

1) **Baseline representation**: a single alphabetic baseline offset is sufficient for editor UI.
2) **Partial recompute**: rely on caching at container granularity; incremental Flex/Grid recompute is deferred.
