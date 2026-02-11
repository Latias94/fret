# ADR 0113: AvailableSpace Constraints and Non-Reentrant Measurement for Declarative Layout

Status: Proposed

## Context

Fret targets a "Tailwind-like primitives + shadcn-like recipes" component ecosystem without relying
on DOM/CSS (ADR 0057, ADR 0062, `docs/tailwind-semantics-alignment.md`). Declarative Flex/Grid are
implemented via `taffy` as an internal algorithm (ADR 0035). The repository has moved away from the
older container-owned persistent `TaffyTree` strategy (ADR 0076) in favor of a window-scoped layout
engine (ADR 0114).

However, the current Taffy integration in `crates/fret-ui` measures child subtrees by *re-entering*
layout (`LayoutCx::layout_in`) inside Taffy's measure callback. When Taffy supplies
`AvailableSpace::{MinContent, MaxContent}`, the runtime approximates these as a huge definite bound
(e.g. `1e9`) and proceeds with a normal layout pass. In certain compositions this creates:

- **Incorrect semantics**: "fill"/percent sizing and `flex-1` are interpreted as if a large definite
  free space exists during an intrinsic measurement phase.
- **Runaway recursion / stack overflow**: invalid or ambiguous compositions (e.g. `flex-1` under a
  parent without a definite main-axis size) can trigger extremely deep recursive re-layout.
  A concrete symptom was observed as `thread 'main' has overflowed its stack` in `shadcn::Tabs`
  layouts, mitigated at the component layer by removing a default `flex-1` recipe and adding a
  regression test.

This is a correctness and stability issue: we want Tailwind-like semantics that are predictable and
that converge, without requiring every recipe author to understand Taffy's constraint phases.

Zed/GPUI provides a useful reference for a non-DOM runtime that still achieves "CSS-like" layout
semantics with Taffy: it keeps `AvailableSpace` as a first-class concept (Definite/Min/Max), and it
does not re-enter layout from the measure callback; instead, measured nodes provide an intrinsic
size function that is evaluated by Taffy during solve.

References:

- Hybrid layout + Taffy as an internal algorithm: `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md`
- Declarative layout semantics + Flex: `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
- Performance hardening for persistent Taffy trees: `docs/adr/0076-declarative-layout-performance-hardening.md`
- Trigger composition without Slot/`asChild`: `docs/adr/0115-trigger-composition-and-no-slot-aschild.md`
- Tailwind semantics mapping: `docs/tailwind-semantics-alignment.md`
- Regression tracker for the observed stack overflow: `docs/todo-tracker.md` (P0 shadcn Components / Layout Correctness)
- GPUI Taffy engine reference: `repo-ref/zed/crates/gpui/src/taffy.rs`
- Refactor roadmap (living doc): `docs/layout-engine-refactor-roadmap.md`
- Migration inventory (living checklist): `docs/layout-engine-v2-migration-inventory.md`

## Goals

1. **Correct constraint semantics**: represent intrinsic measurement phases explicitly
   (`MinContent`/`MaxContent`) rather than approximating them as a huge definite size.
2. **Eliminate layout re-entry from Taffy measurement** for declarative flow layout, so ambiguous
   compositions do not blow the stack.
3. **Preserve existing layering**: docking/splits/scroll/virtualization remain explicit
   editor-friendly containers (ADR 0035, ADR 0042).
4. **Keep Tailwind-like vocabulary typed**: no runtime class parser or CSS cascade.
5. **Provide a migration path** that can be rolled out incrementally and validated with tests.

## Non-goals

- Full CSS parity, selectors/cascade, or a `class="..."` runtime parser.
- A global `z-index` primitive (see ADR 0062).
- Forcing docking, virtualization, or editor-specific layout containers into Taffy.

## Compatibility Notes

This ADR refines existing accepted contracts; it does not contradict them:

- ADR 0035 (hybrid layout) remains the boundary: `taffy` is still an internal algorithm for specific
  declarative containers, and docking/scroll/virtualization remain explicit layout systems.
- ADR 0076 (persistent container-owned Taffy trees) documents a historical implementation shape.
  This ADR tightens the constraint semantics (`AvailableSpace`) and forbids re-entrant layout from
  solver-time measurement regardless of the integration shape.
- ADR 0114 describes a compatible end-state evolution (window-scoped engine + viewport roots) that
  builds on the same measurement and constraint rules defined here.

## Decision

We introduce an explicit "available space" constraint model and a non-reentrant measurement path
for declarative layout:

1. Add a runtime `AvailableSpace` type (`Definite`, `MinContent`, `MaxContent`) and pass it through
   layout/measurement APIs.
2. Introduce a `measure` API that computes an element's **intrinsic size** under a constraint tuple
   without performing a full layout pass for the subtree.
3. Refactor declarative Flex/Grid (and any other Taffy-backed containers) to call `measure(...)`
   from the Taffy measure callback instead of calling back into `layout_in(...)`.
4. Codify Tailwind-like semantics under intrinsic constraints:
   - `Length::Fill` (Tailwind `w-full`/`h-full`) behaves like percent sizing and only resolves to a
     definite value when the parent axis is **definite**; under Min/MaxContent it behaves as `auto`.
   - Flex "free space distribution" (`flex-grow` / `flex-1`) only applies when the container's
     main-axis available space is **definite**; under Min/MaxContent it must not create feedback
     loops by "inventing" free space.
5. Adopt a window-scoped Taffy layout engine for declarative flow layout:
   - Build or update a Taffy tree during the frame's declarative build/prepaint phase.
   - Compute layout once per root and apply bounds to the retained UI tree after solve.
6. Treat docking-driven viewports as independent layout roots:
   - Docking computes **definite** viewport rects.
   - Each viewport's UI root is laid out against its own definite available space.
   - No "free space" coupling or percentage resolution across viewport boundaries.
7. Add engine-level guardrails:
   - Preserve `MinContent`/`MaxContent` semantics end-to-end (no "huge definite probe bounds").
   - Prevent measurement re-entrancy (debug assertion / cycle guard) and use stacksafe execution for
     deep trees.
8. Standardize a two-phase layout protocol:
   - **Request/build phase** constructs the layout graph (Taffy nodes + styles + child edges).
   - **Compute/apply phase** runs the solver and writes definite bounds back into the retained UI
     tree.
   - Intrinsic measurement is pure with respect to bounds: `measure_in` must not assign bounds to
     descendants and must not call `layout_in`, directly or indirectly.
9. Require stable layout node identity across frames:
   - The window-scoped engine maintains a stable mapping from retained `NodeId` to `LayoutId` /
     `TaffyNodeId` and updates styles/children incrementally.
   - Layout invalidation is expressed via `mark_dirty` on affected nodes/roots, enabling caching and
     avoiding full rebuilds.
10. Codify explicit layout barriers and their contract with the engine:
   - Docking, virtualization, scroll, and viewport surfaces remain explicit layout systems (ADR 0035, ADR 0042).
   - Barriers may request intrinsic measurement of their content and/or request layout for their
     mounted child subtree under definite rects, but they must not rely on re-entrant layout during
     solver measurement.

This aligns Fret's typed Tailwind-like semantics with the constraint-phase reality of Taffy, and
removes the primary cause of runaway recursion.

Note: ADR 0076 described an accepted performance hardening step for a container-owned integration,
but the repository now defaults to the window-scoped engine described by ADR 0114. The end-state
still carries forward the same persistence/incremental-update principles (stable identity + bounded
measurement), but the ownership model is window-scoped rather than container-scoped.

## Design (Proposed)

### 1) Constraint types

Add a runtime constraint vocabulary (names illustrative):

- `AvailableSpace`:
  - `Definite(Px)`
  - `MinContent`
  - `MaxContent`
- `LayoutConstraints`:
  - `known: Size<Option<Px>>` (what the solver already determined)
  - `available: Size<AvailableSpace>` (constraint phase)

These should remain renderer/platform independent and live in `crates/fret-ui` (stable contract).

### 2) Measurement vs layout

Add an intrinsic measurement entry point that does *not* assign bounds to descendants:

- `UiTree::measure_in(app, services, node, constraints, scale_factor) -> Size`
- `LayoutCx::measure_in(child, constraints) -> Size`

Rules:

- `measure_in` must not call `layout_in` internally, directly or indirectly.
- `measure_in` is allowed to observe models/globals for invalidation, similar to layout.
- Leaf widgets may use services (text measurement, image metadata, etc.).

### 3) Taffy containers

For Taffy-backed containers (Flex/Grid), the measure callback becomes:

- Convert Taffy's `(known_dimensions, available_space)` into `LayoutConstraints`.
- Call `measure_in(child, constraints)` for children.
- Return the measured size to Taffy.

After computing layout, the container assigns bounds via `layout_in(child, rect)` (as today), but
those `layout_in` calls occur *after* the solve and are provided definite bounds from the solver.

This preserves ADR 0005's "layout writes bounds" contract while removing re-entrant layout during
solve.

### 4) Fill/percent semantics under intrinsic constraints

Fret's `Length::Fill` currently maps to `Dimension::percent(1.0)` for Taffy. To avoid "invented"
space under Min/MaxContent, the runtime must ensure that percent sizing does not behave as "fill
the (huge) probe bounds".

Proposed rule (Tailwind/CSS-aligned contract):

- If the parent axis available space is `Definite`, `Fill` resolves as percent sizing (100%).
- If the parent axis available space is `MinContent` or `MaxContent`, `Fill` resolves as `auto`
  for measurement purposes.

This rule is applied in the style-to-taffy mapping and/or in the measurement path, not in component
recipes. Recipes remain stable and typed.

### 5) Explicit layout barriers

Certain element kinds remain explicit layout systems and are treated as layout barriers:

- docking/splits,
- virtualization containers,
- scroll containers,
- viewport surfaces.

These nodes may:

- provide an intrinsic measurement function when needed, and/or
- require definite constraints in specific axes (and enforce via validation).

### 6) Tests and conformance

Add/extend tests to lock down the new semantics:

- **No recursion/stack overflow**: a minimal layout harness where a `flex-1` item is composed under
  an auto-sized parent must not crash.
- **Definite axis correctness**: under a definite main-axis size, `flex-1` must fill remaining
  space.
- **Percent sizing correctness**: `w-full`/`h-full` acts as 100% only when the parent provides a
  definite size; otherwise it behaves like auto under intrinsic constraints.

Tests should live in `crates/fret-ui` for mechanism semantics, with additional component-layer
regression tests in `ecosystem/*` where appropriate (ADR 0066).

### 7) Window-scoped layout engine (end-state)

Adopt a per-window Taffy engine similar to GPUI:

- A `Window`-scoped layout engine owns the canonical `TaffyTree` (and any per-frame scratch state).
- Declarative element building requests layout nodes from the engine:
  - `request_layout(style, children) -> LayoutId`
  - `request_measured_layout(style, measure_fn) -> LayoutId` (leaf intrinsic measurement only)
- Layout is computed during prepaint using `compute_layout_with_measure`, passing through
  `AvailableSpace::{Definite, MinContent, MaxContent}` (no approximation to large definite bounds).
- After solve, the runtime applies computed bounds back into the retained UI tree via definite rects.

This model centralizes constraint semantics and enables "taffy-by-default" for declarative flow
subtrees while still allowing explicit containers to opt out for performance or control.

### 8) Multi-viewport docking integration

Docking defines a multi-viewport surface for the window. Layout is a two-level system:

1. Docking/splits compute viewport rects (definite bounds) and decide which UI root is mounted in
   each viewport.
2. The window-scoped Taffy engine computes each viewport root independently against its own
   definite available space.

Viewport boundaries are layout barriers: percent/fill and free-space distribution must not resolve
across a viewport edge.

### 9) Guardrails (engineering)

To avoid regressions and make invalid compositions diagnosable:

- Add a re-entrancy/cycle guard to `measure_in` keyed by `(node, constraints, scale_factor)`:
  - debug/test builds: panic with diagnostics (treat as a bug),
  - release builds: return `Size::default()` and emit a rate-limited diagnostic counter.
- Use stacksafe execution for the top-level layout solve and measure callbacks (GPUI reference).
- Prefer rounding in the Taffy engine to reduce subpixel drift where applicable (must remain
  consistent with the snapping policy in ADR 0035 and be idempotent with renderer snapping).

### 10) Stable identity and incremental updates

The window-scoped engine must support incremental layout graph updates:

- `NodeId` is the stable identity for retained UI nodes.
- `LayoutId`/`TaffyNodeId` are derived and cached in the engine (`NodeId -> LayoutId`).
- The engine updates:
  - node style when `LayoutStyle` changes,
  - child lists when the retained tree changes,
  - dirtiness when layout-affecting invalidations occur.

This preserves caching opportunities (including measurement memoization) and keeps per-frame
overhead bounded for large trees.

### 11) Barrier contract (explicit layout systems)

Barriers define boundaries between layout systems. For each barrier kind, the runtime codifies:

- Which axes must be definite to resolve "fill/percent/free-space distribution".
- Which intrinsic measurement queries are performed (`MinContent` vs `MaxContent`), and what they
  are used for (e.g. scroll content extent).
- How definite rects are produced for child subtrees (e.g. viewport rect, scroll viewport rect,
  virtualization visible window).

Initial default policy (subject to conformance tests):

- Virtualization containers are always barriers and only request layout for visible items.
- Scroll containers are barriers; they measure content extent using `MaxContent` constraints and
  lay out the mounted content subtree under a definite viewport rect.

## Rollout Plan (Incremental)

1. Introduce `AvailableSpace` and `LayoutConstraints` types and plumb them through internal layout
   plumbing (no behavior change yet).
2. Add `measure_in` and implement it for leaf primitives used by `Flex`/`Grid` measurement (text,
   images, basic containers).
3. Refactor Taffy containers to use `measure_in` inside measure callbacks and delete the "huge
   probe bounds" fallback.
4. Implement and lock the `Fill`/percent and `flex-grow` constraint-phase rules with tests.
5. Expand intrinsic measurement coverage and/or widen the "taffy island" to include more of the
   declarative flow subtree, reducing cross-subtree measurement costs.
6. Introduce a window-scoped layout engine and migrate Flex/Grid to request nodes into the shared
   engine (removing per-container "island" Taffy trees where possible).
7. Integrate multi-viewport docking by treating each viewport as an independent layout root and add
   a conformance test ensuring no cross-viewport coupling.
8. Migrate geometry-transparent wrappers and other typed primitives into the engine as stable nodes,
   introducing an explicit "contents-like" opt-in mode for wrappers that must not introduce boxes.

## Impacted Areas (Expected)

Mechanism/runtime (`crates/fret-ui`):

- `crates/fret-ui/src/widget.rs` (`LayoutCx` additions / new `MeasureCx` if introduced)
- `crates/fret-ui/src/tree/*` (new measurement entry point)
- `crates/fret-ui/src/declarative/host_widget/layout/flex.rs`
- `crates/fret-ui/src/declarative/host_widget/layout/grid.rs`
- `crates/fret-ui/src/declarative/taffy_layout.rs` (style mapping for Fill/percent under constraints)
- Declarative layout tests: `crates/fret-ui/src/declarative/tests/*`

Component layer (follow-up, not required for the mechanism refactor):

- Remove remaining "defensive defaults" that were only needed to avoid runtime recursion, once the
  new semantics are locked (e.g. optional `flex-1` recipes that are now safe under the correct
  constraints contract).

## Alternatives Considered

1. **Keep the current approach and rely on recipe-level workarounds**
   - Rejected: correctness and stability should be enforced at the runtime contract layer.
2. **Approximate Min/MaxContent as a large definite bound (status quo)**
   - Rejected: creates incorrect semantics and recursion hazards.
3. **Adopt a full CSS engine**
   - Rejected: violates the typed, mechanism-only runtime goals (ADR 0066) and adds large semantic
     surface area.
4. **"Taffy everywhere" globally across all containers**
   - Rejected: conflicts with editor-friendly explicit layout systems (ADR 0035) and virtualization
     constraints (ADR 0042).
   - Note: a "taffy-by-default for declarative flow subtrees" approach may still be an end-state
     for general UI composition, while preserving explicit barriers for docking/scroll/virtualization.

## Defaults (v1)

- Required `measure_in` leaves: `Text`/`StyledText`/`SelectableText`, `TextInput`/`TextArea`,
  `Image`, `Svg`, and any custom primitives that have true intrinsic sizing.
- Invalid compositions: rely on semantic fallbacks + debug assertions, not a separate validation
  pass (e.g. treat `Length::Fill` as `auto` under `AvailableSpace::{MinContent,MaxContent}`).
- Cache keys: bake `scale_factor` + theme revision + `TextFontStackKey` into text measurement
  caches; prefer small stable globals over backend-specific internals.
- Wrapper representation: represent wrappers as pass-through nodes by default to keep a single flow
  island; skipping boxes is not supported by default and requires an explicit validated
  "layout-transparent wrapper" opt-in if introduced later (see ADR 0114, ADR 0115).
- Cycle policy: debug/test builds panic; release builds return `Size::default()` and emit a
  rate-limited diagnostic counter.
- Layout-transparent wrapper (optional, future): validate single-child and forbid geometry-affecting
  properties (padding/margin/overflow/position/size/min/max/flex/grid/transform); this is strictly
  about box introduction/removal and does not imply any Slot/prop-merging mechanism (see ADR 0115).
