# Crate audit (L0) — `fret-render-wgpu`

## Crate

- Name: `fret-render-wgpu`
- Path: `crates/fret-render-wgpu`
- Owners / adjacent crates: `fret-render-core`, `fret-core`, `fret-render` (facade), runner/platform crates that host a `wgpu::Surface`
- Current “layer”: renderer backend (wgpu/WebGPU)

## 1) Purpose (what this crate *is*)

- The `wgpu`-based renderer backend for Fret: scene encoding, GPU resource management, text shaping integration, SVG rasterization/upload, render planning and passes.
- Owns GPU-facing implementation details (pipelines, bind groups, textures, staging) and therefore must remain isolated from core “contract/mechanism” crates.

Evidence anchors:

- `crates/fret-render-wgpu/src/lib.rs`
- `crates/fret-render-wgpu/src/renderer/mod.rs`

## 2) Public contract surface

- Key exports / stable types (observed):
  - `Renderer`, `RenderSceneParams`, `SurfaceState`, target/texture registries (`RenderTargetRegistry`, `ImageRegistry`), SVG helpers (`SvgRenderer`, caches), perf snapshots/stores.
  - `WgpuContext` (async adapter/device/queue acquisition), and env-based backend selection via `FRET_WGPU_BACKEND`.
- “Accidental” exports to consider removing (L0 hypothesis):
  - Some internal-leaning helper types may be re-exported from `lib.rs`; consider tightening the facade once call sites are mapped.

Evidence anchors:

- `crates/fret-render-wgpu/src/lib.rs`

## 3) Dependency posture

- Backend coupling risks: intentionally couples to `wgpu` and a large graphics/text stack (`parley/fontique`, `swash`, `lyon`, `resvg/usvg`, `glam`, `etagere`).
- Compile-time / link-time cost is likely significant; keep public surface small and keep modules discoverable to support incremental refactors.
- JSON use (`serde_json`) exists (likely for dumps/fixtures); keep its usage localized to diagnostics/testing paths where possible.

Evidence anchors:

- `crates/fret-render-wgpu/Cargo.toml`
- `pwsh -NoProfile -File tools/audit_crate.ps1 -Crate fret-render-wgpu`

## 4) Module ownership map (internal seams)

- `renderer/` — render planning, passes, GPU resources, shader sources, render-scene plumbing
  - Files: `crates/fret-render-wgpu/src/renderer/*`
- `text/` — shaping + wrapping + fallback policy and glyph/cache management
  - Files: `crates/fret-render-wgpu/src/text/mod.rs`, `crates/fret-render-wgpu/src/text/wrapper.rs`, `crates/fret-render-wgpu/src/text/parley_shaper.rs`
- `svg*` — SVG rasterization and caches
  - Files: `crates/fret-render-wgpu/src/svg.rs`, `crates/fret-render-wgpu/src/svg_cache.rs`, `crates/fret-render-wgpu/src/svg_cache/*` (if present)
- `surface/targets/images` — surface/config, render targets, image upload/storage
  - Files: `crates/fret-render-wgpu/src/surface.rs`, `crates/fret-render-wgpu/src/targets.rs`, `crates/fret-render-wgpu/src/images.rs`

## 5) Refactor hazards (what can regress easily)

- Shader source drift / WebGPU portability
  - Failure mode: WGSL fails to parse/validate or diverges across platforms/backends.
  - Existing gates: `renderer::tests::{shaders_parse_as_wgsl, shaders_validate_for_webgpu}`.
  - Missing gate to add (optional): if shader strings get split into external `.wgsl` assets, add a “load + validate all” gate for the asset set.
- Render-plan correctness and scissor/mask mapping logic
  - Failure mode: visual artifacts, incorrect clipping/masking, incorrect intermediate reuse.
  - Existing gates: unit tests in `renderer/render_plan.rs` (multiple `#[test]` cases).
  - Missing gate to add: at least one end-to-end `fretboard diag` scene that snapshots a tricky mask/blur/scissor combo (if/when a demo exists).
- Text fallback policy embedded in renderer
  - Failure mode: cross-platform font selection changes; mixed-script text regressions; WASM behavior drift.
  - Existing gates: unit tests in `text/*` and `fret-fonts` bootstrap coverage.
  - Missing gate to add: fixture-driven “fallback chain” expectations per platform (only if we decide to make fallback ordering a contract).
- Environment-based backend selection
  - Failure mode: silent misconfiguration; separators/synonyms parsing regressions.
  - Existing gates: `parse_wgpu_backends_*` unit tests in `lib.rs`.

Evidence anchors:

- `crates/fret-render-wgpu/src/renderer/tests.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan.rs`
- `crates/fret-render-wgpu/src/text/mod.rs`
- `crates/fret-render-wgpu/src/lib.rs`

## 6) Code quality findings (Rust best practices)

- Large-module risk:
  - `text/mod.rs` (~5k LOC) is a “god module” that likely mixes: fallback policy, shaping integration, caches, and scene recording glue.
  - `renderer/shaders.rs` embeds large WGSL strings; consider moving towards “generated or external WGSL assets” once tooling is stable.
- Async boundary:
  - `WgpuContext::new*` are async and necessarily block on adapter/device acquisition; ensure callers keep this off the UI thread (policy likely belongs in runner/app layer).

Evidence anchors:

- `crates/fret-render-wgpu/src/text/mod.rs`
- `crates/fret-render-wgpu/src/renderer/shaders.rs`
- `crates/fret-render-wgpu/src/lib.rs`

## 7) Recommended refactor steps (small, gated)

1. Keep env parsing testable and stable — outcome: reproducible backend selection behavior — gate: `cargo nextest run -p fret-render-wgpu`.
2. Split `text/mod.rs` into a directory module by responsibility (fallback, shaping, caches, scene encoding) — outcome: isolate platform policy from shaping glue — gate: existing text unit tests + `cargo nextest run -p fret-render-wgpu`.
3. Introduce a “shader source boundary” (Rust string vs external asset) and keep a single validation entry point — outcome: safer shader evolution — gate: `renderer::tests::shaders_validate_for_webgpu`.

## 8) Open questions / decisions needed

- Do we want `FRET_WGPU_BACKEND` to be a stable, user-facing contract (documented), or should it remain a developer-only knob?
- Where should font fallback policy live long-term: renderer backend, `fret-fonts`, or a higher-level “platform defaults” crate?
