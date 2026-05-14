# Evidence and Gates

Status: Active
Last updated: 2026-05-14

## Primary Repro

The first repro remains the code-editor resize/paint pressure path:

```bash
cargo run -p fretboard-dev --release -- diag perf ui-code-editor-resize-probes \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --sort time \
  --top 15 \
  --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Use an explicit `--dir target/<descriptive-dir>` for publishable evidence.

## Required Attribution

For every perf claim, run:

```bash
target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 15
```

The summary must mention:

- total/layout/prepaint/paint p50 and p95,
- `layout.engine_solve`,
- `paint.widget`,
- `paint.text_prepare`,
- renderer prepare/encode/upload counters,
- and `code_editor.paint_perf` when the code-editor surface is involved.

## Current Baseline Evidence

Progress ledger:

- `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/PROGRESS.md`
- Global completion contract:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/PROGRESS.md#completion-contract`
- Contract freeze:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M0_CONTRACT_FREEZE_2026-05-14.md`

Most recent pre-lane evidence:

- M0 baseline/source audit:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M0_BASELINE_AUDIT_2026-05-13.md`
- M1 boundary diagnostics slice:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M1_BOUNDARY_DIAGNOSTICS_SLICE_2026-05-13.md`
- Workstream log:
  `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`
- macOS contained-layout run:
  `target/fret-diag-code-editor-resize-probes-contained-layout-20260513/check.perf_thresholds.json`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-contained-layout-20260513/1778661520873/bundle.schema2.json`

Observed result from that run:

- gate failures: `[]`,
- p95/max top total: `1361/1361us`,
- p95/max top layout: `295/295us`,
- p95/max top layout solve: `116/116us`,
- p95/max paint: `1134/1134us`,
- `code_editor.paint_perf` p50/p95 total: `241/401us`.

Most recent boundary-diagnostics slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M1_BOUNDARY_DIAGNOSTICS_SLICE_2026-05-13.md`
- Perf output directory:
  `target/fret-diag-code-editor-resize-probes-boundary-diag-20260513`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-boundary-diag-20260513/1778668519515/bundle.schema2.json`

Observed result from that run:

- time p50/p95: total `1203/1811us`, layout `38/364us`, prepaint `15/34us`,
  paint `949/1737us`,
- hot p50/p95: `layout.engine_solve=0/140us`, `paint.widget=731/1494us`,
  `paint.text_prepare=10/15us`,
- `code_editor.paint_perf` p50/p95 total: `302/743us`,
- renderer prepare/encode/upload counters stayed at zero.

Most recent prepaint-ownership slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M2_CANVAS_PREPAINT_WINDOWED_ROWS_SLICE_2026-05-13.md`
- Perf output directory:
  `target/fret-diag-code-editor-resize-probes-canvas-prepaint-20260513`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-canvas-prepaint-20260513/1778671598958/bundle.schema2.json`

Observed result from that run:

- time p50/p95: total `1117/1356us`, layout `33/335us`, prepaint `168/291us`,
  paint `722/897us`,
- hot p50/p95: `layout.engine_solve=0/129us`, `paint.widget=523/695us`,
  `paint.text_prepare=9/12us`,
- `code_editor.paint_perf` p50/p95 total: `261/433us`,
- `code_editor.paint_perf.us_frame_overlay` sum: `0`,
- row scene replay hit rate: `99%`,
- renderer prepare/encode/upload counters stayed at zero.

This slice moved editor frame-derived ownership out of paint attribution and into prepaint, but it
did not yet finish the full boundary migration or the final 20-30% end-to-end improvement target.

Most recent row-scene prepaint replay-plan slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M3_ROW_SCENE_PREPAINT_REPLAY_PLAN_SLICE_2026-05-13.md`
- Perf output directory:
  `target/fret-diag-code-editor-resize-probes-row-scene-prepaint-plan-20260513`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-row-scene-prepaint-plan-20260513/1778679317011/bundle.schema2.json`

Observed result from that run:

- gate failures: `[]`,
- total p50/p95/max: `1443/1712/1712us`,
- layout p50/p95/max: `387/388/388us`,
- prepaint p50/p95/max: `282/382/382us`,
- paint p50/p95/max: `814/943/943us`,
- row scene replay hit rate: `99-100%`,
- renderer prepare/encode/upload counters stayed at `0`.

Worst-bundle attribution:

- time p50/p95: total `1170/1712`, layout `37/387`, prepaint `324/382`, paint `710/958`
- hot p50/p95: `layout.engine_solve=0/146`, `paint.widget=499/745`,
  `paint.text_prepare=10/12`
- `code_editor.paint_perf` sum planned/used replay entries: `2090/2090`
- max planned/used replay entries per frame: `289/289`
- `code_editor.paint_perf` p50/p95 `us_row_scene_prepaint_plan`: `65/123us`
- `code_editor.paint_perf` p50/p95 `us_row_text`: `0/6us`

This slice proves the planned phase split for cached row scene replay. It does not yet complete the
final `ViewBoundary` store or the final 20-30% end-to-end improvement target.

Most recent windowed-rows canonical row-rect slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M3A_WINDOWED_ROWS_CANONICAL_ROW_RECT_SLICE_2026-05-13.md`
- Perf output directory:
  `target/fret-diag-code-editor-resize-probes-windowed-row-rect-20260513`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-windowed-row-rect-20260513/1778681710195/bundle.schema2.json`

Observed result from that run:

- gate failures: `[]`,
- total p50/p95/max: `1250/1519/1519us`,
- layout p50/p95/max: `335/345/345us`,
- prepaint p50/p95/max: `275/349/349us`,
- paint p50/p95/max: `727/825/825us`,
- row scene replay hit rate: `99%`,
- renderer prepare/encode/upload counters stayed at `0`.

Worst-bundle attribution:

- time p50/p95: total `1125/1519`, layout `35/345`, prepaint `265/380`, paint `672/900`
- hot p50/p95: `layout.engine_solve=0/127`, `paint.widget=456/691`,
  `paint.text_prepare=9/12`
- `code_editor.paint_perf` sum planned/used replay entries: `2090/2090`
- max planned/used replay entries per frame: `289/289`
- `code_editor.paint_perf` p50/p95 `us_row_scene_prepaint_plan`: `67/89us`
- `code_editor.paint_perf` p50/p95 `us_row_text`: `0/12us`

This slice removes code-editor-local fixed-row rect reconstruction from replay planning. It keeps
the replay plan editor-owned, so it is still a transition step before the final boundary fragment
store.

Most recent canvas prepaint-output slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M3B_ROW_SCENE_PREPAINT_OUTPUT_CARRIER_SLICE_2026-05-13.md`
- Focused correctness gate:
  `cargo nextest run -p fret-ui declarative::tests::canvas::canvas_prepaint_output_is_visible_to_canvas_paint --no-fail-fast`
- Perf output directory:
  `target/fret-diag-code-editor-resize-probes-canvas-prepaint-output-20260513`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-canvas-prepaint-output-20260513/1778685213875/bundle.schema2.json`

Observed result from that run:

- time p50/p95/max: total `1103/1576/1576us`,
  layout `35/344/344us`,
  prepaint `251/360/360us`,
  paint `659/877/877us`
- hot p50/p95: `layout.engine_solve=0/133us`, `paint.widget=445/661us`,
  `paint.text_prepare=10/13us`
- `code_editor.paint_perf` p50/p95 total: `175/403us`
- `code_editor.paint_perf.us_row_text` p50/p95: `0/5us`
- planned/used replay entries still matched `2090/2090`
- row scene replay hit rate remained `99%`
- renderer prepare/encode/upload counters stayed at zero

Worst-bundle attribution:

- `target/release/fretboard-dev diag stats target/fret-diag-code-editor-resize-probes-canvas-prepaint-output-20260513/1778685213875/bundle.schema2.json --sort time --top 15`
- time p50/p95: total `1103/1576us`, layout `35/344us`, prepaint `251/360us`,
  paint `659/877us`
- hot p50/p95: `layout.engine_solve=0/133us`, `paint.widget=445/661us`,
  `paint.text_prepare=10/13us`
- `code_editor.paint_perf` sum planned/used replay entries: `2090/2090`
- `code_editor.paint_perf` p50/p95 `us_row_scene_prepaint_plan`: `55/77us`
- `code_editor.paint_perf` p50/p95 `us_row_text`: `0/5us`

Most recent boundary-prepaint-state slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M2B_VIEW_BOUNDARY_PREPAINT_STATE_SLICE_2026-05-13.md`
- Focused correctness gates:
  `cargo nextest run -p fret-ui tree::tests::prepaint::prepaint_output_is_owned_by_view_boundary_state_and_removed_with_node tree::tests::prepaint::prepaint_output_store_is_keyed_by_cache_root_prepaint_key declarative::tests::canvas::canvas_prepaint_output_is_visible_to_canvas_paint --no-fail-fast`
  and
  `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics cache_root_boundary boundary_diagnostics_are_built_from_boundary_stats_with_cache_root_outcomes --no-fail-fast`
- Compile/layering gates:
  `cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-bootstrap --features syntax-rust`
  and `python3 tools/check_layering.py`

Observed result:

- `fret-ui` focused nextest: `3 passed, 928 skipped`.
- `fret-bootstrap` boundary diagnostics nextest: `5 passed, 97 skipped`.
- `cargo check`: passed.
- `tools/check_layering.py`: passed.

This slice is correctness/ownership work, not a new perf claim. It moves typed prepaint output
storage from `Node` to `ViewBoundaryState::prepaint`, updates cache-root boundary diagnostics to
report `prepaint_owner=view_boundary_prepaint_state`, builds top-level `debug.boundaries[]` from
direct `UiTree::debug_boundary_stats()` enumeration, and introduces minimal boundary layout
dependency metadata for contained-relayout eligibility. The final scene-fragment store, boundary
dirty-set migration, and the 20-30% p95/max closeout proof are still pending.

Most recent boundary scene-fragment carrier slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M3C_BOUNDARY_SCENE_FRAGMENT_CARRIER_SLICE_2026-05-14.md`
- Focused correctness gates:
  `cargo nextest run -p fret-ui declarative::tests::canvas::canvas_scene_fragment_is_boundary_owned_and_keyed_by_prepaint_key --no-fail-fast`
  and
  `cargo nextest run -p fret-code-editor --features syntax-rust prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint --no-fail-fast`
  and
  `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics cache_root_boundary boundary_diagnostics_are_built_from_boundary_stats_with_cache_root_outcomes --no-fail-fast`
- Compile gate:
  `cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor --features syntax-rust`

Observed result:

- `fret-ui` canvas scene-fragment nextest: `1 passed, 931 skipped`.
- `fret-code-editor` row-scene replay nextest: `1 passed, 128 skipped`.
- `fret-bootstrap` boundary diagnostics nextest: `4 passed, 97 skipped`.
- `cargo check`: passed.

This slice is a correctness/ownership step, not a new perf claim. It moves the row-scene replay
carrier from generic canvas prepaint output into `ViewBoundaryState::scene_fragment` through
`CanvasSceneFragment<RowSceneFragmentPayload>`, and `debug.boundaries[]` now reports fragment owner,
slot count, and fragment entry count for debug-aware carriers.

Most recent boundary dirty-set slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M4A_BOUNDARY_DIRTY_SET_SLICE_2026-05-14.md`
- Focused correctness gates:
  `cargo nextest run -p fret-ui view_cache::view_cache_runs_contained_relayout_for_invalidated_boundaries view_cache::view_cache_contained_relayout_does_not_force_next_frame_rerender layout_dirty_invalidation_harness scroll_handle_invalidation_harness --no-fail-fast`
  and
  `cargo nextest run -p fret-ui view_cache_runs_contained_relayout_for_invalidated_boundaries --no-fail-fast`
  and
  `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics cache_root_boundary boundary_diagnostics_are_built_from_boundary_stats_with_cache_root_outcomes --no-fail-fast`
- Compile gate:
  `cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-bootstrap --features syntax-rust,diagnostics`

Observed result:

- first `fret-ui` focused run: `3 passed, 929 skipped`; the first namespaced filter did not match
  and was rerun explicitly.
- explicit contained-relayout run: `1 passed, 931 skipped`.
- `fret-bootstrap` boundary diagnostics run: `4 passed, 97 skipped`.
- `cargo check`: passed with no new warnings.

This slice is a correctness/ownership step, not a new perf claim. It removes
`dirty_cache_roots`, `dirty_cache_root_reasons`, and `mark_cache_root_dirty(...)` from the runtime
owner path, keeps `dirty_boundaries` as the hot-path scheduling index, and emits boundary-owned
layout dirty source/detail through `debug.boundaries[]`. The final perf closeout proof and broader
view-cache/paint-cache compatibility cleanup are still pending.

Most recent boundary diagnostics canonicalization slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M4B_BOUNDARY_DIAGNOSTICS_CANONICALIZATION_SLICE_2026-05-14.md`
- Focused correctness gates:
  `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics cache_root_boundary boundary_diagnostics_are_built_from_boundary_stats_with_cache_root_outcomes --no-fail-fast`
  and
  `cargo nextest run -p fret-diag bundle_stats_preserves_cache_root_boundary_summary --no-fail-fast`
- Compile/layering gates:
  `cargo check -p fret-bootstrap -p fret-diag --features diagnostics`,
  `cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-bootstrap --features syntax-rust,diagnostics`,
  and `python3 tools/check_layering.py`

Observed result:

- `fret-bootstrap` boundary diagnostics nextest: `4 passed, 97 skipped`.
- `fret-diag` stats join nextest: `1 passed, 818 skipped`.
- `cargo check -p fret-bootstrap -p fret-diag --features diagnostics`: passed.
- `cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-bootstrap --features syntax-rust,diagnostics`:
  passed.
- `python3 tools/check_layering.py`: passed.
- `cargo fmt`, `WORKSTREAM.json` validation, and `git diff --check`: passed.

This slice deletes the serialized `debug.cache_roots[].boundary` compatibility field. Top-level
`debug.boundaries[]` is now the canonical boundary diagnostics list, and `fret-diag stats` derives
its report-only `top_cache_roots[].boundary` summary from that canonical list.

Most recent boundary hint API slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M4C_BOUNDARY_HINT_API_SLICE_2026-05-14.md`
- Focused correctness gates:
  `cargo nextest run -p fret-ui view_cache_boundary_hints_drive_boundary_layout_dependency view_cache_runs_contained_relayout_for_invalidated_boundaries view_cache_contained_relayout_does_not_force_next_frame_rerender --no-fail-fast`
  and
  `cargo nextest run -p fret-ui-kit cached_subtree_props_boundary_hint_replaces_direct_contained_layout_authoring --no-fail-fast`
- Compile gate:
  `cargo check -p fret-ui -p fret-ui-kit -p fret-ui-gallery -p fret-docking -p fret-workspace -p fret --all-targets`
- Source drift check:
  `rg -n "contained_layout\\(|page_content_cache_contained_layout|ViewCacheProps \\{[^\\n]*contained_layout|contained_layout:" apps/fret-cookbook apps/fret-examples apps/fret-ui-gallery ecosystem/fret-ui-kit/src/declarative crates/fret-ui/src/declarative crates/fret-ui/src/element.rs ecosystem/fret-docking ecosystem/fret-workspace ecosystem/fret/src --glob '*.rs'`

Observed result:

- `fret-ui` focused nextest: `3 passed, 930 skipped`.
- `fret-ui-kit` focused nextest: `1 passed, 519 skipped`.
- `cargo check`: passed.
- source drift check: no matches.

This slice is an authoring-contract step, not a new perf claim. It introduces
`ViewBoundaryHints`, replaces first-party direct `contained_layout` authoring with
`contain_layout_when_bounds_known(...)`, and leaves internal low-level contained-layout flags for
the broader view-cache/build-boundary consolidation.

Most recent view-cache build-boundary store slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M4D_VIEW_CACHE_BUILD_BOUNDARY_STORE_SLICE_2026-05-14.md`
- Direct runtime store gate:
  `cargo nextest run -p fret-ui elements::runtime::tests::view_cache_build_boundary_store_advances_rendered_next_and_clears_frame_local_flags --no-fail-fast`
- View-cache behavior gates:
  `cargo nextest run -p fret-ui declarative::tests::core::view_cache_subtree_membership_includes_nested_cache_roots declarative::tests::view_cache::view_cache_keep_alive_revalidates_recorded_membership_before_touching_stale_detached_elements declarative::tests::view_cache::view_cache_inherits_model_observations_on_cache_hit_layout --no-fail-fast`
- Compile gate:
  `cargo check -p fret-ui --all-targets`
  and
  `cargo check -p fret-ui --features diagnostics --all-targets`

Observed result:

- direct runtime store gate: `1 passed, 933 skipped`;
- view-cache behavior gates: `3 passed, 931 skipped`;
- both `cargo check` commands passed.

This slice is a runtime ownership-consolidation step, not a new perf claim. It replaces
element-runtime flat view-cache build-time rendered/next maps and frame-local side sets with
`ViewCacheBuildBoundaryStore`, while preserving the existing `ElementContext` and declarative mount
method surface. The final migration into `ViewBoundaryState`, paint-cache replay ownership, and
non-code-editor proof surface remain open.

Most recent boundary paint-cache entry slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M4E_BOUNDARY_PAINT_CACHE_ENTRY_SLICE_2026-05-14.md`
- Boundary-owned paint-cache entry gate:
  `cargo nextest run -p fret-ui tree::tests::paint_cache::paint_cache_entry_is_boundary_owned_for_view_cache_roots --no-fail-fast`
- Paint-cache regression gate:
  `cargo nextest run -p fret-ui tree::tests::paint_cache --no-fail-fast`
- View-cache paint-cache gating gate:
  `cargo nextest run -p fret-ui tree::tests::view_cache::view_cache_disables_paint_cache_for_non_boundary_nodes tree::tests::view_cache::view_cache_allows_paint_cache_for_boundary_nodes --no-fail-fast`
- Boundary diagnostics gate:
  `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics cache_root_boundary boundary_diagnostics_are_built_from_boundary_stats_with_cache_root_outcomes --no-fail-fast`
- Compile gates:
  `cargo check -p fret-ui --all-targets`
  and
  `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics`

Observed result:

- boundary-owned paint-cache entry gate: `1 passed, 934 skipped`;
- paint-cache regression gate: `9 passed, 926 skipped`;
- view-cache paint-cache gating gate: `2 passed, 933 skipped`;
- boundary diagnostics gate: `5 passed, 97 skipped`;
- both compile gates passed.

This slice is a runtime ownership-consolidation step, not a new perf claim. It moves boundary-node
`PaintCacheEntry` ownership into `ViewBoundaryState::paint_cache`, adds `paint_cache_owner` to
boundary diagnostics, and preserves node-owned paint-cache entries as the fallback for
non-boundary nodes at the time of M4E. M4F below deletes that fallback. The global previous-op
storage and env-knob retention decisions remain open.

Most recent node paint-cache fallback deletion slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M4F_NODE_PAINT_CACHE_FALLBACK_DELETION_SLICE_2026-05-14.md`
- Paint-cache regression gate:
  `cargo nextest run -p fret-ui tree::tests::paint_cache --no-fail-fast`
- View-cache paint-cache gating gate:
  `cargo nextest run -p fret-ui tree::tests::view_cache::view_cache_disables_paint_cache_for_non_boundary_nodes tree::tests::view_cache::view_cache_allows_paint_cache_for_boundary_nodes tree::tests::view_cache::descendant_layout_invalidation_marks_contained_view_cache_root_dirty --no-fail-fast`
- Ordinary retained-subtree replay and invalidation gates:
  `cargo nextest run -p fret-ui tree::tests::hit_test::paint_cache_replays_subtree_ops_when_clean tree::tests::scroll_invalidation::scroll_offset_changes_do_not_replay_paint_cache tree::tests::models --no-fail-fast`
- Compile gates:
  `cargo check -p fret-ui --all-targets`
  and
  `cargo check -p fret-ui --features diagnostics --all-targets`
- Bootstrap diagnostics gate:
  `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics cache_root_boundary boundary_diagnostics_are_built_from_boundary_stats_with_cache_root_outcomes --no-fail-fast`

Observed result:

- paint-cache regression gate: `10 passed, 926 skipped`;
- view-cache paint-cache gating and contained-boundary dirty reason gate:
  `3 passed, 933 skipped`;
- ordinary retained-subtree replay and invalidation gates: `13 passed, 923 skipped`;
- bootstrap boundary diagnostics gate: `5 passed, 97 skipped`;
- both compile gates passed.

This slice is a runtime ownership-consolidation step, not a new perf claim. It deletes the
node-owned `PaintCacheEntry` field, routes true runtime-boundary entries through
`ViewBoundaryState::paint_cache`, routes plain retained paint-cache entries through
the retained plain-node entry store later named `UiTree::retained_paint_cache_entries`, and kept
`PaintCacheState` as the then-open previous-frame op storage owner for the next paint-cache replay
decision.

Most recent previous-frame paint recording slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M4G_PREVIOUS_FRAME_PAINT_RECORDING_SLICE_2026-05-14.md`
- Paint-cache regression gate:
  `cargo nextest run -p fret-ui tree::tests::paint_cache --no-fail-fast`
- Compile gates:
  `cargo check -p fret-ui --features diagnostics --all-targets`
  and
  `cargo check -p fret-ui --all-targets`
- Merge-conflict resolution support gates after pulling upstream:
  `python3 tools/check_diag_scripts_registry.py`,
  `python3 tools/check_workstream_catalog.py`,
  and `git diff --check`

Observed result:

- paint-cache regression gate: `11 passed, 929 skipped`;
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed;
- `cargo check -p fret-ui --all-targets`: passed;
- diagnostics script registry: up to date;
- workstream catalog: `370 dedicated directories, 47 standalone markdown files`;
- `git diff --check`: passed.

This slice is a local paint-cache ownership split, not a new perf claim. It replaces anonymous
`PaintCacheState::prev_ops`, `prev_text_blob_ids`, and `prev_fingerprint` fields with
`PreviousFramePaintRecording`, keeps generation and replay counters in `PaintCacheState`, and leaves
the final previous-frame recording owner decision open for the next paint-cache replay slice.

Most recent previous-frame paint replay span slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M4H_PREVIOUS_FRAME_PAINT_REPLAY_SPAN_SLICE_2026-05-14.md`
- Paint-cache regression gate:
  `cargo nextest run -p fret-ui tree::tests::paint_cache --no-fail-fast`
- Compile gates:
  `cargo check -p fret-ui --all-targets`
  and
  `cargo check -p fret-ui --features diagnostics --all-targets`
- Boundary/lane gates:
  `python3 tools/check_layering.py`,
  `python3 tools/check_workstream_catalog.py`,
  `python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null`,
  and `git diff --check`

Observed result:

- `cargo check -p fret-ui --all-targets`: passed;
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed;
- paint-cache regression gate: `12 passed, 929 skipped`;
- layering check: passed;
- workstream catalog: passed;
- `WORKSTREAM.json` validation: passed;
- `git diff --check`: passed.

This slice is a local paint-cache replay-owner narrowing step, not a new perf claim. It adds
text blob side-index spans to `PaintCacheEntry`, makes `PreviousFramePaintRecording` own entry
range validation and replay slicing, and uses the precomputed text blob side index during cache-hit
replay. The final previous-frame recording owner decision remains open.

Most recent paint-cache relax view-cache gating deletion slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M4I_PAINT_CACHE_RELAX_VIEW_CACHE_GATING_DELETION_SLICE_2026-05-14.md`
- View-cache paint-cache gating gate:
  `cargo nextest run -p fret-ui tree::tests::view_cache::view_cache_disables_paint_cache_for_non_boundary_nodes tree::tests::view_cache::view_cache_allows_paint_cache_for_boundary_nodes --no-fail-fast`
- Compile gates:
  `cargo check -p fret-ui --all-targets`
  and
  `cargo check -p fret-ui --features diagnostics --all-targets`
- Boundary/lane gates:
  `python3 tools/check_layering.py`,
  `python3 tools/check_workstream_catalog.py`,
  `python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null`,
  and `git diff --check`
- Source-deletion check:
  `rg -n "PAINT_CACHE_RELAX_VIEW_CACHE_GATING|paint_cache_relax_view_cache_gating|relax_view_cache_gating" crates/fret-ui/src apps/fret-ui-gallery/src -g '*.rs'`

Observed result:

- `cargo check -p fret-ui --all-targets`: passed;
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed;
- view-cache paint-cache gating gate: `2 passed, 933 skipped`;
- layering check: passed;
- workstream catalog: passed;
- `WORKSTREAM.json` validation: passed;
- `git diff --check`: passed;
- source-deletion check: no live runtime/code references remain.

This slice deletes an obsolete runtime env branch, not a new perf claim. It locks the M4E/M4F owner
model by making view-cache-active paint-cache recording boundary-gated without an experiment bypass.

Most recent hit-test-only paint-cache replay default slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M4J_HIT_TEST_ONLY_PAINT_CACHE_REPLAY_DEFAULT_SLICE_2026-05-14.md`
- Paint-cache regression gate:
  `cargo nextest run -p fret-ui tree::tests::paint_cache --no-fail-fast`
- Scroll anti-replay gate:
  `cargo nextest run -p fret-ui tree::tests::scroll_invalidation::scroll_offset_changes_do_not_replay_paint_cache --no-fail-fast`
- Compile gates:
  `cargo check -p fret-ui --all-targets`
  and
  `cargo check -p fret-ui --features diagnostics --all-targets`
- Boundary/lane gates:
  `python3 tools/check_layering.py`,
  `python3 tools/check_workstream_catalog.py`,
  `python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null`,
  and `git diff --check`
- Source-deletion check:
  `rg -n "FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY|paint_cache_allow_hit_test_only|PAINT_CACHE_ALLOW_HIT_TEST_ONLY|test_set_paint_cache_allow_hit_test_only" crates/fret-ui/src tools/diag-scripts docs/workstreams/perf-baselines -g '*.rs' -g '*.json'`

Observed result:

- `cargo check -p fret-ui --all-targets`: passed;
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed;
- paint-cache regression gate: `12 passed, 929 skipped`;
- scroll-offset anti-replay gate: `1 passed, 940 skipped`;
- layering check: passed;
- workstream catalog: passed;
- `WORKSTREAM.json` validation: passed;
- `git diff --check`: passed;
- source-deletion check: no live runtime/source/script/baseline references remain.

This slice is an env-knob deletion and behavior promotion step, not a new perf claim. Existing
dedicated probe evidence from `ui-perf-zed-smoothness-v1` proved the local path is reachable and
mixed to neutral on latency; M4J makes the correctness-checked local path canonical, prevents
descendant-originated hit-test-only dirtiness from replaying ancestors, and keeps the counters as
diagnostic evidence.

Most recent previous-frame recording retention slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M4K_PREVIOUS_FRAME_RECORDING_RETENTION_SLICE_2026-05-14.md`
- Paint-cache regression gate:
  `cargo nextest run -p fret-ui tree::tests::paint_cache --no-fail-fast`
- Compile gates:
  `cargo check -p fret-ui --all-targets`
  and
  `cargo check -p fret-ui --features diagnostics --all-targets`
- Boundary/lane gates:
  `python3 tools/check_layering.py`,
  `python3 tools/check_workstream_catalog.py`,
  `python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null`,
  and `git diff --check`
- Source-boundary check:
  `rg -n "paint_cache\\.previous_frame" crates/fret-ui/src -g '*.rs'`

Observed result:

- `cargo check -p fret-ui --all-targets`: passed;
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed;
- paint-cache regression gate: `12 passed, 929 skipped`;
- layering check: passed;
- workstream catalog: passed;
- `WORKSTREAM.json` validation: passed;
- `git diff --check`: passed;
- source-boundary check: no direct `paint_cache.previous_frame` access remains outside
  `PaintCacheState`.

This slice is an owner-retention decision, not a new perf claim. It makes
`PaintCacheState::previous_frame` private and records that `PreviousFramePaintRecording` remains the
per-tree previous-frame linear scene recording source while boundary `PaintCacheEntry` metadata
continues to own the replay decision.

Most recent retained paint-cache entry store slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M4L_RETAINED_PAINT_CACHE_ENTRY_STORE_SLICE_2026-05-14.md`
- Paint-cache regression gate:
  `cargo nextest run -p fret-ui tree::tests::paint_cache --no-fail-fast`
- View-cache paint-cache gating gate:
  `cargo nextest run -p fret-ui tree::tests::view_cache::view_cache_disables_paint_cache_for_non_boundary_nodes tree::tests::view_cache::view_cache_allows_paint_cache_for_boundary_nodes --no-fail-fast`
- Compile gates:
  `cargo check -p fret-ui --all-targets`
  and
  `cargo check -p fret-ui --features diagnostics --all-targets`
- Boundary/lane gates:
  `python3 tools/check_layering.py`,
  `python3 tools/check_workstream_catalog.py`,
  `python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null`,
  and `git diff --check`
- Source-deletion check:
  `rg -n "boundary_paint_cache_entries|test_boundary_paint_cache_side_store_has_entry|side store|side-store" crates/fret-ui/src -g '*.rs'`

Observed result:

- `cargo check -p fret-ui --all-targets`: passed;
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed;
- paint-cache regression gate: `12 passed, 929 skipped`;
- view-cache paint-cache gating gate: `2 passed, 939 skipped`;
- layering check: passed;
- workstream catalog: passed;
- `WORKSTREAM.json` validation: passed;
- `git diff --check`: passed;
- source-deletion check: no old `boundary_paint_cache_entries` / side-store naming remains in
  `crates/fret-ui/src`.

This slice is an owner-retention and naming cleanup step, not a new perf claim. It records
`UiTree::retained_paint_cache_entries` as the explicit retained plain-node paint-cache entry store,
while true runtime boundary entries remain in `ViewBoundaryState::paint_cache`.

Most recent code-editor closeout perf evidence:

- Perf output directory:
  `target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514`
- Threshold report:
  `target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/check.perf_thresholds.json`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/1778700500609/bundle.schema2.json`

Observed result:

- gate failures: `[]`,
- total p50/p95/max: `1205/1396/1396us`,
- layout p50/p95/max: `231/320/320us`,
- prepaint p50/p95/max: `243/339/339us`,
- paint p50/p95/max: `710/839/839us`,
- row scene replay hit rate: `99-100%`,
- renderer prepare/encode/upload counters stayed at `0`.

Worst-bundle attribution:

- `target/release/fretboard-dev diag stats target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/1778700500609/bundle.schema2.json --sort time --top 15`
- time sum: total `11285us`, layout `1009us`, prepaint `2905us`, paint `7371us`
- time p50/p95: total `1151/1396us`, layout `34/337us`, prepaint `255/375us`,
  paint `661/875us`
- hot p50/p95: `layout.engine_solve=0/132us`, `paint.widget=443/650us`,
  `paint.text_prepare=9/11us`
- `code_editor.paint_perf` planned/used replay entries: `2090/2090`
- `code_editor.paint_perf` rows replayed: `2885`
- `code_editor.paint_perf` p50/p95 `us_row_text`: `0/13us`
- `code_editor.paint_perf` p50/p95 `us_row_scene_prepaint_plan`: `54/86us`
- `code_editor.paint_perf` p50/p95 total: `182/385us`

Compared with the M1 boundary-diagnostics bottleneck evidence (`paint.widget` p95 `1494us`, paint
p95 `1737us`), the latest closeout run shows `paint.widget` p95 `650us` and paint p95 `875us`.
That exceeds the required 20-30% improvement for the selected paint-side bottleneck. Total p95
improved from `1811us` to `1396us`, also exceeding the 20% threshold.

## Correctness Gates

Use focused tests first:

```bash
cargo nextest run -p fret-ui <filter>
cargo test -p fret-ui-gallery --features gallery-full --lib <filter>
cargo test -p fret-ui-shadcn --lib <filter>
```

Required for boundary/invalidation changes:

```bash
python3 tools/check_layering.py
```

## Future Paint Stressor

If `ui-code-editor-resize-probes` stops catching the active paint bottleneck, add a narrower
code-editor paint stressor before continuing:

- route directly to the code-editor torture surface,
- keep `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`,
- stress row replay/content resolution without unrelated gallery setup noise,
- and seed a baseline/policy only after the script is deterministic.

## Global Closeout Gate Shape

The final global closeout must include:

- ADR gate: ADR 0327 accepted, revised into an accepted ADR, or superseded by an accepted
  equivalent.
- Correctness gates for the migrated boundary state, selected view-cache path, selected paint-cache
  path, and diagnostics schema.
- Perf gates for:
  - code-editor resize/paint,
  - one broader non-code-editor view-cache or paint-cache proof surface.
- Worst-bundle `diag stats` attribution for every perf claim.
- `python3 tools/check_layering.py`.
- Relevant `cargo check` commands for changed crates.
- `cargo fmt`.
- `git diff --check`.
- A final closeout audit with deletion/retention decisions for old runtime paths.

## Closeout Evidence

Closeout requires:

- final perf run paths,
- final worst-bundle attribution,
- deletion audit path,
- ADR alignment row,
- and exact commands for all promoted gates.
