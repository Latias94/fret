# M4R Second Proof Surface View-Cache Reuse Slice

Date: 2026-05-14
Status: implemented as proof-surface and gate promotion; superseded for final lane status by
`FINAL_CLOSEOUT_AUDIT_2026-05-14.md`

## Purpose

This slice closes the second proof-surface requirement in
`PROGRESS.md#Completion Contract` without changing runtime behavior.

The first proof surface is the code-editor resize/paint path. The second proof surface is
`tools/diag-scripts/ui-gallery/perf/ui-gallery-view-cache-toggle-perf-steady.json`, a broader
non-code-editor UI Gallery surface that exercises shared view-cache reuse and paint-cache replay
through normal gallery composition.

This is not a new optimization slice. It proves that the current Frame Pipeline v2 decisions can
explain a non-editor shared view-cache path through canonical boundary diagnostics:

- `debug.boundaries[]` reports build/layout/prepaint/paint outcomes;
- cache-root records use `layout_dependency`;
- paint-cache replay happens through the current boundary/retained-recording split;
- no live `contained_layout` bundle/schema field is reintroduced.

## Must-Be-True Outcomes

- The non-code-editor view-cache toggle surface runs under `diag perf` on macOS M4.
- The surface has a checked-in macOS M4 perf baseline and validates against it.
- Worst-bundle attribution shows stable view-cache reuse and bounded layout/paint cost.
- `diag stats` view-cache reuse gates pass on the worst bundle.
- The bundle contains boundary and cache-root `layout_dependency` diagnostics.
- New bundle/report output does not contain live `contained_layout` cache-root fields.
- This slice updates the workstream state without marking the global refactor complete before the
  final closeout audit.

## Proof Surface

Script:

```bash
tools/diag-scripts/ui-gallery/perf/ui-gallery-view-cache-toggle-perf-steady.json
```

Why this script:

- it is not code-editor-specific;
- it uses first-party UI Gallery composition rather than a synthetic unit fixture;
- it exercises shared view-cache roots and paint-cache replay;
- it already has Windows baseline history, so this slice only adds the missing macOS M4 proof.

## Perf Gate

New macOS M4 baseline:

```bash
docs/workstreams/perf-baselines/ui-gallery-view-cache-toggle-perf-steady.macos-m4.v1.json
```

Seed command:

```bash
target/release/fretboard-dev diag perf \
  tools/diag-scripts/ui-gallery/perf/ui-gallery-view-cache-toggle-perf-steady.json \
  --dir target/fret-diag-m4r-view-cache-toggle-baseline-seed-20260514 \
  --repeat 7 \
  --warmup-frames 5 \
  --reuse-launch \
  --sort time \
  --top 15 \
  --json \
  --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-view-cache-toggle-perf-steady.macos-m4.v1.json \
  --perf-baseline-headroom-pct 20 \
  --perf-baseline-threshold-surface ui \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --launch -- target/release/fret-ui-gallery
```

Observed seed result:

- repeat: `7`;
- worst bundle:
  `target/fret-diag-m4r-view-cache-toggle-baseline-seed-20260514/1778749752174/bundle.schema2.json`;
- aggregate total p50/p95/max: `574/600/600us`;
- aggregate layout p50/p95/max: `101/109/109us`;
- aggregate prepaint p50/p95/max: `39/44/44us`;
- aggregate paint p50/p95/max: `431/456/456us`;
- `top_view_cache_roots_reused`: `2/2` in the top frame for every run;
- generated thresholds: `max_top_total_us=720`, `max_top_layout_us=131`,
  `max_top_solve_us=0`.

Validation command:

```bash
target/release/fretboard-dev diag perf \
  tools/diag-scripts/ui-gallery/perf/ui-gallery-view-cache-toggle-perf-steady.json \
  --dir target/fret-diag-m4r-view-cache-toggle-baseline-validate-20260514 \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --sort time \
  --top 15 \
  --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-gallery-view-cache-toggle-perf-steady.macos-m4.v1.json \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --launch -- target/release/fret-ui-gallery
```

Observed validation result:

- threshold report:
  `target/fret-diag-m4r-view-cache-toggle-baseline-validate-20260514/check.perf_thresholds.json`;
- `failures`: `[]`;
- worst bundle:
  `target/fret-diag-m4r-view-cache-toggle-baseline-validate-20260514/1778749774595/bundle.schema2.json`;
- aggregate total p50/p95/max: `559/575/575us`;
- aggregate layout p50/p95/max: `96/96/96us`;
- aggregate prepaint p50/p95/max: `36/36/36us`;
- aggregate paint p50/p95/max: `427/443/443us`;
- `top_view_cache_roots_reused`: `2/2` in the top frame for every run.

This is a justified neutral result for the second proof surface: the slice does not change runtime
code, so it establishes and validates a stable macOS M4 contract instead of claiming an optimization
delta.

## Worst-Bundle Attribution

Command:

```bash
target/release/fretboard-dev diag stats \
  target/fret-diag-m4r-view-cache-toggle-baseline-validate-20260514/1778749774595/bundle.schema2.json \
  --sort time \
  --top 15 \
  --check-view-cache-reuse-min 2 \
  --check-view-cache-reuse-stable-min 2
```

Observed result:

- command passed;
- time sum: total `2820us`, layout `972us`, prepaint `373us`, paint `1475us`;
- time p50/p95: total `247/575us`, layout `97/99us`, prepaint `36/43us`,
  paint `114/443us`;
- hot p50/p95: `layout.engine_solve=0/0us`, `paint.widget=28/181us`,
  `paint.text_prepare=0/0us`;
- top frame: total/layout/prepaint/paint `575/96/36/443us`;
- top frame cache roots: `2`, reused: `2`;
- top frame paint-cache replayed ops: `519`;
- stable reuse check:
  `target/fret-diag-m4r-view-cache-toggle-baseline-validate-20260514/check.view_cache_reuse_stable.json`
  reports `failures=[]`, `reuse_snapshots=10`, and `reuse_streak_tail=10`.

## Diagnostics Schema Check

Structured check on the validation worst bundle:

- snapshots: `10`;
- boundary/cache-root records with `layout_dependency`: `50`;
- sample boundary outcomes include:
  - `layout_dependency=contained_when_bounds_known`,
  - `build_outcome=reused`,
  - `paint_outcome=scene_ops_replayed`,
  - `reuse_reason=marked_reuse_root`;
- sample cache-root dependencies include:
  - `contained_when_bounds_known`,
  - `parent_dependent`;
- `contained_layout_count=0`.

Cleanup check:

```bash
rg -n 'contained_layout' \
  target/fret-diag-m4r-view-cache-toggle-baseline-validate-20260514/1778749774595/bundle.schema2.json \
  target/fret-diag-m4r-view-cache-toggle-proof-20260514/stats.view-cache-toggle.worst.json
```

Observed result: no matches.

## Retained And Deleted Paths

Retained intentionally:

- `ViewCacheBuildBoundaryStore` remains the `GlobalElementId`-keyed declarative build-boundary
  store per M4M.
- `ViewBoundaryState` remains the retained-node runtime owner for boundary layout/prepaint/paint
  diagnostics and boundary paint-cache entry metadata.
- `PaintCacheState::PreviousFramePaintRecording` remains the per-tree linear scene recording source
  per M4K.
- `UiTree::retained_paint_cache_entries` remains the plain-node retained paint-cache entry store per
  M4L.

Deleted paths stay deleted:

- no `Node::paint_cache` fallback is used;
- no nested `debug.cache_roots[].boundary` compatibility truth is used;
- no live `cache_roots[].contained_layout` bundle/report field is emitted.

## Next

Completed by `FINAL_CLOSEOUT_AUDIT_2026-05-14.md`: the final closeout batch reran both proof
surfaces, focused tests, `cargo check`, layering, formatting/diff checks, and the deletion/retention
audit.
