# `fret-render`

`fret-render` is the GPU renderer for the Fret workspace.

It is responsible for:

- Driving a `wgpu` device/queue and rendering `fret-core::scene::Scene` recordings.
- Text rendering (font fallback, shaping, atlas management) and related performance snapshots.
- SVG rasterization/caching surfaces used by the UI runtime and higher layers.
- Managing render targets and upload registries for images/SVG/text resources.

This crate is intentionally **not** a platform/windowing layer. Surfaces and swapchains are owned by
runner/platform crates (e.g. winit/web runners) which provide the appropriate `wgpu::SurfaceTarget`
and presentation loop.

## Module ownership map

- `src/renderer/`: the primary renderer implementation and draw orchestration.
- `src/targets.rs`: render target descriptors + registries.
- `src/surface.rs`: surface state helpers (presentation glue, but still renderer-owned).
- `src/images.rs`: image registry + upload helpers.
- `src/svg/`: SVG rasterization and caching.
- `src/text/`: text shaping/wrapping glue and font family configuration.
- `src/perf_store.rs`: frame-level performance sample storage.
- `src/capabilities.rs`: adapter capability detection and surfaced capability flags.
- `src/viewport_overlay/`: immediate-mode overlays rendered in viewport space (debug helpers).

## Public surface

Prefer importing from `fret_render`’s re-exports in `src/lib.rs` (e.g. `Renderer`, registries, perf
snapshot structs). Internal modules are subject to re-grouping as part of the bottom-up fearless
refactor.

## Refactor gates

- Tests: `cargo nextest run -p fret-render`
- Formatting: `cargo fmt`
