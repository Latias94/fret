# Final Closeout Audit - 2026-05-14

Status: global Frame Pipeline v2 closeout complete for the accepted ADR 0327 workstream contract.

This audit closes `ui-frame-pipeline-v2-fearless-refactor-v1` as an execution lane. Future
performance work, renderer display-list evolution, or additional proof surfaces should start as
narrow follow-ons instead of reopening this lane.

## Completion Criteria

The workstream completion contract required:

- accepted ADR 0327 or an accepted replacement;
- one canonical runtime explanation for build/layout/prepaint/paint reuse and diagnostics;
- view-cache, paint-cache, layout containment, prepaint, and scene-fragment paths migrated to the
  boundary model or explicitly retained with a current reason;
- direct `contained_layout` authoring replaced by a boundary-hint API;
- replaced private runtime paths, compatibility diagnostics, and unowned env knobs deleted or
  explicitly retained;
- code-editor resize/paint plus one non-code-editor proof surface passing correctness, perf,
  attribution, layering, cargo check, and deletion-audit gates.

## Verdict

The completion contract is satisfied.

`ViewBoundaryState` is the canonical retained-node runtime owner for layout dirty state, layout
dependency metadata, boundary prepaint outputs, boundary scene fragments, boundary paint-cache entry
metadata, and boundary diagnostics. The intentionally retained stores below are no longer open
migration leftovers; they are documented final mechanisms for identities or data shapes that do not
belong in the retained-node boundary table.

## Canonical Runtime Path

Build:

- `ViewCacheBuildBoundaryStore` remains inside `WindowElementState` as the
  `GlobalElementId`-keyed declarative build-boundary store.
- M4M accepts this owner because build-time cache-hit membership is known before retained `NodeId`
  state is refreshed.
- Runtime membership is rebound to current live nodes before reuse touches retained-node state.

Layout:

- runtime view-cache flags now store `ViewCacheParentLayoutDependency`, not a
  `contained_layout` boolean;
- hot paths derive `layout_contained_when_bounds_known()` only as a predicate;
- `ViewBoundaryState` records boundary layout dependency and dirty source/detail;
- cache-root diagnostics and reports use `layout_dependency`.

Prepaint and scene fragments:

- typed prepaint output storage is boundary-owned;
- code-editor row replay uses boundary-owned
  `CanvasSceneFragment<RowSceneFragmentPayload>`;
- stale replay candidates are validated before paint consumes them.

Paint replay:

- true runtime boundaries own `PaintCacheEntry` metadata through
  `ViewBoundaryState::paint_cache`;
- `UiTree::retained_paint_cache_entries` remains the plain-node entry store for nodes that are not
  runtime boundaries and migrates entries into `ViewBoundaryState::paint_cache` when a node becomes
  a boundary;
- `PreviousFramePaintRecording` remains inside `PaintCacheState` as the per-tree linear
  previous-frame `Scene` recording source.

Diagnostics:

- `debug.boundaries[]` is the canonical boundary truth;
- cache-root summaries are derived views that carry `layout_dependency`;
- live bundle/report output no longer emits `contained_layout` as a cache-root schema field.

## Retained Mechanisms

Retained intentionally:

- `ViewCacheBuildBoundaryStore` in `WindowElementState`, per M4M, as the declarative
  `GlobalElementId` build-boundary mechanism.
- `UiTree::retained_paint_cache_entries`, per M4L, as the plain-node paint-cache entry store.
- `PreviousFramePaintRecording` in `PaintCacheState`, per M4K, as the tree-wide previous-frame
  scene recording carrier required by the current linear `Scene` contract.
- validation-only subtree dirty aggregation env knobs:
  `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE` and
  `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE_PANIC`; these audit the canonical path and do
  not select a different runtime behavior.

These retained mechanisms have owners, tests, and workstream/ADR reasons. They are not compatibility
fallbacks.

## Deleted Or Retired Paths

Deleted from live runtime paths:

- `Node::paint_cache`;
- node-owned typed prepaint output storage as the active owner;
- code-editor-owned row replay frame carriers;
- generic prepaint-output carrier for row-scene replay;
- code-editor-local fixed-row rect reconstruction for replay planning;
- `dirty_cache_roots`, `dirty_cache_root_reasons`, and `mark_cache_root_dirty(...)`;
- nested `debug.cache_roots[].boundary` bundle truth;
- live cache-root `contained_layout` bundle/report field;
- `ViewCacheFlags::contained_layout`;
- `FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING`;
- `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY`;
- layout default-path env branches:
  `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION`,
  `FRET_UI_LAYOUT_ENGINE_SWEEP`,
  `FRET_UI_LAYOUT_SKIP_REQUEST_BUILD_TRANSLATION_ONLY`, and
  `FRET_UI_LAYOUT_FLOW_SKIP_BARRIER_CLEAN_CHILDREN`.

Historical docs and absence assertions may still mention retired names as evidence.

## Proof Surfaces

Code-editor resize/paint:

- gate directory: `target/fret-diag-final-closeout-code-editor-20260514`;
- threshold report: `target/fret-diag-final-closeout-code-editor-20260514/check.perf_thresholds.json`;
- worst bundle:
  `target/fret-diag-final-closeout-code-editor-20260514/1778751374184/bundle.schema2.json`;
- threshold failures: `[]`;
- observed max gate row: total `1601us`, layout `368us`, layout solve `141us`;
- aggregate total p50/p95/max: `1221/1601/1601us`;
- aggregate layout p50/p95/max: `305/368/368us`;
- aggregate prepaint p50/p95/max: `246/350/350us`;
- aggregate paint p50/p95/max: `860/883/883us`;
- row scene replay hit rate: `99%`;
- renderer prepare/encode/upload counters: `0`.

Worst-bundle attribution:

- time sum: total/layout/prepaint/paint = `11583/1074/2972/7537us`;
- time p50/p95: total `1125/1601us`, layout `36/368us`, prepaint `265/377us`,
  paint `660/883us`;
- hot p50/p95: `layout.engine_solve=0/141us`, `paint.widget=451/672us`,
  `paint.text_prepare=9/11us`;
- `code_editor.paint_perf` planned/used replay entries: `2090/2090`;
- rows painted/replayed/stored: `2890/2885/5`;
- `code_editor.paint_perf` p50/p95 total: `192/409us`;
- `us_row_scene_prepaint_plan` p50/p95: `58/86us`;
- `us_row_text` p50/p95: `0/12us`.

Non-code-editor view-cache toggle:

- gate directory: `target/fret-diag-final-closeout-view-cache-toggle-20260514`;
- threshold report:
  `target/fret-diag-final-closeout-view-cache-toggle-20260514/check.perf_thresholds.json`;
- worst bundle:
  `target/fret-diag-final-closeout-view-cache-toggle-20260514/1778751410837/bundle.schema2.json`;
- threshold failures: `[]`;
- observed max gate row: total `593us`, layout `113us`, layout solve `0us`;
- aggregate total p50/p95/max: `593/593/593us`;
- aggregate layout p50/p95/max: `109/113/113us`;
- aggregate prepaint p50/p95/max: `41/41/41us`;
- aggregate paint p50/p95/max: `439/443/443us`;
- top-frame view-cache roots reused: `2/2`.

Worst-bundle attribution:

- command passed with `--check-view-cache-reuse-min 2` and
  `--check-view-cache-reuse-stable-min 2`;
- time sum: total/layout/prepaint/paint = `2991/1092/392/1507us`;
- time p50/p95: total `267/593us`, layout `109/114us`, prepaint `38/42us`,
  paint `118/443us`;
- hot p50/p95: `layout.engine_solve=0/0us`, `paint.widget=27/175us`,
  `paint.text_prepare=0/0us`;
- top frame: cache roots `2`, reused `2`, paint-cache replayed ops `604`;
- steady frames replayed `757` paint-cache ops with no layout solve.

Diagnostics schema checks:

- code-editor bundle: `10` snapshots, `20` boundary records, `10` cache-root records,
  `10` cache-root records with `layout_dependency`, `0` cache-root records with
  `contained_layout`;
- view-cache toggle bundle: `10` snapshots, `30` boundary records, `20` cache-root records,
  `20` cache-root records with `layout_dependency`, `0` cache-root records with
  `contained_layout`;
- observed dependency vocabulary on proof bundles:
  `contained_when_bounds_known` and `parent_dependent`.

## Final Gates

Compile gates:

```bash
cargo check -p fret-ui --all-targets
cargo check -p fret-ui --features diagnostics --all-targets
cargo check -p fret-bootstrap --features ui-app-driver,diagnostics --all-targets
cargo check -p fret-diag --all-targets
```

Observed result: all passed.

Correctness gates:

```bash
cargo nextest run -p fret-ui tree::tests::paint_cache tree::tests::view_cache --no-fail-fast
cargo nextest run -p fret-ui barrier_subtree_layout_dirty_aggregation \
  subtree_layout_dirty_underflow_repair scroll_handle_invalidation_harness --no-fail-fast
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics \
  cache_root_boundary --no-fail-fast
cargo nextest run -p fret-diag \
  bundle_stats_preserves_cache_root_boundary_summary --no-fail-fast
```

Observed result:

- `fret-ui` paint/view cache: `34 passed, 908 skipped`;
- `fret-ui` layout dirty/scroll invalidation: `11 passed, 931 skipped`;
- `fret-bootstrap` cache-root boundary diagnostics: `5 passed, 125 skipped`;
- `fret-diag` bundle stats summary: `1 passed, 818 skipped`.

Perf gates:

```bash
target/release/fretboard-dev diag perf ui-code-editor-resize-probes \
  --dir target/fret-diag-final-closeout-code-editor-20260514 \
  --repeat 3 --warmup-frames 5 --reuse-launch --sort time --top 15 --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

```bash
target/release/fretboard-dev diag perf \
  tools/diag-scripts/ui-gallery/perf/ui-gallery-view-cache-toggle-perf-steady.json \
  --dir target/fret-diag-final-closeout-view-cache-toggle-20260514 \
  --repeat 3 --warmup-frames 5 --reuse-launch --sort time --top 15 --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-gallery-view-cache-toggle-perf-steady.macos-m4.v1.json \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --launch -- target/release/fret-ui-gallery
```

Observed result: both threshold reports have `failures=[]`.

Source and schema deletion checks:

```bash
rg -n "Node::paint_cache|paint_cache: Option|debug\\.cache_roots\\[\\]\\.boundary|cache_roots\\[\\]\\.boundary|FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING|FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY|FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION\\\"|FRET_UI_LAYOUT_ENGINE_SWEEP|FRET_UI_LAYOUT_SKIP_REQUEST_BUILD_TRANSLATION_ONLY|FRET_UI_LAYOUT_FLOW_SKIP_BARRIER_CLEAN_CHILDREN|layout_engine_sweep_policy|layout_skip_request_build_translation_only|layout_flow_skip_barrier_clean_children|subtree_layout_dirty_aggregation_enabled\\(\\)|\\bagg_enabled\\b" \
  crates ecosystem apps tools -g '*.rs' -g '*.json'
rg -n "UiDebugCacheRootStats.*contained_layout|UiCacheRootStatsV1.*contained_layout|BundleStatsCacheRoot.*contained_layout|r\\.get\\(\"contained_layout\"\\)|\"contained_layout\"\\.to_string\\(\\)|contained_layout:" \
  crates/fret-ui/src/tree/debug crates/fret-ui/src/tree/ui_tree_debug \
  ecosystem/fret-bootstrap/src/ui_diagnostics crates/fret-diag/src \
  apps/fret-ui-gallery/src crates/fret-ui/src/tree/tests -g '*.rs'
```

Observed result: no live matches.

Boundary/lane gates:

```bash
cargo fmt --check
python3 tools/check_layering.py
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null
python3 -m json.tool docs/workstreams/perf-baselines/ui-gallery-view-cache-toggle-perf-steady.macos-m4.v1.json >/dev/null
git diff --check
```

Observed result: the final post-audit run of all commands passed.

## Future Work Outside This Lane

Future work should not be treated as missing Frame Pipeline v2 closeout:

- renderer `Scene` display-list contract evolution for per-boundary recording sources;
- additional proof surfaces beyond the two required by the completion contract;
- stricter code-editor paint stressors if the existing resize proof surface stops catching the
  active bottleneck;
- Linux-specific performance closure;
- component-policy work in ecosystem crates.

## Prompt-to-Artifact Checklist

| Requirement | Evidence | Verdict |
| --- | --- | --- |
| ADR 0327 accepted or superseded | `M0_CONTRACT_FREEZE_2026-05-14.md`; ADR status accepted | Complete |
| Final boundary/equivalent owns canonical runtime explanation | `ViewBoundaryState` runtime ownership plus explicit retained mechanisms above | Complete |
| View-cache and paint-cache paths migrated or retained with reason | M4M, M4E-M4L, final retained mechanisms | Complete |
| Direct `contained_layout` authoring replaced | M4C `ViewBoundaryHints`; `contain_layout_when_bounds_known(...)` | Complete |
| Replaced old runtime paths deleted or audited | Deletion audit above; source checks no live matches | Complete |
| Boundary diagnostics canonical | `debug.boundaries[]`; cache-root `layout_dependency`; bundle checks | Complete |
| Two proof surfaces pass | code-editor resize/paint; view-cache toggle | Complete |
| Perf gates and worst-bundle attribution pass | final perf directories and `diag stats` summaries above | Complete |
| Layering, cargo check, focused nextest, diff checks pass | final gate list above | Complete |
| Final deletion/retention audit exists | this file | Complete |
