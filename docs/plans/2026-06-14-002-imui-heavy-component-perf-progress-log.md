---
title: IMUI heavy-component perf progress log
type: progress-log
date: 2026-06-14
execution: code
related_plan: docs/plans/2026-06-14-001-imui-heavy-component-perf-architecture-audit-plan.md
---

# IMUI Heavy-Component Perf Progress Log

## Purpose

This document records execution-time findings, rejected experiments, and next decisions for the
active heavy-component performance goal. It complements the main plan rather than replacing it.

## Current Baseline

- The latest accepted code slice before this follow-up was
  `a2d54dbfe1 perf(gallery): avoid content cache on combobox page`.
- The current follow-up fixes the command/combobox virtual row viewport contract: virtualized
  `CommandPalette` rows now opt the internal `ScrollArea` out of unbounded viewport probing.
- `PAGE_COMBOBOX` opts out of whole-page content cache because the combobox page contains highly
  interactive query/list state.
- The view-cache investigation showed the wrong cache boundary clearly:
  - Whole-page content cache: `total=44825us`, `layout=40500us`,
    `layout_roots_apply=31049us`.
  - Shell-only view cache: `total=12973us`.
  - Combobox page content-cache opt-out: `total=12643us`, `layout_roots_apply=703us`.
  - No-view-cache baseline: `total=12950us`, with most remaining time in paint.
- The main bottleneck has shifted away from broad layout/root apply and toward paint, text
  preparation, renderer encode/finish, and occasional small layout bursts.
- A follow-up stats audit found that the newest `dev-fast-current` bundle's apparent worst frame was
  the scripted `capture_bundle` frame itself. The real top application frames after filtering are
  frames 145 and 146, not frame 148.
- The newest bounded-viewport run reports `total=9591us`, `layout=4436us`,
  `layout.engine_solve=829us`, `paint=4417us`, `roots.apply=516us`, and
  `script_capture_skipped=1`.
- Virtual-list telemetry now shows the long command list as `viewport=272px`, `window_range=0..8`,
  `overscan=8`, and `count=250`; the list model still describes all items, but layout only pays for
  the visible virtual window.
- The current checked-in combobox gate is
  `docs/workstreams/perf-baselines/ui-gallery-combobox-filter-select-steady.dev-fast.windows-rtx4090.v1.json`.
  It was seeded from the latest `target\dev-fast\fret-ui-gallery.exe`, not from the stale release
  binary. Seed p50/p95/max total is `12671/12869/12869us`; the reverse gate passed with
  `failures=[]`.
- This is still above the strict 120Hz target. Treat the gate as a regression guard for the fixed
  failure classes, not as closeout evidence for general-app component parity with GPUI/Zed.
- The latest `fret-ui` view-cache observation-collapse slice keeps that gate green and removes a
  real shared-mechanism cost inside view-cache frames. The accepted implementation relocates only
  descendant observation entries to their nearest view-cache root instead of draining and rebuilding
  the full model/global observation indexes.
- The accepted dev-fast gate after this slice reports worst frame `total=11575us`, `layout=5994us`,
  `layout.engine_solve=927us`, `paint=4900us`, and `failures=[]`. In the comparable baseline gate,
  worst frame was `total=12666us`, `layout=8072us`, `layout.engine_solve=1155us`, and `paint=3953us`.
- The targeted subphase moved as intended: `layout_collapse_layout_observations_time_us` dropped
  from about `1783us` to `386us`, while `paint_collapse_observations_time_us` dropped from about
  `511us` to `137us` on the compared worst frames.
- The current accepted recipe-layer slice is an item-only `CommandPalette` render-row fast path.
  Pure item lists now skip the general pending-row/group/separator trimming pipeline while keeping
  the same cmdk scoring, stable sort, `force_mount`, item vector, and group-slot contract.
- The dev-fast combobox gate after this slice reports `failures=[]`, worst frame `total=11215us`,
  `layout=6978us`, `layout.engine_solve=1211us`, and `paint=3560us`; evidence bundle:
  `target/fret-diag/gate-combobox-filter-select-devfast-command-item-only-fast-path/1781432649868/bundle.schema2.json`.

## Decisions

### D1. Do not cache the whole combobox page content root

Whole-page content caching is the wrong abstraction for pages with active overlays, query state,
virtual rows, and diagnostics selectors. It makes the cache boundary too broad and turns ordinary
interactive invalidation into large root-apply work.

The fix should stay at the gallery policy layer: pages with highly interactive long-list content
should opt out of whole-page content caching. This does not require a `fret-ui` mechanism rewrite.

### D2. Treat section-level doc cache as an unproven experiment

An exploratory edit wrapped each `DocSection` in `cx.cached_subtree(...)`. The code compiled with:

```text
cargo check -p fret-ui-gallery --profile dev-fast -j 1
```

The experiment was reverted before commit because it did not yet meet the evidence bar:

- The first perf command accidentally launched the existing release binary, so it did not measure
  the local `doc_layout.rs` edit.
- Default `CachedSubtreeProps` has no section-specific cache key. That is only safe for content
  whose render inputs are known to be stable through recorded model/global observations.
- The docs scaffold mixes static sections, interactive examples, code tabs, test-id decoration, and
  focus filtering. A broad section boundary could become another oversized cache boundary unless it
  is keyed and scoped deliberately.

Do not revive this approach unless the next slice adds explicit cache keys and a same-binary perf
comparison.

### D3. Exclude script capture frames from perf attribution

`capture_bundle` is diagnostic work, not application work. Counting the bundle dump frame in
`diag stats` can invert the next optimization decision by making a script artifact look like the
application's worst interaction frame.

The fix belongs in `fret-diag` stats attribution, not in shadcn components or runtime scheduling.
Stats now derive a capture-frame filter from the bundle-adjacent `script.result.json` sidecar and
apply it to both materialized schema2 bundles and `frames.index.json` stats-lite paths. The report
prints `script_capture_skipped` so future comparisons can tell when a diagnostic frame was excluded.

### D4. Keep the virtual-list viewport fix in the recipe layer

The bad behavior was not global `ScrollArea` semantics. It was the `CommandPalette` virtualized-row
branch composing an internal scroll container where an unbounded probe let the virtual list observe
the full content extent as its viewport.

The fix stays local: virtualized command rows set `viewport_probe_unbounded(false)` on the internal
`ScrollArea`. That keeps ordinary `ScrollArea` behavior unchanged and avoids pushing a policy
decision into `fret-ui`.

### D5. Name dev-fast baselines explicitly

The available release `fret-ui-gallery.exe` was built before the latest command/combobox fixes, and
a fresh release build had already exceeded the local time budget. Using that binary would seed a
misleading contract from stale code.

The checked-in combobox baseline therefore includes `dev-fast` in its filename and stays out of the
formal Zed smoothness contract matrix. It is valid for the active workstream's quick regression loop:
it should catch unbounded virtual-list viewport probing, full-list row materialization, and broad
whole-page cache invalidation. A release Windows RTX4090 baseline remains a separate follow-up once a
fresh release gallery binary is available.

### D6. Collapse view-cache observations incrementally

The first attempted view-cache collapse optimization was a conservative pre-scan fast path: if all
observation entries were already rooted, return the original index. The evidence rejected it for the
current combobox path. The hot frames still contained descendant observations that needed uplift, so
the implementation still drained and rebuilt the full index; the perf gate stayed around
`total=12578us`, and layout collapse stayed around `1872us`.

The accepted change keeps the same correctness rule but changes the amount of work: compute the
nodes that actually have a distinct nearest view-cache root, remove only those nodes from
`by_node`/reverse indexes, and merge their masks into the target root. Entries already on a
view-cache root or outside a view-cache subtree remain in place.

This belongs in `fret-ui` rather than a recipe crate because model/global observation collapse is a
shared cache mechanism. The new tests cover three invariants for both model and global indexes:
already-rooted observations stay intact, descendant observations still uplift to the nearest root,
and root plus descendant observations for the same dependency union their masks instead of
overwriting each other.

### D7. Reject derived observed-deps presence

An experiment removed the explicit `observed_deps_rendered` / `observed_deps_next` sets and derived
dependency presence from the current model/global observation maps. The intent was to remove hundreds
of empty host-widget observed-dependency lookups during paint.

The evidence rejected the change. The combobox dev-fast gate regressed to `total=23587us`,
`layout=13795us`, and `paint=8701us`, with `failures=5`; evidence bundle:
`target/fret-diag/gate-combobox-filter-select-devfast-observed-deps-presence-derived/1781430201371/bundle.schema2.json`.

The architectural conclusion is that the explicit presence sets are not just a broad cache of the
current observation maps. They preserve dependency-presence continuity across view-cache reuse and
touch paths. Any future optimization in this area must keep that continuity explicit and add stronger
view-cache reuse tests before touching the mechanism again.

### D8. Do not spend the next slice on `CommandAvailabilityCx` input-context borrowing

An experiment changed `CommandAvailabilityCx` to borrow `InputContext` instead of cloning it for each
widget availability route. The focused command-availability publication test passed, and the crate
compiled, but the perf gate did not produce acceptable evidence.

Two runs were noisy in different ways:

- Before rebuilding `target\dev-fast\fret-ui-gallery.exe`, the run reported `total=10451us` but
  failed pointer-move thresholds.
- After rebuilding the dev-fast gallery binary, the run failed layout/solve thresholds with
  `total=14365us`, `layout=9722us`, and `layout.engine_solve=1911us`; evidence bundle:
  `target/fret-diag/gate-combobox-filter-select-devfast-command-availability-borrowed-input-rerun/1781431629778/bundle.schema2.json`.

The architectural conclusion is that the command availability publication Interface is still a real
future Module candidate, but this small public-context borrowing change is not enough leverage to
justify breaking the API or continuing the loop. Prefer a deeper publication Module or move to the
CommandPalette query-window seam before revisiting this area.

### D9. Accept an item-only `CommandPalette` render-row fast path

The current `CommandPalette` long-list path still uses the general render-row builder even when all
entries are plain root items. That general pipeline earns its depth for grouped, separator, loading,
and custom-child palettes, but it becomes shallow overhead for the combobox search adapter: the
caller only needs scored items, `CommandPaletteRenderRow::Item` rows, and a navigation group vector
filled with `None`.

The accepted change keeps the seam local to `ecosystem/fret-ui-shadcn/src/command.rs`. It does not
change the cmdk scoring function, does not change sort stability, does not virtualize a new case,
and does not push policy into `fret-ui`. The focused regression test locks the important invariant:
item-only fast-path results still carry an `item_groups` vector with one `None` slot per item so the
navigation snapshot remains shape-compatible with the general path.

The perf evidence is positive but modest. The gate stayed green (`failures=[]`) with worst frame
`total=11215us`, so this is worth landing as a low-risk recipe improvement. It is not enough to call
the heavy-component lane complete. The next larger opportunity is still a deeper CommandPalette
query/window Module or a paint/text prepared-layout Module, not more ad-hoc micro-branches.

## Current Architecture Read

The current evidence argues against a single framework-level rewrite as the next move. The large
wins came from a sequence of narrower seams:

- Component policy: delayed combobox query clearing during close presence.
- Component rendering: virtualized long command/combobox rows.
- Component rendering: item-only command row construction skips the general grouped-row pipeline.
- Component composition: bounded internal scroll probing for the virtualized command row branch.
- Shared mechanism: command availability interest caching.
- Shared mechanism: incremental view-cache observation collapse.
- Declarative diff: stable single-line plain text content changes avoid layout invalidation.
- Gallery policy: combobox page opts out of whole-page content cache.

This supports a mixed strategy: optimize component recipes where their composition is wasteful, but
promote the fix into `fret-ui` only when repeated component evidence points at a shared mechanism.

## Next Work

1. Continue optimizing the combobox long-list tail; the dev-fast gate is a floor, not the target.
2. Use `diag stats --sort cpu_cycles --top 30` and `--sort time` on each newest bundle before
   changing code again.
3. Treat stats output without `script_capture_skipped` support as stale for scripted capture bundles.
4. If layout remains above budget, focus on popup/overlay solve and scroll-area geometry first; the
   main-page corrected-content relayout and full-list materialization failures are already fixed.
5. If paint/text preparation dominates, inspect static text/code-block/icon preparation and paint
   cache key churn before changing layout code.
6. If renderer finish/encode dominates with low CPU signal, treat it as scheduling/renderer tail
   rather than a component tree problem until a trace proves otherwise.
7. Keep `CommandPalette`, `Combobox`, `DataTable` toolbar recipes, `Sidebar`, and carousel-heavy
   examples as the next heavy-component candidates. Avoid widening to every shadcn recipe until one
   candidate produces a reproducible tail.
8. Do not retry the rejected `observed_deps_presence` derivation or the small
   `CommandAvailabilityCx` borrowing tweak without a new hypothesis and stronger view-cache or
   publication tests.
9. If the next slice stays inside `CommandPalette`, prefer a query/window Module that owns filtered
   item scoring, navigation slots, visible-row metadata, and virtual range inputs behind one
   testable Interface.

## Verification Notes

- `cargo check -p fret-ui-gallery --profile dev-fast -j 1` passed after returning
  `apps/fret-ui-gallery/src/ui/doc_layout.rs` to the mainline shape.
- `cargo run -p fretboard-dev -- diag stats target/fret-diag/imui-heavy-perf-probes-combobox-devfast-current/1781414534335/bundle.schema2.json --sort time --top 5`
  now reports `script_capture_skipped=1`; top frame moved from the old capture frame 148 to frame
  145 (`total=21041us`) followed by frame 146 (`total=17686us`).
- `cargo run -p fretboard-dev -- diag stats target/fret-diag/imui-heavy-perf-probes-combobox-devfast-current/1781414534335/bundle.schema2.json --sort cpu_cycles --top 5`
  reports the same `script_capture_skipped=1` and the same top application frame 145.
- Focused Rust tests in this lane often time out during Windows test target compilation rather than
  failing assertions. Treat check/build plus perf bundles as the practical gate until the local test
  cache is warm or timeout budgets are raised.
- `cargo test -p fret-ui-shadcn --lib command_palette_virtualized_rows_use_bounded_scroll_viewport --profile dev-fast -j 1`
  passed and locks the bounded virtual-row viewport behavior.
- `cargo build -p fret-ui-gallery --profile dev-fast -j 1` passed before the latest perf run.
- `target\debug\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\perf\ui-gallery-combobox-filter-select-steady.json --dir target\fret-diag\imui-heavy-perf-probes-combobox-devfast-bounded-viewport --repeat 1 --warmup-frames 2 --timeout-ms 240000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_UI_GALLERY_START_PAGE=combobox --launch -- target\dev-fast\fret-ui-gallery.exe`
  passed with worst frame `total=9591us`; evidence bundle
  `target/fret-diag/imui-heavy-perf-probes-combobox-devfast-bounded-viewport/1781424587044/bundle.schema2.json`.
- `target\debug\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\perf\ui-gallery-combobox-filter-select-steady.json --dir target\fret-diag\baseline-combobox-filter-select-devfast-windows-rtx4090-v1 --repeat 3 --warmup-frames 5 --prewarm-script tools\diag-scripts\_prelude\tooling-suite-prewarm-fonts.json --prelude-script tools\diag-scripts\_prelude\tooling-suite-prelude-reset-diagnostics.json --perf-baseline-out docs\workstreams\perf-baselines\ui-gallery-combobox-filter-select-steady.dev-fast.windows-rtx4090.v1.json --perf-baseline-headroom-pct 20 --perf-baseline-threshold-surface ui --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_UI_GALLERY_START_PAGE=combobox --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- target\dev-fast\fret-ui-gallery.exe`
  passed and wrote the dev-fast baseline. Seed p50/p95/max total/layout/solve is
  `12671/12869/12869us`, `7762/8074/8074us`, and `893/1157/1157us`.
- `target\debug\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\perf\ui-gallery-combobox-filter-select-steady.json --dir target\fret-diag\gate-combobox-filter-select-devfast-windows-rtx4090-v1 --repeat 1 --warmup-frames 5 --prewarm-script tools\diag-scripts\_prelude\tooling-suite-prewarm-fonts.json --prelude-script tools\diag-scripts\_prelude\tooling-suite-prelude-reset-diagnostics.json --perf-baseline docs\workstreams\perf-baselines\ui-gallery-combobox-filter-select-steady.dev-fast.windows-rtx4090.v1.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_UI_GALLERY_START_PAGE=combobox --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- target\dev-fast\fret-ui-gallery.exe`
  passed; `target/fret-diag/gate-combobox-filter-select-devfast-windows-rtx4090-v1/check.perf_thresholds.json`
  has `failures=[]`.
- `target\debug\fretboard-dev.exe diag stats target\fret-diag\gate-combobox-filter-select-devfast-windows-rtx4090-v1\1781426027088\bundle.schema2.json --sort time --top 5`
  reports `script_capture_skipped=1` and worst frame `total=12666us`, `layout=8072us`,
  `layout.engine_solve=1155us`, `paint=3953us`, `renderer.finish=1511us`, and
  `renderer.encode=800us`.
- `python -m json.tool docs\workstreams\perf-baselines\ui-gallery-combobox-filter-select-steady.dev-fast.windows-rtx4090.v1.json`
  passed.
- `cargo fmt -p fret-ui` passed after incremental view-cache observation collapse.
- `git diff --check` passed after incremental view-cache observation collapse.
- `cargo check -p fret-ui -j 1` passed after incremental view-cache observation collapse.
- `cargo check -p fret-ui --tests -j 1` passed after incremental view-cache observation collapse.
- `cargo test -p fret-ui --lib view_cache_observation_collapse --profile dev-fast -j 1`
  passed: 3 tests, 0 failures.
- `cargo build -p fret-ui-gallery --profile dev-fast -j 1` passed after incremental view-cache
  observation collapse.
- `target\debug\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\perf\ui-gallery-combobox-filter-select-steady.json --dir target\fret-diag\gate-combobox-filter-select-devfast-viewcache-collapse-incremental --repeat 1 --warmup-frames 5 --prewarm-script tools\diag-scripts\_prelude\tooling-suite-prewarm-fonts.json --prelude-script tools\diag-scripts\_prelude\tooling-suite-prelude-reset-diagnostics.json --perf-baseline docs\workstreams\perf-baselines\ui-gallery-combobox-filter-select-steady.dev-fast.windows-rtx4090.v1.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_UI_GALLERY_START_PAGE=combobox --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- target\dev-fast\fret-ui-gallery.exe`
  passed with `failures=[]`; evidence bundle
  `target/fret-diag/gate-combobox-filter-select-devfast-viewcache-collapse-incremental/1781428813597/bundle.schema2.json`.
- `target\debug\fretboard-dev.exe diag stats target\fret-diag\gate-combobox-filter-select-devfast-viewcache-collapse-incremental\1781428813597\bundle.schema2.json --sort time --top 6`
  reports `script_capture_skipped=1` and worst frame `total=11575us`, `layout=5994us`,
  `layout.engine_solve=927us`, `paint=4900us`, `renderer.finish=1493us`, and
  `renderer.encode=819us`.
- Rejected experiment: deriving observed dependency presence from model/global observation maps
  regressed the combobox dev-fast gate to `total=23587us`, `layout=13795us`, `paint=8701us`, with
  `failures=5`. The code was reverted before continuing.
- Rejected experiment: changing `CommandAvailabilityCx` to borrow `InputContext` compiled and passed
  `cargo test -p fret-ui --lib action_availability_snapshot_caches_declarative_interest_within_publication --profile dev-fast -j 1`,
  but the rebuilt gallery perf gate failed on layout/solve thresholds. The code was reverted before
  continuing.
- `cargo fmt -p fret-ui-shadcn` passed after the item-only command render-row fast path.
- `cargo check -p fret-ui-shadcn -j 1` passed after the item-only command render-row fast path.
- `cargo test -p fret-ui-shadcn --lib command_palette_item_only_fast_path_keeps_navigation_group_slots --profile dev-fast -j 1`
  passed and guards the item-only navigation slot contract.
- `cargo test -p fret-ui-shadcn --lib command_palette_virtualized_rows_use_bounded_scroll_viewport --profile dev-fast -j 1`
  passed after the fast path.
- `cargo build -p fret-ui-gallery --profile dev-fast -j 1` passed after the fast path.
- `target\debug\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\perf\ui-gallery-combobox-filter-select-steady.json --dir target\fret-diag\gate-combobox-filter-select-devfast-command-item-only-fast-path --repeat 1 --warmup-frames 5 --prewarm-script tools\diag-scripts\_prelude\tooling-suite-prewarm-fonts.json --prelude-script tools\diag-scripts\_prelude\tooling-suite-prelude-reset-diagnostics.json --perf-baseline docs\workstreams\perf-baselines\ui-gallery-combobox-filter-select-steady.dev-fast.windows-rtx4090.v1.json --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_UI_GALLERY_START_PAGE=combobox --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- target\dev-fast\fret-ui-gallery.exe`
  passed with `failures=[]`; evidence bundle
  `target/fret-diag/gate-combobox-filter-select-devfast-command-item-only-fast-path/1781432649868/bundle.schema2.json`.
- The fast-path gate's worst frame was `total=11215us`, `layout=6978us`,
  `layout.engine_solve=1211us`, and `paint=3560us`; the result protects against regression but
  remains above strict 120Hz.
