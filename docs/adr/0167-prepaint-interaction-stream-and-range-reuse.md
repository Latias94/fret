# ADR 0167: Prepaint Interaction Stream + Range Reuse (GPUI-Aligned)

Status: Proposed

## Context

Fret already defines paint-stream range replay caching as a framework-level optimization:

- record a subtree’s paint output as a contiguous range,
- on cache hit, replay the range without recomputing paint (ADR 0055).

For editor-grade UI correctness and tooling, “paint output” alone is not sufficient:

- hit testing requires stable interaction geometry (hitboxes / pointer regions),
- overlays and dismissal need outside-press observation,
- accessibility requires a semantics snapshot (ADR 0033),
- diagnostics and scripted tests need deterministic “what was interactive this frame” data (ADR 0159).

GPUI addresses this by separating phases and reusing ranges for more than just paint:

- `request_layout` establishes geometry,
- `prepaint` prepares interaction and render-time state,
- `paint` emits draw calls,
- cached views reuse recorded prepaint/paint ranges when not dirty.

This ADR locks a similar contract for Fret’s runtime, while respecting existing layering decisions.

## Decision

### 1) The frame recording has multiple ordered streams

Fret defines a **Frame Recording** as a set of ordered streams (ADR 0055), with P0/P1 staging:

- **Paint stream**: scene ops (already implemented).
- **Interaction stream**: hit regions, pointer listeners, cursor requests, outside-press observers, and any data
  required for deterministic routing in the next tick.
- **Semantics stream**: accessibility semantics snapshot/diff (ADR 0033).

Each stream MUST support “contiguous range contribution per subtree”, enabling range reuse.

### 2) Introduce/standardize a prepaint phase for interaction construction

Fret standardizes a `prepaint` phase whose responsibilities include:

- building the interaction stream for the current frame,
- producing any paint-preparation state that must be ready before paint (e.g. resolved hover state).

The paint phase should be free to assume that interaction geometry is already established, aligning with “no
side-effects in paint other than recording output”.

### 3) Cache hits reuse *all* relevant ranges

When a cache root is clean (ADR 0213) and caching is enabled:

- the runtime MAY skip subtree execution and reuse prior mounted nodes,
- the runtime MUST reuse the cached subtree’s interaction stream range and paint stream range,
- semantics stream reuse is allowed only when the semantics contract is satisfied (e.g. semantics snapshot is still
  valid for the cache key inputs and there are no semantics-affecting dirty reasons).

This avoids the failure mode where paint is reused but hitboxes/semantics are stale or missing.

### 4) Inspection/picking disables caching

When the inspector/tooling is actively picking or inspecting:

- caching MUST be disabled for the inspected window so that hitboxes and semantics are complete and current.

This matches GPUI practice and is required for trustworthy tooling.

## Consequences

- Cached subtree reuse becomes “closed-loop” across rendering, interaction, semantics, and diagnostics.
- The runtime gains a clean place to implement hover/focus pseudoclass tracking without forcing layout churn (ADR 0166).

## Relationship to existing ADRs

- Extends the reserved “multi-stream recording” direction in ADR 0055 by locking a minimum interaction-stream
  vocabulary and prepaint responsibilities.
- Builds on cache-root attribution and nested cache root correctness rules from ADR 0213.

## References

- Zed/GPUI prepaint and range reuse:
  - `repo-ref/zed/crates/gpui/src/element.rs` (element lifecycle includes `prepaint`)
  - `repo-ref/zed/crates/gpui/src/view.rs` (`AnyView::cached`, `reuse_prepaint`, `reuse_paint`, dirty view checks)
- Fret caching and cache root semantics:
  - ADR 0055: `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`
  - ADR 0213: `docs/adr/0213-cache-roots-and-cached-subtree-semantics-v1.md`

