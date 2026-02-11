# ADR 0076: Declarative Layout Performance Hardening (Persistent Taffy Trees)

Status: Accepted (incremental rollout; steps 1–3 implemented)

Update (2026-01-13):

- ADR 0113 introduces constraint-correct `AvailableSpace` and a non-reentrant intrinsic measurement path, so Taffy
  measure callbacks must not re-enter subtree layout.
- ADR 0114 describes a window-scoped layout engine with per-viewport roots (for multi-viewport docking) and is enabled
  by default in this repository.
- The repository no longer implements the container-owned persistent `TaffyTree` integration shape described here by
  default. This ADR is retained for historical context; its incremental-update and caching principles still apply to the
  window-scoped engine end-state.

## Context

Fret's declarative layout semantics are defined in:

- ADR 0035: hybrid layout + optional Taffy integration (Taffy is an internal algorithm)
- ADR 0057: declarative `LayoutStyle` + Flex/Grid semantics (Taffy-backed)
- ADR 0066: keep `fret-ui` runtime contract surface small and optimizable

Today, `Flex` and `Grid` containers in `crates/fret-ui` use `taffy::TaffyTree` and call
`compute_layout_with_measure(...)`. This is the correct high-level approach, but there are two
performance traps that show up quickly as the component surface grows:

1. **Repeated measure calls**: during a single layout solve, Taffy may invoke the measure callback
   multiple times for the same child and constraint tuple (`known` + `avail`). If the callback
   re-enters `UiTree::layout_in(...)`, the runtime can end up doing redundant recursive layouts.
2. **Rebuilding the Taffy tree too often**: when the child list changes, some containers rebuild a
   fresh `TaffyTree`. This throws away node allocations and forces the solver to re-discover
   structure every frame during dynamic UIs (menus, lists, virtualized rows).

GPUI (Zed) demonstrates Taffy as a practical choice, and also uses per-package dev optimization
overrides for Taffy to keep iteration fast.

We want the "clean state" end goal:

- Declarative layout is **incremental**: stable identity → update styles/children → mark dirty
  precisely → compute → write back rects.
- Runtime hot paths stay deterministic and small; component policy remains outside runtime (ADR 0074).

## Decision

We adopt **Scheme C** as the long-term implementation strategy for declarative Flex/Grid layout in the container-owned
integration shape, while allowing a later evolution toward a window-scoped engine (ADR 0114):

Note: ADR 0113 refines the measurement surface by introducing an explicit `AvailableSpace`
(Definite/MinContent/MaxContent) model and a non-reentrant intrinsic measurement path.
ADR 0114 proposes a later evolution that generalizes these persistence/incremental-update principles
into a window-scoped layout engine + viewport roots model for multi-viewport docking.
ADR 0076 remains the accepted near-term performance hardening strategy and the current implementation
shape for `crates/fret-ui`.

### C1) Persistent container-owned Taffy trees (stable identity)

Each `Flex` / `Grid` container owns a persistent `TaffyTree` in element/widget state (ADR 0035),
and maintains a stable mapping:

- `UiTree` child `NodeId` → `taffy::NodeId`

When children change, the container updates the mapping and updates the root's children list rather
than rebuilding the entire `TaffyTree`.

### C2) Incremental updates and precise dirtiness

On each layout pass:

- Only call `set_style(...)` for children whose `LayoutStyle` (or relevant derived fields) changed.
- Only call `mark_dirty(...)` when inputs changed that affect layout.
- Avoid O(n) rebuild patterns for common UI churn (insert/remove/reorder).

### C3) Measure callback memoization (within a solve)

Within a single `compute_layout_with_measure(...)` call, memoize measurement results by:

- `(child, known_w, known_h, avail_w, avail_h)` → `Size`

This prevents repeated recursive `layout_in(...)` calls for identical constraints during one solve.

### C4) Dev build ergonomics

Apply a per-package dev profile override:

- `[profile.dev.package.taffy] opt-level = 3`

This keeps iteration speed acceptable while still running the rest of the workspace in debug.

## Consequences

### Benefits

- Significant reduction in redundant layout work for complex component trees.
- Lower allocation churn and more stable frame times during UI churn.
- Clear path to further optimizations (leaf measurement specialization, style diff hashing).

### Costs / Risks

- More stateful layout caches inside runtime containers; requires careful invalidation rules.
- Mapping child identity to Taffy nodes must handle reorder and removal correctly.
- Virtualization constraints remain: Taffy must never require unbounded children (ADR 0042).

## Rollout Plan

1. Land dev profile override for Taffy. (done)
2. Land per-solve measure memoization in Flex/Grid. (done)
3. Refactor container caches to avoid rebuilding `TaffyTree` on child list changes: (done)
   - introduce `HashMap<NodeId, TaffyNodeId>` mapping
   - update root children list incrementally
4. Add profiling counters / debug stats to validate reductions in:
   - measure callback invocations
   - `layout_in` calls per frame

## References

- ADR 0035: Layout Constraints and Optional Taffy Integration
- ADR 0057: Declarative Layout Style and Flex Semantics
- ADR 0042: Virtualization and Large Lists (no unbounded children in layout engines)
- GPUI uses Taffy internally: `repo-ref/zed/crates/gpui/src/taffy.rs` (reference)
