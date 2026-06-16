# Fixed-Track Virtual Layout Refactor

Date: 2026-06-16
Status: active

## Decision

The next high-leverage performance refactor should deepen the fixed-height retained virtual-list
path, rather than continuing table-local wrapper cleanup.

## Evidence

- The latest retained data-table run still attributes the hot frame to retained `VirtualList` plus
  its parent `Scroll`, even after table wrapper, transform, and diagnostic-anchor reductions.
- `VirtualListMeasureMode::Fixed` currently skips per-row measurement, but still builds
  `measured_updates`, submits barrier roots, solves child roots, and calls `layout_in` for every
  visible child.
- The hot fixed-path behavior is therefore a mechanism cost, not just a shadcn/table recipe cost.

## Refactor Direction

Introduce a narrower fixed-track contract for retained virtualized surfaces:

- Caller supplies deterministic track facts: axis, item count, track extent, gap, scroll margin,
  visible item starts, and a row adapter.
- Runtime owns row placement, barrier-root reuse, and child-layout skip decisions.
- Variable-height and measured rows stay on the existing measured path.

## Non-Goals

- Do not move shadcn/Radix interaction policy into `crates/fret-ui`.
- Do not solve select/combobox/menu tree depth in this slice.
- Do not loosen diagnostics anchors that current perf scripts depend on.

## Candidate Implementation Slices

1. Done: add a focused retained fixed-list test proving a clean cache-hit frame can skip child
   `layout_in` calls entirely.
2. Done: add a child-root skip path that avoids relaying out roots when item bounds, child identity,
   and subtree dirty state are unchanged.
3. Next: re-run the retained data-table perf script and update the retained virtual-list workstream
   evidence.

## Upstream References

- `repo-ref/base-ui/packages/react/src/internals/composite/list/CompositeList.tsx`
- `repo-ref/base-ui/packages/react/src/combobox/list/ComboboxList.tsx`
- `repo-ref/imgui/imgui_demo.cpp`

## Architecture Notes

- This remains a Runtime Substrate mechanism: fixed track geometry, barrier reuse, and layout skip
  are mechanism concerns.
- Collection state, typeahead, dismiss, focus restore, hover intent, and popup policy remain in the
  Policy Layer.

## Progress

### 2026-06-16 Clean Child-Layout Skip

- Added `LayoutCx::can_skip_layout_in`, which reuses the existing `UiTree::can_skip_layout_for_root`
  check instead of duplicating root state rules in `VirtualList`.
- Added reusable `VirtualListLayoutScratch::roots_needing_layout`, so fixed/known/measured list
  placement can keep full barrier-root telemetry while solving and laying out only roots that are
  not clean.
- Preserved scroll child-layout telemetry by recording clean skipped roots in
  `ScrollChildLayoutProfile`.
- Added the focused gate
  `retained_fixed_virtual_list_skips_clean_child_layout_in_on_steady_frame`.

Validation:

- `cargo fmt -p fret-ui --check`
- `cargo check -p fret-ui --lib`
- `cargo nextest run -p fret-ui retained_fixed_virtual_list_skips_clean_child_layout_in_on_steady_frame`
- `cargo nextest run -p fret-ui 'declarative::tests::virtual_list::retained'`

Interpretation:

- This slice fixes avoidable child-layout calls for clean fixed retained windows.
- It is intentionally not claimed as the full answer for the retained data-table filter-shrink hot
  frame, because the latest perf evidence shows that frame is dominated by a real dirty subtree
  (`layout_child_max_subtree_dirty_count=460`).
- The next slice still needs a perf rerun; if the data-table hot frame does not move, continue with
  a deeper fixed-track / dense retained table contract instead of adding more generic wrappers.
