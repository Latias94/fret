# ADR 0127: Frame Capture Options and Determinism (v1)

Status: Proposed

## Context

ADR 0122 defines a portable “request capture via effects, deliver via events” contract for screenshots,
recording, thumbnails, and golden tests.

To make capture usable across:

- user-facing screenshots,
- deterministic golden-image tests,
- remote preview frames,

we must lock:

- the capture option vocabulary,
- what “the captured frame” means (which frame, which color encoding),
- determinism and backpressure expectations,
- wasm constraints and failure modes.

## Decision

### 1) Define `FrameCaptureOptions` (v1)

`FrameCaptureOptions` is a portable struct (no backend types) carried in the capture request effect.

Proposed fields:

- `source: FrameCaptureSource`
- `region: Option<FrameCaptureRegion>`
- `scale: FrameCaptureScale`
- `output: FrameCaptureOutput`
- `determinism: FrameCaptureDeterminism`

#### 1.1) Source selection

- `FrameCaptureSource::WindowCompositedOutput { window: AppWindowId }` (v1 default)

Future reserved:

- capture a specific `RenderTargetId` (engine viewport raw output) for debugging tools.

#### 1.2) Region

Region is optional and is expressed in **logical pixels** by default:

- `FrameCaptureRegion::LogicalRect(Rect)`

Future reserved:

- physical-pixel rects (for pixel-perfect tooling),
- element/overlay-specific capture (semantic selection).

#### 1.3) Scale

- `FrameCaptureScale::WindowScaleFactor` (default)
- `FrameCaptureScale::Fixed(f32)` (e.g. 1.0 for stable goldens)
- `FrameCaptureScale::Fit { max_width_px, max_height_px }` (thumbnails)

#### 1.4) Output format

`FrameCaptureOutput` v1:

- `pixel_format: CapturedPixelFormat` (baseline: `Rgba8`)
- `color_encoding: CapturedColorEncoding` (baseline: `SrgbBytes`)
- `alpha: CapturedAlpha` (`Opaque` or `Include`)

Encoding (PNG/JPEG/MP4) remains app-owned.

### 2) Determinism policy

Capture has two main use cases:

- “as presented” screenshots,
- deterministic goldens.

Define:

- `FrameCaptureDeterminism::AsPresented`
  - captures whatever is on the next presented `FrameId`.
- `FrameCaptureDeterminism::Deterministic {`
  - `scale: Option<f32>` (recommended `1.0`),
  - `disable_nondeterministic_effects: bool,`
`}`

Non-deterministic effects include:

- time-based noise/grain,
- randomized particles,
- device-dependent dithering.

This does not require the framework to “freeze time” globally; it requires that effect implementations provide a
deterministic mode when requested (future work; tied to `RenderPlan` execution).

### 3) Backpressure and bounded in-flight captures

Capture readback is expensive; v1 requires:

- bounded in-flight captures per window (ADR 0122),
- deterministic failure or delay behavior,
- observability counters in debug/perf snapshots.

### 4) Relationship to budgets and degradations

By default, capture reflects the **actual composed output**, including any budget-driven degradations (ADR 0120 / ADR 0123).

For deterministic goldens, apps may choose:

- to configure generous budgets (recommended for test harnesses),
- and/or request deterministic capture mode, which may force stricter fallbacks over device-dependent paths.

### 5) wasm constraints

On wasm, readback can be slow or unsupported depending on backend capabilities (ADR 0124).

The contract remains:

- request capture,
- either receive a frame later or receive a failure event.

## Consequences

- Capture becomes a stable foundation for tests, thumbnails, and screenshot/recording features.
- Deterministic goldens become feasible without hard-coding platform quirks into tests.

## References

- Capture base contract: `docs/adr/0122-offscreen-rendering-frame-capture-and-readback.md`
- Budgets/degradation: `docs/adr/0120-renderer-intermediate-budgets-and-effect-degradation-v1.md`,
  `docs/adr/0123-streaming-upload-budgets-and-backpressure-v1.md`
- Capabilities: `docs/adr/0124-renderer-capabilities-and-optional-zero-copy-imports.md`

