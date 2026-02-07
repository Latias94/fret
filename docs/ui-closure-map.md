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
- **Coordinate closure**: paint, hit-testing, and event coordinates agree under transforms (ADR 0083).

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
  - and anchored overlay geometry queries (ADR 0083).

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
- Draw order is non-semantic: `docs/adr/0082-draworder-is-non-semantic.md`

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

### 7) Text Input + IME + Geometry Queries

**Contract**

- Keyboard/IME boundary: `docs/adr/0012-keyboard-ime-and-text-input.md`
- Text geometry queries (caret/selection/hit test): `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`
- Multiline composition: `docs/adr/0071-text-input-multiline-composition-contract.md`

**Code entry points**

- `crates/fret-ui/src/text_input/mod.rs`
- `crates/fret-ui/src/text_area/mod.rs`
- `crates/fret-ui/src/text_input_style.rs`

**Validation anchors**

- `crates/fret-ui/src/text_input/tests.rs` and `crates/fret-ui/src/text_area/tests.rs` tests

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
- Diagnostics snapshot + scripted interaction tests: `docs/adr/0174-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`

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
     - `docs/adr/0085-virtualized-accessibility-and-collection-semantics.md`

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
   - Contract: `docs/adr/0084-multi-window-degradation-policy.md`

2. **UI inspector surface**
   - Decide a minimal, stable inspector data shape and a scripted interaction test harness surface:
     - `docs/adr/0174-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
     - (observability strategy baseline): `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`

3. **Placement solver expansion**
   - Arrow positioning (ADR 0066 says P1); add when renderer/shape semantics are stable enough.

---

## Suggested Next Closure Sprints (Bottom-Up)

1. **A11y active-descendant closure** (schema + bridge + cmdk adoption + tests).
2. **Transform/clip parity closure** (edge-case tests + renderer conformance linkage).
3. **Multi-window degradation policy** (explicit logical-window mapping rules for single-window platforms).
