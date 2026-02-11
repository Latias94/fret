# Unified Authoring Builder Surface (v1) Workstream

This workstream tracks the incremental rollout of the ecosystem-owned authoring builder surface
proposed in `docs/adr/0160-unified-authoring-builder-surface-v1.md`.

The goal is to make “write UI in Rust” feel closer to GPUI-style ergonomics while preserving Fret’s
mechanism/policy boundaries (ADR 0066) and token-first styling semantics.

## Scope

- A single fluent entry point: `value.ui()`
- A single patch aggregator: `UiBuilder<T>` accumulating `UiPatch { chrome, layout }`
- A consistent vocabulary:
  - chrome/styling via `ChromeRefinement`
  - layout via `LayoutRefinement`
- Patch-only roots still offer a single terminal: `ui().into_element(cx, ...)` (ADR 0160)

## Status (Current Snapshot)

### 1) UiBuilder vocabulary forwarding

`UiBuilder<T>` now forwards the full public vocabulary of:

- `ChromeRefinement` (`ecosystem/fret-ui-kit/src/style/chrome.rs`)
- `LayoutRefinement` + `LayoutRefinement` shorthands (`ecosystem/fret-ui-kit/src/style/layout.rs`,
  `ecosystem/fret-ui-kit/src/style/layout_shorthands.rs`)

Evidence:

- `ecosystem/fret-ui-kit/src/ui_builder.rs`
  - dedicated impl blocks: `impl<T: UiSupportsChrome> UiBuilder<T>` / `impl<T: UiSupportsLayout> UiBuilder<T>`
  - compile-level smoke coverage: `ui_builder_forwards_full_vocabulary_smoke`

### 2) shadcn coverage: nested public surfaces

Two nested public surfaces are now covered by `ui()`:

- `accordion::composable`
  - `AccordionRoot`: layout-only + `ui().into_element(cx)`
  - `AccordionItem/Trigger/Content`: chrome+layout patch-only (supports `ui().build()`)
- `breadcrumb::primitives`
  - `Breadcrumb/BreadcrumbList/BreadcrumbItem`: chrome+layout patch-only
  - `BreadcrumbLink/BreadcrumbPage/BreadcrumbSeparator/BreadcrumbEllipsis`: chrome+layout + `ui().into_element(cx)`

Evidence:

- `ecosystem/fret-ui-shadcn/src/ui_ext/composites.rs`
- `ecosystem/fret-ui-shadcn/src/ui_ext/misc.rs`
- `ecosystem/fret-ui-shadcn/tests/ui_builder_smoke.rs` (`ui_builder_nested_surfaces_compile`)

## Design Notes / Conventions

### Vocabulary rules

- `UiBuilder` method names mirror the underlying refinement methods 1:1.
- The builder does not invent policy; it only aggregates patches.
- Token-first intent should remain the default (prefer `Space/Radius/MetricRef`-backed helpers).

### Patch-only rules

Use patch-only when a type:

- does not have a public `into_element(cx)` terminal (e.g. needs extra parameters), or
- is an internal/composed value passed into another root renderer (e.g. composable accordion items).

When a patch-only type is intended to be rendered directly and has a public terminal that needs extra
arguments, add a `ui_builder_ext` terminal so authoring can stay one-chain:

```rust
SomePatchOnlyType::new()
    .ui()
    .px_2()
    .into_element(cx, /* extra args */)
```

## Known Gaps (Tracked in TODO)

- None tracked in this workstream currently; see `docs/workstreams/unified-authoring-builder-v1-todo.md`.
