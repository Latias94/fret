# Fret Golden Architecture (Module Closure Index)

This document is the **navigation layer** for the repository:

- ADRs (`docs/adr/`) remain the **source of truth** for contracts.
- This file makes the architecture **actionable** by mapping each module to:
  - the authoritative docs/ADRs to read,
  - the code entry points,
  - the validation anchors (tests / demos),
  - and the “closure checklist” to prevent late rewrites.

If you are new, read `docs/README.md` first, then use this file as your always-open index.

---

## How We “Close” A Module (Bottom-Up Loop)

For each module, we consider it “closed enough to scale” when:

1. **Contract is explicit**: ADR(s) are `Accepted` (or an explicit decision gate exists).
2. **Implementation matches contract**: code follows layering rules and determinism invariants.
3. **Validation exists**: at least one unit/integration test (or a stable demo harness when UI behavior is hard to test).
4. **Observability exists**: logs/debug hooks exist to diagnose drift.
5. **No backdoor dependencies**: no accidental `winit`/`wgpu` bleed into contract crates.

---

## System Layering (What Depends On What)

**Contracts (portable):**

- `crates/fret-core` — IDs, geometry, events, scene ops, docking graph/ops/persistence shapes, semantics schema.

**Runtime boundary (portable host services):**

- `crates/fret-runtime` — `UiHost`, `Effect`, `InputContext`, keymap/when, model store/registry types.

**Default integrated runtime:**

- `crates/fret-app` — default `UiHost` implementation (`App`), models/effects/commands, settings loading.

**UI substrate (mechanism-only):**

- `crates/fret-ui` — tree/layout/paint, routing/focus/capture, overlays substrate + placement solver, scroll/virtualization, text input.

**Component layer (policy-heavy):**

- `ecosystem/fret-ui-kit` — headless policies + composition helpers (action hooks, overlays policy, tokens/recipes infra).
- `ecosystem/fret-ui-shadcn` — shadcn/ui v4-aligned component surface (recipes, variants, behaviors).
- `ecosystem/fret-docking` — docking UI + interaction policy (B-route; policy outside runtime).

**Backends (not portable):**

- `crates/fret-render` — wgpu renderer implementation for `fret-core::Scene`.
- `crates/fret-platform` — portable platform I/O contracts (no `winit`).
- `crates/fret-runner-winit` — winit glue (event mapping + AccessKit adapter).
- `crates/fret-launch` — concrete desktop glue wiring winit + renderer + effect draining.

**Ergonomics + demos:**

- `crates/fret-ui-app` — convenience layer binding `fret-ui` to `fret-app::App` (integrated app ergonomics).
- `apps/fret-examples` — end-to-end harness code.
- `apps/fret-demo` — native harness shells.
- `apps/fret-demo-web` — wasm harness shell.
- `crates/fret` — facade crate (re-exports).

Hard rules: see `docs/dependency-policy.md` and ADR 0037.

---

## Module Index

Each section below answers:

- What is the module responsible for?
- Which ADRs define the “hard-to-change” contracts?
- Where to start reading code?
- What must be validated before scaling?

### `fret-core` (Portable Contracts)

**Read first**

- `docs/dependency-policy.md`
- ADR 0002 (`display list / scene ops`): `docs/adr/0002-display-list.md`
- ADR 0004 (`resource handles`): `docs/adr/0004-resource-handles.md`
- ADR 0013 (`docking ops/persistence`): `docs/adr/0013-docking-ops-and-persistence.md`
- ADR 0019 (`scene state stack/layers`): `docs/adr/0019-scene-state-stack-and-layers.md`
- ADR 0078/0079 (`scene transform/clip + layer markers`): `docs/adr/0078-scene-transform-and-clip-composition.md`, `docs/adr/0079-scene-layers-marker-only-v1.md`
- ADR 0080 (`vector path contract`): `docs/adr/0080-vector-path-contract.md`

**Code entry points**

- `crates/fret-core/src/lib.rs`
- `crates/fret-core/src/scene.rs`
- `crates/fret-core/src/dock.rs`, `crates/fret-core/src/dock_op.rs`, `crates/fret-core/src/dock_layout.rs`
- `crates/fret-core/src/semantics.rs`
- `crates/fret-core/src/input.rs`

**Closure checklist (P0)**

- Scene ordering invariants are testable (renderer conformance + UI-level invariants).
- Docking model + ops are stable and serializable (layout versioning stays explicit).
- Event payloads remain portable (no `PathBuf` / no backend types).

### `fret-runtime` (Host Boundary)

**Read first**

- ADR 0052: `docs/adr/0052-ui-host-runtime-boundary.md`
- ADR 0001 (effects): `docs/adr/0001-app-effects.md`
- ADR 0022 (when expressions): `docs/adr/0022-when-expressions.md`
- ADR 0054 (capabilities): `docs/adr/0054-platform-capabilities-and-portability-matrix.md`

**Code entry points**

- `crates/fret-runtime/src/ui_host.rs`
- `crates/fret-runtime/src/effect.rs`
- `crates/fret-runtime/src/input.rs`
- `crates/fret-runtime/src/when_expr.rs`

**Closure checklist (P0)**

- `UiHost` stays small and portable (no backend types; prefer `Effect`).
- Capability gates are used for enable/disable, not ad-hoc widget branching.

### `fret-app` (Default Integrated Runtime)

**Read first**

- ADR 0031 (models + leasing): `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- ADR 0034 (timers/scheduling): `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- ADR 0023 (command metadata): `docs/adr/0023-command-metadata-menus-and-palette.md`
- ADR 0014 (settings files): `docs/adr/0014-settings-and-configuration-files.md`

**Code entry points**

- `crates/fret-app/src/app.rs`
- `crates/fret-app/src/ui_host.rs`
- `crates/fret-app/src/settings.rs`

**Closure checklist (P0)**

- Effects draining is deterministic and bounded.
- Settings schema versioning is explicit and migrations are planned.

### `fret-ui` (Mechanism-Only Runtime Substrate)

**Read first**

- Contract surface lock: ADR 0066 `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Declarative model: ADR 0028 `docs/adr/0028-declarative-elements-and-element-state.md`
- Authoring ergonomics: ADR 0039 `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
- Action hooks: ADR 0074 `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md` + `docs/action-hooks.md`
- Layout contract: ADR 0035 / 0057 / 0062 / 0076 (constraints, Tailwind semantics, perf hardening)
- Overlays: ADR 0011 / 0067 / 0069 (`multi-root`, `policy architecture`, `outside press`)
- Placement: ADR 0064 `docs/adr/0064-overlay-placement-contract.md`
- RenderTransform (paint + hit-test + overlay anchors): ADR 0083 `docs/adr/0083-render-transform-hit-testing.md`
- Frame recording + replay caching: ADR 0055 `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`
- Virtualization: ADR 0070 + 0047
- Text input: ADR 0044/0045/0046/0071 + ADR 0012/0043 for IME arbitration
- Semantics / A11y boundary: ADR 0033 / 0073 + `docs/a11y-acceptance-checklist.md`
- Virtualized collection accessibility: ADR 0085
- UI closure map (this subsystem): `docs/ui-closure-map.md`

**Code entry points**

- `crates/fret-ui/src/tree/mod.rs` (routing + focus/capture + layers + paint cache)
- `crates/fret-ui/src/elements/mod.rs`, `crates/fret-ui/src/declarative.rs`
- `crates/fret-ui/src/action.rs` (action hook substrate)
- `crates/fret-ui/src/overlay_placement/mod.rs`
- `crates/fret-ui/src/scroll.rs`, `crates/fret-ui/src/virtual_list.rs`
- `crates/fret-ui/src/text_input/mod.rs`, `crates/fret-ui/src/text_area/mod.rs`

**Validation anchors**

- Runtime conformance suite: `cargo nextest run -p fret-ui` (most tests live in `crates/fret-ui/src/tree/tests/`)
- RenderTransform closure (hit testing + event coordinates + visual bounds):
  - `crates/fret-ui/src/tree/tests/` (`render_transform_affects_hit_testing_and_pointer_event_coordinates`)
  - `crates/fret-ui/src/tree/tests/` (`nested_render_transforms_compose_for_pointer_event_coordinates`)
  - `crates/fret-ui/src/tree/tests/` (`hit_test_respects_rounded_overflow_clip_under_render_transform`)
  - `crates/fret-ui/src/tree/tests/` (`overlay_render_transform_affects_hit_testing_and_event_coordinates`)
  - `crates/fret-ui/src/tree/tests/` (`visual_bounds_for_element_includes_ancestor_render_transform`)
  - `crates/fret-ui/src/tree/tests/` (`non_invertible_render_transform_is_ignored_for_paint_and_visual_bounds`)
- Anchored placement solver invariants:
  - `crates/fret-ui/src/overlay_placement/tests.rs` (`keeps_bottom_when_it_fits`)
  - `crates/fret-ui/src/overlay_placement/tests.rs` (`flips_from_bottom_to_top_when_bottom_overflows`)
  - `crates/fret-ui/src/overlay_placement/tests.rs` (`sized_variant_prefers_side_with_less_main_axis_overflow`)

**Subsystem index (UI — find things fast)**

```mermaid
flowchart LR
  Host[Runner / UiHost] --> UiTree[UiTree (layers + routing + layout + paint)]
  UiTree --> Scene[Scene ops (paint stream)]
  UiTree --> Semantics[SemanticsSnapshot (a11y stream)]
  UiTree --> Effects[Effects (IME / commands / requests)]
  Scene --> Render[fret-render (wgpu)]
  Semantics --> PlatformA11y[fret-runner-winit (AccessKit bridge)]
```

- **Event routing + capture + focus**: ADR 0005 / 0020 / 0068; code: `crates/fret-ui/src/tree/mod.rs`
- **Overlays + barriers + outside press**: ADR 0011 / 0067 / 0069; code: `crates/fret-ui/src/tree/mod.rs`; policy: `ecosystem/fret-ui-kit/src/window_overlays/*`
- **Anchored overlays (placement)**: ADR 0064; code: `crates/fret-ui/src/overlay_placement/mod.rs`; reference: `repo-ref/floating-ui`
- **RenderTransform + anchor geometry**: ADR 0083; code: `crates/fret-ui/src/tree/mod.rs` (hit-test mapping + `visual_bounds_for_element` recording) + `crates/fret-ui/src/elements/mod.rs` (cross-frame geometry queries)
- **Layout**: ADR 0035 / 0057 / 0062 / 0076; code: `crates/fret-ui/src/declarative.rs`, `crates/fret-ui/src/element.rs`
- **Painting + clip/transform semantics**: ADR 0002 / 0019 / 0063 / 0078 / 0082; code: `crates/fret-ui/src/paint.rs` + declarative paint emission
- **Performance primitives**: ADR 0051 / 0055 / 0034; code: `crates/fret-ui/src/tree/mod.rs`, `crates/fret-ui/src/elements/mod.rs`
- **Scrolling + virtualization**: ADR 0042 / 0047 / 0070; code: `crates/fret-ui/src/scroll.rs`, `crates/fret-ui/src/virtual_list.rs`
- **Text input + IME**: ADR 0012 / 0043 / 0044 / 0045 / 0046 / 0071; code: `crates/fret-ui/src/text_input/mod.rs`, `crates/fret-ui/src/text_area/mod.rs`
- **A11y / AT surface (semantics)**: ADR 0033 / 0073; code: `crates/fret-ui/src/tree/mod.rs` (snapshot) + `crates/fret-a11y-accesskit/src/lib.rs` (AccessKit mapping) + `crates/fret-runner-winit/src/accessibility.rs` (adapter glue)

**Closure checklist (P0)**

- No policy leaks: shadcn/Radix/APG “outcomes” are implemented in components via hooks.
- Determinism: hit testing + paint + semantics snapshots remain consistent across overlay roots.
- Performance: layout/paint invalidation is explicit and test-covered for key edge cases.

### `fret-ui-kit` (Headless Policies + Composition Helpers)

**Read first**

- `docs/foundation-first-workflow.md`
- `docs/action-hooks.md`
- ADR 0074 (policy migration) and ADR 0066 (runtime stays small)

**Code entry points**

- `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs`
- `ecosystem/fret-ui-kit/src/window_overlays/*`

**Closure checklist (P0)**

- Policy is testable without a runner (unit/integration tests).
- Overlay dismissal/focus-restore rules are centralized here (not in runtime).

### `fret-ui-shadcn` (shadcn/ui v4 Surface)

**Read first**

- `docs/shadcn-declarative-progress.md`
- Upstream reference: `repo-ref/ui` (registry + recipes) and Radix UI Primitives (upstream: <https://github.com/radix-ui/primitives>; pinned locally, see `docs/repo-ref.md`)

**Closure checklist (P0)**

- Components validate runtime mechanisms (popover/menu/cmdk are “acceptance tests” for focus, overlays, semantics).

### `fret-docking` (Docking UI + Policy)

**Read first**

- ADR 0075 (B-route): `docs/adr/0075-docking-layering-b-route-and-retained-bridge.md`
- Docking contracts: ADR 0013 / 0017 / 0015 / 0072
- Single-window platforms (degradation policy): ADR 0084 `docs/adr/0084-multi-window-degradation-policy.md`
- Viewport tools boundary: ADR 0049 + ADR 0025

**Code entry points**

- `ecosystem/fret-docking/src/dock/space.rs` (DockSpace UI)
- `ecosystem/fret-docking/src/dock/manager.rs` (DockManager + ops integration)
- `ecosystem/fret-docking/src/dock/viewport.rs` (viewport hit mapping → `ViewportInputEvent`)
- `ecosystem/fret-docking/src/dock/mod.rs` (`DockViewportOverlayHooks`)

**Closure checklist (P0)**

- Cross-window drag + overlays + viewport capture arbitration is locked (ADR 0072).
- Keep-alive / early submission / programmatic close have tests or stable demo harness coverage.

### `fret-render` (wgpu Renderer)

**Read first**

- ADR 0009 (ordering/batching): `docs/adr/0009-renderer-ordering-and-batching.md`
- ADR 0030 (shape semantics over SDF): `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
- ADR 0029 (text pipeline): `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
- Scene transform/clip: ADR 0078/0079

**Code entry points**

- `crates/fret-render-wgpu/src/renderer/mod.rs`
- `crates/fret-render-wgpu/src/text.rs`

**Closure checklist (P0)**

- Scene validation + renderer conformance tests cover transforms/clips/layers.
- Text caching keys include all configuration that affects glyph output (font stack, locale, etc.).

### `fret-platform` + `fret-launch` (Desktop Backend)

**Read first**

- ADR 0003 (platform boundary): `docs/adr/0003-platform-boundary.md`
- ADR 0054 (capabilities): `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- ADR 0041/0053 (DnD + payload portability)

**Code entry points**

- Runner: `crates/fret-launch/src/runner/mod.rs`
- Accessibility bridge: `crates/fret-runner-winit/src/accessibility.rs` (winit glue) + `crates/fret-a11y-accesskit/src/lib.rs` (AccessKit mapping)

**Closure checklist (P0)**

- Single authority for window registry and event translation (avoid duplicated registries).
- Effect execution is centralized and deterministic.

### `fret-demo` (Stable Harnesses)

**Code entry points**

- Components gallery: `apps/fret-examples/src/components_gallery.rs`
- Docking demo: `apps/fret-examples/src/docking_demo.rs`
- Docking arbitration harness (docking + viewport + overlays): `apps/fret-examples/src/docking_arbitration_demo.rs` (validates ADR 0072; see `docs/docking-arbitration-checklist.md`)

**Closure checklist**

- Each “hard-to-test” behavior has a stable demo surface (and a short manual checklist if needed).

---

## Reference Sources (repo-ref)

Pinned references are documented in `docs/repo-ref.md`. The typical mapping is:

- GPUI/Zed substrate patterns: `repo-ref/zed`, `repo-ref/gpui-component`
- shadcn/Radix outcomes: `repo-ref/ui`, Radix UI Primitives (upstream: <https://github.com/radix-ui/primitives>; pinned locally, see `docs/repo-ref.md`)
- placement vocabulary: `repo-ref/floating-ui`
- virtualization vocabulary: `repo-ref/virtualizer` (Rust engine; primary); see `docs/repo-ref.md`
- docking UX vocabulary: `repo-ref/imgui`, `repo-ref/dear-imgui-rs`

---

## Quick Drift Checks (Useful Commands)

- Find stale docking entry points in docs: `rg -n "fret-ui/src/dock\\.rs" docs -S`
- Validate workspace health: `cargo nextest run --workspace`
- Validate layering: `rg -n "use winit|use wgpu" crates/fret-core crates/fret-ui crates/fret-runtime -S`
