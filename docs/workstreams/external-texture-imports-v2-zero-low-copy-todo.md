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

- [ ] EXTV2-meta-020 Extend the imported-target metadata surface (if required):
      add the minimum set of fields needed for video/camera correctness (colorspace hints,
      range/transfer/matrix as a bounded enum set), with deterministic degradation rules.
  - Evidence anchors:
    - `crates/fret-render-core/src/lib.rs` (`RenderTargetMetadata`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/viewport_surface.rs`

## Capability matrix + observability

- [x] EXTV2-cap-030 Publish a capability matrix for v2 strategies (native/wasm/mobile) and ensure
      “requested vs effective” ingest attribution is always present in perf snapshots/bundles.
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/capabilities.rs`
    - `crates/fret-render-wgpu/src/renderer/types.rs` (`RenderPerfSnapshot`)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiFrameStatsV1`)
    - `docs/adr/0282-external-texture-imports-v2-zero-low-copy.md` (Capability matrix + fallback chain)

- [!] EXTV2-diag-040 Add explicit counters/hints for metadata degradations (not just ingest fallbacks):
      e.g. “colorspace_hint_dropped”, “orientation_hint_ignored”.
  - Blocked until we introduce additional metadata fields beyond v1’s portable set (or land a
    backend that cannot preserve existing metadata and must degrade deterministically).

## Native/mobile implementations (staged)

- [~] EXTV2-native-100 Land a native low/zero-copy ingestion path where supported:
      integrate platform-decoder produced frames via a capability-gated adapter, with deterministic
      fallback to GPU copy / CPU upload and observable attribution.
  - Evidence anchors:
    - `crates/fret-launch/src/runner/native_external_import.rs`
    - `crates/fret-launch/src/runner/imported_viewport_target.rs`
      - `ImportedViewportRenderTarget::push_native_external_import_update_with_requested_ingest_strategy_or_fallback(...)`
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
  - Remaining:
    - Land a real platform/decoder-backed `NativeExternalTextureFrame` implementation that can
      produce `ExternalZeroCopy` on capable backends (and deterministically degrade otherwise) (M2B+).

- [ ] EXTV2-mobile-110 Define iOS/Android capability-gated plans (blocked until backend support exists):
      document prerequisites and the deterministic fallback behavior.

## Web zero-copy (explicitly blocked)

- [!] EXTV2-web-120 WebCodecs `VideoFrame` → WebGPU `ExternalTexture` zero-copy sampling:
      land only when the backend supports `ExternalTexture` end-to-end.
  - Blocker: upstream backend support (wgpu WebGPU backend).

## Perf gates

- [ ] EXTV2-perf-200 Add v2 steady-state perf scripts + baselines for any landed v2 strategy.
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
