# Scroll Optimization Workstream (v1) — Milestones

Date: 2026-05-08
Status: Active

## M0 — Baseline + evidence (1–2 days)

- Establish a minimal “scroll correctness” script set (ui-gallery).
- Add a thumb-drag stability repro + gate script.
- Document current invariants (HitTestOnly scrolling, nested wheel routing).

## M1 — Mechanism hardening (2–4 days)

- Reduce barrier/scroll foot-guns (single helper paths where possible).
- Add unit tests around barrier relayout + subtree dirty aggregation.

## M2 — Wheel/trackpad coalescing prototype (3–5 days)

- Implement an opt-in coalescing mode.
- Add a torture script for wheel input and basic perf telemetry capture (bundle capture; perf threshold TBD).
- Ensure nested scrollables still route correctly (deepest-first).

## M3 — Scrollbar drag baseline lock (2–4 days)

- Stabilize thumb while dragging under content changes.
- Add a deterministic gate (diag script + bounded assertions on semantics).

## M4 — Extents observation hardening (2–4 days)

- Expand post-layout overflow observation coverage with gates.
- Validate budget-hit fallback probes prevent pinned extents.
- Separate retained seed extents from authoritative extent commits and lock the contract with
  mechanism tests.
- Ensure authoritative observations can finish deferred invalidation cleanup even when the
  observed extent is unchanged.

## M5 — Dirty-frontier resize churn reduction (2–4 days)

- Keep contained view-cache dirty work inside the contained relayout + nearest-scroll follow-up
  path instead of promoting clean scroll direct child roots to `Layout` invalidation.
- Keep post-layout overflow observation authoritative when one direct child remains dirty and a
  different child has descendant-only shrink work; synthetic scroll content roots must not keep
  stale pinned extents ahead of the observed child frontier.
- Preserve non-retained virtual-list view-cache rerender pressure when wheel scrolling escapes the
  rendered visible range, while retained virtual lists continue using the retained reconcile path
  without notifying the cache root.
- Profile the remaining direct-child-invalidated / resize-measure path separately before attempting
  another layout skip or apply-only branch.
  - 2026-05-15 normalized view-cache resize-stress attribution no longer shows
    direct-child-invalidated / resize-measure as the steady-frame bottleneck; worst considered
    frames are paint-dominant with bounded layout solves and no invalidation walks.
- Keep representative `diag perf` samples normalized; repair stale prewarm command forms before
  using them as p95 baselines.

## M6 — Clean root-solve / geometry propagation split (1–2 days)

- Skip barrier/root Taffy solves only for engine-backed clean roots during small-step interactive
  width-only resize when child bounds can be derived from previous clean geometry.
- Keep `Scroll` as a side-effectful layout boundary: parent geometry can be propagated to it, but
  `Scroll` layout still publishes viewport/content handles, deferred-probe state, overflow
  observation, and child transforms.
- Keep `ViewCache` and `VirtualList` off this fast path until each has a dedicated retained/render
  or visible-window proof.
- Record local perf evidence separately from RTX4090 closeout. RTX4090 remains follow-up evidence,
  not this slice's completion condition.

## M7 — Layout side-effect / geometry propagation contract (follow-on)

- Classify remaining `Semantics`, root `Stack`, and editor `PointerRegion` small-width-delta solves
  before widening the fast path.
- Prefer an explicit contract for pure geometry nodes, side-effectful layout boundaries, and
  safe leaf-only nodes over another ad hoc whitelist expansion.
- Add diagnostics that explain the first unsupported kind/reason when a clean root cannot skip its
  engine solve.
- Treat `Canvas` leaf participation as a small optional proof, not the next primary perf owner.
- 2026-05-17 minimum slice landed: `CleanGeometryNodeContract` separates pure pass-through
  geometry, no-wrap vertical flex, safe leaves, and side-effect boundaries; diagnostics now expose
  the per-frame rejection count plus first reason/kind for rejected clean root-solve skips.
- 2026-05-17 follow-up: layout-engine solve diagnostics now attach the clean-geometry rejection
  reason/kind to the individual rejected solve. Fresh no-4090 resize-jitter evidence points to
  `Container` as the next meaningful blocker; `Canvas` is measured too small and `Scroll` remains a
  side-effect boundary.

## M8 — Conservative Container geometry contract (candidate)

- Prove a narrow `Container` clean-geometry subset before widening the root-solve skip.
- Start with static children, px padding/border/spacing, definite or fill width propagation, and no
  auto-height reflow dependency.
- Keep absolute children, non-px/fractional insets, retained/windowing surfaces, wrap flex, and
  layout-side-effect descendants on the full solve path until each has its own proof.
- Validate with focused Rust tests first, then rerun the no-4090 resize-jitter diagnostics to learn
  the next blocker instead of assuming the fast path is complete.
- 2026-05-17 minimum slice landed: `Container` supports a px-inset/static-child manual bounds proof
  and remains manual-bounds-only in the clean propagation path. Fresh no-4090 evidence shows the next
  meaningful blocker is `auto_child_height`, with wrap flex still present under the content
  `Semantics` root.

## M9 — Auto-height / line-break stability classification (candidate)

- Classify `auto_child_height` rejections before widening the clean-geometry proof.
- Separate stable wrapper auto-height from width-dependent text/wrap/flex reflow.
- Keep wrap flex blocked until line-break stability can be proven against previous geometry.
- 2026-05-17 classification landed: stable auto-height `Container` wrappers and stable auto-height
  children inside vertical no-wrap `Flex` can participate through recursive geometry proof, while
  changing-size text leaves reject with `text_reflow`. Fresh no-4090 evidence moves the next
  blockers to `Grid` and horizontal `Flex`; wrap flex remains visible under the content root, and
  `Scroll` remains a side-effect boundary.
- 2026-05-17 model refactor landed: `CleanGeometryNodeContract` now records separate layout-effect,
  child-bounds, and width-delta size-stability axes. Boundary detection also reads that contract,
  instead of keeping a separate `Scroll` special-case table.
- Do not treat the current supported node set as permission to expand by name. The next grid,
  row-flex, retained/cache, canvas, layout-query, or transform slice still needs a focused proof and
  evidence.

## M10 — Horizontal fixed Flex geometry contract (candidate)

- Prove horizontal no-wrap `Flex` only for fixed main-axis child distribution before attempting
  `Grid` or flex-grow/fill semantics.
- Accept horizontal cross-axis alignment when the resize proof is width-only and parent height plus
  px vertical padding remain stable; keep vertical non-stretch cross-axis alignment rejected until it
  has its own proof.
- Keep grow/shrink/order/basis/align-self and auto/fill/fraction main-axis widths on the full solve
  path with a dedicated `flex_item_sizing` rejection.
- 2026-05-17 minimum slice landed: horizontal fixed `Flex` gets a dedicated
  `HorizontalFixedFlex` child-bounds strategy, pure leaf geometry is separated from text
  stable-computed leaves, and focused tests cover stretch horizontal acceptance, center horizontal
  acceptance, vertical center rejection, and horizontal grow rejection.
- Fresh no-4090 resize-jitter evidence moves the app-shell/nav blocker from `flex_cross_align` to
  `flex_item_sizing`; content `Semantics` remains blocked by `unsupported_kind=Grid` with
  `wrap_nodes=1`, `Canvas` remains too small to prioritize, and `Scroll` remains a side-effect
  boundary.

## M11 — Narrow Grid geometry contract (candidate)

- Prove only the real card-header-like grid shape before attempting any general grid or text
  reflow optimization.
- Accept one-column grids with explicit non-empty `Auto` / `Px` row tracks, px spacing, start
  alignment, static children, simple grid lines, and stable child size semantics.
- Keep flexible tracks, multi-column placement, item self-alignment, non-px spacing, positioned
  children, text reflow, retained/windowing surfaces, and side-effect boundaries on the full solve
  path until each has a focused proof.
- 2026-05-17 minimum slice landed: `Grid` now has a
  `SingleColumnAutoRowsGrid` child-bounds strategy for this narrow subset, plus a negative flexible
  track guardrail. Fresh no-4090 evidence moves the content blocker from `unsupported_kind=Grid` to
  `text_reflow / Text`; the root blocker is now `missing_measured_size / Stack` through sidebar
  nav `ScrollArea`, while `Canvas` and root `Scroll` remain separate follow-ups.
