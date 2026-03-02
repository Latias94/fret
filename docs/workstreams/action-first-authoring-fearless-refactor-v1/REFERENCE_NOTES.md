# Action-First Authoring + View Runtime (Fearless Refactor v1) — Reference Notes

Last updated: 2026-03-01

This file is a quick pointer index to relevant upstream and in-tree references.

---

## Upstream (non-normative)

- Zed/GPUI action dispatch and key routing:
  - `repo-ref/zed/crates/gpui/src/action.rs`
  - `repo-ref/zed/crates/gpui/src/key_dispatch.rs`
  - `repo-ref/zed/crates/gpui/src/window.rs` (dispatch + availability queries)

- gpui-component authoring ergonomics:
  - `repo-ref/gpui-component/crates/ui/src/styled.rs`
  - `repo-ref/gpui-component/crates/ui/src/virtual_list.rs`

---

## In-tree (authoritative)

### Authoring + state helpers

- Authoring paradigm ADR: `docs/adr/0223-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`
- Selector + deps rails:
  - `ecosystem/fret-selector/src/lib.rs`
  - `ecosystem/fret-selector/src/ui.rs`
- Query:
  - `ecosystem/fret-query/src/lib.rs`
- imui authoring facade:
  - `docs/workstreams/imui-authoring-facade-v2.md`
  - `ecosystem/fret-imui`
  - `ecosystem/fret-authoring`

### UI IR + caching

- Element IR:
  - `crates/fret-ui/src/element.rs` (`AnyElement`, `ElementKind`)
  - `crates/fret-ui/src/elements/cx.rs` (`ElementContext`, keyed identity)
- Cache roots:
  - ADR: `docs/adr/0213-cache-roots-and-cached-subtree-semantics-v1.md`
  - Workstream: `docs/workstreams/gpui-parity-refactor.md`

### Diagnostics + scripts

- ADR: `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
- Inspect + selectors guide: `docs/debugging-ui-with-inspector-and-scripts.md`
- Protocol selector vocabulary:
  - `crates/fret-diag-protocol/src/lib.rs` (`UiSelectorV1`)

### GenUI (data-driven specs)

- Workstream: `docs/workstreams/genui-json-render-v1.md`
- Core:
  - `ecosystem/fret-genui-core/src/spec.rs`
  - `ecosystem/fret-genui-core/src/render.rs`
  - `ecosystem/fret-genui-core/src/actions.rs`

