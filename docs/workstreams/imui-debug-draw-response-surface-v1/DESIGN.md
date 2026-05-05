# ImUi Debug Draw Response Surface v1

Status: Closed narrow follow-on
Last updated: 2026-05-05

`imui_debug_draw_basics` proved that app authors can draw Dear ImGui-style debug geometry, but the
helper still behaved like a paint-only call: authors had to capture summaries inside the draw
closure, and the canvas could not participate in normal IMUI response queries.

This lane adds a Fret-shaped response surface:

- `debug_draw(...)` / `debug_draw_with_options(...)` return `DebugDrawResponse`.
- The response carries source-level `DebugDrawListSummary` and command summaries after channel
  merge.
- `DebugDrawInteractionOptions` can opt the canvas into a pressable item response.
- The default remains non-interactive so paint-only overlays do not start consuming events.

## Ownership

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs` owns the helper API and lowering.
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` owns the facade return type.
- `apps/fret-cookbook/examples/imui_debug_draw_basics.rs` owns the public teaching proof.
- The existing cookbook diagnostics script owns first-open smoke evidence.

## Must-Be-True Outcomes

- Default debug draw remains a plain `Canvas`.
- Opt-in interaction wraps the canvas in a `Pressable` and returns `ResponseExt`-compatible state.
- Summary data is available after the draw call, not only inside the draw closure.
- No renderer callback, raw vertex/index buffer, or backend draw-call contract is introduced.

## Non-Goals

- No per-shape hit testing.
- No Dear ImGui `AddCallback` equivalent.
- No public raw `VtxBuffer` / `IdxBuffer` mutation.
- No pixel-perfect diagnostics gate.
