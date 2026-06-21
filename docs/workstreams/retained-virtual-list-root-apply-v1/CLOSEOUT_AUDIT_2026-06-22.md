# Retained VirtualList Root Apply v1 - Closeout Audit - 2026-06-22

## Verdict

Close this lane.

The retained data-table path is no longer primarily blocked on a broad retained `VirtualList`
runtime bug. The latest retained repro shows the fixed-height data-table hot path is within the
current acceptable budget for this lane, and the remaining owners are either the outer content
`Scroll`, first-solve work for row `Pressable` roots, or a future denser fixed-row/table primitive.

## Shipped Decisions

- Keep the retained `ViewCache` settle characterization as a test artifact fix only. No runtime
  reuse-root marking change should land from that evidence.
- Keep `Pressable` as the retained data-table row interaction boundary. It owns row selection,
  hover/pressed visuals, focus routing, hit-test capture, and list-item semantics.
- Do not add a `PressableProps` background API just to remove the selected/hover row background
  `Container`; that would widen runtime mechanism for a table-local policy concern.
- Treat the inline fixed-row cell-padding slice as the last optimization owned by this lane. It
  removed the measured per-cell wrapper breadth without changing runtime contracts.

## Evidence

Latest post-padding retained repro:

```text
target/fret-diag/retained-vlist-inline-cell-padding-codex-20260621/1782066104208/bundle.json
```

`diag stats` reported:

```text
p95.us(total/layout/prepaint/paint)=1983/1642/76/306
layout.root apply=1366
layout.nodes=250
```

`layout-perf-summary` for the same bundle attributed the worst frame to:

```text
Scroll       inclusive=1326us layout=401us
VirtualList  inclusive=824us  layout=194us
Text         inclusive=18us   layout=18us
```

Top layout solves:

```text
Pressable batch_roots=33 subtree_nodes=66 solve_time=123us reason=first_solve
Semantics  batch_roots=1  subtree_nodes=73 solve_time=113us reason=new_frame_key_changed
Stack      batch_roots=1  subtree_nodes=100 solve_time=4us
```

The previous fixed-row inline cell-padding slice moved the retained child path from:

```text
layout_children_first_pass=1770us
nodes_performed=330
Container nodes=132
```

to:

```text
layout_children_first_pass=667us
nodes_performed=198
Container nodes=0
```

## Interpretation

- The deleted per-cell `Container` shell was the last table-local wrapper breadth proven by this
  lane.
- The remaining `Pressable` cost is row first-solve work for newly mounted fixed rows, not a missed
  clean-geometry propagation case. The older `pressable-clean-geometry-propagation-v1` lane already
  proved the clean-geometry path for `Pressable`.
- The remaining `Scroll` cost belongs to viewport/content shell ownership, not retained
  `VirtualList` reconciliation.
- A future fixed-row/table primitive may be worthwhile, but it needs a separate target state because
  it would be a denser table/list mechanism rather than another root-apply cleanup.

## Follow-Ons

- Start a narrow `Scroll` owner lane if fresh evidence keeps the content viewport `Scroll` above
  the retained table/list subtree.
- Start a fixed-row/table primitive lane if fresh evidence shows first-solve row roots are still the
  dominant table-local cost after this closeout.
- Keep code-view/editor-controls on their existing heavy-component perf track; do not route those
  through this retained data-table lane.

## Final Status

Closed on 2026-06-22.
