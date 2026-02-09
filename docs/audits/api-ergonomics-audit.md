# API Ergonomics Audit (Draft)

Date: 2026-02-09  
Scope: authoring ergonomics for app and ecosystem component code

This audit evaluates whether Fret's public authoring surfaces are ergonomic enough for:

- application authors (fast iteration, high code density),
- ecosystem authors (third-party crates composing cleanly),
- "foreign UI" integration (embedding other UI systems without collapsing Fret's contracts).

Repository principle reminders:

- Kernel vs ecosystem boundaries must remain intact (see `docs/adr/0066-fret-ui-runtime-contract-surface.md`).
- Prefer a single "golden path" authoring surface in ecosystem crates; keep kernel contracts stable and minimal.

## Executive Summary

Fret already has the core primitives required for a GPUI-style authoring experience:

- Declarative per-frame element tree + stable identity (`ElementContext::{scope,keyed,named}`, `GlobalElementId`).
- Externalized cross-frame element state (`with_state_for`, `with_state` patterns).
- Explicit model/global observation with invalidation strength (ADR 0051).
- Ecosystem-level composition surface (`UiExt::ui()` / `UiBuilder`) that can converge into a single authoring dialect.

The main ergonomics limitations today:

- "vector-first authoring" friction (`Vec<AnyElement>` everywhere).
- two competing authoring dialects (direct runtime props vs `UiBuilder` patch surface).
- small, repeated boilerplate patterns that fragment across crates (`test_id`, common semantics stamping, etc.).

## Current Wins (Keep These)

- Stable identity is explicit and predictable (no hidden diff engine).
- Invalidation semantics are explicit and precise (good for editor-grade correctness).
- Policy stays in ecosystem: `fret-ui` is mechanisms/contracts, not a component library.

## Friction Hotspots

### H1) `Vec<AnyElement>` as the default children boundary

Symptoms:

- Frequent `vec![...]` and `collect::<Vec<_>>()` boilerplate.
- Hard to stream/compose children with iterators.
- Third-party crates often implement local helper traits/macros instead of reusing a shared pattern.

Recommended direction:

- Prefer iterator-friendly boundaries in constructors and setters:
  - `children: impl IntoIterator<Item = AnyElement>`
  - store `Vec<AnyElement>` internally, but accept iterators at the boundary.
- Prefer `Elements` (owned wrapper) or `impl IntoIterator` for authoring-facing returns instead of exposing raw `Vec`.

### H2) View function signatures and `fn` pointer wiring

Many "golden path" apps are wired through a `fn` pointer signature:

- `type ViewFn<S> = for<'a> fn(&mut ElementContext<'a, App>, &mut S) -> Elements`
- `pub type ViewElements = Elements`
- Evidence: `ecosystem/fret-bootstrap/src/ui_app_driver.rs`

This choice has real ergonomics implications:

- `-> impl Trait` return types are not usable inside a `type ViewFn = fn(...) -> ...` alias; `fn` pointers require a concrete, nameable return type.
- `fn` pointers are intentionally friendly to hotpatch boundaries and keep monomorphization under control.

Practical recommendation:

- Keep the default driver view signature returning a concrete `Elements`.
- If we want closure capture ergonomics, provide it as an optional "advanced driver" surface (boxed dyn closure) instead of changing the golden path.

### H3) Micro-boilerplate: semantics test ids

The codebase often stamps automation identifiers like:

```rust
node = node.attach_semantics(SemanticsDecoration::default().test_id("..."));
```

This is pure authoring noise in most cases.

Recent improvement (small win):

- `AnyElement::test_id(...)` exists as a single-call helper, keeping the mechanism in `fret-ui` and avoiding ecosystem dependency requirements.
- Evidence: `crates/fret-ui/src/element.rs`

## Comparisons (What to Learn, What to Avoid)

This section is intentionally focused on what an app author experiences and how an ecosystem author plugs in.

| Framework | Authoring surface | View return type | Composition feel | Takeaways for Fret |
|---|---|---|---|---|
| egui | immediate-mode closures | no return (mutates `Ui`) | very low boilerplate, state is explicit in user structs | consider "no-return" builder patterns for some layers; keep Fret's retained contracts |
| iced | MVU (`update/view/message`) | `Element<Message>` | typed messages + declarative composition | borrow typed routing and "one default entry point" without adopting widget model wholesale |
| Dioxus / Leptos | React-like + macro DSL | `Element` / view nodes | high density DSL, hooks for state | macros can be an optional layer, but kernel should stay macro-free |
| Slint | dedicated markup language | compiled components | strong tooling, static layouts | good for product UI; less aligned with editor-grade docking/viewport focus |
| GPUI (Zed) | builder-style declarative tree rebuild | typically `impl IntoElement` (per-method) | dense authoring, state externalized | long-term direction is similar; focus on converging the golden path and reducing vector boilerplate |

## Recommendations (Small Changes First)

### P0) Make the golden path the default (docs + examples)

- Update demos/templates to consistently use `UiExt::ui()` / `UiBuilder` rather than mixing two dialects.
- Keep kernel APIs stable; move "what authors should do" into ecosystem docs and examples.

### P1) Reduce vector-first boundaries (incremental, no flag day)

- In ecosystem components, accept iterator-friendly children everywhere.
- Provide common "children collection" helpers where it improves readability (avoid bespoke helpers per crate).

### P1.1) Standardize "test id" stamping (done)

- Prefer `AnyElement::test_id(...)` for diagnostics/automation-only ids.
- Avoid local ad-hoc extension traits in leaf crates unless layering forces it.

### P2) Optional advanced driver: captured closures (explicit tradeoffs)

If needed for app ergonomics (e.g. captured environment), consider an opt-in driver:

- `Box<dyn for<'a> FnMut(&mut ElementContext<'a, App>, &mut State) -> Elements + 'static>`

Tradeoffs:

- dynamic dispatch + allocation,
- potentially more complicated hotpatch story,
- but improves ergonomics for certain application architectures.

## Evidence (Pointers)

- Authoring conversion boundary and `Elements`: `crates/fret-ui/src/element.rs`
- View wiring signature: `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- Unified builder surface: `ecosystem/fret-ui-kit/src/ui_builder.rs`

