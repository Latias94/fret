# `repo-ref/` Reference Sources (Pinned for Accurate Reading)

This repository contains local reference checkouts under `repo-ref/` to validate design decisions against proven implementations.
These directories are **not** build dependencies of Fret; they exist to avoid “reading the wrong upstream code”.

## `third_party/` (Versioned build dependencies)

Unlike `repo-ref/`, paths under `third_party/` are intended for versioned, reproducible build inputs.

- `third_party/lucide` is a git submodule used by icon generation scripts.
- `third_party/radix-icons` is a git submodule used by icon generation scripts.
- Keep it pinned and update intentionally (submodule bump + regenerated outputs).

## Important: `repo-ref/` is local state (not committed)

`repo-ref/` is intentionally ignored by git in this repository. That means:

- Fresh clones will not include any upstream checkouts under `repo-ref/`.
- Many docs (especially `docs/audits/*`) reference file paths under `repo-ref/` as a reading aid.
- To reproduce those paths locally, use `tools/fetch_repo_refs.py` (recommended) or clone the refs manually.

## Pin Policy

- `repo-ref/` is a curated set of upstream sources that we actively reference in ADRs and design reviews.
- For fast-moving upstreams, tracking `main`/`master` is acceptable, but record the commit SHA when referencing behavior.
- For crate dependencies (e.g. `winit`, `wgpu`), prefer reading the **exact version** recorded in `Cargo.lock`.

## Dependency Sources (Crates in `Cargo.lock`)

This repository generally avoids vendoring crate dependency sources under `repo-ref/` (for example, most crates like `wgpu` are read via their version in `Cargo.lock` when needed).
However, we may keep a pinned checkout for certain upstream dependencies (for example `repo-ref/winit`) when it is useful for design review and debugging.

When you need to cite or inspect dependency behavior:

- Find the exact version in `Cargo.lock`.
- Use `cargo vendor` and inspect `vendor/<crate>/src/...` (or fetch that crate version from crates.io).

## Recorded HEADs (Fast-Moving References)

These directories may track `main`/`master`/`trunk`. When you cite behavior from them, also cite the commit SHA.
As a baseline, the project tracks the following reference SHAs (local checkouts may include only a subset):

- `repo-ref/aria-practices`: `84b921a0`
- `repo-ref/cmdk`: `dd2250e`
- `repo-ref/dear-imgui-rs`: `5768e5a`
- `repo-ref/echarts`: `09198192b`
- `repo-ref/egui_plot`: `ed3d2c2`
- `repo-ref/floating-ui`: `0681dbb6`
- `repo-ref/fret-ui-precision`: `c52a90d`
- `repo-ref/gpui-component`: `c78cec71`
- `repo-ref/icons`: `112af91`
- `repo-ref/imgui`: `396b33d0d`
- `repo-ref/implot`: `81b8b19`
- `repo-ref/implot3d`: `5981bc5`
- `repo-ref/lucide`: `0c4ac91b`
- Radix UI Primitives: <https://github.com/radix-ui/primitives> (`90751370`)
- `repo-ref/tailwindcss`: `9720692e`
- `repo-ref/ui` (shadcn/ui): `d07a7af8`
- `repo-ref/vello`: `cc2dd70e`
- `repo-ref/virtualizer`: `f9c72f7a`
- `repo-ref/zed`: `f4aad4bb`
- `repo-ref/makepad`: `b40b9af49`
- `repo-ref/winit`: `da622006`

### Bootstrap script (recommended)

If you want the two refs most frequently used by goldens + audits:

- `python3 tools/fetch_repo_refs.py` (defaults to `ui` + `primitives`)
- `python3 tools/fetch_repo_refs.py --ui-only`
- `python3 tools/fetch_repo_refs.py --primitives-only`

## Optional checkouts (not always present)

Some references are useful but are not guaranteed to be present in every workspace checkout:

- transform-gizmo: clone into `repo-ref/transform-gizmo` when needed (used by gizmo ADRs/audits).
- Vello: clone into `repo-ref/vello` when needed.
- (Historical) TanStack Virtual: we no longer keep a checkout by default; use `repo-ref/virtualizer` as the primary reference.

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
- IME / composition handling patterns (platform-specific; reference only):
  - `repo-ref/zed/crates/gpui/src/platform.rs` (search `InputHandler`)
  - `repo-ref/zed/crates/gpui/src/platform/windows/events.rs` (search `handle_ime_composition`)
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

## shadcn/ui (Component recipes + variants vocabulary)

This is the upstream reference for shadcn component structure, variants, and interaction affordances.
We do **not** copy the React implementation, but we do borrow:

- component decomposition (primitives vs composites),
- variants vocabulary (`variant`/`size`/`intent`),
- UX details (focus ring, disabled states, loading patterns).

Where to look:

- Component recipes live in the v4 registry sources:
  - `repo-ref/ui/apps/v4/registry/` (e.g. `*/ui/button.tsx`, `*/ui/input.tsx`, `*/ui/popover.tsx`)

## TailwindCSS (Token scales + naming conventions)

This is the upstream reference for token scales and naming conventions (spacing, radii, typography, colors).
Fret will not implement Tailwind’s runtime/class parser; instead, we use the vocabulary to define typed tokens and
compose component “recipes” (see MVP 45).

Where to look:

- `repo-ref/tailwindcss/packages/`
- `repo-ref/tailwindcss/crates/` (for how Tailwind models tokens/scales internally)

## Floating UI (Popover/menu placement vocabulary)

This is the upstream reference for “floating element” placement (menus, popovers, tooltips). We borrow vocabulary
and the collision/flip/shift mental model; we do not copy the JS runtime.

Where to look:

- `repo-ref/floating-ui/packages/` (algorithms and docs sources)

## `virtualizer` (Rust virtualization engine reference)

This is a Rust, UI-agnostic virtualization engine inspired by TanStack Virtual. It is useful as a
code-level reference when validating our `VirtualList` algorithms and invariants:

- `repo-ref/virtualizer`

## ImGui / Dear ImGui (Docking + multi-viewport vocabulary)

Useful for multi-window docking UX vocabulary and “global frame counter” patterns:

- Upstream ImGui docking/multi-viewport code:
  - `repo-ref/imgui/` (see `docs/` and docking branches upstream; this checkout may track master)
- Frame lifecycle (`NewFrame`, `FrameCount`, `Render`):
  - `repo-ref/dear-imgui-rs/dear-imgui-sys/third-party/cimgui/imgui/imgui.cpp`
- Winit multi-viewport backend logic (echo suppression via frame count):
  - `repo-ref/dear-imgui-rs/backends/dear-imgui-winit/src/multi_viewport.rs`

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
