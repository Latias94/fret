Status: Active (workstream tracker)

This workstream tracks **importing GPU-produced frames into the UI** via `RenderTargetId` +
`SceneOp::ViewportSurface`, without leaking backend handles into `fret-ui`.

It is the practical “runs end-to-end in-repo” companion to:

- ADR 0234: `docs/adr/0234-imported-render-targets-and-external-texture-imports-v1.md`

## Goals (v1)

1. Close the loop for the “contract path”:
   - a driver can refresh a `wgpu::TextureView` each frame via runner-applied deltas
     (`EngineFrameUpdate.target_updates`),
   - and the UI can display it as a `ViewportSurface` with resize + fit + lifecycle coverage.
2. Establish the minimal **metadata seam** required by real imports:
   - alpha semantics (`premul` vs `straight`),
   - orientation/transform hints,
   - frame timestamp hints (diagnostics only).
3. Provide a capability-gated “true import” story with deterministic fallback:
   - start with copy-based paths (GPU copy / CPU upload),
   - track web zero-copy (`ExternalTexture`) explicitly as blocked until the backend supports it.
4. Leave a diagnostics/perf closure:
   - a `fretboard-dev diag run` script that produces screenshots,
   - and a committed steady-state perf baseline + policy preset.

## Non-goals (v1)

- Shipping zero-copy imports everywhere.
- Owning codecs/decoders/media engines in the framework (ADR 0119).
- Exposing backend handles (wgpu/Vulkan/Metal/WebGPU) to UI/component code.

## Tracking

- TODOs: `docs/workstreams/external-texture-imports-v1/external-texture-imports-v1-todo.md`
- Milestones: `docs/workstreams/external-texture-imports-v1/external-texture-imports-v1-milestones.md`

## v2 direction — zero/low-copy imports (capability-gated)

The v1 workstream intentionally proves the *contract path* first. The next meaningful step is to
raise the performance ceiling for high-frequency, high-resolution sources (video, camera, remote
desktop, large canvas) by adding a **capability-gated zero/low-copy import path**.

Key properties (v2):

- **Bounded and deterministic**: the import strategy is selected from a small, ordered set, and
  every unsupported path must fall back deterministically to a copy-based path.
- **No backend handles in UI**: UI/component code continues to consume `RenderTargetId` +
  `SceneOp::ViewportSurface` only.
- **Metadata is first-class**: colorspace/transform/orientation/alpha semantics travel with the
  frame so the copy and zero-copy paths converge to the same observable behavior.
- **Perf gated**: steady-state perf baselines exist for the copy path today; v2 adds baselines for
  any zero/low-copy path we land, especially on wasm/mobile.

Tracking:

- ADR: `docs/adr/0282-external-texture-imports-v2-zero-low-copy.md`
- Workstream (v2): `docs/workstreams/external-texture-imports-v2-zero-low-copy/external-texture-imports-v2-zero-low-copy.md`
