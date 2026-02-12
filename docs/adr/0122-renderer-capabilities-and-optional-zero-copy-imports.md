# ADR 0122: Renderer Capabilities and Optional Zero-Copy Imports (v1)

Status: Proposed

## Context

Fret targets desktop-first now and wasm/WebGPU later. Capabilities differ by platform and GPU:

- external texture import (e.g. WebCodecs/VideoFrame paths),
- supported texture formats (YUV planes, sRGB views),
- binding array support (batching improvements),
- readback performance constraints.

To keep the core contracts stable, we need a framework-owned way to:

- discover renderer/backend capabilities,
- enable optional fast paths without breaking portability,
- define deterministic fallbacks when capabilities are missing.

This ADR is about **capability negotiation** at the app/runner boundary. UI code must remain backend-agnostic.

## Decision

### 1) Define `RendererCapabilities` as a stable, serializable-ish struct (app-visible)

Expose a renderer capability snapshot to the app/runner layer (not `fret-core`):

- max texture size,
- supported sample counts,
- supported input pixel formats for streaming images (Rgba8, Nv12/I420 if available),
- support for external texture import (capability-gated),
- support for binding arrays / related advanced features (capability-gated),
- support for readback (and any known constraints).

Exact API surface and placement are implementation-defined (likely `crates/fret-render` + runner glue).

#### 1.2) Capability-gated GPU fast paths (non-zero-copy)

Some “fast paths” are not zero-copy but still reduce CPU work and/or upload bytes (e.g. GPU-assisted YUV conversion).
These paths must also be capability-gated and observable.

Example (informative):

- `streaming_images.nv12_gpu_convert`: upload NV12 planes and run a small GPU conversion pass into RGBA storage.

### 1.1) Capabilities must be observable and stable for a session

- Capabilities should be captured once per renderer initialization and treated as stable for the session.
- Capabilities should be included in renderer perf snapshots/debug dumps so regressions can be correlated with
  feature paths (e.g. binding arrays enabled vs disabled).

### 2) Capability-gated imports are opt-in and never leak raw backend handles into UI code

If external texture import is supported:

- apps may register/import an external texture as an `ImageId` source (or as a `RenderTargetId` update),
- UI references only `ImageId` / `RenderTargetId`.

If not supported:

- apps fall back to byte uploads (ADR 0119), with budgets/backpressure (ADR 0121).

### 3) Deterministic fallback rules

Capabilities must be used only to select between:

- “fast path” (zero-copy or GPU-native),
- “portable path” (byte upload),
- and “unsupported” (explicit error).

Fallback selection must be deterministic for a given platform/backend configuration.

Implementation note (informative):

- Runners may expose a config switch to enable an experimental capability-gated fast path, but the fast path must
  still be conditioned on the capability snapshot. Debug env overrides may exist, but should not be required for
  normal operation.

## Crate Placement and Implementation Sketch (Non-normative)

- `crates/fret-render`:
  - computes and exposes `RendererCapabilities` (backend-facing).
- `crates/fret-launch` (runner glue):
  - reads capabilities and selects ingestion/capture strategies (portable vs fast path),
  - reports capability-driven fallbacks in logs/diagnostics.
- Apps:
  - may branch on capabilities, but must remain correct under the portable path.

## Consequences

- wasm and desktop can share the same UI/component code while using different ingestion paths.
- Optional fast paths do not cause silent drift: missing capabilities are explicit and handled.

## Validation / Acceptance Criteria

Implementation is considered conformant when:

- Capability snapshots are stable for a renderer session and observable in debug/perf dumps.
- For missing capabilities, the system deterministically falls back to the portable path (or explicit error).
- UI code does not require backend handles to use capability-gated paths (only `ImageId`/`RenderTargetId`).

## References

- Streaming image ingestion: `docs/adr/0119-streaming-images-and-video-surfaces.md`
- Upload budgets/backpressure: `docs/adr/0121-streaming-upload-budgets-and-backpressure-v1.md`
- Offscreen capture/readback: `docs/adr/0120-offscreen-rendering-frame-capture-and-readback.md`
- Extensibility (capability-gated shaders): `docs/adr/0123-renderer-extensibility-materials-effects-and-sandboxing-v1.md`
