# ADR 0234: Imported Render Targets and External Texture Imports (v1)

Status: Proposed

## Context

Fret supports embedding GPU-produced content into the UI via:

- `RenderTargetId` + `SceneOp::ViewportSurface` (ADR 0007),
- a runner-owned submission coordinator where apps/drivers **record** engine GPU work and the runner
  **submits** it in a deterministic order (ADR 0038).

This integration is sufficient when the app/engine renders into a `wgpu::Texture` allocated on the
shared `Device` (the “owned offscreen target” path).

However, some platforms can produce GPU-native frames outside an app-owned render graph:

- Web: WebCodecs `VideoFrame` → WebGPU “external texture” (capability-gated; ADR 0122).
- Desktop: platform decoders / camera pipelines that can expose GPU textures (capability-gated).

We want a stable, portable contract that lets apps display these frames without:

- leaking backend handles into `fret-ui` (ADR 0092 / ADR 0123),
- breaking queue ownership rules (ADR 0038),
- or forcing the app to always fall back to CPU byte uploads (ADR 0119).

At the same time, we want to avoid getting blocked by platform-specific interop details. The work
must be staged so “the contract path runs end-to-end” can be validated inside this repository
before any true external import path is implemented.

## Decision

### D1 — Split the work into two layers: “contract path” then “true external import”

We explicitly separate:

1) **Contract path (must run and be verifiable in-repo)**
   - The driver can obtain a GPU-resident view (`wgpu::TextureView`) each frame (even if it comes
     from a texture the driver created itself on the shared `Device`).
   - The driver updates the renderer’s render target registry via `EngineFrameUpdate` target deltas
     (ADR 0038) so a stable `RenderTargetId` can be referenced by UI elements.
   - The UI displays the target via `ViewportSurface` with stable resizing/object-fit behavior
     (ADR 0231 + `ViewportSurfaceProps.fit`).

2) **True external import (capability-gated, platform-specific, optional)**
   - The underlying frame is produced by a platform decoder / external system.
   - The ingestion path is selected by a renderer capability snapshot (ADR 0122) and must have a
     deterministic fallback (ADR 0119).
   - No backend handles are exposed to UI/component code; only `ImageId`/`RenderTargetId` are used.

This staged approach prevents the project from being blocked by platform and decoder interop while
still converging on a real “works in practice” end state.

### D2 — “Contract path” render target updates are expressed as explicit deltas (already shaped by ADR 0038)

The driver records engine work and returns `EngineFrameUpdate`:

- `command_buffers: Vec<wgpu::CommandBuffer>`
- `target_updates: Vec<RenderTargetUpdate>`

The runner applies `target_updates` to the renderer registry **before** recording/submitting the UI
command buffer (ADR 0038 + ADR 0015), so UI sampling is correct within the same frame.

This ADR locks the semantics of `RenderTargetUpdate::Update`:

- It may be used for both “first registration” and “view refresh” of a stable `RenderTargetId`.
- The `RenderTargetDescriptor` metadata (`size`, `format`, `color_space`) is authoritative for the
  UI compositor (ADR 0040).
- The view update must preserve strict ordering invariants: submission order remains “engine first,
  then UI” (ADR 0038).

### D3 — Lifetime rule for imported views: the runner must keep resources alive through submission

For the contract path:

- Drivers typically use `ViewportRenderTarget`-style helpers that own the `wgpu::Texture` and keep
  it alive across frames, so the registry can store the `TextureView` safely.

For true external imports:

- A `TextureView` (or external-texture handle) may only be valid for the duration of a frame.
- The runner must ensure that any imported per-frame GPU resource remains alive until:
  - the engine command buffers are submitted, and
  - the UI command buffer that samples the target is submitted.

Implementation strategy is intentionally left open, but the contract requires:

- a deterministic, per-frame “keepalive” ownership mechanism at the runner/renderer boundary,
  without leaking backend handles into `fret-ui`.

### D4 — Metadata seam: render target descriptors must remain explicit and capability-aware

The render target registry entry must carry enough metadata for correct compositing:

- `size`
- `format` (including sRGB-ness as relevant to the backend)
- `color_space` (ADR 0040)

v1 introduces a minimal, explicit metadata seam carried by `RenderTargetDescriptor.metadata`
(`RenderTargetMetadata`):

- alpha semantics (`premul` vs `straight`),
- orientation/transform metadata (for camera/video sources),
- frame timing hints (for diagnostics, not UI logic).

This metadata is stored in the render target registry. v1 does not require the renderer to apply
orientation transforms during sampling yet; the metadata exists to prevent backend paths from
forking implicit conventions and to keep room for capability-gated fast paths.

Future (expected) metadata extensions, if required by real imports:

- YUV plane formats and conversion strategy (GPU vs CPU),
- additional orientation/transform nuance beyond the v1 subset.

Any such extensions must:

- be capability-gated where needed (ADR 0122),
- have deterministic fallbacks (ADR 0119),
- and remain outside `fret-core` scene types if they require backend-specific handles (ADR 0092).

### D5 — Tooling and validation: “contract path runs end-to-end” is a required milestone

Before implementing any true external import:

- Add an in-repo validation harness that:
  - produces a per-frame GPU texture (simple renderer pass: clear + checkerboard),
  - registers/updates it as a `RenderTargetId`,
  - and displays it via `ViewportSurface` with resize + fit coverage.
- Add diagnostics/perf baselines that make the cost visible:
  - draw calls, upload bytes, intermediate usage where relevant (ADR 0095 / ADR 0118).

This milestone must be reproducible on desktop and (when available) on wasm/WebGPU.

## Non-goals (v1)

- Shipping a fully working zero-copy path on every platform/backend.
- Exposing raw `wgpu`/platform texture handles to `fret-ui` or ecosystem component crates.
- Taking ownership of decoders, codecs, audio, or streaming stacks (ADR 0119).

## Consequences

- Fret gains a practical, staged path to “real external texture import” without destabilizing the UI
  contract surfaces.
- The ecosystem can build rich viewports and video panels using `RenderTargetId` today, while
  capability-gated imports can be added incrementally later.

## References

- Viewport surfaces: `docs/adr/0007-viewport-surfaces.md`
- Submission coordinator: `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- Streaming frames: `docs/adr/0119-streaming-images-and-video-surfaces.md`
- Optional zero-copy imports: `docs/adr/0122-renderer-capabilities-and-optional-zero-copy-imports.md`
- Color/compositing: `docs/adr/0040-color-management-and-compositing-contracts.md`
- Foreign UI embedding (isolated surfaces): `docs/adr/0174-foreign-ui-embedding-isolated-surfaces.md`
