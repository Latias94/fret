# ADR 0081: `DrawOrder` Is Non-semantic (v1)

Status: Accepted

## Context

`fret-core::SceneOp` includes a `DrawOrder` field on multiple draw primitives.

However, Fret’s rendering contract is explicitly **in-order** (ADR 0002 / ADR 0009): `Scene.ops`
order is authoritative and must be preserved across primitive kinds.

If `DrawOrder` is treated as a sorting key, it will immediately cause contract drift:

- components will start using `DrawOrder` as a z-index,
- renderers may start sorting/bucketing across non-adjacent ops,
- viewport overlays and modal surfaces will break.

ADR 0009 already states `DrawOrder` is not a general-purpose sorting key, but does not fully lock what
it *is* for. This ADR fixes that ambiguity.

## Decision

### 1) `DrawOrder` must not affect compositing order

- Renderers must treat `Scene.ops` ordering as authoritative.
- Renderers must not reorder draw ops based on `DrawOrder`.
- Producers must not rely on `DrawOrder` to change layering.

### 2) `DrawOrder` is a legacy/debug field in v1

In v1, `DrawOrder` is considered **non-semantic**:

- It may be used for debugging/profiling instrumentation (labels, counters).
- It may be used as part of cache keys/fingerprints to avoid accidental cache collisions.
- It must not change visible output when `Scene.ops` order is unchanged.

### 3) Producer guidance

Producers should generally use `DrawOrder(0)` for all primitives.

If a producer wants different layering, it must express it by emitting ops in the correct order
(or by using explicit state ops like `PushLayer` markers; ADR 0079).

## Consequences

- The ordering contract stays simple and robust for editor-grade overlay composition.
- We retain `DrawOrder` for compatibility while preventing it from becoming an implicit z-index.

## Future Work

- Consider removing `DrawOrder` from the public `SceneOp` contract in a future breaking revision, once
  downstream code no longer depends on it.

