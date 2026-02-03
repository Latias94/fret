# Immediate-Mode Authoring Facade ("imui") v1

Status: Baseline (historical reference; superseded by v2)
Last updated: 2026-02-03

This document proposes an **immediate-mode authoring facade** for Fret that feels closer to `egui` / Dear ImGui (and
to `repo-ref/dear-imgui-rs`), while remaining aligned with Fret’s core runtime direction:

- **Per-frame declarative element tree** mounted into a retained substrate (`UiTree`) (ADR 0028).
- **Mechanism-only core** (`crates/fret-ui`), with interaction policy and “recipes” living in ecosystem crates (ADR 0066).
- **Editor-grade requirements** (docking, multi-window, multi-viewport surfaces, GPU layered rendering).

Tracking:

- TODO tracker: `docs/workstreams/imui-authoring-facade-v1-todo.md`
- Fearless v2 plan: `docs/workstreams/imui-authoring-facade-v2.md`
- Fearless v2 tracker: `docs/workstreams/imui-authoring-facade-v2-todo.md`
- Architecture baseline: `docs/architecture.md` (declarative mount + retained semantics)
- Authoring ergonomics notes: `docs/ui-ergonomics-and-interop.md`, `docs/workstreams/authoring-ergonomics-fluent-builder.md`
- Docking/multi-window direction: `docs/workstreams/docking-multiwindow-imgui-parity.md`, `docs/workstreams/docking-multiviewport-arbitration-v1.md`

---

## 1) Problem Statement

Fret’s declarative authoring model is powerful, but many users (especially engine/editor developers) want an API that:

- feels “write UI in-order” (immediate-mode ergonomics),
- composes well in loops without macro-heavy boilerplate,
- is easy to extend by third-party ecosystem crates,
- does not lock policy into `crates/fret-ui`,
- remains compatible with retained semantics (focus/IME/overlays/caching/virtualization),
- works across windows and viewports (Unity/Unreal-style workflows).

We want to achieve this **without introducing a second UI runtime** and without forcing a future migration.

---

## 2) Scope and Non-Goals

In scope (v1):

- An ecosystem crate (proposed: `ecosystem/fret-imui`) that provides:
  - an `ImUi` façade type,
  - a `Response` model (clicked/changed/hovered/focused/drag state),
  - a small set of foundational layout + widget helpers (text, button, checkbox, separators, row/column/scroll),
  - a stable identity story equivalent to `push_id()` (`ui.id(key, |ui| ...)`).
- A third-party integration story:
  - ecosystem crates can expose `imui` feature gates and implement `fn widget(ui: &mut ImUi, ...) -> Response`.
- Interop escape hatches:
  - allow embedding existing declarative elements and complex surfaces (canvas, viewport surfaces, docking host).

Non-goals (v1):

- No new renderer backend or platform backend (imui is authoring only).
- No “policy bundle” in core. Default spacing/padding/focus rings remain ecosystem-level.
- No requirement to match Dear ImGui 1:1 (we align on ergonomic outcomes, not API parity).
- No new retained widget framework. imui must compile down to the existing element-based runtime.

---

## 2.1 Invariants (Do Not Break)

These are the “hard-to-change” seams that imui must preserve. If any of these become false, we will likely
force a large refactor later.

- **Identity stability**: stable IDs under dynamic lists and reordering (keyed scopes remain the canonical story).
- **Input/focus correctness**: focus, capture, IME targets, overlays, and arbitration keep working the same way.
- **Multi-window + multi-root**: multiple windows and overlay roots remain first-class and consistent with
  `ElementContext` scoping.
- **Docking + viewport surfaces**: docking policy and embedded engine viewports keep their interaction boundaries.
- **No “second runtime”**: imui is authoring only; it must compile down to the existing element taxonomy.

---

## 3) High-Level Architecture (How imui Fits Fret)

### 3.1 The key idea: immediate-like authoring over a declarative substrate

imui is a thin authoring façade that produces a declarative element list:

- Users write `ui.button("OK")` in-order.
- Under the hood, imui emits `AnyElement` nodes using the existing `ElementContext` mechanisms.
- The returned element list is mounted via `render_root(...)` into `UiTree` (ADR 0028).

This keeps all “hard-to-change” semantics in one place:

- stable identity → `GlobalElementId` mapping → `NodeId` reuse,
- focus/capture/IME targets,
- overlays and multi-root z-order,
- cache roots, view cache reuse, windowed surfaces.

### 3.2 Backends: platform and renderer stay out of imui

imui should depend on:

- `fret-ui` (for `ElementContext`, element taxonomy, input/focus mechanisms),
- `fret-core` (shared geometry/types),
- optionally `fret-runtime` if needed for model observation helpers.

It should **not** depend on:

- `fret-render` (wgpu),
- `winit`,
- runner crates.

This keeps imui compatible with:

- native winit + wgpu today,
- wasm runner today,
- future renderers that consume `fret-core::SceneOp` differently,
- future runners (SDL, custom engine host, etc.).

---

## 4) Proposed Public Surface (v1)

### 4.1 Entry point

imui is invoked inside an existing root render closure, producing elements for the declarative mount pass:

- `imui(cx, |ui| { ... }) -> Elements`
- `imui_vstack(cx, |ui| { ... }) -> Elements` (wraps output in a `Column`)
- `imui_build(cx, out: &mut Vec<AnyElement>, |ui| { ... })` (sink-based variant)

Note: `fret_ui::element::Elements` is a small owned wrapper around `Vec<AnyElement>` specifically intended for
authoring-facing APIs. It keeps signatures “iterator-friendly” without forcing callers into a bare `Vec`.

The contract remains the same as ADR 0028:

- call `render_root(...)` once per frame before layout/paint,
- let the runtime own invalidation and retained semantics.

### 4.2 `ImUi` (core façade type)

`ImUi` is an authoring helper that:

- holds a mutable reference to `ElementContext`,
- accumulates child elements in order (or pushes into an explicit sink),
- exposes small widget/layout helper methods.

It must provide:

- `ui.id(key, |ui| ...)` — stable identity scope (dynamic loops, trees),
- `ui.with_style(...)` / `ui.theme_*` hooks as *optional* (v1 minimal; richer styling belongs in A2),
- `ui.cx_mut()` — escape hatch to raw `ElementContext`,
- `ui.push(child: AnyElement)` or `ui.add(...)` for custom elements.

#### Why a sink-based variant exists

Fret already uses a “return `IntoIterator` or provide a `_build` sink” pattern in ecosystem authoring helpers to avoid
iterator borrow pitfalls (e.g. closures that capture `&mut cx` cannot be returned as iterators). imui should follow the
same approach:

- `imui(...) -> Elements` for the common “Hello World” path,
- `imui_build(...)` for cases where callers naturally want to push into an existing output buffer or are building from
  iterators that cannot be returned directly.

#### Minimal Hello World (prototype)

This workstream currently ships a minimal demo (native runner):

```rust
fret_imui::imui_vstack(cx, |ui| {
    ui.text("Hello, imui!");
    if ui.button("OK").clicked() {
        // ...
    }
})
```

See: `apps/fret-examples/src/imui_hello_demo.rs` and `apps/fret-demo/src/bin/imui_hello_demo.rs`.

### 4.3 `Response` (interaction results)

To be ecosystem-friendly, widgets should return a `Response` instead of a bare `bool`.

Minimum v1 fields:

- `hovered`, `pressed`, `focused`,
- `clicked` (activation),
- `changed` (value mutation),
- `rect` (last layout bounds if available) for popover placement and tooling.

Implementation direction:

- derive instantaneous states from existing runtime state (e.g. the pressable focused/hovered tracking),
- derive “edge-triggered” events (clicked/changed) using element-local state keyed by `GlobalElementId`
  (store “last fired frame” or “pending flag”, clear-on-read).

---

## 5) Identity and Loops (Avoiding the #1 Immediate-Mode Footgun)

Immediate-mode UIs rely on stable IDs:

- Dear ImGui: `PushID` / label `##suffix`
- egui: `Id` / `ui.push_id(...)`

In Fret:

- unkeyed callsite-based IDs are convenient, but **dynamic lists must be keyed** (ADR 0028).

Therefore imui must:

- make the “keyed scope” path obvious and cheap:
  - `ui.id(key, |ui| { ... })`
- provide collection helpers that enforce keys (optional, but recommended):
  - `ui.for_each_keyed(items, |ui, key, item| { ... })`

In the current prototype, `for_each_keyed` is provided by `ecosystem/fret-imui` to make the keyed path the easy path.

### 5.1 Key stability rules (recommended, v1)

`ui.id(key, ...)` is a *semantic identity* boundary. If the key changes, the runtime is allowed to treat it as a
different element instance (losing focus state, cached layout, transient responses, etc.).

Recommended key types:

- `Uuid` (best for persisted editor layout and cross-session stability),
- `&'static str` or `Arc<str>` / `String`,
- numeric IDs (`u64`, `u32`) from a stable registry,
- tuples of stable keys (e.g. `(PanelId, TabId)`).

Avoid / footguns:

- pointer/address-based keys (not stable across runs),
- keys derived from iteration order of `HashMap`/`HashSet` (not stable),
- floats (NaN edge cases; instability under formatting/precision changes),
- using `DefaultHasher` (not deterministic across Rust versions / platforms).

### 5.2 Persisted editor layout keys (decision)

For editor-grade UX, docking layouts must persist across restarts and must remain compatible with plugins (ADR 0013 and
ADR 0016). This requires a **human-reviewable, namespaced, versionable** key scheme.

Decision (v1):

- Docking persistence uses `fret-core` panel identity types:
  - `PanelKind(String)` as the *stable* persisted identifier,
  - `PanelKey { kind: PanelKind, instance: Option<String> }` for optional multi-instance panels.
- `PanelKind` strings are the canonical “official persisted keys” for dock panels and must be treated as a stable
  contract.

Recommended conventions:

- Built-in panels: prefix with `core.` (e.g. `core.scene`, `core.game`, `core.todo`).
- Plugin panels: prefix with the plugin ID (e.g. `plugin.acme.foo.panel`, `plugin.zed.workspace.search`).
- Keep keys stable: renaming a `PanelKind` is a breaking change unless you ship a migration that rewrites persisted
  layout files.
- `PanelKey.instance` is for multiple instances of the *same* kind:
  - prefer a stable logical instance key (file path, asset ID, project-relative URI),
  - or a persisted UUID string if the instance is user-created and must survive restarts.

How this relates to imui `ui.id(...)`:

- When authoring dock panels in imui, prefer `ui.id(panel_key.clone(), |ui| { ... })` so UI identity aligns with the
  persisted docking identity.
- Within a panel, use additional scoped keys under the panel root (e.g. `ui.id(("search", result_id), ...)`).

### 5.3 Hashing: the “stable hash” seam (decision)

imui must not introduce a second hashing scheme.

Decision (v1): `ui.id(key, ...)` delegates to `ElementContext::keyed(...)`, which uses the same deterministic hashing
strategy as the element runtime (FNV-1a 64 via `crates/fret-ui/src/elements/hash.rs`), not `DefaultHasher`.

Design rule:

- When a user writes a loop without `ui.id(...)`, we should prefer debug diagnostics (consistent with the existing
  “unkeyed element list order changed” warning policy).
- In v1, `ImUi` exposes `ui.for_each_unkeyed(items, ...)` as an explicit opt-in to callsite-based identity. In debug
  builds, this will warn if the list order changes between frames.

---

## 6) Ecosystem Integration (Official and Third-Party)

### 6.1 Feature gates: a consistent pattern

To keep ecosystem crates portable and to avoid unnecessary dependencies, adopt:

- `default = []`
- `headless` (model/algorithms only, no UI dependency) where feasible
- `ui` (declarative integration with `fret-ui`)
- `imui` (immediate-mode integration; should imply `ui`)

Rule of thumb (updated by v2 consolidation, 2026-02-03):

- `imui` depends on `fret-authoring` (`UiWriter`), not on a concrete frontend (`fret-imui`).
- Apps can depend on `fret-imui` (or any future frontend) separately.
- “recipes” / opinionated styling integrations depend on `fret-ui-kit` / `fret-ui-shadcn` (A2).

### 6.2 Third-party widget contract

Third-party crates should expose widgets like:

- `pub fn widget(ui: &mut impl fret_authoring::UiWriter<H>, ...)`

This keeps:

- composition easy (any crate can call any other crate’s widget functions),
- dependencies minimal (no need to depend on `UiTree` or runner details),
- compatibility with multi-window (the window is already in `ElementContext`).

Interactive widgets may still choose to depend on `fret-imui` for a shared `Response` type, but the
core embedding surface should remain frontend-agnostic.

#### Third-party checklist (recommended)

1) Keep default features minimal:

- `default = []`
- `headless` for model-only / algorithms
- `ui` for declarative (`fret-ui`) integration
- `imui` for immediate-mode adapters (`fret-authoring` + `UiWriter`) (and it should imply `ui`)

2) Gate the adapter module behind `imui`:

```rust
#[cfg(feature = "imui")]
pub mod imui;
```

3) Prefer the “widget function” contract:

```rust
pub fn widget<H: fret_ui::UiHost>(
    ui: &mut impl fret_authoring::UiWriter<H>,
    /* args */
) {
    /* ... */
}
```

4) Only drop to substrate when necessary:

- for existing declarative builders: `ui.mount(|cx| -> impl IntoIterator<Item=AnyElement> { ... })`
- for advanced mechanisms: `ui.with_cx_mut(|cx| { ... })`
- for retained widgets: `ui.with_cx_mut(|cx| cx.retained_subtree(...))` (feature-gated)

### 6.3 Escape hatches (avoid ecosystem dead-ends)

Some ecosystem surfaces will need deeper integration:

- custom canvas drawing,
- engine viewport surfaces (`ViewportSurface`),
- docking hosts and overlay controllers.
- retained widget subtrees that are not yet expressible as declarative elements (e.g. the node graph editor).

imui must expose “drop to substrate” options:

- `ui.cx_mut()` for direct `ElementContext` authoring,
- `ui.mount(|cx| -> impl IntoIterator<Item=AnyElement>)` as a bridge for existing declarative builders.
- `cx.retained_subtree(...)` to host a retained widget subtree inside the declarative element runtime
  (feature-gated: `fret-ui/unstable-retained-bridge`).

#### Retained subtree hosting (prototype)

Some official ecosystem crates (e.g. `fret-node`) are currently implemented as retained widgets (`UiTreeRetainedExt`)
and produce a retained root `NodeId`. imui must be able to embed these surfaces without forcing a rewrite.

Prototype mechanism:

- `crates/fret-ui` exposes a feature-gated `RetainedSubtree` element (`unstable-retained-bridge`).
- The element mounts an internal `ElementHostWidget` container whose only child is the retained root node returned by
  a factory closure `Fn(&mut UiTree<H>) -> NodeId`.
- The factory closure is invoked only on first mount (or after node GC); otherwise the retained subtree is preserved
  across frames like any other retained widget.

Prototype adapter example (node graph):

```rust
use fret_ui::retained_bridge::UiTreeRetainedExt as _;

fret_imui::imui(cx, |ui| {
    // `build` is invoked only when the subtree is first mounted.
    fret_node::imui::retained_subtree(ui, |ui_tree| {
        // Build a retained subtree and return its root `NodeId`.
        // (Concrete node graph setup is app-specific; see existing demos in `apps/fret-examples`.)
        ui_tree.create_node_retained(fret_node::ui::NodeGraphEditor::new())
    });
})
```

Demo (native):

- `apps/fret-examples/src/imui_node_graph_demo.rs`
- Run via: `cargo run -p fret-demo --bin imui_node_graph_demo --features node-graph-demos`

#### Docking in imui (official adapter)

`fret-docking` ships a small `imui` adapter module so apps don’t need to hand-roll the retained bridge:

```rust
fret_imui::imui(cx, |ui| {
    fret_docking::imui::dock_space(ui, |app, window| {
        // Ensure the dock graph/panels exist for this window and update viewport targets if needed.
        // (App-specific; see `apps/fret-examples/src/imui_editor_proof_demo.rs` for a complete example.)
        let _ = (app, window);
    });
})
```

Stability policy (decision):

- The retained subtree host remains feature-gated (`fret-ui/unstable-retained-bridge`) until the editor-grade proof
  points (docking + multi-window + viewport surfaces) are validated and we have at least one end-to-end demo that uses
  it in anger (see M7 in the TODO tracker).

---

## 7) Docking and Multi-Viewport: What imui Must Not Break

imui is authoring; docking is policy and runtime state. The hard requirements for editor-grade docking are:

- stable identity for panels/tabs (to preserve focus, drag state, and persisted layout),
- multi-window layouts that can degrade to in-window floatings on wasm/mobile,
- embedded engine viewports (`ViewportSurface`) with correct input/focus boundaries.

imui supports this by:

- making stable keys the default story (`ui.id(PanelKey, ...)`),
- keeping “window + root” scoping consistent with existing `ElementContext` / `GlobalElementId` rules,
- not bypassing the existing focus/IME/input routing mechanisms.

### 7.1 Editor-grade proof demo (native)

This workstream includes a minimal proof demo that opens **two OS windows**, each hosting an imui root that embeds:

- a docking host (`fret-docking`), and
- a viewport surface panel (`SceneOp::ViewportSurface`) driven by an app-owned offscreen render target.

Demo:

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- Run via: `cargo run -p fret-demo --bin imui_editor_proof_demo`

Notes:

- The second window is created via `CreateWindowKind::DockRestore { logical_window_id: "aux" }`.
- wasm/mobile fallback (degrade multi-window → in-window floatings) is a planned follow-up (see M7 in the TODO tracker).
  For a native simulation of wasm/mobile constraints, run the same demo with
  `FRET_IMUI_EDITOR_PROOF_SINGLE_WINDOW=1` to disable multi-window tear-off at the capability layer.

---

## 8) A1 vs A2 (Where Things Live)

### A1: `ecosystem/fret-imui` (this workstream)

Contains:

- `ImUi`, `Response`, id scope helpers
- minimal widget/layout foundation (mechanism-level ergonomics)
- zero styling policy by default

### A2: ecosystem “batteries” layers (optional, separate workstreams)

Examples:

- `fret-ui-kit` extension traits (small, reusable policy primitives)
- `fret-imui-shadcn` (or `fret-ui-shadcn` adapters) for opinionated visuals and recipes

Goal:

- allow users to start minimal,
- allow official demos to be batteries-included without infecting core contracts.

---

## 9) Open Questions (need decisions before locking v1)

### 9.1 Decisions locked for v1 (2026-02-02)

1) Hashing for `ui.id(...)`:
   - Use the existing `fret-ui` stable hashing strategy (FNV-1a 64) via `ElementContext::keyed(...)`.
   - Do not introduce `DefaultHasher`-based IDs.

2) Retained widget interop:
   - Keep retained subtree hosting behind `fret-ui/unstable-retained-bridge` until M7 proof points are complete.

3) Persisted dock panel identity:
   - Use `fret-core` `PanelKind` / `PanelKey` string-based identities as the canonical persisted keys (ADR 0013 / 0016).

### 9.2 Remaining open questions

1) Response semantics:
   - Should `clicked()` mean “activated since last frame” or “activated since last call-site evaluation”?
   - Decision (prototype): store edge-triggered flags as **window-scoped transient events**, keyed by
      `(GlobalElementId, u64)` and **clear-on-read**, with a **one-frame delivery buffer** to handle “input between
      frames” (see `crates/fret-ui/src/elements/runtime.rs` and `ecosystem/fret-imui/src/lib.rs`).

2) Styling surface:
   - Should A1 include any theme token helpers, or should it be 100% policy-free?
   - Recommendation: keep A1 policy-free; add styling via A2 extension traits.

3) Layout convenience:
   - Do we model `ui.horizontal/vertical` directly, or map to existing `Row/Column/Flex` elements only?
   - Recommendation: provide `row/column` plus a minimal `grid` wrapper only if needed by early demos.

4) Interop with existing ecosystem components:
   - Do we expose `imui` adapters inside each ecosystem crate, or provide a single “adapters” crate?
   - Decision: per-crate `imui` feature providing small wrappers (keeps ownership local).

5) Naming for sink-based APIs:
   - Do we standardize on `*_build` (align with `fret-ui-kit`) or use a different naming convention?
   - Decision: use `imui_build` (and later `row_build`, `column_build`, etc.) for consistency.

6) Return type for authoring helpers:
   - Decision: prefer `Elements` for authoring-facing “children lists”, and keep `_build` sink variants for
     performance-sensitive and borrow-sensitive code paths.

7) State binding direction:
   - Decision: prefer model-based widgets (`Model<T>`) as the default surface (aligns with the long-term
     “externalized state across frames” direction). A local `&mut T` API may exist as a convenience layer, but it
     should be built on top of the model story, not replace it.

---

## 10) v1 Freeze and v2 Direction (Fearless Refactor)

v1 status (2026-02-03):

- v1 is considered feature-complete enough for internal editor-grade prototyping.
- Land only correctness fixes, portability fixes, and small ergonomic adjustments that do not expand the public surface.

Motivation for v2:

- The ecosystem now has a separate “golden path” authoring surface for styling/layout patches:
  `ui()` / `UiBuilder<T>` (ADR 0175 + related workstreams).
- If we keep expanding both the immediate-mode façade (imui) and the fluent builder chain independently, we will
  end up with duplicated widget APIs and long-term ecosystem maintenance cost.

v2 direction (summary):

- Keep the **same runtime substrate**: imui remains authoring-only; it must still compile down to the declarative
  element taxonomy (no second runtime).
- Converge authoring surfaces: treat the unified builder (`ui()` / `UiBuilder<T>`) as the primary patch vocabulary and
  make imui a thin imperative frontend / bridge so ecosystem widgets can be authored once and consumed from both styles.
- Keep the layering rule: policy and recipes stay in ecosystem (`fret-ui-kit`, `fret-ui-shadcn`, and friends), not in
  `crates/fret-ui`.

Migration policy:

- This repository is not yet public, so we can do a flag-day migration from v1 → v2.
- Still land changes in staged slices (feature gates, demos kept green) so the editor-grade proof points remain a
  reliable regression harness during the refactor.

See:

- `docs/workstreams/imui-authoring-facade-v2.md`
- `docs/workstreams/imui-authoring-facade-v2-todo.md`
