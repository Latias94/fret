# UI Ergonomics & Interop Notes (Iced / GPUI Comparison)

This note is a design-oriented snapshot focused on two questions:

1. Why does Fret authoring sometimes feel “more complex” than frameworks like `iced`?
2. What is the most realistic way to interop with other UI ecosystems without breaking Fret’s
   editor-grade goals (multi-window, docking, viewports, GPU-layered rendering)?

This document is not an ADR. If we agree on a direction, we should promote the chosen contract
surface(s) into an ADR (and keep it narrow and hard-to-change).

## Background work (portable) and "heavy app" adapters

Fret is intentionally main-thread oriented for UI/runtime mutation. The scalable pattern is:

- background work runs off the UI lane (or best-effort cooperatively on wasm),
- results return as **data-only** messages,
- the runner is woken to the next driver boundary where inboxes are drained and redraw is scheduled.

This surface is locked in `docs/adr/0199-execution-and-concurrency-surface-v1.md` and aligns with the golden-path guidance in `docs/adr/0112-golden-path-ui-app-driver-and-pipelines.md`.

### Heavy app recipe: Tokio thread + inbox + wake

For editor-grade apps (indexing, LSP, asset IO, compilation), a realistic adapter story is:

1. Run a Tokio runtime on a dedicated background thread (or use an existing runtime handle).
2. Send results into an inbox (pure data).
3. Call `wake()` so the runner reaches the next driver boundary promptly.
4. Drain inboxes on the UI thread, apply updates to models/globals, and request redraw.

This avoids forcing Tokio on small apps while giving large apps an explicit, debuggable concurrency boundary.

## Mental Model: Three Things That Often Get Mixed

When users say “the API feels complex”, it usually comes from these layers bleeding together:

1. **Mechanism runtime** (`crates/fret-ui`): tree, layout, hit-testing, events, effects, IDs.
2. **Policy / authoring surface** (`ecosystem/fret-ui-kit`, `ecosystem/fret-ui-shadcn`, `fret-kit`):
   default padding/row height, focus policy, dismiss semantics, hover intent, tokens → styles.
3. **Embedding surfaces** (viewport panels, retained-widget bridge): how to host “foreign” systems.

To keep the core contract stable, the ergonomics work should focus on (2) while (1) stays minimal.

## Comparing `iced` vs `gpui` vs Fret (High-Level)

### `iced`

- **User code feels simple** because it is opinionated: `Message`, `update`, `view`, subscriptions.
- **Widget tree is declarative** but the runtime is effectively retained/diffed: widgets are rebuilt
  in user code, but the framework keeps per-widget state and diffs the tree.
- **Interop** tends to be “all-in” (use iced widgets), or embed foreign rendering via custom widgets.

### `gpui`

- **Authoring is immediate-mode-ish**: rebuild element tree every frame, store state outside the
  tree (`Model`, `State`, etc.), and use identity keys to preserve locality.
- **Interop** is typically done by hosting external render surfaces (engine/canvas/video) and
  translating input.

### Fret (current + target)

- Current: a retained tree prototype exists (`UiTree`), with a long-term goal of a GPUI-style
  “rebuild each frame + cross-frame state externalized” authoring model.
- Contract philosophy: lock hard-to-change runtime contracts first, then iterate policy-heavy
  component surfaces in `ecosystem/`.

## Interop Recommendation: Tiered Embedding (Not “Same Tree” Mixing)

Trying to directly “mix” two full UI runtimes in the same widget tree tends to fail on:
focus, accessibility semantics, input capture, text IME, layout, and animation timing.

Instead, a practical interop strategy is:

- **Tier A (recommended): Viewport surface embedding**
  - Foreign UI renders into an app-owned `RenderTargetId` (offscreen texture).
  - Fret hosts that texture via `ViewportSurfaceProps` (`SceneOp::ViewportSurface`).
  - Pointer/wheel input is forwarded as `Effect::ViewportInput` using a `ViewportMapping`.
  - This matches engine viewports, code editors, node graphs, video surfaces, etc.
- **Tier B: Retained widget bridge (feature-gated)**
  - Use `Widget` trait to embed policy-heavy widgets while migrating retained components.
  - Keep this unstable/feature-gated; avoid making it the primary end-user authoring surface.
- **Tier C (avoid): “Same-tree” interoperability**
  - Only consider if we are willing to define a unified focus/semantics/text model across runtimes.

### Tier A code sketch (engine-style embedding)

The existing `ViewportRenderTarget` helper (`crates/fret-launch/src/runner/viewport_target.rs`) is
the intended glue for Tier A:

```rust
// 1) Maintain an offscreen target
let (id, view) = state.target.ensure_size(context, renderer, desired_size, Some("external-ui"));

// 2) Render your foreign system (iced/egui/etc) into `view`
// iced_render_into_view(&mut iced_state, &view, ...);

// 3) In the Fret UI tree, embed it
cx.viewport_surface_props(ViewportSurfaceProps {
    target: id,
    target_px_size: desired_size,
    fit: ViewportFit::Contain,
    ..ViewportSurfaceProps::new(id)
});

// 4) Forward input by translating pointer/wheel into `Effect::ViewportInput`
// See `ecosystem/fret-ui-kit/src/declarative/viewport_surface.rs`.
```

This keeps contracts clean: foreign runtime owns its layout/state; Fret owns docking, windowing,
semantics boundaries, and compositing.

## Ergonomics Recommendation: Keep Core Minimal, Add “Authoring Sugar” Where It Belongs

Fret already has an ecosystem authoring surface (`UiBuilder`, `.ui()`, style/layout refinements).
The remaining high-impact ergonomics improvements tend to be:

1. **Reduce “children container” boilerplate**
   - Prefer accepting `IntoIterator<Item = AnyElement>` in high-frequency APIs.
   - Provide a tiny authoring helper for iterator-heavy lists (examples: `.elements()` for `Vec`, `.elements_owned()` for `Elements`).
2. **Make root rendering accept iterables**
   - Root render fns should accept iterable children, not force `Vec`.
3. **Bias toward key-based identity**
   - Make “keyed list” helpers the default for dynamic collections (reorder/remove/insert).
4. **Prefer `fret-ui-kit` for policy-heavy defaults**
   - Hover intent, focus trap/restore, dismiss rules, and theme token resolution should stay in
     ecosystem crates.

## “Todo app” authoring: what to measure

If we want to evaluate ergonomics concretely, measure:

- How many times the user has to write `vec![...]` / `.collect::<Vec<_>>()`.
- How much state wiring is required (`Model` + observation + invalidation).
- How easy it is to embed a foreign viewport panel (Tier A) next to normal UI.

The current `apps/fret-examples/src/todo_demo.rs` is a good baseline because it already exercises:
input, buttons, tabs, list rendering, and style tokens.

