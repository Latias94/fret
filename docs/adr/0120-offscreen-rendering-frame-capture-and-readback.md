# ADR 0120: Offscreen Rendering, Frame Capture, and Readback (v1)

Status: Proposed

## Context

Editor-grade apps frequently need to turn UI rendering into data:

- screenshots (user-facing),
- recording UI previews / tutorials (app-owned encoding),
- golden-image conformance testing (framework-owned harnesses),
- exporting thumbnails for asset browsers,
- remote collaboration previews (encode frames elsewhere).

Fret must support these without violating core invariants:

- `Scene.ops` order is authoritative (ADR 0002 / ADR 0009),
- compositing is linear and explicit (ADR 0040),
- queue submission is coordinated by the runner (ADR 0038),
- `fret-core` / `fret-ui` remain `wgpu`-free (ADR 0092).

The framework should not own:

- video encoding formats/codecs,
- disk I/O policies,
- network streaming.

It should provide a **portable, bounded, observable** mechanism for:

- rendering a window/scene into an offscreen target,
- optionally reading it back to CPU memory.

## Decision

### 1) Capture is requested via effects and delivered via events

Frame capture is expressed as an `Effect` (app/runner drained, ADR 0001 / ADR 0034):

- `Effect::FrameCaptureRequest { window: AppWindowId, token: FrameCaptureToken, options: FrameCaptureOptions }`

When capture completes (same tick or later), the runner delivers:

- `Event::FrameCaptured { token: FrameCaptureToken, frame: CapturedFrame }`
- or `Event::FrameCaptureFailed { token: FrameCaptureToken, message: String }`

The token is app-owned (like other I/O tokens) to keep the UI runtime backend-agnostic.

### 2) Capture source: display-referred output of the UI compositor

Capture semantics:

- capture the window’s final composed UI output for a specific `FrameId` (ADR 0034),
- include viewport surfaces as they are composited (as-visible), not raw engine buffers.

This makes capture results meaningful for screenshots and conformance tests.

### 3) Output formats: raw pixels first, encoding is app-owned

`CapturedFrame` v1 should provide raw pixel bytes, plus metadata:

- dimensions (px),
- row stride,
- pixel format (baseline: RGBA8),
- color encoding tag (baseline: sRGB bytes).

Encoding to PNG/MP4/etc is explicitly out of scope for the framework.

### 4) Offscreen rendering is a renderer capability, not a UI semantic

Offscreen targets used for capture are renderer-owned:

- capture may render into an intermediate texture and then read back,
- or reuse an existing output texture if safe (implementation-defined).

No new `SceneOp` semantics are required for capture.

### 5) Backpressure and budgets are mandatory

Frame capture requests are potentially expensive (GPU→CPU readback).

v1 requirements:

- requests must be coalescible by `(window, token)` with latest-wins semantics (ADR 0125),
- the runner must enforce bounded in-flight captures per window,
- if the system is overloaded, requests may be delayed or failed deterministically.

This keeps editors responsive under stress.

### 6) Web/wasm constraints are explicit

On wasm/WebGPU, readback may be slow or constrained. The contract remains the same:

- request via effect,
- deliver result via event (possibly later),
- allow failure with a clear error message.

Apps should treat capture as an optional capability and degrade gracefully.

## Crate Placement and Implementation Sketch (Non-normative)

This section describes a likely implementation placement consistent with ADR 0092.

- **Effect + event surface**
  - Define request effects in `crates/fret-runtime` (effect enum).
  - Define capture result events in `crates/fret-core` (`Event`), keeping payload types `wgpu`-free.
- **Runner**
  - `crates/fret-launch` drains capture effects, enforces backpressure (bounded in-flight), and coordinates
    submission and readback timing.
  - The runner should prefer to capture a specific `FrameId` (ADR 0034) for determinism.
- **Renderer**
  - `crates/fret-render` owns offscreen target allocation and readback implementation details:
    - render-to-texture,
    - optional format conversion for the requested output,
    - GPU→CPU transfer using the backend’s supported mechanism.

Note: capture should not require any new `SceneOp` semantics; it is a renderer/runner feature layered on top of
existing composition.

## Consequences

- Apps can implement screenshots, recording, thumbnails, and remote previews without owning GPU submission.
- The framework can build golden-image tests without special-case hooks in UI code.
- Readback costs are bounded and observable.

## Follow-up Work

- Define `FrameCaptureOptions` in detail (clip rects, scaling, alpha policy, color encoding).
- Lock options and determinism vocabulary:
  - `docs/adr/0125-frame-capture-options-and-determinism-v1.md`
- Define capture budgets and diagnostics hooks.
  - Related: intermediate budgets (ADR 0118) and streaming upload backpressure (ADR 0121).
- Add a headless/offscreen test harness that validates ordering and effect degradation interactions.

## References

- Submission coordinator: `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- Scheduling + frame identity: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- Color/compositing: `docs/adr/0040-color-management-and-compositing-contracts.md`
- Upload backpressure (related): `docs/adr/0121-streaming-upload-budgets-and-backpressure-v1.md`
