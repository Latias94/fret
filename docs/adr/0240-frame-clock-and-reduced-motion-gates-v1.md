# ADR 0240: Frame Clock and Reduced Motion Gates (v1)

Status: Proposed

## Context

Fret’s scheduling model is explicitly runner-owned and event-driven (ADR 0034). This is a strong
foundation for performance, but ecosystems quickly re-invent "time" in ad-hoc ways when building
animated components:

- using "number of paints" as time,
- storing timestamps in component-local statics,
- mixing wall-clock time reads into view building (breaking view-cache reuse),
- inconsistent handling of `prefers-reduced-motion` across components.

MagicUI-style components frequently need a stable, monotonic clock:

- shimmer / sparkle / noise drift,
- pointer-follow easing and spring-like motions,
- long-running ambient animations.

We want a contract that:

- provides a **monotonic, per-frame time base** that is consistent across native + wasm,
- makes reduced-motion a first-class gate (ADR 0232),
- keeps view-cache semantics predictable (no accidental “time becomes a dependency key”),
- supports diagnostics and scripted tests by making time *observable and controllable* in harnesses.

Related ADRs:

- Scheduling primitives: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- Environment queries (`prefers_reduced_motion`): `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
- Observability / diagnostics bundles: `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`,
  `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`

## Decision

### 1) Runner provides a monotonic frame clock snapshot (per window, per frame)

Each rendered frame has an associated clock snapshot:

- `frame_id: u64` (monotonic, increments per window)
- `now_monotonic: Duration` (monotonic time since an arbitrary origin; never wall-clock)
- `delta: Duration` (time since the previous rendered frame for that window; best-effort)

The exact origin of `now_monotonic` is runner-defined, but it MUST be monotonic and stable within a
single frame.

### 2) UI/runtime exposes frame clock reads as *non-reactive* values

`fret-ui` MUST expose the frame clock snapshot to widget contexts (e.g. `EventCx`, `PaintCx`,
`LayoutCx`) as a plain value read.

Crucially:

- Reading the frame clock MUST NOT participate in view-cache dependency tracking.
- Components must request frames explicitly (ADR 0034) when they want time to advance their own
  state.

Rationale:

- Time changes every frame; treating it as a reactive dependency would defeat caching and
  determinism.
- Scheduling is the explicit mechanism that “turns on time”.

### 3) Canonical animation pattern (mechanism vs policy)

Mechanism-level contract:

- A component that needs animation holds a `ContinuousFrames` lease or requests animation frames
  while active (ADR 0034).
- The component stores its own animation state (e.g. start time, phase, spring state) in
  element-owned state or a model.
- During each frame, the component advances that state using `frame_clock.delta` and triggers a
  repaint (`Invalidation::Paint`) as needed.

Policy-level guidance (ecosystem):

- `ecosystem/fret-ui-kit` should provide helpers for common interpolation/easing/spring patterns,
  but the runtime remains mechanism-only.

### 4) Reduced motion is a first-class gate

`prefers_reduced_motion` is provided via environment queries (ADR 0232).

Contract intent:

- Ecosystem components SHOULD treat reduced motion as "avoid continuous animation and large motion".
- Components SHOULD prefer:
  - disabling ambient animations (sparkle/shimmer),
  - snapping transitions (or using shorter, non-easing fades),
  - keeping user-driven motion (scroll, drag) intact.

The runtime does not enforce style; it provides the observable signal and consistent time base.

### 5) Diagnostics and scripted tests

For diagnostics and scripted interaction tests (ADR 0159):

- Bundles SHOULD record the per-window `frame_id` and a summarized clock snapshot.
- Harness runners SHOULD be able to run with a deterministic, fixed `delta` (test mode) to reduce
  flakiness in animation-driven screenshots and perf gates.

This may be implemented as a runner flag or a diag-mode override; it does not change the public
scene contracts.

## Consequences

- Ecosystem layers can implement MagicUI-like motion without abusing paint counts or wall-clock.
- View caching remains predictable: time reads do not poison cache keys.
- Reduced motion becomes consistent across components because the signal is centralized and the
  recommended response pattern is explicit.

## Non-goals

- This ADR does not standardize a full timeline/animation graph system.
- This ADR does not guarantee that `delta` is stable across background/foreground transitions;
  components must handle large deltas defensively.

