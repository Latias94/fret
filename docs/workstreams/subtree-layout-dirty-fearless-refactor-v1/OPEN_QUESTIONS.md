# Open questions

This document records unresolved design questions and a recommended direction for v1.

## 1) Propagation strategy across view-cache roots

Should aggregation deltas always propagate to the root immediately, or should we:

- propagate eagerly to the nearest view-cache root, then
- propagate to ancestors in a deferred repair pass?

Trade-off:

- eager-to-root is simplest but risks higher per-keystroke overhead
- deferred propagation is more complex but keeps the “truncation performance intent”

### Recommendation (v1)

Adopt a **two-tier propagation strategy**:

1. **Eager propagation within the current cache root** (walk parent pointers until the nearest
   `ElementKind::ViewCache` boundary or tree root).
2. **Deferred propagation across cache roots** via a once-per-frame “repair” pass that applies
   accumulated deltas to ancestors above the cache root.

Rationale:

- Preserves the performance intent of truncation while still making subtree-dirty observable at
  higher levels (correctness for cache reuse decisions).
- Keeps the hot-path update cost bounded by the local cache-root height (common case).

Practical v1 staging:

- Land the mechanism behind a runtime flag and ship **eager-to-root** first if it is simpler to
  implement and measure.
- If metrics show parent-walk cost is high in real workloads, switch to the two-tier design without
  changing the consumer API.

## 2) Counter vs epoch

Alternative designs:

- `subtree_layout_dirty_count` (counter, incremental)
- `subtree_layout_dirty_epoch` (monotonic max across children, resets via per-frame stamping)

Counters are straightforward but require correct decrement paths.
Epochs avoid decrements but require per-frame stamping and can be harder to reason about under
view-cache reuse.

### Recommendation (v1)

Use **`subtree_layout_dirty_count: u32`** as the primary aggregation structure.

Rationale:

- Clear semantics and O(1) query for consumers.
- Supports nested dirty/clean transitions without recomputation.
- Easy to validate in debug builds by recomputing counts for a subtree and comparing.

When to revisit:

- If the long-term authoring model moves to “rebuild element tree every frame” and invalidation
  becomes largely implicit, an **epoch/stamp** approach may become simpler than decrement bookkeeping.
  That would be a v2 internal swap while keeping `subtree_layout_dirty(node) -> bool` stable.

## 3) Parent pointer correctness

Aggregation updates rely on walking parent pointers. If parent pointers can be stale in rare cases,
we may need:

- periodic repair passes (already exist in some subsystems), or
- a fallback path for aggregation repair (diagnostic-only).

### Recommendation (v1)

Treat “parent pointers are correct” as a **hard `UiTree` invariant**. Do not complicate the primary
update path to tolerate stale pointers.

Instead, add debug-only drift detection:

- a diagnostic (or targeted debug asserts) that recomputes subtree counts and reports mismatches with
  evidence anchors (node ids, element kinds, parent ids).

This makes pointer bugs loud and actionable rather than silently papered over.

## 4) Consumer semantics

Do consumers want:

- “is my subtree dirty?” (any descendant)
- “is my direct child root dirty?” (current behavior)

For scroll correctness, the answer is clearly the subtree query at the edge. For other systems we
should be explicit and avoid accidental churn.

### Recommendation (v1)

Keep consumer semantics explicit via two internal queries:

- `self_layout_dirty(node) -> bool` (current invalidation bit)
- `subtree_layout_dirty(node) -> bool` (aggregation: “any descendant needs layout”)

Guidance:

- Consumers that decide whether to **reuse cached measurement/extent** should consult
  `subtree_layout_dirty(...)`.
- The aggregation is a **signal**, not an invalidation; consumers should not “force mark dirty” as a
  workaround once the aggregation is in place.
