# ImUi Debug Draw Command Metadata v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui exposes draw-list command metadata through `ImDrawCmd`: clip rect, texture reference,
vertex/index offsets, element counts, and optional callbacks. Fret should not mirror callback-heavy
or raw-buffer ownership in the generic scene/runtime contract. This lane keeps the equivalent
bounded and ecosystem-owned: IMUI debug draw lists expose command kind, channel, image reference, and
payload counts for testing, diagnostics, and editor overlay tooling.

## Ownership

- `fret-ui-kit::imui` owns the debug draw command buffer and public introspection API.
- `crates/fret-core` and `crates/fret-render-wgpu` stay unchanged for this lane.
- Renderer callbacks, raw writable vertex/index buffers, and batch-level GPU metadata stay out of
  this lane.

## Must-Be-True Outcomes

- Public command metadata can be inspected without exposing private `DebugDrawCommand` variants.
- Active `ChannelsSplit` state reports summaries in the same channel order that `ChannelsMerge`
  will paint.
- Image-backed commands expose their `ImageId` as optional command metadata.
- Mesh and vertex-level commands report payload counts without claiming renderer batching or exact
  backend draw-call counts.
- The public facade remains compile-smoke covered through `fret-ui-kit::imui` re-exports.

## Non-Goals

- No `AddCallback` / user renderer callback.
- No raw mutable `CmdBuffer`, `VtxBuffer`, or `IdxBuffer` exposure.
- No renderer draw-call batching contract.
- No hit-test-aware debug interaction model yet.
