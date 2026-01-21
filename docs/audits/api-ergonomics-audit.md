# API Ergonomics Audit (Draft)

Date: 2026-01-21  
Scope: authoring ergonomics for app and ecosystem component code

This audit focuses on whether Fret’s public surfaces are ergonomic enough for:

- application authors (fast iteration, high code density),
- ecosystem authors (third-party crates composing cleanly),
- “foreign UI” integration (reusing other UI ecosystems without collapsing Fret’s contracts).

Repository principle reminders:

- Kernel vs ecosystem boundaries must remain intact (see `docs/adr/0066-fret-ui-runtime-contract-surface.md`).
- Prefer a single “golden path” authoring surface in ecosystem crates; keep kernel contracts stable and minimal.

## Executive Summary

Fret already has the core primitives required for a GPUI-style authoring experience:

- Declarative per-frame element tree + stable identity (`ElementContext::{scope,keyed,named}`, `GlobalElementId`).
- Externalized cross-frame element state (`with_state_for`).
- Explicit model/global observation with invalidation strength (ADR 0051).
- Ecosystem-level unified patch/builder surface (`UiExt::ui()` / `UiBuilder`, ADR 0175 Proposed).

However, ergonomics are currently limited by:

- pervasive `Vec<AnyElement>` requirements (children closures and component constructors),
- inconsistent use of the unified builder surface in examples/templates,
- lack of a single “default” composition path that third-party crates can implement once and reuse everywhere.

## Evidence (Current State)

### `Vec<AnyElement>` friction hotspots

- `ElementContext` builder-style helpers use `FnOnce -> Vec<AnyElement>` in many places.
  - Evidence: `crates/fret-ui/src/elements/cx.rs` has many `FnOnce(&mut Self) -> Vec<AnyElement>` occurrences.
- Many `fret-ui-shadcn` components accept `children: Vec<AnyElement>` in constructors and setters.
  - Evidence: `ecosystem/fret-ui-shadcn/src` contains many `children: Vec<AnyElement>` fields and `new(children: Vec<AnyElement>)` constructors.

This forces application code and third-party crates into a “vector-first” authoring style, reducing code density and composability with iterators/arrays.

### Unified builder surface exists (but must become the default)

`fret-ui-kit` provides a patch aggregator and a single authoring entrypoint:

- `UiExt::ui()` and `UiBuilder<T>::into_element(cx)`
  - Evidence: `ecosystem/fret-ui-kit/src/ui_builder.rs`

`fret-ui-shadcn` already wires many components into this builder surface via macros:

- Evidence: `ecosystem/fret-ui-shadcn/src/ui_ext/support.rs`

The remaining gap is adoption and completeness: examples and templates must make this the “one way”.

### Model observation ergonomics exist (component-layer sugar)

`ModelWatchExt` provides “observe + read” sugar while preserving explicit invalidation semantics:

- Evidence: `ecosystem/fret-ui-kit/src/declarative/model_watch.rs`

## What’s Already Good (Keep It)

- `ElementContext::{scope,keyed,named}` provides deterministic identity without a diff engine.
- `for_each_keyed` is the correct direction for list identity and safety.
- Explicit invalidation strength (Paint/Layout/HitTest) is appropriate for editor-grade correctness, as long as:
  - the default path is ergonomic (`ModelWatchExt`, `UiExt`),
  - “power user” paths remain available (`observe_model`, direct runtime props).
- `fret-ui-shadcn` taxonomy + `fret-ui-kit` policy infrastructure is the right ecosystem layering.

## Main Ergonomics Risks

### R1) “Vector-first authoring” reduces composability

Symptoms:

- Frequent `vec![...]` boilerplate.
- Harder to stream/compose child lists (iterators), conditionally append children, etc.
- Third-party crates tend to fork local helpers instead of sharing a universal pattern.

### R2) Two authoring dialects in app code

Symptoms:

- Apps mix:
  - direct runtime props (`ContainerProps`, `LayoutStyle`, manual `Theme::global(...)`),
  - per-component `refine_style/refine_layout`,
  - builder surface `.ui()...`.

This fragments teaching, makes examples harder to scan, and increases the cost of third-party component integration.

### R3) “Foreign UI mixing” can collapse contracts

Attempting to mix other UI runtimes (Iced/Egui/etc) inside the same tree tends to require deep semantic bridges:

- layout, hit testing, event routing, focus/IME, a11y, invalidation.

This is high-risk and likely to bloat kernel contracts. The recommended approach is “isolated embedding”.

## Recommendations (Prioritized)

### P0 — Make the golden path the default (docs + examples)

- Update representative examples to use:
  - `UiExt::ui()` (builder surface),
  - `ModelWatchExt` (observe+read sugar),
  - `stack::{hstack,vstack}` (avoid direct runtime props unless needed).
- Ensure `fret-kit` templates and `fretboard new` outputs follow this style.

Acceptance: new users can build a non-trivial UI without touching `LayoutStyle` directly.

### P1 — Reduce `Vec<AnyElement>` requirements via `IntoIterator`

Change high-frequency APIs to accept `IntoIterator<Item = AnyElement>`:

- `ElementContext` subtree builders (`container`, `row`, `column`, etc).
- `fret-ui-shadcn` constructors/setters that currently take `Vec<AnyElement>`.

This should be source-compatible for callers passing `Vec`, while enabling:

- arrays/slices,
- iterators,
- incremental composition without intermediate allocations.

### P2 — Standardize “third-party component integration contract”

Define and document a small set of traits a third-party component should implement once:

- `UiPatchTarget` (+ optional `UiSupportsChrome/Layout`) for patch aggregation,
- `UiIntoElement` for rendering,
- optionally `RenderOnce` for deeper kernel-level composition (if needed).

Make this guidance part of:

- component author docs,
- a checklist in `docs/component-author-guide.md` or a dedicated ecosystem audit doc.

### P3 — Formalize “foreign UI embedding” as isolated surfaces

Add an ADR (or update existing integration docs) stating:

- Supported: embed foreign UI as a render target surface + event forwarding.
- Not supported: mixing foreign widget trees as first-class Fret elements (no shared a11y/focus/IME semantics).

This keeps kernel contracts stable and prevents accidental scope creep.

## Checklist for New Ecosystem Components

**Authoring surface**

- Expose a `ui()` entrypoint via `UiPatchTarget` and `UiIntoElement`.
- Prefer accepting children as `impl IntoIterator<Item = AnyElement>`.
- Avoid leaking `Theme` resolution into app code; resolve tokens in `into_element(cx)`.

**State and invalidation**

- Use `ModelWatchExt` for “observe + read” in component code.
- Use `Invalidation::Paint` by default; escalate to `Layout` only when layout changes.
- Ensure dynamic lists use `for_each_keyed` (or explicit keys) for stable identity.

**Boundaries**

- Keep policy (dismissal, focus trap/restore, hover intent) in `fret-ui-kit` / shadcn layer.
- Keep `crates/fret-ui` as mechanism-only.

## Follow-up Work Items

- Promote ADR 0175 from Proposed → Accepted once example migration proves the UX.
- Implement P1 changes (`IntoIterator`) and run `cargo fmt` + tests.

