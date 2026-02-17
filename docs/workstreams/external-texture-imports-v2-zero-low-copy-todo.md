# External Texture Imports v2 (Zero/Low-Copy) — TODO Tracker

Status: Draft (workstream tracker)

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `EXTV2-{area}-{nnn}`

When completing an item, leave 1–3 evidence anchors (paths + key functions/tests), and prefer
`fretboard diag` scripts/bundles where applicable.

## Design lock

- [x] EXTV2-adr-010 Lock ADR 0282 to “executable” detail:
      bounded strategy set, capability gating rules, deterministic fallback chain, and
      wasm/mobile correctness checklist.
  - ADR: `docs/adr/0282-external-texture-imports-v2-zero-low-copy.md`
  - Exit criteria:
    - every strategy has explicit prerequisites + fallback,
    - every metadata field has an explicit “preserve or degrade” rule,
    - perf gate checklist is explicit per target (native/wasm/mobile).
  - Evidence anchors:
    - `docs/adr/0282-external-texture-imports-v2-zero-low-copy.md` (strategy set, fallback chain, metadata rules)
    - `docs/workstreams/external-texture-imports-v2-zero-low-copy.md` (execution order + web WS notes)
    - `crates/fret-render-core/src/lib.rs` (`RenderTargetMetadata`, `RenderTargetIngestStrategy`)

## Metadata semantics

- [x] EXTV2-meta-020 Extend the imported-target metadata surface (portable + bounded):
      add the minimum set of fields needed for video/camera correctness, expressed as bounded enums
      (no open-ended strings), with deterministic degradation rules.
  - Proposed minimum:
    - content color encoding hints (primaries, transfer, matrix, range),
    - explicit “unknown” values for every enum (portable default),
    - serde defaults for forward/backward compatibility in diag baselines.
  - Evidence anchors:
    - `docs/adr/0282-external-texture-imports-v2-zero-low-copy.md` (metadata rules, deterministic degradation)
    - `crates/fret-render-core/src/lib.rs` (`RenderTargetMetadata`)
      - `RenderTargetColorEncoding`, `RenderTargetColorPrimaries`, `RenderTargetTransferFunction`,
        `RenderTargetMatrixCoefficients`, `RenderTargetColorRange`
      - unit test: `render_target_metadata_color_encoding_defaults_when_missing`
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/viewport_surface.rs`

## Capability matrix + observability

- [x] EXTV2-cap-030 Publish a capability matrix for v2 strategies (native/wasm/mobile) and ensure
      “requested vs effective” ingest attribution is always present in perf snapshots/bundles.
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/capabilities.rs`
    - `crates/fret-render-wgpu/src/renderer/types.rs` (`RenderPerfSnapshot`)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiFrameStatsV1`)
    - `docs/adr/0282-external-texture-imports-v2-zero-low-copy.md` (Capability matrix + fallback chain)

- [x] EXTV2-diag-040 Add explicit counters/hints for metadata degradations (not just ingest fallbacks):
      e.g. “color_encoding_dropped”, “colorspace_hint_dropped”, “orientation_hint_ignored”.
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/renderer/resources.rs` (`render_target_color_encoding_conflicts_with_portable_rgb_assumption`)
    - `crates/fret-render-wgpu/src/renderer/types.rs` (`RenderPerfSnapshot` field: `render_target_metadata_degradations_color_encoding_dropped`)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiFrameStatsV1.renderer_render_target_metadata_degradations_color_encoding_dropped`)

## Native/mobile implementations (staged)

- [x] EXTV2-native-101 M2A: wire a real native frame source end-to-end (Windows MF), even if it
      initially lands as `CpuUpload`, with stable metadata semantics and gates.
  - Evidence anchors:
    - Adapter-request attribution (requested vs effective) is now verifiable in a native demo:
      - `apps/fret-examples/src/external_texture_imports_demo.rs` (KeyN adapter path requests `ExternalZeroCopy`)
      - Perf script: `tools/diag-scripts/external-texture-imports-contract-path-native-adapter-perf-steady.json`
      - Baseline: `docs/workstreams/perf-baselines/external-texture-imports-contract-path-native-adapter.windows-local.v1.json`
    - First real native frame source (Windows/MF, stage M2A = CPU upload):
      - `apps/fret-examples/src/external_video_imports_mf_demo.rs` (`wmf::MfVideoReader`, CPU upload loop, test_ids)
      - `apps/fret-demo/src/bin/external_video_imports_mf_demo.rs` (demo entrypoint)
      - Perf script: `tools/diag-scripts/external-video-imports-mf-cpu-upload-perf-steady.json`
      - Baseline: `docs/workstreams/perf-baselines/external-video-imports-mf-cpu-upload.windows-local.v1.json`
      - Correctness script (requires `FRET_DIAG_SCREENSHOTS=1` + `--check-pixels-changed external-video-imports-mf-surface`):
        - `tools/diag-scripts/external-video-imports-mf-cpu-upload-correctness.json`
    - Tooling note (native gates):
      - Native sessions may omit embedding `bundle.json` into WS messages. `fret-diag-export`
        supports this by reading the exported bundle from disk using `out_dir` + `dir`.
        - Evidence: `apps/fret-diag-export/src/main.rs` (`wait_for_bundle_dumped` filesystem fallback)
  - Local verification (Windows, 2026-02-17):
    - Start a devtools WS hub (token can be fixed for repeatable scripts):
      - `FRET_DEVTOOLS_TOKEN=<token> cargo run -p fret-devtools-ws`
    - Run the demo (DX12 optional; any native backend works for CPU upload):
      - `FRET_DEVTOOLS_WS=ws://127.0.0.1:7331/ FRET_DEVTOOLS_TOKEN=<token> FRET_MF_VIDEO_PATH=<dir_or_file> cargo run -p fret-demo --features devtools-ws --bin external_video_imports_mf_demo`
      - `FRET_MF_VIDEO_PATH` may point to a directory; the demo picks the first supported video file (sorted by filename).
    - List sessions and run the correctness script:
      - `FRET_DEVTOOLS_WS=ws://127.0.0.1:7331/ cargo run -p fret-diag-export -- --list-sessions --token <token>`
      - `FRET_DEVTOOLS_WS=ws://127.0.0.1:7331/ cargo run -p fret-diag-export -- --script tools/diag-scripts/external-video-imports-mf-cpu-upload-correctness.json --token <token> --session-id <id> --out-dir target/fret-diag-mf/exports`
    - Expected artifacts:
      - Bundles: `target/fret-diag-mf/exports/<ts>-bundle/bundle.json`
      - Screenshots (if `FRET_DIAG_DIR` is set): `<diag_dir>/screenshots/<ts>-script-step-*/window-*.png`

- [x] EXTV2-native-103 M2B: prove a capability-gated **shared allocation** write path on native:
      a synthetic producer writes into a renderer-owned `wgpu::Texture` via the backend’s native queue,
      with deterministic state transitions and a minimal gate.
  - Intent:
    - This is the practical “no-copy” uplift when “import a foreign platform texture handle into wgpu”
      is blocked upstream.
    - This typically classifies as `Owned` in the bounded strategy set (the texture is renderer-owned).
  - Evidence anchors:
    - `apps/fret-examples/src/external_texture_imports_demo.rs` (DX12 clear shared-allocation mode; env-gated)
    - Correctness script (DX12-only; requires `FRET_WGPU_BACKEND=dx12` and shared-allocation flag):
      - `tools/diag-scripts/external-texture-imports-dx12-shared-allocation-clear-correctness.json`

- [x] EXTV2-native-104 Add a runner-facing shared allocation export helper (DX12-only):
      centralize “export queue/resource + wgpu transitions” so real producers can write into
      renderer-owned textures without duplicating unsafe backend plumbing.
  - Evidence anchors:
    - `crates/fret-launch/src/runner/shared_allocation.rs` (`dx12::Dx12SharedAllocationWriteGuard`)
    - `apps/fret-examples/src/external_texture_imports_demo.rs` (uses the helper in DX12 clear mode)

- [~] EXTV2-native-105 Prototype a real native frame source landing as DX12 GPU copy (MF, env-gated):
      decode via Media Foundation with a DXGI device manager and copy frames into a shared allocation
      (requested `ExternalZeroCopy`, effective `GpuCopy`), with deterministic fallback to checker/CPU paths.
  - Notes:
    - This is intentionally capability-gated and experimental; it should not be considered portable until
      the constraints are understood across driver/backends.
    - Known failure modes:
      - The MF SourceReader may still return a CPU-backed `IMFMediaBuffer` even when a DXGI device
        manager is configured, so the DX12 path cannot obtain an `IMFDXGIBuffer` and deterministically
        falls back to `CpuUpload`.
      - Some codecs / drivers deliver DXGI-backed frames as NV12 surfaces. The demo handles this by
        converting NV12 -> BGRA on GPU (D3D11 video processor) into a temporary texture before copying
        into the DX12 shared allocation.
  - Evidence anchors:
    - `apps/fret-examples/src/external_video_imports_mf_demo.rs` (`ExternalVideoImportsMode::MfVideoDx12GpuCopy`)
    - `crates/fret-launch/src/runner/shared_allocation.rs` (`dx12::Dx12SharedAllocationWriteGuard::export_raw`)
    - Correctness script (requires `FRET_WGPU_BACKEND=dx12`, `FRET_EXTV2_MF_DX12_GPU_COPY=1`, and a playable `FRET_MF_VIDEO_PATH`):
      - `tools/diag-scripts/external-video-imports-mf-dx12-gpu-copy-correctness.json`
    - Perf script:
      - `tools/diag-scripts/external-video-imports-mf-dx12-gpu-copy-perf-steady.json`
    - Baseline:
      - `docs/workstreams/perf-baselines/external-video-imports-mf-dx12-gpu-copy.windows-local.v1.json`
  - Local verification (Windows DX12, 2026-02-17):
    - `FRET_WGPU_BACKEND=dx12 FRET_EXTV2_MF_DX12_GPU_COPY=1 FRET_MF_VIDEO_PATH=<dir_or_file> ... external_video_imports_mf_demo`
    - Run: `tools/diag-scripts/external-video-imports-mf-dx12-gpu-copy-correctness.json` via `fret-diag-export` and confirm screenshots show decoded video frames.

- [~] EXTV2-native-100 Land a native low/zero-copy ingestion path where supported:
      integrate platform-decoder produced frames via a capability-gated adapter, with deterministic
      fallback to GPU copy / CPU upload and observable attribution.
  - Evidence anchors:
    - `crates/fret-launch/src/runner/native_external_import.rs`
    - `crates/fret-launch/src/runner/imported_viewport_target.rs`
      - `ImportedViewportRenderTarget::push_native_external_import_update_with_requested_ingest_strategy_or_fallback(...)`
      - `ImportedViewportRenderTarget::push_native_external_import_update_with_deterministic_fallback(...)`
  - Remaining:
    - Land a real platform/decoder-backed `NativeExternalTextureFrame` implementation that can
      produce the best available path on capable backends (and deterministically degrade otherwise):
      - Prefer shared allocation where external-handle import is blocked (producer writes into renderer-owned texture).
      - Use true external-handle import only when upstream exposes a supported mechanism.
    - Investigate native `ExternalZeroCopy` feasibility on wgpu 28:
      - likely requires a supported way to wrap/import platform GPU textures (e.g. D3D12/Metal/IOSurface)
        into a `wgpu::Texture`/`TextureView` safely; treat as capability-gated and potentially blocked
        until upstream exposes the necessary APIs.

- [x] EXTV2-native-102 M2B (time-box): feasibility spike for a true `ExternalZeroCopy` path on
      native, behind explicit capabilities (e.g. Windows D3D12-only import).
  - Resolution (2026-02-17):
    - **Blocked** for “wrap a foreign platform texture handle into wgpu” as a general mechanism:
      wgpu’s `Device::create_texture_from_hal` requires a `wgpu-hal` texture value that is created
      by the same wgpu device, and `wgpu-hal` does not expose a public constructor for importing an
      arbitrary platform handle (e.g. `ID3D12Resource`) into a `wgpu-hal::dx12::Texture`.
    - **Viable alternative** for “no-copy” on native: **shared allocation** (allocate a
      `wgpu::Texture` in the runner/renderer and hand its native handle to the producer/decoder),
      which can often be classified as `Owned` rather than `ExternalZeroCopy` in our bounded
      strategy set.
  - Next revisit:
    - Re-evaluate once wgpu exposes a safe, supported “import platform texture handle” API for the
      relevant backends (D3D12/Metal/IOSurface), or once we adopt a backend that provides it.
  - Evidence anchors:
    - `apps/fret-examples/src/external_texture_imports_demo.rs` (env-gated DX12 handle probe:
      `FRET_EXTV2_DX12_SHARED_TEXTURE_PROBE=1`)
    - `docs/adr/0282-external-texture-imports-v2-zero-low-copy.md` (capability gating + strategy set)

- [ ] EXTV2-mobile-110 Define iOS/Android capability-gated plans (blocked until backend support exists):
      document prerequisites and the deterministic fallback behavior.

## Web zero-copy (explicitly blocked)

- [!] EXTV2-web-120 WebCodecs `VideoFrame` → WebGPU `ExternalTexture` zero-copy sampling:
      land only when the backend supports `ExternalTexture` end-to-end.
  - Blocker: upstream backend support (wgpu WebGPU backend).

## Perf gates

- [~] EXTV2-perf-200 Add v2 steady-state perf scripts + baselines for any landed v2 strategy.
  - Keep v1 copy-path baselines green:
    - `tools/diag-scripts/external-texture-imports-web-copy-perf-steady.json`
    - `docs/workstreams/perf-baselines/external-texture-imports-web-copy.web-local.v1.json`
    - Evidence (2026-02-16): web DevTools WS perf gate run is runnable end-to-end again:
      - `fix(web): wake redraw on DevTools WS inbox`
      - `fretboard diag perf tools/diag-scripts/external-texture-imports-web-copy-perf-steady.json ... --perf-baseline docs/workstreams/perf-baselines/external-texture-imports-web-copy.web-local.v1.json`
  - Native adapter-path gate (requested vs effective attribution in perf bundles):
    - Script: `tools/diag-scripts/external-texture-imports-contract-path-native-adapter-perf-steady.json`
    - Baseline: `docs/workstreams/perf-baselines/external-texture-imports-contract-path-native-adapter.windows-local.v1.json`
  - MF CPU-upload gate (stage M2A; real source):
    - Script: `tools/diag-scripts/external-video-imports-mf-cpu-upload-perf-steady.json`
    - Baseline: `docs/workstreams/perf-baselines/external-video-imports-mf-cpu-upload.windows-local.v1.json`
  - DX12 shared-allocation (stage M2B; synthetic native writer):
    - Script: `tools/diag-scripts/external-texture-imports-dx12-shared-allocation-clear-perf-steady.json`
    - Baseline: `docs/workstreams/perf-baselines/external-texture-imports-dx12-shared-allocation-clear.windows-local.v1.json`
    - Seed policy: `docs/workstreams/perf-baselines/policies/external-texture-imports-dx12-shared-allocation-clear.v1.json`
