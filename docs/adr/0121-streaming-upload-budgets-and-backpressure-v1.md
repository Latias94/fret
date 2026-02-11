# ADR 0121: Streaming Upload Budgets and Backpressure (v1)

Status: Proposed

## Context

Streaming image updates (ADR 0119) enable video playback UIs, remote-desktop previews, camera feeds, and
scrubbable timeline thumbnails without framework-owned decoding.

Even with “latest-wins” coalescing, streaming frames can overload systems in two distinct ways:

1) **Upload bandwidth**: CPU→GPU transfers (queue writes / staging buffers) can saturate the GPU queue.
2) **Staging memory**: temporarily buffered frame bytes can spike memory usage if producers outpace consumption.

These are separate from intermediate texture budgets for postprocessing passes (ADR 0118).

We need deterministic guardrails that keep the UI responsive while preserving correctness.

## Decision

### 1) Budget scope: per-window upload budgets with optional per-stream caps

The runner/renderer enforces upload budgets per window:

- max bytes uploaded per rendered frame (normative for v1),
- max staging bytes retained for pending updates.

Optionally, apps may configure per-stream caps for fairness (implementation-defined in v1).

Note:

- If partial (dirty-rect) updates are used (ADR 0119), budget accounting should track the actual bytes uploaded,
  not the full logical image size.

### 2) Backpressure strategy: latest-wins + bounded staging + deterministic drop policy

v1 requirements:

- Coalesce updates per streaming image generation `(ImageId, stream_generation)` (ADR 0124 / ADR 0119).
- Bound pending staging memory:
  - if exceeded, drop older pending updates first,
  - keep the most recent update when possible (latest-wins).
- If uploads exceed per-frame/per-second budgets, prefer:
  1) delay applying updates until the next frame,
  2) drop intermediate updates (keep newest),
  3) as a last resort, refuse updates with a reported error (deterministic).

### 2.1) Alignment and repacking are expected (and must be budgeted)

GPU upload APIs commonly require row alignment (e.g. `bytes_per_row` alignment).
Therefore:

- the ingestion contract must support stride/`bytes_per_row` (ADR 0119),
- the runner/renderer may repack rows into an aligned staging buffer,
- repacking cost and staging memory must be accounted for under these budgets.

### 3) Layout invariants

Any dropping/delay is purely visual freshness. It must not affect:

- layout,
- hit-testing,
- input routing,
- event ordering within the app.

### 4) Observability

The system must expose counters (debug/perf snapshot):

- bytes uploaded per frame/window (both budgeted and actual),
- dropped update count per image/window,
- peak staging bytes,
- time-to-present for the latest frame (optional).

Implementation note:

- The budget decision often needs a conservative estimate (e.g. assume YUV updates expand to RGBA for budgeting).
  Therefore observability should report *both*:
  - `upload_bytes_budgeted`: the estimate used for budget decisions, and
  - `upload_bytes_applied`: the actual CPU->GPU upload bytes performed by the chosen apply path.

### 5) Web/wasm constraints

On wasm, CPU→GPU uploads can be significantly slower and are often subject to browser throttling.
The budget/backpressure contract remains the same:

- keep queues bounded,
- prefer latest-wins,
- allow deterministic degradation (drop/delay) over unbounded buffering.

## Consequences

- Streaming surfaces become safe to use broadly (video, remote, camera) without risking unbounded memory growth.
- Behavior remains deterministic and debuggable under load.

## Validation / Acceptance Criteria

Implementation is considered conformant when:

- Pending update queues remain bounded by the staging budget under adversarial producers (no unbounded growth).
- Coalescing is latest-wins per `(ImageId, stream_generation)` (ADR 0124), and behavior is deterministic.
- Upload byte accounting reflects actual uploaded bytes (including repacking/alignment overhead).
- Debug/perf snapshots expose: bytes uploaded per frame/window, dropped update counts, and peak staging bytes.

## References

- Streaming image semantics: `docs/adr/0119-streaming-images-and-video-surfaces.md`
- Intermediate budgets (postprocessing): `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`
