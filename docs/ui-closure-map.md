---
title: UI Subsystem Closure Map (fret-ui)
---

# UI Subsystem Closure Map (fret-ui)

This document is the **closure-oriented index** for Fret’s UI substrate and its immediate neighbors:

- `crates/fret-ui` (mechanism-only runtime substrate)
- `ecosystem/fret-ui-kit` (policy + headless infra)
- `ecosystem/fret-ui-shadcn` (recipes + shadcn v4 surface)
- `ecosystem/fret-docking` (docking UI + policy)
- `crates/fret-platform` (portable platform I/O contracts)
- `crates/fret-runner-winit` (AccessKit bridge + winit glue)

It is intentionally **not** a full spec. ADRs remain the source of truth; this file exists to:

- make the UI architecture **navigable**,
- define “module closure” checklists (contract → code → tests → demo),
- and keep a **risk register** of gaps that would otherwise cause late rewrites.

See also:

- Golden index: `docs/golden-architecture.md`
- Runtime contract gates: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Behavior references: `docs/reference-stack-ui-behavior.md`

---

## Closure Loop (What “Done Enough” Means)

For each UI sub-module below, we consider it “closed enough to scale” when:

1. **Contract is locked** (Accepted ADR, or an explicit “decision gate”).
2. **Mechanism/policy boundary is enforced** (`fret-ui` is mechanism-only; policies in components).
3. **At least one regression test exists** (runtime unit test, or component unit/contract test).
4. **A demo harness exists** for behaviors that are hard to test (and a short manual checklist).
5. **No coordinate-space ambiguity remains** (especially under `render_transform`, clip stacks, and multi-root overlays).

---

## High-Level Pipeline (Per Window)

```mermaid
flowchart LR
  Platform[Platform Events] --> App[App/Models/Effects]
  App --> Build[UI Build (declarative render_root)]
  Build --> Layout[Layout (Taffy-backed; constraints)]
  Layout --> Paint[Paint (Scene ops emission)]
  Paint --> Scene[Scene finalize]
  Layout --> Semantics[SemanticsSnapshot (a11y stream)]
  Paint --> Semantics
  Scene --> Render[fret-render (wgpu)]
  Semantics --> Bridge[fret-runner-winit (AccessKit bridge)]
```

Key invariants:

- **Determinism**: same inputs → same hit-testing / layering / placement results.
- **Identity stability**: element identity survives churn (ADR 0028 / ADR 0033).
- **Multi-root correctness**: overlays and modal barriers are first-class roots (ADR 0011).
- **Coordinate closure**: paint, hit-testing, and event coordinates agree under transforms (ADR 0082).

---

## Current Convergence Closure Target

The active fearless-refactor plan is
`docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md`. It
builds on the closed Phase 2 convergence plan
`docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md`.
This closure map should be read through the active plan when deciding what to break, delete, or
gate.

Must-be-true outcomes for the next convergence pass:

- Default app surfaces teach `fret::app::prelude::*`, `View`, `AppUi`, `LocalState`, typed actions,
  data/effects helpers, and `notify`; they do not teach raw `UiTree`, retained widgets, or
  low-level driver seams.
- `crates/fret-ui` exposes generic mechanisms for layers, focus, capture, outside interactions,
  dirty propagation, frame phases, and geometry; policy-coded overlay/component vocabulary lives in
  ecosystem crates.
- Runtime mechanism submodules may expose low-level configuration types when component layers need
  them, but `crates/fret-ui` root exports stay narrower: resizable split chrome is available via
  `fret_ui::resizable_panel_group::ResizablePanelGroupStyle`, not as a root policy-style export.
- Declarative identity, stable node liveness, retained placement, and view/entity identity are
  separate: `GlobalElementId`, `StableNodeHandle`, `NodeId`, `ViewId`, and `BoundaryId` are not
  interchangeable.
- Current-frame topology, not repaired retained parent pointers, is the target authority for live
  focus, scroll, command, semantics, layer attachment, cache-root lookup, and dirty propagation
  queries. Any topology snapshot must carry a build/freeze/invalidate/consume epoch contract.
- Dirty work is attributable by entity-first `ViewId` / `ViewBoundary`, with cache-root-first and
  boundary-node behavior treated as compatibility mappings rather than the final runtime model.
- View-cache-owned layout/paint observations are recorded directly under boundary subscribers;
  cache-root observation collapse is no longer a normal-path bridge.
- Prepaint products, boundary hit-testing inputs, reusable boundary semantics subtrees, text-layout
  indexes, and scene fragments are owned by boundaries where locality is proven.
- Dispatch snapshots, command routing and availability, final semantics snapshots, hit-test path
  routing, focus/capture state, active layer roots, modal barriers, and tree-wide paint recording
  stay window/layer-forest owned unless a later ADR proves a narrower owner.
- Renderer/text costs for local edits are bounded by scene chunks with explicit closure metadata,
  render-plan reuse, dirty upload ranges, and explicit text/glyph/wasm cache budgets.
- Flat renderer input is not normal-path evidence once a frame class is in the chunk-launch support
  matrix. Supported native/web fixtures must render through authoritative chunk manifests with zero
  normal-path `FlatCompat` usage; unsupported frames report structured reasons and stay outside
  bridge-deletion evidence.
  Current source-selection evidence lives in `crates/fret-render-wgpu/src/renderer/mod.rs`, while
  renderer-owned chunk payload/cache assembly state lives behind
  `crates/fret-render-wgpu/src/renderer/render_scene/frame_assembler.rs`.
  Native/web launch now share `RenderSceneSourcePolicy::chunk_manifest_when_supported()` in
  `crates/fret-launch/src/runner/desktop/runner/window_redraw_render_scene.rs` and
  `crates/fret-launch/src/runner/web/render_loop.rs`. Frame stats expose
  `renderer_render_scene_source_chunk_manifest_frames`,
  `renderer_render_scene_source_flat_compat_frames`, and structured unsupported fallback counters
  so diagnostics can prove zero normal-path `FlatCompat` usage for supported classes.
  Manifest closure evidence lives in `crates/fret-core/src/scene/manifest.rs` and
  `crates/fret-core/src/scene/chunk.rs`; boundary scene chunk diagnostics use entry
  bounds/origin-sensitive fingerprints from `crates/fret-ui/src/tree/view_boundary.rs`.
- Text chunk/resource closure is not complete until WGPU `TextShape` residency metadata preserves
  shaping cluster/run facts for ligatures, RTL, combining marks, fallback font runs,
  selection/caret, decorations, and atlas reset generation.
- Renderer dirty upload expansion is stream-class gated. Resource-free quad instances and
  `VertexColor`-only viewport vertices may use partial writes after payload-plan alignment and
  coverage gates pass; image, viewport-surface, text, path, mask, material, clip, and effect streams
  stay on full upload until their closure metadata, fallback reasons, and per-stream write
  count/byte budgets are complete.

High-risk compatibility paths that need either deletion or an explicit retention gate:

- hash-keyed retained identity fallback scans after stable-handle diagnostics exist,
- parent repair and GC reachability work that can scale with the active retained tree; parent repair
  deletion needs a non-mutating would-repair oracle, not only zero repair calls after the repair path
  is disabled,
- the deleted `ViewId(pub NodeId)` wrapper returning through implicit conversions,
- the deleted `BoundaryId(NodeId)` wrapper and raw `NodeId`-keyed boundary storage returning,
- layout dirty iteration regressing from boundary candidates back to raw dirty `NodeId` ownership,
- cache-root observation collapse returning as a post-layout/post-paint pass,
- flat `Scene` bridges used as the normal renderer input or the only replay unit for local
  text/caret/selection changes; production chunk payload replay through temporary flat scenes must
  not return after the closure-supported native payload path, and `FlatCompat` must remain explicit
  debug/parity oracle only,
- ambiguous renderer chunk inputs returning as `scene_chunks: Option<_>`; launch-owned manifests are
  diagnostics unless passed through an explicit authoritative chunk source,
- full-blob text resource helpers returning to normal renderer chunk/resource paths; chunk keys use
  visible glyph residency, while shaping-aware cluster/run closure remains a gated follow-up,
- non-quad stream partial uploads broadening without a per-stream closure owner, fallback reason,
  write-count/byte counter, and coverage-gap proof,
- `fret-ui` public names that encode Dialog/Popover/Menu/Tooltip/dismissal policy,
- first-party examples that make advanced/manual assembly look like the default app path,
- source-policy allowlist entries that do not name an owner, reason, allowed seams, and retirement
  criteria.

Phase 3 classifies remaining bridge matches as one of four buckets before closeout:

- **Normal path:** must be deleted or split into a new owner lane with a failing deletion gate.
- **Debug/parity oracle:** allowed only when opt-in and excluded from normal launch/runtime evidence.
- **Compatibility alias/reader:** allowed only with old-bundle or migration tests.
- **Explicit advanced/raw seam:** allowed only when the public/default path has a replacement and the
  source-policy record names owner, reason, allowed seams, and retirement criteria.

---

## Coordinate Spaces (The Non-Negotiables)

### Units and DPI

- **Logical pixels** are the core UI coordinate unit (ADR 0017).
- Render backends convert logical to physical pixels; UI contracts stay portable.

### Spaces we must keep explicit

- **Local node space** (a node’s `bounds` and its children).
- **Window space** (logical px; what placement, semantics, and hit-testing ultimately reason in).
- **Scene space** (the root paint space; typically the same as window space in UI rendering).

### RenderTransform closure

- `render_transform` must affect:
  - paint emission,
  - hit-testing,
  - pointer event coordinates,
  - and anchored overlay geometry queries (ADR 0082).

Implementation anchors:

- `crates/fret-ui/src/tree/mod.rs` (transform propagation, hit-test mapping)
- `crates/fret-ui/src/elements/mod.rs` (`visual_bounds_for_element`, last-frame geometry)
- Component anchoring: `ecosystem/fret-ui-kit/src/overlay.rs`

Validation anchors:

- `crates/fret-ui/src/tree/tests/` (`render_transform_affects_hit_testing_and_pointer_event_coordinates`)
- `crates/fret-ui/src/tree/tests/` (`nested_render_transforms_compose_for_pointer_event_coordinates`)
- `crates/fret-ui/src/tree/tests/` (`hit_test_respects_rounded_overflow_clip_under_render_transform`)
- `crates/fret-ui/src/tree/tests/` (`overlay_render_transform_affects_hit_testing_and_event_coordinates`)
- `crates/fret-ui/src/tree/tests/` (`visual_bounds_for_element_includes_ancestor_render_transform`)
- `crates/fret-ui/src/tree/tests/` (`non_invertible_render_transform_is_ignored_for_paint_and_visual_bounds`)

---

## Subsystem Map (Contracts → Code → Validation)

### 1) UI Tree, Input Routing, Focus, Capture (Core Mechanism)

**Contract**

- Retained tree substrate + capture semantics: `docs/adr/0005-retained-ui-tree.md`
- Focus + command routing: `docs/adr/0020-focus-and-command-routing.md`
- Focus-visible: `docs/adr/0061-focus-rings-and-focus-visible.md`
- Runtime contract gates: `docs/adr/0066-fret-ui-runtime-contract-surface.md`

**Code entry points**

- `crates/fret-ui/src/tree/mod.rs`
- `crates/fret-ui/src/focus_visible.rs`

**Validation anchors**

- `cargo nextest run -p fret-ui` (many routing/focus tests live in `crates/fret-ui/src/tree/tests/`)

**Common failure modes to guard**

- capture vs click-through outside press interference (ADR 0069)
- modal barrier scoping errors (pointer/keyboard reaching underlay)

### 2) Multi-Root Overlays, Barriers, Outside Press (Mechanism + Policy Split)

**Contract**

- Multi-root overlays: `docs/adr/0011-overlays-and-multi-root.md`
- Overlay policy architecture (split): `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Non-modal outside press observer (click-through): `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- Focus traversal scope: `docs/adr/0068-focus-traversal-and-focus-scopes.md`

**Mechanism (runtime)**

- Overlay root stack + barrier flags in `crates/fret-ui/src/tree/mod.rs`
- Window-scoped input arbitration snapshot is published via `WindowInputContextService`
  (`fret_runtime::InputContext.window_arbitration`), not via a separate arbitration service.

**Policy (components)**

- `ecosystem/fret-ui-kit/src/window_overlays/*`
- shadcn surfaces in `ecosystem/fret-ui-shadcn/src/*`

**Demo harness**

- `apps/fret-examples/src/components_gallery.rs` (popover/tooltip/hover-card/etc.)

### 3) Anchored Placement (Floating UI Vocabulary)

**Contract**

- Placement solver: `docs/adr/0064-overlay-placement-contract.md`
- Behavior targets: `docs/reference-stack-ui-behavior.md` (Floating UI)

**Code entry points**

- `crates/fret-ui/src/overlay_placement/mod.rs`

**Validation anchors**

- `crates/fret-ui/src/overlay_placement/tests.rs`

**Closure requirement**

- Anchor geometry must be in **window logical space** and must track what the user sees under transforms.

### 4) Paint, Transforms, Clip Stack (Scene Semantics)

**Contract**

- Scene state stacks: `docs/adr/0019-scene-state-stack-and-layers.md`
- Rounded clipping: `docs/adr/0063-rounded-clipping-and-soft-clip-masks.md`
- Transform + clip composition: `docs/adr/0078-scene-transform-and-clip-composition.md`
- Draw order is non-semantic: `docs/adr/0081-draworder-is-non-semantic.md`

**Code entry points**

- UI paint emission: `crates/fret-ui/src/paint.rs`, `crates/fret-ui/src/declarative.rs`
- Scene rendering: `crates/fret-render-wgpu/src/renderer/mod.rs`

**Validation anchors**

- Runtime-level: hit-testing parity tests (overflow clip / rounded clip) in `crates/fret-ui/src/tree/tests/`
- Renderer-level: `crates/fret-render/tests/affine_clip_conformance.rs` (deep stacks, affine + clip-local evaluation)

**Non-goals (for v1)**

- Isolated opacity groups (ADR 0078 explicitly excludes this; would require new ops).

### 5) Declarative Layout (Flex/Grid/Tailwind Vocabulary + Perf)

**Contract**

- Constraints + optional Taffy: `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md`
- Declarative layout semantics (Flex): `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
- Tailwind layout primitives: `docs/adr/0062-tailwind-layout-primitives-margin-position-grid-aspect-ratio.md`
- Perf hardening (persistent Taffy trees): `docs/adr/0076-declarative-layout-performance-hardening.md`
- Virtualization constraint: `docs/adr/0042-virtualization-and-large-lists.md`

**Code entry points**

- `crates/fret-ui/src/declarative.rs`
- `crates/fret-ui/src/element.rs`

**Validation anchors**

- Layout semantics tests in `crates/fret-ui/src/declarative.rs`

### 6) Scrolling + Virtualization (Mechanism + Policy Split)

**Contract**

- Scroll + large list constraints: `docs/adr/0042-virtualization-and-large-lists.md`
- Virtual list contract (TanStack vocabulary): `docs/adr/0070-virtualization-contract.md`

**Code entry points**

- `crates/fret-ui/src/scroll.rs`
- `crates/fret-ui/src/virtual_list.rs`

**Validation anchors**

- `crates/fret-ui/src/scroll.rs` + `crates/fret-ui/src/virtual_list.rs` tests

### 7) Text Input + Read-only Selection + Geometry Queries

**Contract**

- Keyboard/IME boundary: `docs/adr/0012-keyboard-ime-and-text-input.md`
- Text geometry queries (caret/selection/hit test): `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`
- Multiline composition: `docs/adr/0071-text-input-multiline-composition-contract.md`
- Read-only selectable text baseline: `docs/adr/0137-readonly-text-selection-and-clipboard.md`
- View-cache reuse must preserve interaction-visible retained text state: `docs/adr/0176-declarative-liveness-roots-and-gc-under-view-cache-reuse.md`
  and `docs/adr/0224-view-cache-subtree-reuse-and-state-retention.md`

**Code entry points**

- `crates/fret-ui/src/text_input/mod.rs`
- `crates/fret-ui/src/text_area/mod.rs`
- `crates/fret-ui/src/text_input_style.rs`
- `crates/fret-ui/src/declarative/host_widget/event/selectable_text.rs`
- `crates/fret-ui/src/elements/runtime.rs`

**Validation anchors**

- `crates/fret-ui/src/text_input/tests.rs` and `crates/fret-ui/src/text_area/tests.rs` tests
- `crates/fret-ui/src/elements/runtime.rs`
  (`selectable_text_span_bounds_can_be_read_by_element`)
- `crates/fret-ui/src/declarative/tests/view_cache.rs`
  (`view_cache_preserves_selectable_text_interactive_span_bounds`)
- `tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-interactive-links-activation.json`

**Closure requirement**

- Cache-hit frames must preserve `SelectableText` interactive-span geometry so gallery/docs surfaces remain activation-safe and diagnostics-explainable under `ViewCache` reuse.

### 8) A11y / Semantics (AT-Ready Infrastructure)

**Contract**

- Semantics tree + AccessKit bridge: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- Active descendant for composite widgets: `docs/adr/0073-active-descendant-and-composite-widget-semantics.md`
- Acceptance checklist: `docs/a11y-acceptance-checklist.md`

**Code entry points**

- Snapshot production: `crates/fret-ui/src/tree/mod.rs` (semantics snapshot)
- Platform bridge: `crates/fret-runner-winit/src/accessibility.rs` (winit glue) + `crates/fret-a11y-accesskit/src/lib.rs` (AccessKit mapping)

**Closure requirement**

- Multi-root overlays and modal barriers must restrict **semantics reachability** exactly as they restrict input (ADR 0033 / ADR 0011).

### 9) Docking + Multi-Viewport + Multi-Window (Policy Outside Runtime)

**Contract**

- Docking model + ops + persistence (portable): `docs/adr/0013-docking-ops-and-persistence.md`
- Docking interaction arbitration: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Multi-window + DPI: `docs/adr/0017-multi-window-display-and-dpi.md`
- Docking layering B-route: `docs/adr/0075-docking-layering-b-route-and-retained-bridge.md`
- Viewport input forwarding: `docs/adr/0025-viewport-input-forwarding.md`

**Code entry points**

- Docking UI/policy: `ecosystem/fret-docking/src/dock/*`
- Docking demo (baseline): `apps/fret-examples/src/docking_demo.rs`
- Docking arbitration harness (ADR 0072 conformance): `apps/fret-examples/src/docking_arbitration_demo.rs`
- Conformance checklist: `docs/docking-arbitration-checklist.md`

**Important portability note**

- The docking graph can represent multiple logical windows.
- Platforms that do not support multiple OS windows should degrade by mapping logical windows into a single OS window (floating/teardown policy lives above `fret-ui`).

### 10) Observability / Inspector Hooks (Debuggability as a Contract)

**Contract**

- Observability strategy: `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`
- Diagnostics snapshot + scripted interaction tests: `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`

**Code entry points**

- `crates/fret-ui/src/tree/mod.rs` (debug stats structs; overlay stack/focus/capture visibility)
- Renderer metrics: `crates/fret-render-wgpu/src/renderer/mod.rs`

**Closure requirement**

- Each “hard-to-test” UI behavior must have either a regression test or a stable demo surface with a manual checklist.

---

## Risk Register (Gaps / Non-Closed Areas)

This is the “do it now or pay later” list, ordered by expected rewrite cost.

### P0 (high rewrite risk)

1. **Composite widget A11y closure beyond the cmdk baseline**
   - `active_descendant` exists end-to-end (schema + snapshot + AccessKit mapping) and is already used by `Command`.
   - Remaining closure work is to make this a reusable, predictable pattern across composite widgets:
     - combobox/listbox variants that keep focus in the input,
     - menu/list keyboard navigation that does not fight semantics reachability under modal barriers,
     - and explicit constraints when virtualization is involved.
    - Reference: `docs/adr/0073-active-descendant-and-composite-widget-semantics.md`
   - Virtualized collections baseline (locked):
     - `docs/adr/0084-virtualized-accessibility-and-collection-semantics.md`

2. **Docking + overlay + viewport capture conformance**
   - Status:
     - targeted regressions exist in `ecosystem/fret-docking/src/dock/tests.rs`,
     - end-to-end harness exists in `apps/fret-examples/src/docking_arbitration_demo.rs`,
     - manual checklist lives in `docs/docking-arbitration-checklist.md`.
   - Remaining: expand coverage for cross-window edge cases (tear-off + drag cancel + modal barrier) and any platform-specific pointer capture quirks.

3. **Transform + clip + hit-testing parity in edge cases**
   - Baseline parity tests exist (including rounded overflow clip under `render_transform`), but we still need to harden:
     - deeper clip stacks under mixed transforms (including scale + non-axis-aligned cases),
     - multi-root overlay edge cases (barriers + outside press + transformed overlays),
     - and explicit coverage for rotation/shear transforms if we intend to support them in v1.

### P1 (important, but can stage)

1. **Multi-window capability degradation policy**
   - Define a single, explicit policy for “logical windows” on platforms without OS multi-window.
   - Tie it to the capabilities matrix (ADR 0054) and docking layout persistence rules (ADR 0013/0017).
   - Contract: `docs/adr/0083-multi-window-degradation-policy.md`

2. **UI inspector surface**
   - Decide a minimal, stable inspector data shape and a scripted interaction test harness surface:
     - `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
     - (observability strategy baseline): `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`

3. **Placement solver expansion**
   - Arrow positioning (ADR 0066 says P1); add when renderer/shape semantics are stable enough.

---

## Suggested Next Closure Sprints (Bottom-Up)

1. **A11y active-descendant closure** (schema + bridge + cmdk adoption + tests).
2. **Transform/clip parity closure** (edge-case tests + renderer conformance linkage).
3. **Multi-window degradation policy** (explicit logical-window mapping rules for single-window platforms).
