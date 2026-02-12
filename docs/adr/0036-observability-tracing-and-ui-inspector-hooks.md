# ADR 0036: Observability (Tracing, Frame Markers, and UI Inspector Hooks)

Status: Accepted

## Context

Fret aims to support complex editor UIs where correctness and performance issues can be subtle:

- draw-order bugs (viewport overlays, popups),
- focus/IME edge cases,
- multi-window scheduling and idle CPU usage,
- text atlas cache churn,
- GPU submission stalls.

Without a stable observability strategy, teams often add ad-hoc logging and custom profiling hooks that later
become incompatible or too noisy.

References:

- Threading/logging/error strategy:
  - `docs/adr/0008-threading-logging-errors.md`
- Frame pipeline and submission ordering:
  - `docs/adr/0015-frame-lifecycle-and-submission-order.md`
- Renderer ordering constraints:
  - `docs/adr/0009-renderer-ordering-and-batching.md`

## Decision

### 1) Adopt structured tracing as the primary instrumentation mechanism

Use a structured tracing framework (e.g. `tracing`) across crates with consistent spans and events.

Define canonical spans for each frame/tick (aligned with ADR 0015 phases):

1. platform events → core events
2. app/model updates
3. UI build/layout/paint
4. scene finalize
5. renderer prepare/submit
6. present
7. effects drain (fixed-point loop)

### 2) Introduce a stable frame identifier and window markers

Expose:

- `FrameId` (from ADR 0034),
- `AppWindowId`,
- optional “reason” markers for redraw (input, timer, animation, engine viewport, etc.).

These become the backbone for correlating CPU and GPU work.

### 3) Provide debug/inspector hooks (framework-level, optional at runtime)

Define optional hooks for a UI inspector overlay/tooling:

- hover node path (retained: `NodeId`; declarative: `GlobalElementId`),
- focus/capture state,
- overlay root stack (ADR 0011),
- layout bounds visualization,
- semantics tree visualization (ADR 0033).

These hooks must be:

- disabled by default in production,
- cheap when disabled (no heavy string allocation).

Follow-up (tooling contract, non-normative):

- The concrete, versioned “diagnostics bundle” shape and scripted interaction test harness are defined in
  `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`.

### 4) Renderer metrics are standardized

Define a stable set of renderer metrics that can be exported for profiling:

- primitive counts by kind,
- batch counts and batch-break reasons (clip changes, texture changes),
- atlas stats (allocations, evictions, upload bytes),
- frame-in-flight stats (buffers/textures).

The exact metric plumbing is internal to `fret-render`, but the conceptual contract is stable.

Defaults (P0):

- GPU timestamps are **optional** (feature-gated and capability-checked). CPU-side tracing spans and `FrameId` are mandatory.
- Trace volume is controlled by:
  - a small fixed set of canonical spans always available,
  - optional sampling for high-frequency events,
  - optional ring-buffer capture for inspector tooling.
- Public observability surface is a small, versioned data shape:
  - `ObservabilityVersion`,
  - `FrameId`, `AppWindowId`,
  - per-frame counts (scene ops by kind, batches, batch-break reasons),
  - atlas stats (alloc/evict/upload bytes),
  - frame-in-flight stats (buffers/textures).
  Everything else remains internal debug-only.

## Consequences

- Performance regressions and correctness issues become debuggable early.
- The project can evolve rendering/text/layout systems without losing visibility.
- Tooling (inspector, diagnostics panels) can be built as example apps without entangling core crates.

## Open Questions (To Decide Before Implementation)

### Locked P0 Choices

1) **Metrics naming**: Rust structs as the canonical contract.
   - Metrics are exposed as a small, versioned Rust data shape (as described in “Defaults (P0)”).
   - Optional serialization (JSON) is allowed for tooling, but the Rust struct is the source of truth.

2) **Inspector transport**: in-process only for P0.
   - Inspector/diagnostics are exposed via in-process hooks and a ring-buffer capture mechanism.
   - Debug IPC transport is deferred until a concrete need appears.

Additional locked behavior:

- Every render submission that increments `FrameId` must emit a tracing span boundary and attach `FrameId` + window markers.
- GPU timestamps remain feature-gated and capability-checked; CPU spans are always available.
