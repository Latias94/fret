Status: Active (workstream tracker)

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `EXT-{area}-{nnn}`

Leave 1–3 evidence anchors when completing an item (paths + key functions/tests), and prefer
`fretboard diag` scripts/bundles where applicable.

## Open items (v1 follow-ups)

- [!] EXT-web-100 Web v1 zero-copy import: WebCodecs `VideoFrame` → WebGPU `ExternalTexture`
      (capability-gated) with deterministic fallback.
  - Blocker: wgpu WebGPU backend does not implement `ExternalTexture` yet (wgpu v28 has an
    `unimplemented!` stub in `wgpu/src/backend/webgpu.rs`).
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/capabilities.rs`
    - `docs/workstreams/creative-recipes-v1-todo.md` (P1 external texture imports section)

- [ ] EXT-meta-110 Consume `RenderTargetMetadata` for sampling transforms where applicable:
      - alpha semantics (`straight` → premul policy),
      - orientation/transform mapping for camera/video sources.
  - Notes:
    - ADR 0234 v1 only requires the seam to exist; this item is about turning the metadata into
      an end-to-end observable behavior when we have real sources.
  - Candidate evidence anchors:
    - `crates/fret-render-wgpu/src/targets.rs` (`RenderTargetDescriptor.metadata`)
    - `crates/fret-render-core/src/lib.rs` (`RenderTargetMetadata`)

- [ ] EXT-native-120 Native “true external import” adapter seam (optional v1.1):
      accept a platform-decoder produced GPU frame via a capability-gated interface, without
      leaking handles into `fret-ui`, and with deterministic fallback to the copy paths.

- [ ] EXT-perf-130 Add comparative diag/perf baselines for copy paths:
      - native CPU upload path vs native GPU-offscreen path,
      - web GPU copy path (when stable),
      and document the expected deltas (uploads/intermediates) in the baseline notes.

