# imui Ecosystem Facade v1 Performance Guide

Status: Draft (living guide)
Last updated: 2026-02-06

This guide documents practical performance rules for the immediate-mode ecosystem facade
(`fret_ui_kit::imui`) while keeping `fret-imui` policy-light.

Related:

- `docs/workstreams/imui-ecosystem-facade-v1.md`
- `docs/workstreams/imui-ecosystem-facade-v1-todo.md`
- `docs/adr/0042-virtualization-and-large-lists.md`
- `docs/adr/0070-virtualization-contract.md`

---

## 1) Scope

Applies to:

- facade wrappers in `ecosystem/fret-ui-kit/src/imui.rs`,
- adapter modules that expose egui/imgui-like ergonomics,
- demo/proof surfaces that exercise immediate-mode interaction loops.

Does not replace crate-level profiling or renderer-specific tuning.

---

## 2) Core Rules (v1)

1) Stable identity first

- Always use keyed identity for dynamic/reorderable collections (`ui.push_id(...)`, `ui.keyed(...)`,
  model-keyed rows/items).
- Avoid index-only identity for collections that can insert/remove/reorder.

2) Keep wrappers allocation-light

- Avoid per-frame `String` building and temporary `Vec` materialization in hot paths.
- Reuse option structs/state buffers where practical.
- Prefer borrowed data (`&str`, slices, iterators) over newly allocated owned containers.

3) Keep interaction state single-sourced

- `ResponseExt` edge signals stay transient (clear-on-read).
- Long-lived sessions (drag/resize/selection) stay in element-local/canonical component state.
- Do not duplicate complex widget state machines in facade wrappers.

4) Use two-frame geometry stabilization intentionally

- For popup/context-menu/floating alignment, use last-frame bounds semantics.
- Treat first-frame geometry misses as expected and avoid compensating allocations/retries per frame.

5) Bound work with virtualization/caching

- Large lists/tables/trees must use the runtime virtualization contract.
- Keep cache boundaries explicit; avoid forcing full subtree rerender from localized interactions.

---

## 3) Recommended Patterns

- Prefer canonical adapters (Tier 1) before primitive fallback wrappers.
- Keep wrapper APIs thin: map signals and identity, delegate behavior to canonical policies.
- When adding a new wrapper, include one behavior-focused nextest and avoid demo-only validation.
- For floating/window overlays, compose existing layer primitives (`floating_layer`, overlay controller)
  instead of adding parallel z-order registries.

---

## 4) Common Perf Pitfalls

- Recomputing large option lists every frame for select/menu wrappers.
- Unstable keys that invalidate cached layout/interaction state.
- Building nested closures that capture large owned data repeatedly in immediate loops.
- Mixing unrelated responsibilities in wrapper code (layout policy + interaction policy + data transforms).

---

## 5) Validation Workflow

1) Behavior gate (fast)

- `cargo nextest run -p fret-imui -p fret-ui-kit`

2) Compile gate (cross-target smoke)

- `cargo check -p fret-authoring -p fret-imui -p fret-ui-kit --features imui --target wasm32-unknown-unknown`

3) Interaction regression gate (scripted)

- `cargo run -p fretboard -- diag run --script tools/diag-scripts/imui-float-window-drag-resize-context-menu.json`

4) Demo sanity gate (manual)

- `cargo run -p fret-demo --bin imui_response_signals_demo`
- `cargo run -p fret-demo --bin imui_floating_windows_demo`

---

## 6) Promotion Gate for Heavier Work

Before adding heavier facade functionality (for example popup-select state machines or generalized
window promotion), require all of the following:

- a clear ownership decision (runtime vs ui-kit policy),
- at least one nextest regression gate,
- one `fretboard diag` scripted path when interaction choreography is non-trivial,
- explicit fallback behavior for non-native/missing capability targets.

---

## 7) Relationship to Text Ecosystem

Text editing remains owned by the text/code-editor ecosystem workstream. The imui facade should
integrate via adapters and must not fork text-editing interaction engines.
