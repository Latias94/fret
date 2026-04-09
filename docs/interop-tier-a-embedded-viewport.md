# Tier A Interop Cookbook: Embedded Viewport Surface

Tier A is the recommended interop strategy in Fret: **host**, don’t “mix runtimes in one tree”.

Instead of trying to unify focus/IME/semantics/scheduling across UI frameworks, Tier A embeds a
foreign system as an isolated render surface:

- Foreign UI renders into an offscreen `RenderTargetId`.
- Fret presents that target via `ViewportSurface`.
- Input is forwarded explicitly as `ViewportInputEvent` (surface UV + target pixels).

This matches editor/engine workflows: viewports, video panels, node graphs, code editors, etc.

## Run the demo

Recommended (cookbook example):

```bash
cargo run -p fretboard-dev -- dev native --example embedded_viewport_basics
```

Maintainer harness (legacy demo):

```bash
cargo run -p fret-demo --bin embedded_viewport_demo
```

The demo shows:

- render target allocation + resizing (preset buttons),
- a viewport panel embedding the target,
- input forwarding with visible feedback (click counter + color changes).

## Minimal steps (what you implement in an app)

### 1) Store an embedded surface in your window state

Use `EmbeddedViewportSurface` in your window state and pick an initial pixel size:

- `embedded_viewport::EmbeddedViewportSurface::new(format, color_space, initial_px_size)`

### 2) Install the global viewport input hook

Install `embedded_viewport::handle_viewport_input` as the app-wide viewport input hook.

If you use the builder hook-preserving path, the extension helper composes directly there:

- `fret::FretApp::new("my-app").window("my-app", (960.0, 720.0)).view_with_hooks::<MyView>(|d| d.drive_embedded_viewport())?`

### 3) Record rendering into the offscreen target each frame

The embedded helper owns the boilerplate:

- ensure the target exists at the desired size,
- create an encoder,
- push the command buffer into the engine update.

For “render it yourself” apps, implement `EmbeddedViewportRecord` and wire the recorder:

- `driver.record_engine_frame(embedded_viewport::record_engine_frame::<S>)`

Or use `drive_embedded_viewport()` which installs both input + recorder.

### 4) Embed the surface in your UI

Render the panel in your UI tree:

- `surface.panel(cx, EmbeddedViewportPanelProps { forward_input: true, .. })`

## Common pitfalls (learn these early)

- **Don’t try to mix full runtimes in one tree**: focus/IME/semantics/input capture become undefined fast.
- **Keep the boundary explicit**: Tier A is “a surface + forwarded input”, not shared widget ownership.
- **Make resize behavior intentional**: decide who owns target size (fixed presets, panel-measured, or window-driven).
- **Forward input only when you mean it**: `forward_input: true` is the opt-in.

## References

- Design note: `docs/ui-ergonomics-and-interop.md`
- Implementation: `ecosystem/fret/src/interop/embedded_viewport.rs`
