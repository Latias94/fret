# ADR 0038: Engine Render Hook and Submission Coordinator (Viewport Recording Without Queue Ownership)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Vello: https://github.com/linebender/vello

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

Fret’s primary goal is to support complex engine editors (Unity/Unreal-class UX):

- multiple engine viewports embedded in dock panels (ADR 0007),
- tear-off windows and cross-window docking (ADR 0013 / ADR 0017),
- UI overlays that sample engine-rendered textures in the same frame (ADR 0015),
- a host-provided shared GPU context (ADR 0010),
- event-driven rendering by default (ADR 0034).

If the integration boundary between “engine rendering” and “UI rendering + presentation” is not explicit early,
real engine users will inevitably build ad-hoc glue that later becomes impossible to standardize without a rewrite.

The most failure-prone area is **queue ownership**:

- if both engine and UI submit to `wgpu::Queue` independently, ordering and observability become fragile,
- multi-window presentation and frames-in-flight pools become much harder to debug,
- “continuous rendering” requirements can accidentally spread through the whole app.

## Decision

### 1) Single coordinator submits to the GPU queue

Within the canonical frame pipeline (ADR 0015):

- the platform runner owns **the act of calling** `queue.submit(...)` and presenting surfaces,
- the engine is allowed to **record** GPU work, but does not “own the queue”.

This keeps submission order deterministic and makes tracing/profiling uniform (ADR 0036).

### 2) Engine integration is a wgpu-facing API that lives outside `fret-core`

To preserve the “wgpu-free core” rule (ADR 0004 / ADR 0037):

- `fret-core` / `fret-ui` must not expose `wgpu` types,
- the engine render hook API lives in a wgpu-facing crate (e.g. `fret-render-wgpu` or a runner crate such as
  `fret-desktop-winit-wgpu`), which is already allowed to depend on `wgpu`.

### 3) Mainline integration shape: “record commands” + “update targets”

The engine-side contract must support these responsibilities:

- **Record GPU work for one frame** (commands that render into textures registered as `RenderTargetId`).
- **Maintain render target registrations** (create/update on resize; expose resolved views when MSAA is used).

The UI/runtime contract remains unchanged:

- UI emits `SceneOp::ViewportSurface { target: RenderTargetId, ... }` (ADR 0007),
- renderer resolves `RenderTargetId` → GPU resources (registry in `fret-render-*`).

#### “Embedded 3D viewport” is just a `RenderTargetId` (not a `wgpu::TextureView` in UI)

UI code should not receive or store `wgpu::TextureView`. The correct editor-grade contract is:

- the engine renders into its own textures using its own render graph/passes,
- the engine registers (or updates) the final sampled view as a `RenderTargetId` entry,
- the UI paints it via `SceneOp::ViewportSurface { target: RenderTargetId, ... }` (ADR 0007).

This keeps `fret-core` / `fret-ui` wgpu-free and allows multiple viewports across windows.

#### “Present ready” and frame-graph integration (P0 correctness)

The engine may have its own internal frame graph (multiple passes, multiple encoders).
Fret does not need to own that graph. The integration contract is:

- for a given `FrameId`, the engine returns **recorded** `wgpu::CommandBuffer`s that render into the
  textures registered as `RenderTargetId`,
- the runner submits engine command buffers **before** the UI command buffer (ADR 0015),
- the UI samples those textures in the same frame.

Because submission is centralized and the mainline contract assumes a single `wgpu::Queue` (ADR 0015),
submission order is sufficient to guarantee that the viewport texture is ready to sample.

If an engine wants to render “ahead” (decoupled simulation/render cadence), it should provide its own
double/triple-buffered viewport targets and update the registry to point at the latest completed frame
(future work, not required for P0).

### 4) Queue ownership rule (hard constraint)

In the mainline integration:

- the engine must not call `queue.submit(...)` for work that participates in Fret’s frame pipeline,
- the engine must not call “present” on platform surfaces (presentation is platform-runner owned),
- the engine may create GPU resources on the shared `Device` (pipelines, bind groups, textures, buffers).

If an engine requires full control of submission/present, it should host the runner itself and embed Fret as a UI
library (still compatible with ADR 0010, but a different app topology).

### 5) Frame identity is explicit for correctness and debugging

The engine render hook API must receive:

- `TickId` and `FrameId` (ADR 0034), to distinguish “event loop turns” from “actual renders/presents”,
- window/viewport identity and sizes (logical + physical) (ADR 0017 / ADR 0025),
- access to diagnostics hooks for tracing/metrics (ADR 0036).

## Implementation Notes

The current desktop glue (`crates/fret-launch`) implements the “record commands + update targets”
shape by extending the `WinitDriver` hook:

- The driver may implement `record_engine_frame(...) -> EngineFrameUpdate`.
- `EngineFrameUpdate` contains:
  - `target_updates: Vec<RenderTargetUpdate>` (explicit render target deltas),
  - `command_buffers: Vec<wgpu::CommandBuffer>` (engine work for this frame).
- The runner applies `target_updates` to the renderer’s render target registry **before** recording and submitting
  the UI command buffer, so UI sampling is correct in the same frame (ADR 0015).

This keeps the queue ownership rule intact while avoiding demo-only glue code that directly mutates the renderer
registry during interaction.

## Relationship to Vello (Reference Design)

Vello’s architecture separates:

- `Scene` → `Encoding` (a linearized buffer representation),
- `Encoding` → `Recording` (an ordered list of GPU-relevant commands),
- `Recording` → backend engine execution (currently `WgpuEngine`).

This is conceptually aligned with Fret’s “engine records, runner submits” rule:

- We can keep `fret-core::SceneOp` as the stable semantic contract (ADR 0002 / ADR 0009),
- and still adopt an internal “encoding/recording” separation inside `fret-render` to improve caching,
  debuggability, and testability, without changing queue ownership or submission ordering.

Important difference: Fret’s `SceneOp` stream must support strict interleaving across primitive kinds
(viewport surfaces, quads, text, clips) (ADR 0009), so Vello is not a drop-in UI renderer backend.
The practical reuse story is to treat Vello as an **offscreen texture producer** (icons/vector views),
then composite the resulting texture via Fret resource handles (`ImageId` / `RenderTargetId`).

## Consequences

- Correct-by-construction ordering: engine viewport writes happen before UI samples them (ADR 0015).
- Multi-window rendering remains debuggable because submit/present is centralized.
- The framework remains UI-focused: it enables engine editors without becoming an engine render framework itself.

## Alternatives Considered

### A) Engine owns the queue and calls `queue.submit` itself

Pros:

- maximal engine freedom.

Cons:

- makes UI/engine ordering and multi-window present correctness easy to break,
- makes observability and scheduling policies much harder to enforce consistently,
- increases the probability of a large integration rewrite.

### B) Shared queue ownership with informal “be careful” guidance

Pros:

- easy to start.

Cons:

- not stable at editor complexity; the first serious multi-window + overlays integration will force a redesign.

## Open Questions (To Decide Before Implementation)

### Locked P0 Choices

#### 1) API shape: engine returns recorded command buffers (runner submits)

The mainline API shape is:

- engine records GPU work using its own `wgpu::CommandEncoder`s,
- engine returns `Vec<wgpu::CommandBuffer>` for this frame,
- the runner submits them (engine work first, then UI) and presents (ADR 0015).

Rationale:

- minimal coupling: engines can keep their own render graph/encoder structure,
- deterministic ordering: submission remains centralized,
- avoids forcing an encoder factory into every engine integration.

#### 2) Target registry updates: push deltas (explicit updates)

Render target registrations are updated via explicit “delta updates” returned from the engine hook:

- create/update target entries when size/format/sample-count change,
- remove entries when a target is destroyed.

Rationale:

- avoids hidden “pull” queries across threads,
- makes resize behavior explicit and debuggable.

### Still Deferred (Not P0)

- Multi-queue engines: require explicit synchronization adapters (deferred).
- Headless/offscreen-only runners: compatible with the contract, but not required for P0.

## References

- Vello architecture overview (Scene → Encoding → Recording → Engine):
  - optional checkout at `repo-ref/vello` (see `docs/repo-ref.md`)
  - `repo-ref/vello/doc/ARCHITECTURE.md`
- Vello `Recording` command list (engine-facing execution plan):
  - `repo-ref/vello/vello/src/recording.rs`
