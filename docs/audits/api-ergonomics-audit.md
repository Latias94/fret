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

ADR 0189 formalizes the policy (and the kernel already supports the mechanism). The recommended ecosystem entrypoint is `fret-kit`’s embedded viewport helper surface.

Supported surface (ecosystem-level):

- `EmbeddedViewportSurface` owned by window state
- `EmbeddedViewportForeignUi` (object-safe boundary) registered per window via `set_foreign_ui(...)`
- single-call driver wiring: `drive_embedded_viewport_foreign()`

This enables “reuse other ecosystems” without collapsing focus/IME/a11y contracts.

Policy statement (keep):

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

## Comparative References (Iced, GPUI)

This section is a quick “design vocabulary alignment” to make future ADR discussion concrete.

### Iced (Elm-style MVU, task/subscription driven)

Key authoring traits:

- State as a plain Rust value, updated by messages (`update(&mut State, Message) -> Task<Message>`).
- UI described as an element tree built each frame (`view(&State) -> Element<Message>`).
- Message routing is typed and implicit (widgets produce `Message` directly).
- No explicit invalidation model in user code; the runtime schedules redraws.

Evidence anchors (reference checkout):

- `F:\SourceCodes\Rust\fret\repo-ref\iced\src\application.rs` (`iced::application(...) -> Application`).
- `F:\SourceCodes\Rust\fret\repo-ref\iced\src\lib.rs` pocket guide (“update/view/message” narrative).

Ergonomic takeaway for Fret: typed message routing and “single default entry point” can be borrowed
without adopting Iced’s widget model or layout approach.

### GPUI (hybrid immediate/retained; entities + render each frame)

Key authoring traits:

- “Views” are entities implementing `Render`, called once per frame to rebuild the element tree.
- Element state is owned by the framework (entity storage), not by the tree itself.
- Context objects are the primary interface surface (AppContext / WindowContext).

Evidence anchors (reference checkout):

- `F:\SourceCodes\Rust\fret\repo-ref\zed\crates\gpui\README.md` (“three registers” overview).

Ergonomic takeaway for Fret: Fret’s long-term direction already resembles GPUI; the main gap is
authoring surface polish (density, defaults, unified patterns), not core capability.

## Prototype: `fret_kit::mvu` (Typed Commands Without String Parsing)

To test how much of the Iced “typed message” ergonomics we can adopt without changing kernel
contracts, this worktree adds a small ecosystem-level MVU helper:

- `ecosystem/fret-kit/src/mvu.rs`
  - `Program` trait (`init/update/view`) with `State` + `Message`.
  - `MessageRouter<Message>` that allocates per-frame `CommandId` and resolves it in the driver hook.
  - `fret_kit::mvu::app::<P>(...)` mirrors the existing `fret_kit::app(...)` golden path.

Why this shape:

- The golden-path driver uses `fn` pointers for hotpatch predictability, so a trait-based program is
  the simplest way to keep “no captured closures in wiring” while still improving ergonomics.

Limitations (current):

- This does not remove the model observation/invalidation responsibilities yet (it only eliminates
  stringly `CommandId` parsing and prefix handling in app code).
- `MessageRouter` is frame-local; it is intentionally not a stable command registry.

## Code Size Snapshot (TODO demo family)

Line counts (worktree):

- `apps/fret-examples/src/todo_demo.rs`: 536
- `apps/fret-examples/src/todo_mvu_demo.rs`: 518
- `apps/fret-examples/src/todo_mvu_interop_demo.rs`: 313
- `apps/fret-examples/src/todo_interop_demo.rs`: 512
- `apps/fret-examples/src/todo_interop_kit_demo.rs`: 306

Interpretation:

- The MVU prototype currently reduces “command routing complexity” more than raw LOC.
- The largest LOC savings still come from using `fret-kit` golden-path hooks instead of writing a
  full custom driver, which supports the “single default path” recommendation.

Recent ergonomic improvement:

- `fret-kit` now includes `interop::embedded_viewport` to reduce “isolated embedding” boilerplate
  (publish target id as models + global viewport input filtering + surface panel helper).
- `interop::embedded_viewport::{EmbeddedViewportUiAppDriverExt, EmbeddedViewportMvuUiAppDriverExt}`
  further reduce wiring boilerplate (install input + frame recording in one call).
