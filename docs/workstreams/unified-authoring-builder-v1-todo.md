# Unified Authoring Builder Surface (v1) TODO

This is a tracking document for `docs/workstreams/unified-authoring-builder-v1.md`.

## MVP0 (Builder Vocabulary)

- [x] Forward full `ChromeRefinement` vocabulary onto `UiBuilder<T>` (`UiSupportsChrome`)
- [x] Forward full `LayoutRefinement` vocabulary onto `UiBuilder<T>` (`UiSupportsLayout`)
- [x] Add a compile-level smoke test to lock the vocabulary surface

## MVP1 (shadcn Surface Coverage)

- [x] Cover `accordion::composable` with `ui()` (root + patch-only subvalues)
- [x] Cover `breadcrumb::primitives` with `ui()` (patch-only roots + full leaf elements)
- [x] Extend `ecosystem/fret-ui-shadcn/tests/ui_builder_smoke.rs` to compile nested surfaces

## MVP2 (Patch-Only Terminals / Ergonomics)

- [x] Add `ui_builder_ext` terminals for `breadcrumb::primitives::{Breadcrumb,BreadcrumbList,BreadcrumbItem}`:
  - target ergonomics: `*.ui().into_element(cx, |cx| ...)`
  - keep the existing `into_element(cx, children)` signature authoritative

## MVP3 (Semantics Decorators / Late Landing)

Goal:

- Reduce cases where authoring code must call `.into_element(cx)` early purely to attach diagnostics/a11y
  decorators (`test_id`, role), by allowing those to be expressed on the fluent builder path and applied at the
  terminal.

Backlog:

- [x] Expose `test_id(...)` on the fluent authoring path (applied during `.into_element(cx)`).
  - Implemented as `UiElementTestIdExt` (no early landing required for patch targets).
  - Evidence: `ecosystem/fret-ui-kit/src/declarative/semantics.rs`
- [x] Expose minimal semantics decorators on the fluent builder path (applied during `.into_element(cx)`).
  - Implemented on `UiBuilder<T>` as late-landing semantics decoration:
    `test_id(...)`, `a11y_role(...)`, `a11y_label(...)`.
  - Evidence: `ecosystem/fret-ui-kit/src/ui_builder.rs`
- [x] Add compile-only coverage for builder-level decorators.
  - Evidence:
    - `ecosystem/fret-ui-kit/src/ui.rs`
    - `ecosystem/fret-ui-shadcn/tests/ui_builder_smoke.rs`

## Ongoing (Audit / Guardrails)

- [ ] Expand compile coverage in `ecosystem/fret-ui-shadcn/tests/ui_builder_smoke.rs` as new nested public
      surfaces are introduced
- [ ] Decide and document when a nested “subcomponent value” should:
  - be patch-only (`ui().build()` only), vs
  - expose `ui().into_element(cx)` (if signature matches and it is a user-facing leaf element)
