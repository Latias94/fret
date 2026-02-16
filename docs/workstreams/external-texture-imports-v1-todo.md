Status: Active (workstream tracker)

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `EXT-{area}-{nnn}`

Leave 1–3 evidence anchors when completing an item (paths + key functions/tests), and prefer
`fretboard diag` scripts/bundles where applicable.

## Open items (v1 follow-ups)

- [~] EXT-adr-200 External texture imports v2 contract (zero/low-copy):
      define the bounded strategy set, capability gating, deterministic fallback order, and the
      required metadata semantics for correctness parity.
  - ADR: `docs/adr/0282-external-texture-imports-v2-zero-low-copy.md`
  - Exit criteria:
    - the fallback chain is explicit and bounded (no “best effort”),
    - metadata (colorspace/orientation/alpha) semantics are locked,
    - and a minimal perf-gate checklist is recorded for wasm/mobile.

- [x] EXT-diag-210 Surface per-frame ingestion strategy counters in renderer perf snapshots:
      record declared `RenderTargetMetadata` ingestion strategy counts for `RenderTargetId` updates and
      viewport sampling, and include them in UI diagnostics bundles for perf attribution.
  - Notes:
    - `RenderTargetMetadata.requested_ingest_strategy` (requested) and `RenderTargetMetadata.ingest_strategy`
      (effective) are both surfaced so capability-gated fallbacks are measurable.
    - `render_target_updates_ingest_fallbacks` counts requested != effective at update time (best-effort).
  - Evidence anchors:
    - `crates/fret-render-core/src/lib.rs` (`RenderTargetIngestStrategy`)
    - `crates/fret-render-wgpu/src/renderer/resources.rs` (`perf_pending_render_target_updates_by_ingest`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (snapshot fields + breakdown)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiFrameStatsV1` renderer fields)

- [!] EXT-web-100 Web v1 zero-copy import: WebCodecs `VideoFrame` → WebGPU `ExternalTexture`
      (capability-gated) with deterministic fallback.
  - Notes:
    - This is the web backend’s primary v2 “ceiling path”, but it remains blocked until the
      WebGPU backend supports `ExternalTexture` ingestion end-to-end.
    - Contract details are tracked in ADR 0282.
  - Blocker: wgpu WebGPU backend does not implement `ExternalTexture` yet (wgpu v28 has an
    `unimplemented!` stub in `wgpu/src/backend/webgpu.rs`).
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/capabilities.rs`
    - `docs/workstreams/creative-recipes-v1-todo.md` (P1 external texture imports section)

- [x] EXT-meta-110 Consume `RenderTargetMetadata` for sampling transforms where applicable:
      - alpha semantics (`straight` → premul policy),
      - orientation/transform mapping for camera/video sources.
  - Notes:
    - ADR 0234 v1 only requires the seam to exist; this item is about turning the metadata into
      an end-to-end observable behavior when we have real sources.
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/targets.rs` (`RenderTargetRegistry::metadata`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/viewport_surface.rs`
    - `crates/fret-render-wgpu/tests/viewport_surface_metadata_conformance.rs`

- [x] EXT-native-120 Native “true external import” adapter seam (optional v1.1):
      accept a platform-decoder produced GPU frame via a capability-gated interface, without
      leaking handles into `fret-ui`, and with deterministic fallback to the copy paths.
  - Evidence anchors:
    - `crates/fret-launch/src/runner/native_external_import.rs`
    - `crates/fret-launch/src/runner/imported_viewport_target.rs`
    - `apps/fret-examples/src/external_texture_imports_demo.rs` (KeyN toggles adapter path)

- [x] EXT-perf-130 Add comparative diag/perf baselines for native copy paths:
      - native CPU upload path vs native GPU-offscreen path,
      and document the expected deltas (uploads/intermediates) in the baseline notes.
  - Evidence anchors (native):
    - `tools/diag-scripts/external-texture-imports-contract-path-perf-steady.json`
    - `docs/workstreams/perf-baselines/external-texture-imports-contract-path.windows-local.v1.json`
    - `tools/diag-scripts/external-texture-imports-decoded-png-cpu-copy-perf-steady.json`
    - `docs/workstreams/perf-baselines/external-texture-imports-decoded-png-cpu-copy.windows-local.v1.json`

- [x] EXT-web-perf-131 Web GPU copy path perf baseline (when stable):
      add a steady-state perf script + baseline for the wasm copy path demo.
  - Evidence anchors:
    - `apps/fretboard/src/demos.rs` (`external_texture_imports_web_demo`)
    - `tools/diag-scripts/external-texture-imports-web-copy.json`
    - `tools/diag-scripts/external-texture-imports-web-copy-perf-steady.json`
    - `docs/workstreams/perf-baselines/policies/external-texture-imports-web-copy.v1.json`
    - `docs/workstreams/perf-baselines/external-texture-imports-web-copy.web-local.v1.json`
    - `apps/fret-examples/src/external_texture_imports_web_demo.rs`
    - `apps/fret-demo-web/src/wasm.rs` (`demo=external_texture_imports_web_demo`)
  - Notes:
    - Export bundles (devtools-ws):
      - `FRET_DEVTOOLS_WS=ws://127.0.0.1:7331/ FRET_DEVTOOLS_TOKEN=<token> cargo run -p fret-diag-export -- --list-sessions`
      - `FRET_DEVTOOLS_WS=ws://127.0.0.1:7331/ FRET_DEVTOOLS_TOKEN=<token> cargo run -p fret-diag-export -- --script tools/diag-scripts/external-texture-imports-web-copy-perf-steady.json --session-id <id> --out-dir target/fret-diag-web-copy/exports`
    - Generate baseline:
      - `cargo run -p fretboard -- diag perf-baseline-from-bundles tools/diag-scripts/external-texture-imports-web-copy-perf-steady.json target/fret-diag-web-copy/exports/<export1> target/fret-diag-web-copy/exports/<export2> --perf-baseline-headroom-pct 20 --warmup-frames 5 --perf-baseline-out docs/workstreams/perf-baselines/external-texture-imports-web-copy.web-local.v1.json`
  - Baseline record:
    - Date: 2026-02-15
    - Exports:
      - `target/fret-diag-web-copy/exports/1771140829044-bundle`
      - `target/fret-diag-web-copy/exports/1771140845261-bundle`
