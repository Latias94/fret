# `repo-ref/` Reference Sources (Pinned for Accurate Reading)

This repository contains local reference checkouts under `repo-ref/` to validate design decisions against proven implementations.
These directories are **not** build dependencies of Fret; they exist to avoid “reading the wrong upstream code”.

## Pin Policy

- Prefer pinning to **the exact crate versions used by this workspace** (see `Cargo.lock`) for `winit` and `wgpu`.
- For fast-moving UI projects (`zed`/`gpui-component`/`godot`), tracking `main`/`master` is acceptable, but record the commit SHA when referencing behavior.

## Pinned Versions (Matches `Cargo.lock`)

- `winit` (crate `0.30.12`): `repo-ref/winit` @ `v0.30.12` (detached) — commit `f6893a4390df`
- `wgpu` (crate `28.0.0`): `repo-ref/wgpu` @ `v28.0.0` (detached) — commit `3f02781bb5a0`

Verify locally:

- `git -C repo-ref/winit describe --tags --always`
- `git -C repo-ref/wgpu describe --tags --always`

Note: `wgpu 28.x` requires Rust `1.92+` (see `rust-toolchain.toml`).

## Recorded HEADs (Fast-Moving References)

These directories may track `main`/`master`/`trunk`. When you cite behavior from them, also cite the commit SHA.
As a baseline, this workspace currently has:

- `repo-ref/zed`: `637ff3425455`
- `repo-ref/gpui-component`: `5bb53ef9ff2b`
- `repo-ref/godot`: `1ea6b0ccff99`
- `repo-ref/dear-imgui-rs`: `a3261f5ed219`

## GPUI / Zed (Rendering, Elements, Text, Scene)

Core “GPUI-style declarative UI” and rendering references:

- Element lifecycle / build-layout-paint-drop:
  - `repo-ref/zed/crates/gpui/src/element.rs`
- Element identity and cross-frame state access patterns:
  - `repo-ref/zed/crates/gpui/src/window.rs` (search `ElementId`, `GlobalElementId`, `with_element_state`)
- Scene composition model (layers + primitives):
  - `repo-ref/zed/crates/gpui/src/scene.rs`
- Text system architecture:
  - `repo-ref/zed/crates/gpui/src/text_system.rs`
  - `repo-ref/zed/crates/gpui/src/platform/linux/text_system.rs` (cosmic-text integration)
- SDF/border/shadow + text quality shader helpers:
  - `repo-ref/zed/crates/gpui/src/platform/blade/shaders.wgsl`

## `gpui-component` (shadcn-inspired components + themes)

Component library patterns (primitives + themes + examples):

- Component crate entry:
  - `repo-ref/gpui-component/crates/ui`
- Theme files (schema-driven):
  - `repo-ref/gpui-component/themes`
- “Storybook”-style demos:
  - `repo-ref/gpui-component/crates/story`
  - `repo-ref/gpui-component/examples`
- High-level overview and positioning:
  - `repo-ref/gpui-component/README.md`

## `fret-ui-precision` (UI design reference: Tailwind-like tokens + recipes)

This is a pinned **design sandbox** (React + Tailwind) used to validate editor UI composition and theming before
committing to framework-level contracts.

What to borrow (conceptually):

- A 2-axis token model:
  - *Theme colors* (surface/text/semantic colors),
  - *UI style* tokens (spacing/radius/shadows/motion knobs).
- “Recipes” (utility compositions) for consistent component chrome (buttons, inspector rows, panels, tabs).

Where to look:

- Theming + tokens architecture notes:
  - `repo-ref/fret-ui-precision/docs/`
- Tailwind token mapping:
  - `repo-ref/fret-ui-precision/tailwind.config.ts`
- Token usage and component recipes:
  - `repo-ref/fret-ui-precision/src/index.css`

## Godot (Editor docking + viewports + frame counters)

Useful for editor workflow and persistence patterns:

- Dock manager / editor docking plumbing:
  - `repo-ref/godot/editor/docks/editor_dock_manager.cpp`
  - `repo-ref/godot/editor/editor_node.cpp`
- Render/idle behavior and “frames drawn” counter:
  - `repo-ref/godot/main/main.cpp` (search `increment_frames_drawn`)
  - `repo-ref/godot/core/config/engine.cpp` (search `frames_drawn`)

## Dear ImGui (Docking + multi-viewport + frame counters)

Useful for multi-window docking UX vocabulary and “global frame counter” patterns:

- Frame lifecycle (`NewFrame`, `FrameCount`, `Render`):
  - `repo-ref/dear-imgui-rs/dear-imgui-sys/third-party/cimgui/imgui/imgui.cpp`
- Winit multi-viewport backend logic (echo suppression via frame count):
  - `repo-ref/dear-imgui-rs/backends/dear-imgui-winit/src/multi_viewport.rs`

## Winit (Event loop, multi-window, DPI)

Matches crate `0.30.12` used by this workspace:

- Event loop and proxy:
  - `repo-ref/winit/src/event_loop.rs`
- Window events and DPI/scale-factor behavior:
  - `repo-ref/winit/src/event.rs`
  - `repo-ref/winit/src/dpi.rs`

## Wgpu (Surface creation, WebGPU/wasm direction)

Matches crate `28.0.0` used by this workspace:

- `Instance::create_surface` and safe/unsafe targets:
  - `repo-ref/wgpu/wgpu/src/api/instance.rs`
- Surface types and presentation constraints:
  - `repo-ref/wgpu/wgpu/src/api/surface.rs`
- Web canvas surface paths (wasm):
  - `repo-ref/wgpu/wgpu-hal/src/gles/web.rs`

## Zed Blog Posts (Design Inspiration)

These are not “pinned code”, but they are useful high-level references when discussing perf and UX constraints:

- https://zed.dev/blog/videogame (frame pacing and UI-on-GPU mindset)
- https://zed.dev/blog/120fps (scheduler + latency discipline)
- https://zed.dev/blog/settings-ui (schema-driven settings UI patterns)
- https://zed.dev/blog/gpui-ownership (app-owned models and borrow-friendly updates)

When citing behavior from these posts in ADRs, include the URL and the date you accessed it.

## Makepad (Portability posture + redraw/caching vocabulary)

Useful as a reference posture for wasm/mobile portability and redraw/caching vocabulary (do not copy APIs):

- Web entrypoint message pump:
  - `repo-ref/makepad/platform/src/os/web/web.rs`
- Incremental redraw / drawlist rebuild skipping:
  - `repo-ref/makepad/draw/src/draw_list_2d.rs` (search `begin_maybe`)
- “Computed tokens” theme approach (inspiration for future derived tokens; not required for P0):
  - `repo-ref/makepad/widgets/src/theme_desktop_dark.rs`

## Vello (Rendering pipeline layering vocabulary)

Useful as a vocabulary reference for “recording” separation, caching, and testability (not a drop-in backend):

- `repo-ref/vello/doc/ARCHITECTURE.md`
