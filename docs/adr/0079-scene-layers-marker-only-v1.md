# ADR 0079: Scene Layers (Marker-only v1)

Status: Proposed

## Context

ADR 0019 reserves `PushLayer / PopLayer` as part of the `Scene` state stack.

Layers are an attractive hook for batching and caching, but if their semantics are left ambiguous they
quickly become a source of contract drift:

- Some producers may treat `layer` as a z-index / sorting key.
- Some renderers may treat `layer` as an implicit offscreen group (isolated compositing).
- Some components may accidentally depend on implementation-defined layer behavior.

Fret’s core display list contract requires **strict in-order compositing** (ADR 0002 / ADR 0009), and
the engine viewport embedding contract depends on it. We need a v1 definition that is safe to lock
before scaling component surface area.

## Decision

### 1) `PushLayer / PopLayer` are range markers, not a sorting key

`PushLayer { layer }` begins a *layer range*; `PopLayer` ends it.

The `layer: u32` value is a **tag** (for debugging and profiling), not a z-index. Renderers must not
reorder operations based on `layer`.

### 2) Layers do not change compositing semantics (no implicit offscreen)

In v1, a layer range:

- does not imply an isolated compositing group,
- does not change alpha blending rules,
- does not imply an offscreen render target,
- does not change clip/transform semantics.

If Fret needs isolated compositing (opacity groups, filters, backdrop blur), it must be expressed
via dedicated ops (future ADR), not by overloading `PushLayer`.

### 3) What layers are allowed to influence (perf/debug only)

Renderers are allowed to treat layer ranges as **implementation boundaries** that do not affect
visible output, such as:

- forcing a batch flush,
- segmenting internal encodings/recordings,
- attaching debug labels and timing zones,
- creating cache keys or replay segments (ADR 0055),
- collecting stats (draw call counts, clip depth, atlas usage).

### 4) Producer guidance (recommended usage)

Producers may use layer ranges to communicate intent and improve debuggability, for example:

- base UI roots vs overlay roots (ADR 0011),
- drag-preview or transient overlay composition,
- viewport overlay UI vs viewport surface content.

However, producers must continue to rely on **scene op order** for correctness.

## Consequences

- `PushLayer` is safe to use early for debugging, profiling, and cache segmentation.
- Order correctness remains locked by ADR 0009 and cannot be accidentally weakened by “layer sorting”.
- Future “isolated groups / effects” can be added without breaking existing layer usage.

## Future Work

- Define whether and how layer ranges should participate in conformance testing (e.g. “must flush” is
  not required, but “must not reorder” is).
- Define explicit isolated group/effect ops if needed (opacity groups, filters, backdrop blur).

