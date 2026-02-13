Status: Active (workstream tracker)

This file is the execution checklist for:

- `docs/workstreams/external-texture-imports-v1.md`

## M0 — Contract path runs end-to-end (in-repo verifiable)

- [x] EXT-m0-000 Demo registers a stable `RenderTargetId`, refreshes its `TextureView` via
      runner-applied deltas, and displays it via `ViewportSurface` with:
  - [x] resize handling (realloc on window resize),
  - [x] fit coverage (contain/cover/stretch),
  - [x] lifecycle toggle (unregister/register to validate cleanup + ID stability).
- [x] EXT-m0-010 Diagnostics closure: `fretboard diag run` produces a bundle + screenshots.
- [x] EXT-m0-020 Perf closure: committed steady-state baseline + seed policy preset.

Evidence:

- ADR: `docs/adr/0234-imported-render-targets-and-external-texture-imports-v1.md`
- Helper: `crates/fret-launch/src/runner/imported_viewport_target.rs` (`ImportedViewportRenderTarget`)
- Demo (native): `apps/fret-demo/src/bin/external_texture_imports_demo.rs`
- Demo (examples): `apps/fret-examples/src/external_texture_imports_demo.rs`
- Diag scripts:
  - `tools/diag-scripts/external-texture-imports-contract-path.json`
  - `tools/diag-scripts/external-texture-imports-contract-path-perf-steady.json`
- Perf policy + baseline:
  - `docs/workstreams/perf-baselines/policies/external-texture-imports-contract-path.v1.json`
  - `docs/workstreams/perf-baselines/external-texture-imports-contract-path.windows-local.v1.json`

## M1 — Capability-gated “true import” (copy-based paths)

- [x] EXT-m1-100 Web copy path exists (GPU copy, no CPU readback):
      `ExternalImageSource` → `Queue::copy_external_image_to_texture` → imported render target.
  - Evidence:
    - `apps/fret-examples/src/external_texture_imports_web_demo.rs`
    - `apps/fret-demo-web/src/wasm.rs` (`demo=external_texture_imports_web_demo`)

- [x] EXT-m1-110 Native copy policy exists (software decode → CPU upload → imported render target)
      with an explicit, deterministic fallback story.
  - Evidence:
    - `apps/fret-examples/src/external_texture_imports_demo.rs` (`I` toggles source)

- [x] EXT-m1-120 Renderer reports capability gating for external texture import.
  - Evidence:
    - `crates/fret-render-wgpu/src/capabilities.rs`
    - `docs/workstreams/diag-extensibility-and-capabilities-v1/capabilities.md`

## M2 — Metadata seam (explicit descriptors)

- [x] EXT-m2-200 Render target descriptors carry explicit import metadata:
      alpha semantics, orientation/transform hints, timestamp hints (diagnostics only).
  - Evidence:
    - `crates/fret-render-core/src/lib.rs` (`RenderTargetMetadata`)
    - `crates/fret-render-wgpu/src/targets.rs` (`RenderTargetDescriptor.metadata`)

## M3 — Web zero-copy (blocked)

- [!] EXT-m3-300 / EXT-web-100 Web v1 zero-copy import via WebGPU `ExternalTexture` is implemented
      and gated (with deterministic fallback).
  - Blocker: wgpu WebGPU backend missing `ExternalTexture` implementation (wgpu v28).

## M4 — Follow-ups (v1.x; not required for closure)

- [x] EXT-meta-110 Consume `RenderTargetMetadata` for sampling transforms where applicable
      (alpha/orientation).
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/viewport_surface.rs`
    - `crates/fret-render-wgpu/tests/viewport_surface_metadata_conformance.rs`
- [ ] EXT-native-120 Native “true external import” adapter seam (platform-decoder produced GPU
      frame, capability-gated, deterministic fallback).
- [ ] EXT-perf-130 Comparative diag/perf baselines for copy paths (native CPU upload vs GPU
      offscreen; web GPU copy when stable).
