# ADR 0330: Retained Runtime Internal and Compat Surface

Status: Accepted

## Context

Fret has always had two related but different UI concepts:

- a retained `UiTree` runtime substrate that owns layout, hit testing, focus, paint state,
  invalidation, and renderer-facing orchestration;
- a declarative authoring surface (`Render`, `RenderOnce`, `IntoElement`, `ElementContext`,
  `AnyElement`) that rebuilds the element tree while externalizing cross-frame state by stable
  identity.

ADR 0066 already states that retained widgets (`UiTree` + `Widget`) are an internal hosting
mechanism, not the component ecosystem authoring model. The remaining risk was that
`crates/fret-ui` still exported `Widget`, `EventCx`, `LayoutCx`, `PaintCx`, and sibling retained
contexts from the crate root by default. That made retained authoring look like the stable public
API even after the retained bridge and several retained component paths were deleted.

This also created an architectural conflict for low-level surfaces such as `fret-node`: its legacy
retained canvas/editor implementation used to need runtime contexts while the rest of the node graph
surface was already declarative-first and should not teach retained widget authoring. That
compatibility island has now been deleted; the supported node graph authoring surface is
binding/controller/declarative composition.

The GPUI/Zed reference split supports the same direction: `repo-ref/zed/crates/gpui/src/element.rs`
exposes `Render`, `RenderOnce`, `Element`, `IntoElement`, and `AnyElement` as the authoring model,
while component ecosystems compose above that instead of exposing an arbitrary retained widget
trait as the default component API.

## Decision

1. Retained runtime stays.
   - `UiTree`, retained nodes, retained invalidation, layout, hit-test, focus, paint, and prepaint
     mechanisms remain valid inside `crates/fret-ui`.
   - Frame Pipeline v2 and diagnostics may continue to store runtime state on retained nodes where
     that is the current mechanism owner.

2. Retained widget authoring is not the default public authoring surface.
   - `Widget`, `EventCx`, `CommandCx`, `CommandAvailabilityCx`, `LayoutCx`, `PrepaintCx`,
     `PaintCx`, and `SemanticsCx` must not be exported from the default `fret-ui` root surface.
   - Those names may be exported only behind an explicit compatibility feature or another
     intentionally named low-level adapter surface.

3. Declarative authoring is the ecosystem contract.
   - Component and recipe crates author through `Render` / `RenderOnce` / `IntoElement`,
     `ElementContext`, headless engines, pure policy kernels, and explicit mechanism helpers.
   - New reusable UI primitives must not require downstream crates to implement retained
     `Widget`.

4. Compatibility islands are explicit and delete-planned.
   - Existing low-level legacy surfaces may keep using retained contexts only when a workstream
     records why a declarative/binding-first adapter is not yet sufficient.
   - Such crates must opt into `fret-ui/compat-retained-widgets` explicitly.
   - Compatibility islands must be source-policy gated and tracked by workstreams, not normalized
     as the public ecosystem API.
   - `fret-node` no longer has a retained canvas compatibility island or a
     `compat-retained-canvas` feature.

5. Shared mechanism contract types can remain public.
   - `Invalidation` and `CommandAvailability` are still public mechanism data types because they
     are consumed by declarative hosts and command/invalidating mechanisms outside retained widget
     authoring.
   - If future audits prove these types are still too coupled to `widget.rs`, move their
     definitions to a neutral mechanism module before changing their public path.

## Consequences

- The default app/component story now matches the documented declarative-only ecosystem boundary.
- Retained runtime work can continue without forcing every ecosystem crate to treat `Widget` as a
  stable authoring contract.
- `fret-node` now uses binding/controller/declarative composition for supported UI authoring; the
  legacy retained canvas feature and node-local retained widget island were deleted.
- Any new component crate or first-party example that needs retained widget contexts must justify
  that as a low-level adapter, not as ordinary component authoring.

## Implementation Notes

- `crates/fret-ui/src/lib.rs` keeps `CommandAvailability` and `Invalidation` on the default root
  surface and gates `Widget` plus retained contexts behind `compat-retained-widgets`.
- `ecosystem/fret-node/Cargo.toml` no longer defines `compat-retained-canvas`, and
  `ecosystem/fret-node/src/ui/canvas` now contains only supported declarative-surface support
  modules such as geometry, route math, spatial lookup, and resize handle types.
- `ecosystem/fret-node/src/surface_policy_tests.rs` locks the retained exit by checking that the
  feature, raw retained transport queue, and retained widget source island stay removed.
- `docs/workstreams/retained-public-surface-exit-v1/` tracks the public-surface exit and the next
  adapter migration slices.
- The broader six-cut convergence plan is tracked by
  `docs/workstreams/fearless-architecture-convergence-v1/`.

## References

- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0028-declarative-elements-and-element-state.md`
- `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
- `docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md`
- `docs/shadcn-declarative-progress.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`
- `docs/workstreams/retained-public-surface-exit-v1/DESIGN.md`
- `crates/fret-ui/src/lib.rs`
- `ecosystem/fret-node/Cargo.toml`
- `repo-ref/zed/crates/gpui/src/element.rs`
