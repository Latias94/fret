# Immediate-Mode Authoring Facade ("imui") v2 - Fearless Refactor Plan

Status: Draft (workstream note; not an ADR)
Last updated: 2026-02-03

This document proposes a fearless refactor of the `imui` authoring surface after v1.

The motivating idea: **imui should remain an authoring frontend**, not a second runtime, and it should not grow into a
parallel ecosystem surface that competes with the unified patch chain (`ui()` / `UiBuilder<T>`) introduced by ADR 0175.

Tracking:

- TODO tracker: `docs/workstreams/imui-authoring-facade-v2-todo.md`
- v1 baseline: `docs/workstreams/imui-authoring-facade-v1.md`
- Unified patch chain ADR: `docs/adr/0175-unified-authoring-builder-surface-v1.md`
- Unified builder workstream: `docs/workstreams/unified-authoring-builder-v1.md`
- Fluent builder ergonomics audit: `docs/workstreams/authoring-ergonomics-fluent-builder.md`
- Architecture baseline: `docs/architecture.md`
- Docking parity notes: `docs/workstreams/docking-multiwindow-imgui-parity.md`

Decision snapshot (2026-02-03):

- `UiWriter` lives in `ecosystem/fret-authoring` (tiny, policy-light, no cycles).
- Official ecosystem `imui` feature gates depend on `fret-authoring` (not `fret-imui`).
- Canonical widget rule (official crates): one source-of-truth implementation; adapters (imui/ui-kit/shadcn) delegate.
- Stability policy: keep `UiWriter` stable once depended on; expect churn in bridge utilities and imui ergonomics.
- Token/preset policy: keep patch vocabulary + generic scales/presets in `fret-ui-kit`; keep shadcn-aligned tokens and recipes in `fret-ui-shadcn`; keep app-specific tokens in the app layer.

---

## 1) Motivation

v1 `imui` proved that immediate-mode ergonomics can coexist with Fret’s retained substrate and contracts.

However, the ecosystem is also converging on a separate “golden path” authoring interface for styling/layout patches:

- `ui()` / `UiBuilder<T>` (ADR 0175)

If we continue expanding `imui` *and* expanding `ui()` as separate authoring worlds, we will incur:

- duplicated widget APIs (two ways to author the same component),
- inconsistent patch vocabulary (style/layout shorthands drift),
- long-term maintenance cost across official ecosystem crates.

v2’s goal is to keep the benefits of immediate-mode control flow while making ecosystem components **authorable once**
and consumable from multiple authoring frontends.

---

## 2) Invariants (Do Not Break)

These are the “hard-to-change” seams from v1 that remain non-negotiable:

- **No second runtime**: imui must compile down to the declarative element taxonomy mounted into `UiTree` (ADR 0028).
- **Stable identity**: keyed identity must remain the canonical story (`ui.id(...)`); no new hashing scheme.
- **Input/focus correctness**: focus, capture, IME, overlays, and multi-root z-order keep working the same way.
- **Multi-window + multi-root**: windows remain first-class; wasm/mobile can degrade to in-window floatings.
- **Docking + viewport surfaces**: docking policies and embedded engine viewports must retain correct boundaries.
- **Layering**: keep policy and recipes in ecosystem crates (ADR 0066); avoid leaking “defaults” into `crates/fret-ui`.

---

## 3) Design Goals

1) **Single authoritative implementation per widget**

- A widget should have one source-of-truth implementation.
- Any additional authoring entry points should be thin adapters, not parallel implementations.

2) **Single patch vocabulary**

- The unified builder (`ui()` / `UiBuilder<T>`) remains the primary patch vocabulary for chrome/layout.
- imui should not duplicate the full patch surface by re-exporting a second copy of “tailwind-ish” shorthands.

3) **Keep imui policy-light**

- `fret-imui` should remain a small façade over `ElementContext`.
- Integration with token scales and recipes should stay in ecosystem layers (`fret-ui-kit`, `fret-ui-shadcn`).

4) **Preserve escape hatches**

- `cx_mut()` for advanced mechanisms.
- `mount(...)` for embedding existing declarative builders.
- retained subtree hosting remains feature-gated and explicit.

---

## 4) Proposed Shape (Bikesheddable)

### 4.1 A single “writer” contract for immediate-style composition

Introduce a small ecosystem-level “writer” trait (name bikesheddable, e.g. `UiWriter`) that captures the minimal needs
of immediate-style UI:

- access to an underlying `ElementContext`,
- an ordered sink for `AnyElement` nodes,
- stable identity scoping (`keyed(key, ...)`) via the runtime’s canonical hashing rules.

Then:

- `ImUi` implements this writer trait.
- other authoring frontends may also implement it (e.g. helper wrappers that want imperative child emission while still
  using the unified patch chain).

This pushes third-party widget signatures toward a single, stable surface:

- `fn widget<H: UiHost>(ui: &mut impl UiWriter<H>, ...) -> Response` (interactive widgets), or
- `fn into_element(cx: &mut ElementContext<'_, H>, ...) -> AnyElement` (render-only building blocks).

### 4.2 Bridge `ui()` / `UiBuilder<T>` into imui without coupling crates

To avoid pulling `fret-ui-kit` into `fret-imui`, keep the dependency direction:

- `fret-imui` depends on `fret-ui` (mechanisms),
- `fret-ui-kit` depends on `fret-ui` and can optionally add an `imui` integration module.

Concretely:

- add a `fret-ui-kit` `imui` feature that provides extension traits on `UiWriter`:
  - “render this `UiBuilder<T>` into the current output list”
  - “construct common layout nodes in a patchable way, but author children imperatively”

This makes it possible to write immediate-mode control flow while still using the unified patch chain for styling.

Status (2026-02-03):

- Implemented in `fret_ui_kit::imui` behind the `imui` feature.
- Entry point: `UiWriterUiKitExt::add_ui(...)`.

Example (imui):

```rust
use fret_ui_kit::imui::UiWriterUiKitExt as _;

let builder = fret_ui_kit::ui::text(ui.cx_mut(), "Hello").text_sm().px_2();
ui.add_ui(builder);
```

### 4.3 Demos remain the regression harness

The editor-grade proof demo (`imui_editor_proof_demo`) must remain runnable and should be treated as a contract test for
the v2 refactor:

- docking host embedding,
- viewport surfaces,
- multi-window + fallback modes,
- pointer capture / focus boundaries.

---

## 5) Migration Plan (Internal “Flag Day”, staged for stability)

Because the repository is not yet public, we can do a breaking “flag day” migration for v2.

To keep the refactor safe and reviewable:

1) Land v2 APIs behind feature gates (temporary).
2) Migrate official demos + the minimal set of ecosystem adapters needed for editor-grade flows.
3) Delete v1 APIs and update docs once the new demos are green and the key tests pass.

---

## 6) Acceptance Criteria (v2 “done”)

- `cargo nextest run` passes for relevant crates (`fret-imui`, docking, and any new bridge crate).
- Native demos still run:
  - `cargo run -p fret-demo --bin imui_editor_proof_demo`
- wasm coverage at least includes a compile-only smoke harness for the authoring surface.
- Evidence: `apps/fret-imui-wasm-smoke` (run `cargo check -p fret-imui-wasm-smoke --target wasm32-unknown-unknown`).
- Official ecosystem crates do not ship duplicated authoring APIs without a clear reason.

---

## 7) Golden Path and Gotchas

### 7.1 Recommended authoring shape (v2)

- Use `fret_imui::imui(cx, |ui| ...)` (or `_build` variants) for immediate-mode control flow and `Response` signals.
- Use `fret_ui_kit::ui::*` / `UiBuilder<T>` for chrome/layout patches, and emit builders via
  `fret_ui_kit::imui::UiWriterUiKitExt::add_ui(...)`.
- In official ecosystem crates, prefer accepting `&mut impl UiWriter<H>` instead of a concrete `ImUi`.

### 7.2 When to drop to `cx_mut()` / `with_cx_mut(...)`

Use the escape hatch when you need substrate-level mechanisms that are not (and should not be) expressed as ui-kit
patch vocabulary:

- Docking host embedding (`fret_docking::imui::dock_space_with(...)`).
- Viewport surfaces / embedded engine views (render targets, capture boundaries).
- Canvas-like custom drawing elements (low-level paint/layout wiring).
- Retained subtree interop (explicit, feature-gated).

### 7.3 Common pitfalls

- Avoid borrow conflicts by constructing `UiBuilder<T>` first, then calling `add_ui(builder)` (don’t keep `&mut cx` alive).
- Always use keyed identity (`ui.id(...)` / `ui.keyed(...)`) for dynamic collections that can reorder.

---

## 8) Open Questions

1) How far should `Response` expand in v2?

- keep v1 minimal (`clicked/changed/hovered/pressed/focused/rect`), or
- add more editor-grade signals (drag started/ended, context menu requests, etc.).

2) Where should token/preset helpers live long-term?

- kit vs shadcn vs app (see `IMUI2-bridge-021`).
