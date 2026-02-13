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
   - a `fretboard diag run` script that produces screenshots,
   - and a committed steady-state perf baseline + policy preset.

## Non-goals (v1)

- Shipping zero-copy imports everywhere.
- Owning codecs/decoders/media engines in the framework (ADR 0119).
- Exposing backend handles (wgpu/Vulkan/Metal/WebGPU) to UI/component code.

## Tracking

- TODOs: `docs/workstreams/external-texture-imports-v1-todo.md`
- Milestones: `docs/workstreams/external-texture-imports-v1-milestones.md`

