# fret-ui Layout Architecture Inventory

Date: 2026-05-18
Task: FLA-010

## Summary

The current `fret-ui` layout architecture is complex, but the complexity is mostly attached to
real contracts: retained tree layout, Taffy-backed flow roots, explicit barrier roots, resize-time
geometry propagation, and diagnostics attribution.

The main architecture smell is not the model itself. The smell is that
`crates/fret-ui/src/tree/layout/node.rs` now contains both the ordinary widget layout execution path
and the clean-geometry proof model. That makes future audits harder even when the code is correct.

Initial recommendation: do not redesign the model yet. First consider a behavior-preserving
organization split if the next audit step confirms that reviewability is the limiting factor.

## File map

Current line counts:

| File | Approx lines | Role |
| --- | ---: | --- |
| `crates/fret-ui/src/tree/layout/node.rs` | 2912 | Per-node layout execution plus clean-geometry propagation/proof helpers. |
| `crates/fret-ui/src/tree/layout/solve.rs` | 429 | Barrier-root solve orchestration, batching, fallback reporting, clean-geometry skip call sites. |
| `crates/fret-ui/src/tree/layout/entrypoints.rs` | 2502 | Public/internal layout entry points, root iteration, snapshots, frame-level layout orchestration. |
| `crates/fret-ui/src/layout/engine.rs` | 2622 | Taffy engine wrapper, node/layout id maps, solve cache/stamps, measure cache, diagnostics profile. |
| `crates/fret-ui/src/layout/engine/flow.rs` | separate module | Builds Taffy flow subtrees from `ElementInstance` and style data. |

This already suggests a useful boundary: `layout/engine.rs` is a backend wrapper; `tree/layout/*`
is retained-tree orchestration; clean-geometry proof code is a retained-tree optimization contract.

## Execution model

Primary entry points:

- `UiTree::layout_all(...)` and `layout_all_with_pass_kind(...)` in
  `tree/layout/entrypoints.rs`.
- `UiTree::layout_in(...)` and `layout_in_with_pass_kind(...)` in `tree/layout/entrypoints.rs`.
- `UiTree::layout_node(...)` in `tree/layout/node.rs`.

Barrier/root solve path:

- `UiTree::solve_barrier_flow_root_if_needed(...)` in `tree/layout/solve.rs`.
- `UiTree::solve_barrier_flow_roots_if_needed(...)` in `tree/layout/solve.rs`.
- `TaffyLayoutEngine::compute_root_for_node_with_measure_if_needed(...)` and
  `compute_independent_roots_with_measure_if_needed(...)` in `layout/engine.rs`.

Taffy flow build path:

- `build_viewport_flow_subtree(...)` in `layout/engine/flow.rs`.
- `build_flow_subtree_impl(...)` maps declarative element records and layout style into Taffy
  style/child graphs.

The separation is mostly sound: Taffy-specific state stays in `layout/engine.rs` and
`layout/engine/flow.rs`; tree orchestration stays in `tree/layout/*`.

## Clean-Geometry Model

The current proof model has explicit internal axes:

- `CleanGeometryLayoutEffect`
  - `Pure`
  - `SideEffectBoundary`
- `CleanGeometryChildBoundsStrategy`
  - `None`
  - `PreserveLocalOrigins`
  - `ContainerPxInsets`
  - `VerticalNoWrapFlex`
  - `HorizontalFixedFlex`
  - `SingleColumnAutoRowsGrid`
- `CleanGeometryWidthDeltaSizeStability`
  - `Propagated`
  - `NoWrapTextCachedMetrics`

This is the right conceptual direction. It avoids a single flat enum that would mix node type,
side-effect policy, child geometry derivation, and leaf size stability.

The important call sites:

- `can_skip_clean_geometry_engine_solve_for_resize(...)`
- `clean_manual_geometry_subtree_supported_checked(...)`
- `clean_geometry_node_contract(...)`
- `clean_nowrap_text_cached_metrics_supported(...)`
- `try_propagate_clean_engine_layout(...)`

Current supported classes are conservative:

- pass-through wrappers and hit/focus/pointer policy wrappers when they preserve local child
  origins,
- container px-inset/static-child subsets,
- selected no-wrap flex subsets,
- selected one-column auto/px-row grid subsets,
- pure leaves,
- explicit zero driver leaves,
- cached nowrap text with clip/start alignment and matching measure fingerprint.

Current intentional stop conditions include:

- wrapped text reflow,
- root `Scroll` layout side effects,
- `Canvas` unless fresh evidence makes it worth proving,
- retained/cache semantics unless a boundary-specific proof exists,
- broad measured-size sentinel migration unless ambiguity recurs outside explicit zero driver
  leaves.

## Diagnostics Model

The layout stack already exposes evidence that supports architecture decisions:

- layout engine solve profiles,
- measure hotspot counts and cache hits,
- clean-geometry rejection reason/kind/node attribution,
- view-cache reuse outcomes,
- layout request/build and root solve timings.

This means the next architecture decision can be empirical. We should not need speculative
rewrites to learn whether a class is the current bottleneck.

## Risks

### R1 - Reviewability of `node.rs`

`tree/layout/node.rs` mixes:

- ordinary widget layout execution,
- layout observation recording,
- debug hotspot tracking,
- clean-geometry proof classification,
- manual child bounds derivation,
- text-specific fingerprint checks,
- helper validation for flex/grid/container/absolute children.

This is the clearest refactor candidate. The current shape is understandable after a deep read, but
future proof additions will be expensive to review.

### R2 - Contract drift if extraction is careless

The clean-geometry helpers rely on `UiTree` internals, element records, node dirty state, measured
size, and debug attribution. A mechanical split must preserve the narrow proof behavior exactly.

The first extraction, if chosen, should be behavior-preserving and private.

### R3 - False pressure to broaden text

The text stop condition is currently correct. Wrapped text has width-derived layout/measure/paint
constraints. It should not be folded into clean geometry without a dedicated computed-box /
line-break stability proof.

### R4 - Scroll side-effect boundary

`Scroll` still publishes viewport/content handles, deferred-probe state, overflow observation, and
child transforms. Treating it as pure geometry would be a model bug, not an optimization.

## Decision Candidates

### Candidate A - Keep current structure

Use this if FLA-020 shows no significant local layout owner and we expect little future
clean-geometry expansion.

Pros:

- no churn,
- no risk of introducing behavior drift,
- current gates already cover the hard parts.

Cons:

- `node.rs` remains hard to audit,
- future clean-geometry additions become more likely to mix concepts.

### Candidate B - Extract clean geometry into a private module

Move the proof model and helpers out of `node.rs`, keeping behavior unchanged.

Pros:

- improves reviewability,
- preserves the current model,
- creates a clearer owner surface for future proofs.

Cons:

- non-trivial because helpers need broad `UiTree` access,
- may require careful module privacy choices to avoid widening internal APIs.

### Candidate C - Redesign the model

Introduce a stronger internal node-role/proof registry or precomputed classification.

Pros:

- could reduce repeated matching and make future proofs more declarative.

Cons:

- not justified yet by current evidence,
- higher risk,
- likely to touch more of the retained tree and element frame machinery.

## Initial Recommendation

Prefer Candidate B only if FLA-020 or source-review pressure shows that more clean-geometry work is
likely. Otherwise, keep the current model and move to the next true owner lane.

Do not choose Candidate C without fresh evidence that the current axes are insufficient.
