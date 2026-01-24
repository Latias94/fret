# Viewport Panels (Tier A): Engine Viewports, Video, and GPU-Heavy Widgets

This document explains the "Tier A" integration path for embedding **engine-owned GPU content** into
Fret's UI tree, without turning `fret-ui` into a GPU framework.

Tier A is the recommended choice for:

- Game/engine viewports (scene rendering, debug overlays).
- Video playback / NLE-style panels (scrubbing, thumbnails, GPU filters).
- Any component that wants substantial custom rendering work (custom shaders, bespoke pipelines).

## Contracts (what stays stable)

- UI embedding uses a stable handle: `RenderTargetId` (ADR 0007).
- UI draws a `ViewportSurface` that samples the target by id:
  - declarative leaf: `ViewportSurfaceProps` / `cx.viewport_surface(...)`
  - convenience wrapper with input forwarding: `fret-ui-kit::viewport_surface_panel(...)`
- The runner provides an engine render hook:
  - `WinitAppDriver::record_engine_frame(...) -> EngineFrameUpdate` (ADR 0038).
- Input is forwarded from UI to the app via:
  - `Effect::ViewportInput(ViewportInputEvent)` (ADR 0007 / ADR 0098),
  - `ViewportInputEvent` carries `target`, `cursor_px` (window logical px), `uv`, `target_px`, plus
    button/modifiers and pointer identity (ADR 0147).

## End-to-end shape (recommended)

1. **App owns the engine subsystem** (or a "viewport renderer" for a specific panel).
2. Each frame, the driver records engine command buffers in `record_engine_frame`.
3. The engine renders into a `wgpu::Texture` you own (offscreen).
4. Register/update that texture as a `RenderTargetId` in the UI renderer.
5. The UI tree embeds the target via `ViewportSurface`.
6. Pointer/wheel events are forwarded into the engine via `Effect::ViewportInput`.

## Where to start in code

- Minimal engine viewport proof (clears a texture and embeds it as a viewport surface):
  - `apps/fret-examples/src/plot3d_demo.rs`
- More complex integration + overlays:
  - `apps/fret-examples/src/gizmo3d_demo.rs`
- Gizmo + viewport integration guide:
  - `docs/gizmo-viewport-integration.md`
- Engine hook + submission coordination:
  - `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
  - `crates/fret-launch/src/runner/common.rs` (`WinitAppDriver`, `EngineFrameUpdate`)
- Helper to allocate/resize `RenderTargetId`-backed textures:
  - `crates/fret-launch/src/runner/viewport_target.rs` (`ViewportRenderTarget`)
- Declarative viewport embedding + input forwarding helper:
  - `ecosystem/fret-ui-kit/src/declarative/viewport_surface.rs`

## Render target lifecycle (what you must do)

You own:

- the `wgpu::Texture` (and its `TextureView`) used for the viewport output,
- resize decisions (choose pixel size; handle DPI),
- and the engine commands that write into the texture.

Fret owns:

- compositing that texture into the UI tree (sampling it in `ViewportSurface` draws),
- and ordering guarantees (engine commands are submitted before UI draws sample the target).

In `record_engine_frame`, the driver receives `&WgpuContext` and `&mut Renderer`, so you can:

- `renderer.register_render_target(RenderTargetDescriptor { ... })` once to obtain a `RenderTargetId`
- `renderer.update_render_target(id, RenderTargetDescriptor { ... })` when the view/size changes
- `renderer.unregister_render_target(id)` when the target is no longer needed

Note: on desktop, a simple `wgpu::TextureFormat::Bgra8UnormSrgb` target is usually fine.

## Helper: `ViewportRenderTarget` / `ViewportRenderTargetWithDepth`

To reduce boilerplate for "allocate/resize + register/update", use:

- `ViewportRenderTarget` (color only)
- `ViewportRenderTargetWithDepth` (color + depth)

Advanced configuration:

- `.with_usage(...)` and `.with_view_formats(...)` let you opt into extra texture usage flags or view formats
  (useful for video/postprocess panels that need additional bindings).
- `.ensure_size_owned_view(...)` / `.ensure_size_owned_views(...)` return cloned `TextureView`s for callers that
  want to avoid borrowing across additional state reads.

## Input forwarding (best practice)

If your viewport panel needs interaction, embed it with `fret-ui-kit::viewport_surface_panel(...)`.
That helper:

- emits a `ViewportSurface` leaf,
- computes `ViewportMapping` (fit + target px size),
- forwards pointer + wheel as `Effect::ViewportInput` with consistent uv/px mapping,
- handles pointer capture so drags remain stable.

In your driver, implement:

- `fn viewport_input(&mut self, app: &mut App, event: ViewportInputEvent)`

and route events to the engine subsystem that owns `event.target`.

When consuming viewport input for editor-style tooling, treat `ViewportInputKind::PointerCancel`
as an explicit teardown signal for pointer capture (e.g. clear hot/active tool state).

## Video playback guidance

Tier A options:

1) **Streaming image updates (preferred for video decode)**:
   - Use `Effect::ImageUpdateNv12/I420/Rgba8` to upload decoded frames (ADR 0121/0123/0126).
   - UI composes the image like any other texture.

2) **RenderTargetId (preferred for GPU postprocess / compositor-heavy video UIs)**:
   - Decode however you want (CPU/GPU), do postprocess in your own command buffers,
   - present the result via `RenderTargetId + ViewportSurface`.

## Common pitfalls

- **Don’t pass window-space pointer coordinates into the engine.** Always use `ViewportInputEvent.uv` or
  `ViewportInputEvent.target_px` (viewport-local in render-target pixels).
- **Be explicit about color space** for render targets (`RenderTargetColorSpace::Srgb` vs `Linear`).
- **Don’t leak wgpu into components.** Keep engine rendering in the driver/engine subsystem (Tier A),
  keep UI policy in ecosystem components (Tier B).
