# ADR 0037: Workspace Boundaries and External Components Repository (`fret-components`)

Status: Accepted

## Context

Fret’s goal is to be a **game-editor-grade UI framework**:

- docking + tear-off windows,
- multiple engine viewports,
- layered overlays/menus/modals,
- high-performance GPU rendering (wgpu now, WebGPU/wasm later).

Editor UIs also require a large surface area of reusable UI components (trees, inspectors, tables, pickers),
but growing the core framework into a “kitchen sink” tends to cause:

- slow iteration due to tight coupling,
- accidental dependency cycles (platform/render leaking into UI),
- large rewrites when porting to new backends (wasm/mobile).

GPUI’s ecosystem demonstrates a healthy split:

- a small, stable runtime (`gpui`),
- a separate component library repo (`gpui-component`) that can evolve quickly.

We want the same outcome for Fret, while keeping the framework/editor-app scope boundary explicit (ADR 0027).

Zed/GPUI code anchors (non-normative):

- Zed’s monorepo also reflects a similar “runtime substrate vs policy UI surfaces” split:
  - runtime substrate: `repo-ref/zed/crates/gpui`
  - policy-heavy UI surfaces: `repo-ref/zed/crates/ui`

Current workspace note:

- `fret-core` is kept layout-engine-free. Layout engines such as `taffy` are allowed only in `fret-ui` and/or
  `fret-components-*` (see ADR 0035).

## Decision

### 1) Two repositories: `fret` (core) and `fret-components` (component ecosystem)

**Repo A: `fret`**

Contains the framework core and backends:

- `fret-core`: backend-agnostic contracts (IDs, geometry, events, scene/display list, resource handles).
- `fret-app`: app runtime (models, effects, commands, scheduling hooks).
- `fret-ui`: UI runtime substrate (execution model, layout/invalidation, hit-testing, focus/capture, **overlay layer mechanism**, docking UX infrastructure).
  - Note: “shadcn-like” overlay *surfaces* (popover/dialog/menu/tooltip/toast/command palette/menubar) should live in
    `fret-components-*` so sizing/variants/tokens can converge without fighting runtime widgets.
- `fret-render-*`: rendering backends (wgpu today; future WebGPU/wasm reuse).
- `fret-platform-*`: platform backends (winit today; future web/mobile).
- `fret`: facade crate (re-exports the stable public surface).
- `fret-demo`: examples and integration tests for framework-level milestones.

**Repo B: `fret-components`**

Contains reusable UI components as a workspace with multiple crates.

This repository is explicitly expected to ship an **editor kit**:

- a shadcn-inspired component set (primitives with consistent tokens/variants),
- higher-level editor UI patterns (tree, inspector, table, command UI),
- optional specialized visualization components (charts, etc.).

Example crate layout (names are placeholders):

- **Primitives / shadcn-style UI**
  - `fret-ui-kit`: reusable component infrastructure (token-driven styling, `StyledExt`, size/density vocabulary, low-level building blocks).
  - `fret-ui-shadcn`: shadcn/ui (v4) aligned component surface (names + variants + policies), built on top of `fret-ui-kit`.
  - `fret-icons`: icon primitives and common icon sets (as data + paint, not platform APIs).
- **Editor patterns**
  - `fret-components-tree`: tree view + virtualization (hierarchy/project browser patterns).
  - `fret-components-inspector`: inspector/form patterns and field widgets.
  - `fret-components-table`: tables/grids for editor panels.
  - `fret-components-command-ui`: command palette UI, menu surfaces, keybinding help.
- **Specialized**
  - `fret-components-charts`: charts/plots (time series, histograms, etc.).
  - `fret-components-text-edit`: text fields and editor-grade text widgets (incremental; depends on ADR 0006/0029).

Optionally:

- `fret-components`: a meta crate that re-exports the recommended set.

Component crates can be added or removed without changing the core framework repository.

Non-goal (hard rule):

- `fret-components` must remain **domain-agnostic**: no direct dependency on an engine’s asset database,
  ECS/scene model, gizmo/tool policy, or project system. Those remain app-owned (ADR 0027).

### 2) Dependency direction is a hard rule

To preserve portability and avoid dependency cycles:

- `fret-core` depends on no other `fret-*` crate.
- `fret-app` depends on `fret-core`.
- `fret-ui` depends on `fret-core` + `fret-app`.
- `fret-render-*` depends on `fret-core` (and backend crates like `wgpu`), but must not depend on `fret-ui`.
- `fret-platform-*` depends on `fret-core` (and backend crates like `winit`), but must not depend on `fret-ui` or `fret-render-*`.
- A dedicated runner/integration crate (e.g. `fret-launch`) depends on `fret-platform-*` + `fret-render-*` + `fret-ui`
  and is responsible for wiring event loop + GPU presentation together.
- `fret-components-*` depends on `fret-ui` (and thus `fret-app`/`fret-core`), and must not depend on `fret-platform-*` or `fret-render-*`.

Rationale:

- component crates should be portable and testable without a windowing backend,
- platform/render backends must remain swappable (desktop/web/mobile).

### 3) “Contracts vs implementations” split within the core repo

The core repo must keep “contracts” stable and “implementations” swappable:

- backend-agnostic types live in `fret-core`,
- backend-specific code lives in `fret-render-*` and `fret-platform-*`,
- UI runtime is independent from platform/render implementation details (consumes only contracts and app runtime services).

### 4) Layout engine dependencies are not allowed in `fret-core`

To keep the core contract minimal:

- `fret-core` must not depend on layout engines (e.g. `taffy`) or any “UI algorithm” implementation crates.
- Layout engines are allowed in `fret-ui` and/or `fret-components-*` (see ADR 0035).

This prevents “contract crates” from pulling in heavy dependencies and makes wasm/mobile portability easier.

### 5) Versioning and compatibility policy

We treat the `fret-*` crates in the core repo as a single “platform”:

- publish and tag them with the same version,
- changes that break the public contracts require a semver-major bump.

For `fret-components`:

- component crates track the same major version as `fret` (compatibility),
- minor versions may move faster, but must declare compatible `fret` versions in `Cargo.toml`.

### 6) Examples live in core; component demos live with components

- Framework-level demos and “integration proof” apps live in the `fret` repo (`fret-demo`).
- Component-specific demos and storybook-like showcases live in the `fret-components` repo.

## Consequences

- The core framework stays small and stable, which reduces long-term refactor risk.
- Component development can iterate rapidly without destabilizing platform/render backends.
- wasm/mobile ports are easier because UI components do not depend on desktop-only backends.

## Open Questions (To Decide Before Implementation)

### Locked P0 Choices

1) **Publishing strategy**:
   - Start `fret-components` as a separate repository consumed via git dependency.
   - Publish to crates.io later, once the `fret-ui` authoring/runtime surface is stable.

2) **Compatibility enforcement**:
   - Add CI checks to enforce the dependency direction rules and forbid cycles (required before expanding crate count).
   - Recommended local/CI gate in this repo: `python3 tools/check_layering.py`.

3) **Facade surface**:
   - The `fret` crate facade remains **framework-only** (no re-export of `fret-components`).
   - `fret-components` may provide its own meta crate for convenience.

4) **Runner packaging**:
   - Introduce a dedicated runner/integration crate (e.g. `fret-launch` or `fret-desktop`)
     that wires platform + renderer + app/ui together.
   - Keep `fret-platform-*` focused on platform IO (windows/events/clipboard/IME/DnD) rather than GPU composition policy.

Current status:

- Implemented in this workspace as `crates/fret-launch`.
- MVP45 incubation: `ecosystem/fret-ui-kit` exists in-tree to validate the component API and token wiring before
  extracting to a separate `fret-components` repository.

## Clarification (2025): General-purpose UI first, editor kit is layered

Although Fret’s motivating use case is an editor, the component ecosystem should remain **general-purpose by default**:

- `fret-ui-kit` should target application UIs (forms, navigation, dialogs, lists) and avoid engine/editor domain concepts.
- Editor-only patterns (inspector, hierarchy, scene graph affordances) should live in separate crates (e.g. `fret-components-editor`) or in the app.

Component styling guidance:

- Prefer a “Tailwind-like” internal authoring model: small typed tokens + recipe/variant composition, not CSS selector strings.
- Namespaced theme keys (ADR 0050 §5.1) are the extensibility escape hatch for component libraries and plugins.
- Prefer composable “style patches” (`StyleRefinement`) and ergonomic extension traits (`StyledExt`) in the component layer so recipes/variants can stack predictably without hard-coding magic numbers in the runtime.

Reference posture:

- `repo-ref/gpui-component` for Rust component ergonomics + theme schema patterns.
- `repo-ref/fret-ui-precision` for token taxonomy and “recipe” composition vocabulary (as a design reference, not an API dependency).
