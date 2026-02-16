# ADR 0015: Frame Lifecycle and GPU Submission Order


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

Fret targets a Unity/Unreal-class editor where, in a single frame:

- the engine renders one or more viewports into textures,
- the UI renders chrome and overlays over those viewports,
- multiple OS windows may be presented (tear-off docks),
- the GPU may operate asynchronously (frames in flight).

If the framework does not define **frame phases** and **submission ordering**, we risk subtle
race conditions (sampling stale viewport textures, inconsistent overlays), and later rewrites
once engines integrate more deeply.

References:

- Zed’s “in-flight buffer” lessons (triple buffering / async presentation):
  - https://zed.dev/blog/120fps
- Existing Fret contracts:
  - Display list ordering (ADR 0002 + ADR 0009)
  - Viewports via `RenderTargetId` (ADR 0007)
  - Host-provided GPU context (ADR 0010)
  - Timers/animation/redraw scheduling (`TickId` vs `FrameId`) (ADR 0034)
- Zed/GPUI (non-normative code anchors):
  - per-window draw vs present split (produce scene, then draw it at the platform boundary):
    `repo-ref/zed/crates/gpui/src/window.rs` (`Window::draw`, `Window::present`)
  - scene assembly and ordering/batching by derived order keys:
    `repo-ref/zed/crates/gpui/src/scene.rs`

## Decision

### 1) Define a canonical frame pipeline

Each platform “tick” (event-loop turn) executes this canonical pipeline on the main thread:

1. Drain platform events → produce `fret-core` input events.
2. Apply events to app/UI state (models, widget tree, docking).
3. Build display lists (`Scene`) for each window.
4. Prepare/submit engine viewport rendering for this frame.
5. Submit UI rendering for this frame (consuming `Scene`).
6. Present surfaces for windows that were rendered.
7. Drain app effects (redraw requests, window create/close), bounded fixed-point loop.

This ordering ensures UI overlays can sample the engine’s viewport textures in the same frame.

### 2) Submission ordering is explicit

Within a frame, command buffers must be submitted in this order:

- engine viewport rendering submissions (writes `RenderTargetId` textures),
- UI rendering submission (samples those textures via `SceneOp::ViewportSurface`),
- surface present.

When using a single `wgpu::Queue`, submission order provides correct GPU-side ordering.

### 3) `TickId` and `FrameId` are distinct (avoid “event loop turns” vs “renders” confusion)

For correctness and debugging, Fret treats:

- `TickId`: increments on each event-loop turn (platform events + effects draining),
- `FrameId`: increments only when a render/present actually occurs.

This prevents subtle bugs in multi-window + docking scenarios where some ticks do not render (event-driven mode),
and avoids “echo” problems for multi-window handling that rely on a global frame counter (ImGui-style patterns).

See ADR 0034 for scheduling semantics and how redraw requests are coalesced.

### 4) Initial synchronization constraint (P0 correctness)

For the first stable architecture, Fret assumes:

- a single `wgpu::Queue` is used for both engine and UI submissions within a frame.

If an engine wants multi-queue execution, it must provide explicit synchronization primitives
and an integration adapter; this is deferred until the core contracts are proven.

## Consequences

- Viewport surfaces can be sampled safely in the same frame as they are rendered (mainline case).
- Multi-window presentation remains deterministic and debuggable.
- Renderers can keep using “frames in flight” pools without corrupting in-use buffers.

## Future Work

- Define a formal “engine hook” API:
  - `engine_render(frame_cx) -> Vec<wgpu::CommandBuffer>` or a closure-based submission model.
- Use `FrameId` consistently for tracing and resource pooling (defined in ADR 0034).
- Document multi-queue integration requirements when needed.
