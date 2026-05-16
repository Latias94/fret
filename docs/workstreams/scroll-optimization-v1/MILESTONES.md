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
