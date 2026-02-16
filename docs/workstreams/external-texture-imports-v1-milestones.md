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
    - `tools/diag-scripts/external-texture-imports-web-copy.json`
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
- [x] EXT-native-120 Native “true external import” adapter seam (platform-decoder produced GPU
      frame, capability-gated, deterministic fallback).
  - Evidence:
    - `crates/fret-launch/src/runner/native_external_import.rs`
    - `crates/fret-launch/src/runner/imported_viewport_target.rs`
- [x] EXT-perf-130 Comparative diag/perf baselines for native copy paths (native CPU upload vs GPU
      offscreen).
  - Evidence (native):
    - `docs/workstreams/perf-baselines/external-texture-imports-contract-path.windows-local.v1.json`
    - `docs/workstreams/perf-baselines/external-texture-imports-decoded-png-cpu-copy.windows-local.v1.json`
- [x] EXT-web-perf-131 Web GPU copy path perf baseline (when stable).
  - Evidence:
    - `tools/diag-scripts/external-texture-imports-web-copy-perf-steady.json`
    - `apps/fretboard/src/demos.rs` (`external_texture_imports_web_demo`)
    - `docs/workstreams/perf-baselines/policies/external-texture-imports-web-copy.v1.json`
    - `docs/workstreams/perf-baselines/external-texture-imports-web-copy.web-local.v1.json`
  - Baseline record:
    - Date: 2026-02-15
    - Exports:
      - `target/fret-diag-web-copy/exports/1771140829044-bundle`
      - `target/fret-diag-web-copy/exports/1771140845261-bundle`

- [x] EXT-diag-210 Import ingest observability (requested vs effective) is surfaced in perf snapshots
      and diagnostics bundles.
  - Notes:
    - The caller should set:
      - `RenderTargetMetadata.requested_ingest_strategy` (desired), and
      - `RenderTargetMetadata.ingest_strategy` (effective, after capability-gated fallback).
    - The renderer reports:
      - `render_target_updates_requested_ingest_*` (requested distribution),
      - `render_target_updates_ingest_*` (effective distribution),
      - `render_target_updates_ingest_fallbacks` (requested != effective),
      - and `viewport_draw_calls_ingest_*` (draw-side attribution).
    - This is best-effort observability only; it must not change import behavior by itself.
  - Evidence:
    - `crates/fret-render-core/src/lib.rs` (`RenderTargetMetadata.requested_ingest_strategy`)
    - `crates/fret-render-wgpu/src/renderer/resources.rs` (update-time counters)
    - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (perf snapshot plumbing)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiFrameStatsV1` fields)

## M5 — Zero/low-copy ceiling (v2; capability-gated)

- [~] EXT-m5-400 Contract v2 is documented and bounded:
      define the strategy set, capability gating, deterministic fallback order, and metadata
      semantics so copy and zero/low-copy paths converge to the same observable behavior.
  - ADR: `docs/adr/0282-external-texture-imports-v2-zero-low-copy.md`

- [ ] EXT-m5-410 Perf-gate checklist is expanded for v2:
      add a v2 steady-state perf script + baseline for any zero/low-copy path we land, and keep
      the existing copy-path baselines as non-regression anchors (web + native).
  - Evidence anchors (existing copy-path gates):
    - `tools/diag-scripts/external-texture-imports-web-copy-perf-steady.json`
    - `docs/workstreams/perf-baselines/external-texture-imports-web-copy.web-local.v1.json`
  - Checklist (new v2 baselines):
    - Record both requested vs effective ingest distributions and the fallback count:
      - `render_target_updates_requested_ingest_*`
      - `render_target_updates_ingest_*`
      - `render_target_updates_ingest_fallbacks`
    - Keep `viewport_draw_calls_ingest_*` stable for the target workload (draw-side attribution).
    - If `requested=ExternalZeroCopy` is used in a demo, the baseline must explicitly tolerate
      fallback to `effective=GpuCopy` on unsupported targets (and show it via the counters).

- [!] EXT-m5-420 Web zero-copy implementation (blocked):
      WebCodecs `VideoFrame` → WebGPU `ExternalTexture` → imported render target, capability-gated
      with deterministic fallback to the GPU copy path.
  - Blocker: wgpu WebGPU backend missing `ExternalTexture` implementation (wgpu v28).
