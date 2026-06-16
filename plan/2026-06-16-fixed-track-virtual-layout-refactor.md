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
3. Done: re-run the retained data-table perf script and update the retained virtual-list evidence.

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

### 2026-06-16 Inherited Fixed Text Layout Skip

- Root cause slice: dense retained table cells use ecosystem typography scopes to declare stable
  control text, but the declarative text-content diff previously checked only `TextProps::style`.
  That missed inherited fixed line-height policies and kept many single-line cell text updates on
  the layout invalidation path.
- Mechanism change: declarative text content invalidation now resolves the effective text style
  from the current theme snapshot plus inherited typography before deciding whether a text-only
  update can skip layout.
- Policy-layer change: `text_table_cell` and `text_table_cell_emphasis` now fill their fixed column
  slot while retaining `min-width: 0`, nowrap, and ellipsis semantics. This makes table-cell text
  compatible with the mechanism-level fixed-width/fixed-line-box skip.
- Added regression coverage:
  `inherited_fixed_line_height_text_content_changes_are_paint_only_in_declarative_diff` and an
  updated `retained_table_text_uses_shared_table_cell_role` assertion.

Validation:

- `cargo fmt -p fret-ui -p fret-ui-kit --check`
- `cargo nextest run -p fret-ui inherited_fixed_line_height_text_content_changes_are_paint_only_in_declarative_diff stable_unwrapped_text_content_changes_are_paint_only_in_declarative_diff`
- `cargo nextest run -p fret-ui-kit retained_table_text_uses_shared_table_cell_role`
- `cargo build --release -p fretboard-dev -p fret-ui-gallery`

Perf note:

- Attempted fresh m6 capture under
  `target/fret-diag/retained-vlist-root-apply-m6-text-slot-v1`.
- The run failed before reaching the retained DataTable page:
  step 6 waited for `ui-gallery-nav-data-table-torture`, while the failure bundle showed only
  `ui-gallery-nav-search` and `ui-gallery-nav-scroll`. The search field had value length 19
  (`datatable (torture)`), so this is a navigation/filter prelude failure, not valid DataTable
  performance evidence.
- The timeout bundle did show post-change steady nav/content frames around `3.2ms total /
  2.9ms layout`, but those frames are not the target retained table workload and should not be
  compared against m5.
- Added a direct-start retained DataTable perf script:
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change-direct.json`.
  It starts with `FRET_UI_GALLERY_START_PAGE=data_table_torture` and declares
  `required_launch_features=["gallery-dev"]`, avoiding the sidebar search/filter prelude that
  made the prior m6 capture non-actionable.
- Valid direct m6 bundle:
  `target/fret-diag/retained-vlist-root-apply-m6-text-slot-direct-v4/1781627048872/bundle.schema2.json`.
  `diag stats --sort cpu_cycles --top 8` reported `top_total_time_us=8934`,
  `top_layout_time_us=8015`, `top_layout_engine_solve_time_us=4618`,
  `layout.root apply=6720`, and `layout.nodes=417`.
- Interpretation: the inherited fixed text-slot slice is a small positive movement over m5
  (`9278` / `8669` / `5977` / `7897` / `417`), but the hot frame is still a retained
  DataTable input-change frame dominated by parent `Scroll` / retained `VirtualList` root apply.
  Continue with fixed-track/dense retained table contract work rather than more text-cell cleanup.

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
- The retained data-table perf rerun moved modestly:
  `target/fret-diag/retained-vlist-root-apply-m5-clean-root-skip-v1/1781600101441/bundle.schema2.json`.
  `diag stats --sort cpu_cycles --top 10` reported `top_total_time_us=9278`,
  `top_layout_time_us=8669`, `top_layout_engine_solve_time_us=5977`,
  `layout.root apply=7897`, and `layout.nodes=417`.
- This is a real improvement over the prior cell-anchor toggle bundle
  (`9965` / `9328` / `6595` / `8546` / `417`), but it does not move the owner away from
  retained `VirtualList` plus parent `Scroll`.
- Continue with a deeper fixed-track / dense retained table contract instead of adding more generic
  wrapper cleanup.

### 2026-06-17 Declarative Invalidation Detail Attribution

- The valid direct m6 bundle still showed 132 layout invalidation walks rooted at
  `text_table_cell` paths, but the existing diagnostics labeled those walks as generic
  `other/unknown`. That made it impossible to tell whether the remaining hot frame was still a
  text-content fallback or a broader retained-window remount/reconcile effect.
- Added `UiDebugInvalidationDetail::DeclarativeInstanceChanged` and routed declarative pending
  invalidations through `invalidate_with_source_and_detail`.
- Text content changes now mark a separate diagnostic bit, so both paint-only and layout-affecting
  text updates are reported as `declarative_text_content_changed`; other declarative instance
  changes are reported as `declarative_instance_changed`.
- Added focused assertions to the text-content diff tests so diagnostic attribution cannot silently
  regress back to `unknown`.
- Validation:
  - `cargo fmt -p fret-ui --check`
  - `cargo check -p fret-ui --lib`
  - `git diff --check -- crates/fret-ui/src/declarative/mount.rs crates/fret-ui/src/tree/debug/invalidation.rs crates/fret-ui/src/declarative/tests/text_cache.rs`
- Focused `nextest` compile/run and `cargo test -p fret-ui --lib
  stable_unwrapped_text_content_changes_are_paint_only_in_declarative_diff --no-run` both timed out
  before producing pass/fail output. A later retry happened with no active Rust build processes,
  so treat this as a `fret-ui` test-binary build blocker to investigate separately.

### 2026-06-17 Retained Row Fixed-Track ManagedSurface

- Replaced the retained fixed-row single-group cell strip from a row-local `h_row`/Flex layout with
  a `ManagedSurface` fixed-track owner.
- The new helper keeps the existing cell containers, padding, borders, renderer output, test ids,
  semantics, and hit-test bounds. It only replaces row-internal column placement with direct
  known-width geometry.
- This keeps policy in `fret-ui-kit` while using the existing `fret-ui` mechanism primitive for
  first-class child bounds; it avoids the earlier rejected absolute-cell shortcut that broke
  sidecar/test-id geometry.
- Focused validation:
  - `cargo fmt -p fret-ui-kit --check`
  - `cargo check -p fret-ui-kit --lib`
  - `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_plain_rows_omit_background_wrapper table_virtualized_retained_selected_rows_keep_background_wrapper table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform --no-fail-fast --no-capture`
- Perf rerun:
  `target\release\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\data-table\ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change-direct.json --repeat 1 --warmup-frames 5 --dir target\fret-diag\retained-vlist-row-managed-surface-m7-v1 --env FRET_UI_GALLERY_DATA_TABLE_RETAINED=1 --env FRET_UI_GALLERY_START_PAGE=data_table_torture --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --sort cpu_cycles --top 8 --json --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness`
- Evidence bundle:
  `target/fret-diag/retained-vlist-row-managed-surface-m7-v1/1781634040521/bundle.json`.
- `diag perf` reported `top_total_time_us=2564`, `top_layout_time_us=1693`,
  `top_layout_engine_solve_time_us=392`, `layout.root apply=1082`, and `layout.nodes=66`.
  The previous direct m6 evidence was `8934` / `8015` / `4618` / `6720` / `417`.
- Bounded bundle triage confirms the same retained filter-shrink scenario still occurred on frame
  26: `window_shift_reason=inputs_change`, `window_shift_kind=escape`, `items_len=111`,
  `prev_count=50000`, `count=111`.
- Interpretation: this is the first material 120Hz-level result for the retained DataTable
  torture path. The optimization validates the architectural direction: dense fixed-track
  components should expose deterministic geometry to the runtime instead of paying a generic
  Flex solve per visible row. The next deeper slice can either move more cell content into a
  direct dense paint path, or generalize this fixed-track owner into a reusable table/list
  primitive once more callers need it.

### 2026-06-17 Non-Retained View-Cache Row Geometry

- Added a direct-start view-cache DataTable perf script:
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change-direct.json`.
  It starts on `data_table_torture` with `FRET_UI_GALLERY_VIEW_CACHE=1`, avoiding the same
  navigation-search prelude instability that previously made some table captures non-actionable.
- Baseline evidence bundle:
  `target/fret-diag/data-table-view-cache-direct-m2/1781634607902/bundle.json`.
- Bounded triage confirmed the hot frame is the same filter-shrink scenario:
  `frame=27`, `window_shift_reason=inputs_change`, `window_shift_apply_mode=non_retained_rerender`,
  `window_shift_kind=escape`, `items_len=111`, `prev_count=50000`, `count=111`.
- Baseline `diag stats --sort cpu_cycles --top 8` reported `top_total_time_us=12941`,
  `top_layout_time_us=11387`, `top_layout_engine_solve_time_us=6023`, `layout.root apply=8814`,
  and `layout.nodes=843`.
- The layout perf sidecar attributed the hot solve to the normal non-retained body row path:
  a `Pressable` root under `ecosystem/fret-ui-kit/src/declarative/table.rs:8983`, with
  `subtree_nodes=726` and `measure_calls=792`.
- Refactor slice: generalized the retained fixed-row `ManagedSurface` helper into
  `table_fixed_row_group(...)`, then reused it for the normal non-retained fixed body row
  single-strip path. `Pressable`, row semantics, cell containers, padding, clipping, test ids,
  and shared horizontal scroll transform are retained; only the known-width cell strip placement
  moves from row-local Flex to deterministic geometry.
- Public DataTable helpers now require `H: UiHost + 'static` because `ManagedSurface` stores
  element-local layout/paint hooks. This is an intentional heavy-component API tradeoff rather than
  a runtime-layer policy leak.
- Focused validation passed:
  - `rustfmt --edition 2024 --check ecosystem/fret-ui-kit/src/declarative/table.rs ecosystem/fret-ui-shadcn/src/data_table.rs ecosystem/fret-ui-shadcn/src/ui_builder_ext/data.rs`
  - `cargo check -p fret-ui-kit --lib`
  - `cargo check -p fret-ui-shadcn --lib`
  - `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_unpinned_body_uses_shared_horizontal_transform table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform table_virtualized_retained_plain_rows_omit_background_wrapper --no-fail-fast --no-capture`
- The first m3 perf attempt spent several minutes rebuilding `fret_ui_gallery` and then left only
  a waiting `fretboard-dev` process with no `ready.touch`, script result, or bundle. That process
  was cleaned up because it was this slice's stale diagnostics launcher and had no child process.
- Valid rerun:
  `target/fret-diag/data-table-view-cache-direct-m4-managed-row/1781636557298/bundle.json`.
- `diag stats --sort cpu_cycles --top 5` reported `top_total_time_us=10594`,
  `top_layout_time_us=8774`, `top_layout_engine_solve_time_us=985`, `layout.root apply=6945`,
  and `layout.nodes=810`.
- Bounded bundle triage confirmed the same non-retained filter-shrink scenario still occurred on
  frame 27: `window_shift_reason=inputs_change`, `window_shift_apply_mode=non_retained_rerender`,
  `window_shift_kind=escape`, `items_len=111`, `prev_count=50000`, `count=111`.
- The hot layout solve itself moved from `6023us` / `792 measure_calls` / `subtree_nodes=726` to
  `985us` / top solve `33 measure_calls` / `subtree_nodes=165`. This proves the deterministic
  row geometry slice removed the row-local fixed-column Flex solve.
- The remaining miss is now different: `layout.root apply=6945`, `inv.calls=271`, and
  `inv.nodes=4613` dominate the frame. The next non-retained view-cache optimization should target
  invalidation/root-apply churn during full `non_retained_rerender` window shifts, not more
  row-internal column placement.
