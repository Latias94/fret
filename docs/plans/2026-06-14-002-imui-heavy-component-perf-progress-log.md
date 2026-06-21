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
- The editor-controls follow-up narrowed several single-child shells in `fret-ui-editor`:
  `PropertyGrid` now returns a single default row directly, `editor_input_group_row` returns a lone
  child directly, `ColorEdit` input mounts `TextInput` without an outer `PointerRegion`, and
  `ColorEdit` popup numeric/options surfaces now return a lone visible child directly when no
  `test_id` is requested.
- `EnumSelect` trigger now renders the caret as a direct centered `Flex` around the icon instead of
  wrapping the icon in a dedicated caret `Container`.
- Validation for that slice passed with
  `cargo nextest run -p fret-ui-editor -j 1 --no-fail-fast` and
  `cargo nextest run -p fret-ui-editor enum_select --no-fail-fast`.
- `TextAssistField` then tightened its inline empty-state shell: inline now returns `TextField`
  directly when there is no `inline_panel` and no `empty_label`, while inline empty-label cases
  still keep the shell. The regression tests now cover inline direct return, overlay direct
  return, and inline empty-label preservation.
- Validation for that slice passed with
  `cargo fmt --all --check` and
  `cargo nextest run -p fret-ui-editor inline_surface_without_panel_or_empty_label_returns_the_field_root anchored_overlay_surface_without_panel_or_empty_label_returns_the_field_root inline_surface_with_empty_label_keeps_the_shell_visible --no-fail-fast`.
- The gallery internal preview cleanup kept the typed helper lane explicit while preserving the
  raw `Vec<AnyElement>` seams only where the scaffold still needs them. `overlay_status_text` and
  `overlay_scroll_row_text` now return typed helpers, `status_flags`/`portal_geometry` land those
  helpers explicitly at the vector boundary, and `tree_torture` regained its action-listener
  import after the test coverage was tightened.
- The inspector direct-entry surface remains the current hot page shell candidate: the
  direct-entry probe still routes through the static content stack in `apps/fret-ui-gallery/src/ui/content.rs`,
  while the latest evidence says the outer `ui-gallery-content-viewport` `Scroll` is still the
  dominant owner rather than the inspector `VirtualList`.
- Validation for this follow-up passed with
  `cargo fmt --all --check` and
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews --test inspector_perf_surface --test code_view_perf_surface --no-fail-fast`.
- A current-state perf refresh attempt for
  `ui-gallery-inspector-torture-scroll-direct-entry` reached the launch binary and then failed on a
  local compile mismatch inside `overlay/widgets.rs` before capture; that was fixed by collapsing
  the double-borrow map chain and aligning the gallery tests with the new typed-helper inventory.
- No fresh inspector bundle was captured from that retried run before the follow-up moved back to
  structural cleanup, so the existing evidence still stands: code-view steady direct-entry is
  light, combobox long-list is light, and the inspector page-shell / content viewport remains the
  next substantive hotspot to cut.
- The direct-entry probe then moved one layer deeper: row clicks now restore focus to the stable
  `ui-gallery-inspector-root` host instead of the recycled row node, and the fresh bundle
  `target/fret-diag/inspector-direct-entry-stable-root-focus-20260621/1782019574150/bundle.schema2.json`
  reports `subtree_no_focus_fallback=0`.
- Validation for that follow-up passed with
  `cargo nextest run -p fret-ui-gallery --no-fail-fast inspector_scroll_direct_entry_perf_script_starts_on_target_page_without_nav_search inspector_scroll_perf_script_keeps_nav_transition_setup gallery_inspector_torture_uses_fixed_row_text_roles gallery_inspector_torture_stamps_row_root_semantics_and_action_state gallery_inspector_torture_keeps_selected_row_model_on_paint_invalidation gallery_inspector_torture_keeps_row_shell_shrunk gallery_inspector_torture_keeps_tight_virtual_list_overscan gallery_inspector_torture_wraps_the_retained_list_in_a_stable_root_semantics_host`
  and
  `cargo run -p fretboard-dev --release -- diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-torture-scroll-direct-entry.json --dir target/fret-diag/inspector-direct-entry-stable-root-focus-20260621 --repeat 3 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
  plus the `jq` count check at `0`.
- The editor-controls shell-shrink batch is consistent with the shell-shrink direction:
  `PropertyGrid`, `ColorEdit`, `DragValue`, `EnumSelect`, `TextAssistField`, and
  `editor_input_group_row` each now have targeted structural coverage.
- `MiniSearchBox` is already thin enough that further shell removal would likely belong in
  `editor_joined_input_frame`, not the control itself.
- `AssetRefField` still carries a meaningful multi-action shell because it composes value text,
  status badge, and optional action segments; it is a better candidate for a future bounded slice
  than for a forced one-line shrink.
- Validation for the batch passed with
  `cargo fmt --all --check` and
  `cargo nextest run -p fret-ui-editor property_grid color_edit drag_value enum_select text_assist_field input_group --no-fail-fast`.
- `color_edit::popup::options::color_picker_options` now returns the single visible option directly
  even when a popup-level `test_id` is present, so the popup no longer keeps an extra vertical
  shell for the one-option case.
- `test_id` is preserved as a layout-transparent semantic anchor on the returned option, so
  diagnostics and UI automation can still locate the node without paying for a wrapper.
- Regression coverage now locks both the plain direct-return path and the `test_id`-decorated
  direct-return path.
- Validation for this slice passed with `cargo fmt --all --check` and
  `cargo nextest run -p fret-ui-editor color_edit::popup::options --no-fail-fast`.

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

### D10. Split code-view transition and steady mount probes explicitly

The existing `ui-gallery-code-view-torture-mount.json` script drives nav search, nav result
visibility, and page click before capture. That is a valid gallery transition surface, but it is not
the same thing as a steady code-view direct-entry mount.

The accepted follow-up keeps two distinct probe contracts:

- `ui-gallery-code-view-torture-mount.json`: transition surface that includes nav/search/page-switch
  work.
- `ui-gallery-code-view-torture-mount-direct-entry.json`: steady mount surface that starts on
  `code_view_torture`, waits for the page/code-view anchors plus font-stack stability, then resets
  diagnostics before capture.

Do not justify another code-view-local implementation slice from the old transition probe alone.
Use the direct-entry probe for steady mount evidence, and treat the old script as gallery-transition
evidence until it is renamed more explicitly.

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

- 2026-06-20 focused long-list launched correctness refresh:
  `target/debug/fretboard-dev diag run tools/diag-scripts/ui-gallery/perf/ui-gallery-combobox-long-list-focused-filter-select-steady.json --dir target/fret-diag/combobox-long-list-focused-minrun --timeout-ms 600000 --env FRET_DIAG_SEMANTICS=1 --launch -- cargo run -p fret-ui-gallery`
  passed. The evidence bundle
  `target/fret-diag/combobox-long-list-focused-minrun/1781896246714-ui-gallery-combobox-long-list-focused-filter-select-steady/bundle.schema2.json`
  now resolves the expected focused anchors and trigger lane (`docsec-long-list-content`,
  `ui-gallery-combobox-long-list-trigger`, `ui-gallery-combobox-long-list-query`,
  `ui-gallery-combobox-long-list-selected`).
- 2026-06-20 focused long-list current-state perf refresh:
  `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-combobox-long-list-focused-filter-select-steady.json --dir target/fret-diag/combobox-long-list-focused-perf-current --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-ui-gallery`
  passed with worst frame `total=244us`, `layout=14us`, `prepaint=130us`, `paint=100us`; evidence
  bundle `target/fret-diag/combobox-long-list-focused-perf-current/1781896419197/bundle.schema2.json`.
  This focused probe is therefore valid again, but on the current macOS surface it is no longer a
  dominant heavy-component hotspot.
- 2026-06-20 current-state heavy-surface rerank:
  - `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-torture-scroll.json --dir target/fret-diag/inspector-torture-scroll-current --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
    passed with worst frame `total=3396us`, `layout=3082us`, `solve=1241us`; evidence bundle
    `target/fret-diag/inspector-torture-scroll-current/1781897141976/bundle.schema2.json`.
  - `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount.json --dir target/fret-diag/code-view-torture-mount-current-macos --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
    passed with worst frame `total=3736us`, `layout=3456us`, `solve=946us`; evidence bundle
    `target/fret-diag/code-view-torture-mount-current-macos/1781897154059/bundle.schema2.json`.
  - `target/debug/fretboard-dev diag perf tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-click-stress.json --dir target/fret-diag/cookbook-imui-editor-controls-click-stress-current --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-cookbook --features cookbook-imui,cookbook-diag --example imui_editor_controls_basics`
    passed with worst frame `total=1348us`, `layout=1180us`, `solve=513us`; evidence bundle
    `target/fret-diag/cookbook-imui-editor-controls-click-stress-current/1781897209391/bundle.schema2.json`.
  Current ranking on this machine/runtime shape is therefore `code-view > inspector >
  editor-controls`, with `combobox long-list` much lighter still.
- 2026-06-20 code-view current-state node attribution:
  `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount.json --dir target/fret-diag/code-view-torture-mount-node-profile-current --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_LAYOUT_NODE_PROFILE=1 --env FRET_LAYOUT_NODE_PROFILE_TOP=20 --env FRET_LAYOUT_NODE_PROFILE_MIN_US=200 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
  passed with worst frame `total=3746us`, `layout=3420us`, `solve=954us`; evidence bundle
  `target/fret-diag/code-view-torture-mount-node-profile-current/1781897332013/bundle.json`.
  The hot frames are dominated by the outer page scroll root `ui-gallery-content-viewport`
  (roughly `2.2ms-2.6ms` self time after navigation settles), not by the inner
  `ui-gallery-code-view-root` `VirtualList`.
- 2026-06-20 code-view content-scroll bisect:
  `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount.json --dir target/fret-diag/code-view-torture-mount-bisect-disable-content-scroll --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_UI_GALLERY_BISECT=128 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
  passed with worst frame `total=3227us`, `layout=3027us`, `solve=683us`; evidence bundle
  `target/fret-diag/code-view-torture-mount-bisect-disable-content-scroll/1781897433562/bundle.json`.
  This supports the current hypothesis that the next slice belongs on the outer
  `ui-gallery-content-viewport` contract rather than inside code-block-local text preparation.
- 2026-06-20 rejected code-view page-local static shell experiment:
  a temporary `content.rs` change routed `PAGE_CODE_VIEW_TORTURE` through the existing static
  content-shell branch to mimic `BISECT_DISABLE_CONTENT_SCROLL` without widening runtime behavior.
  The implementation was reverted after evidence review.
  Validation/evidence:
  - `cargo fmt --all`
  - `cargo check -p fret-ui-gallery --features gallery-dev`
  - `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount.json --dir target/fret-diag/code-view-torture-mount-page-static-shell --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
    produced `total=3904us`, `layout=3571us`, `solve=1001us`; evidence bundle
    `target/fret-diag/code-view-torture-mount-page-static-shell/1781898593104/bundle.json`
  - repeat-3 rerun at
    `target/fret-diag/code-view-torture-mount-page-static-shell-rerun/1781898666393/bundle.schema2.json`
    reported `p50=3763us`, `p95=max=3807us`, still worse than the current-state baseline
    (`3736us`) and far from the full bisect result (`3227us`)
  - node profile at
    `target/fret-diag/code-view-torture-mount-page-static-shell-node-profile/1781898744109/bundle.json`
    still attributed the hot frames to `ui-gallery-content-viewport`, meaning the measured path was
    not actually exercising the intended page-local branch
  - control rerun with `FRET_UI_GALLERY_START_PAGE=code_view_torture` at
    `target/fret-diag/code-view-torture-mount-page-static-shell-start-page/1781898825559/bundle.schema2.json`
    dropped to `p50=532us`, `p95=max=543us`
  Interpretation: the page-local static shell itself is not obviously bad, but the current mount
  probe is still dominated by nav/search/page-switch transition work. The next slice should narrow
  the probe surface before landing another code change on the outer content-shell contract.
- 2026-06-20 direct-entry code-view mount clarification:
  - added `tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount-direct-entry.json`
    plus `apps/fret-ui-gallery/tests/code_view_perf_surface.rs` so the steady mount probe is gated
    separately from the old transition probe
  - refreshed steady mount perf:
    `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount-direct-entry.json --dir target/fret-diag/code-view-torture-mount-direct-entry-refresh --repeat 3 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
    reported `p50.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=306/11/0/189/106/0/0`,
    `p95=313/12/0/197/118/0/0`, and `max=313/12/0/197/118/0/0`; evidence bundle
    `target/fret-diag/code-view-torture-mount-direct-entry-refresh/1781899846277/bundle.json`
  - refreshed direct-entry node profile:
    `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount-direct-entry.json --dir target/fret-diag/code-view-torture-mount-direct-entry-node-profile-refresh --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_LAYOUT_NODE_PROFILE=1 --env FRET_LAYOUT_NODE_PROFILE_TOP=20 --env FRET_LAYOUT_NODE_PROFILE_MIN_US=100 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
    reported `top.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=310/11/0/197/102/0/0`;
    evidence bundle
    `target/fret-diag/code-view-torture-mount-direct-entry-node-profile-refresh/1781899876398/bundle.schema2.json`
  - interpretation:
    - direct-entry steady mount is not the current heavy hotspot
    - `layout` is now about `11-12us`, `solve=0`, and the measured worst frame stays near
      `310us`
    - the old `ui-gallery-code-view-torture-mount.json` result should therefore be read as a
      nav/search/page-switch transition surface, not as proof that code-view steady mount still
      needs a local runtime/component optimization
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

## 2026-06-20 Inspector Direct-Entry Static Content-Stack Note

- The inspector direct-entry probe now routes through the static content stack in
  `apps/fret-ui-gallery/src/ui/content.rs`, while the `code_view_torture` path keeps its own
  scroll shell and fixed preview height.
- This keeps the inspector page contract narrow without turning the change into a framework-level
  scroll rewrite.
- Validation for the slice passed:
  - `cargo fmt --all --check`
  - `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews --no-fail-fast`
  - `cargo nextest run -p fret-ui-gallery --test inspector_perf_surface --no-fail-fast`
  - `cargo nextest run -p fret-ui-gallery --test code_view_perf_surface --no-fail-fast`
- Perf evidence:
  - direct-entry worst frame after the change:
    `target/fret-diag/inspector-direct-entry-after-static-content-stack/1781910619755/bundle.schema2.json`
    with `top_total_time_us=2895`, `layout_time_us=2323`, and
    `layout.root_phases.roots(total/apply)=1473/1473`
  - node-profile rerun:
    `target/fret-diag/inspector-direct-entry-after-static-content-stack-node-profile/1781911121082/bundle.schema2.json`
    with worst frame `total=3052us`
- Node attribution says the remaining heavy node is still the outer `ui-gallery-content-viewport`
  `Scroll`, while the inspector `ui-gallery-inspector-root` `VirtualList` sits around
  `self_us=627-1064` and `total_us=770-1350` on the later hot frames.
- Interpretation: this slice is worth keeping, but it does not eliminate the outer content viewport
  as the current hotspot. The next inspector cut should likely target the page shell/content
  viewport contract again, not the inner row shape.

## 2026-06-20 Inspector Direct-Entry Nav Scroll Intrinsic-Mode Note

- The current direct-entry inspector evidence now points at the fixed-width sidebar
  `ui-gallery-nav-scroll` as the visible hot node rather than the inspector row tree.
- `apps/fret-ui-gallery/src/ui/nav.rs` pins that sidebar scroll viewport to
  `ScrollIntrinsicMeasureMode::Viewport` and `viewport_probe_unbounded(false)`, matching the
  already-bounded content viewport pattern instead of recursively measuring the full nav list
  during intrinsic sizing.
- Validation for the slice passed:
  - `cargo fmt --all --check`
  - `cargo nextest run -p fret-ui-gallery gallery_sidebar_nav_scroll_is_explicit_flex_fill_slot --no-fail-fast`
- Perf rerun:
  - `cargo run -p fretboard-dev --release -- diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-torture-scroll-direct-entry.json --dir target/fret-diag/inspector-direct-entry-nav-scroll-rerun --repeat 3 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
  - bundle: `target/fret-diag/inspector-direct-entry-nav-scroll-rerun/1781938135186/bundle.schema2.json`
- Result: the rerun stayed in the same multi-ms band rather than producing a breakout win
  (`p50 total/layout/solve/prepaint/paint = 3278/2539/1115/209/429`, `p95 = 3284/2640/1200/220/525`).
- Interpretation: the sidebar scroll owner is now narrower and easier to reason about, but the
  direct-entry inspector surface still has another hotspot in the outer shell/root-apply path.
  The next cut should stay evidence-led and avoid widening back into row-local repair unless a new
  bundle points there.

## 2026-06-20 Inspector Direct-Entry A/B Note

- Re-ran the same direct-entry inspector script with `FRET_UI_GALLERY_VIEW_CACHE=1` and
  `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1` to test whether the cached shell was the missing lever.
- Result: the cache-enabled run did not improve the surface. It landed at
  `p50 total/layout/solve/prepaint/paint = 3431/2810/985/83/535` and
  `p95 = 3740/3122/1075/84/540`, which is worse than the current no-cache direct-entry band.
- Re-ran the same script with `FRET_UI_GALLERY_INSPECTOR_KEEP_ALIVE=0` to test whether the
  retained row pool was the real owner.
- Result: the keep-alive-off run stayed in the same multi-ms range
  (`p50 total/layout/solve/prepaint/paint = 3170/2517/1160/215/427`,
  `p95 = 3379/2704/1190/248/453`).
- `diag stats` on the keep-alive-off bundle still shows a heavy layout root-apply surface
  (`roots(total/apply)=1593/1593` at p95/max) and a stable `paint.widget` tail; the remaining
  owner is still the outer inspector page shell rather than a clear win from shell caching or
  keep-alive tuning.
- Decision: stop spending the next slice on cache toggles or row-retention knobs. The next
  evidence-led cut should go back to `apps/fret-ui-gallery/src/ui/content.rs` and the inspector
  direct-entry page shell contract, then only split a narrower retained VirtualList follow-on if a
  fresh bundle moves the owner there.

## 2026-06-20 Inspector Direct-Entry Overscan-8 Note

- Tightened the retained inspector list window from overscan `12` to `8` in
  `apps/fret-ui-gallery/src/ui/previews/gallery/torture/inspector_torture.rs`.
- Refreshed direct-entry perf:
  `target/fret-diag/inspector-direct-entry-overscan-8/1781940494893/bundle.json`
  reported `p50 total/layout/solve/prepaint/paint = 2161/1538/849/199/328` and
  `p95 = 2564/1899/993/229/419`.
- Refreshed node-profile perf:
  `target/fret-diag/inspector-direct-entry-overscan-8-node-profile/1781941064314/bundle.schema2.json`
  still shows the outer `ui-gallery-content-viewport` `Scroll` as the dominant owner
  (`self_us=7449`, `total_us=11499` on the hot frame), while `ui-gallery-inspector-root`
  `VirtualList` sits lower (`self_us=840`, `total_us=1617`).
- Conclusion: overscan `8` is a real improvement, but it is not the final owner shift. The next
  slice should stay on `apps/fret-ui-gallery/src/ui/content.rs` and the inspector direct-entry
  shell contract rather than shrinking the retained list window further first.

## 2026-06-20 Inspector Direct-Entry Shell-Root-Prune Note

- I tested removing the extra `ui_gallery.content_root` key from the non-cache `content_view`
  branch in `apps/fret-ui-gallery/src/driver/shell.rs`, keeping only the selected-page content key
  as the page boundary.
- Validation passed:
  - `cargo fmt --all --check`
  - `cargo nextest run -p fret-ui-gallery --test inspector_perf_surface --no-fail-fast`
- Perf rerun:
  - `target/fret-diag/inspector-direct-entry-shell-root-prune/1781943312483/bundle.json`
  - `p50 total/layout/solve/prepaint/paint = 2102/1483/829/228/391`
  - `p95 = 2721/2024/942/302/398`
- Result: this was not a win. The direct-entry surface got worse versus the overscan-8 run, and
  the hot path still sat in the outer shell/root-apply region rather than moving cleanly onto the
  retained inspector list.
- Decision: keep this as a rejected shell-prune experiment. The next slice should come from a
  different evidence-backed boundary in `apps/fret-ui-gallery/src/ui/content.rs`, or from the
  retained inspector list only if a future bundle moves the owner there first.

## 2026-06-20 Inspector Direct-Entry Command Availability Follow-up

- I revisited the inspector direct-entry perf lane after the no-focus subtree fallback route split
  landed in `fret-ui`. The route-aware pruning stayed useful, but the broader no-focus subtree
  summary cache did not show a clear net win, so I removed that extra layer instead of keeping a
  speculative mechanism branch around.
- Validation for the command-availability slice passed with
  `cargo fmt --all --check` and
  `cargo nextest run -p fret-ui window_command_action_availability_snapshot --no-fail-fast`.
- Fresh direct-entry perf evidence for the inspector surface landed at
  `target/fret-diag/inspector-direct-entry-no-focus-subtree-cache-current/1781955248948/bundle.schema2.json`.
  The repeated run stayed in the same low-ms band, with worst frame
  `total/layout/solve/prepaint/paint = 2762/2151/911/220/391` and `window_runtime_snapshot.command_availability`
  still dominated by `edit.copy@subtree_no_focus_fallback` around `620us` on the hot frames.
- Interpretation: the remaining inspector cost is still mostly outer shell/root-apply work, not the
  no-focus subtree route itself. The command-availability route split is worth keeping, but the next
  meaningful perf cut should return to the inspector page shell / content viewport boundary rather
  than expanding the command-availability cache further.

## 2026-06-20 Gallery Header Semantics Shrink Note

- `ui-gallery-content-header` now attaches semantics directly to `header_content` instead of
  introducing a dedicated wrapper `Semantics` node.
- The header keeps the same layout and the same diagnostics anchor, but the page shell loses one
  real layout node in the header branch.
- Validation:
  - `cargo fmt --all --check`
  - `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_content_shell --no-fail-fast`
- I did not rerun the perf probe for this note. The current evidence still says the outer content
  viewport is the dominant owner, so this is a structural cleanup rather than a new perf
  conclusion.

## 2026-06-20 Inspector Direct-Entry Short Shell Note

- `preview_inspector_torture` now returns the retained inspector list directly, so the direct-entry
  `inspector_torture` page skips the generic preview card shell while still keeping the inspector
  diagnostics root.
- Regression coverage now locks both boundaries: the generic preview card shell stays absent on
  inspector direct-entry, and `ui-gallery-inspector-root` stays present.
- Validation passed:
  - `cargo fmt --all --check`
  - `cargo nextest run -p fret-ui-gallery --features gallery-dev inspector_torture_skips_preview_card_shell --no-fail-fast`
- Perf rerun:
  - `target/fret-diag/inspector-direct-entry-short-shell-v2/1781958586751/bundle.json`
  - `p50 total/layout/solve/prepaint/paint = 2361/1846/741/165/350`
  - `p95 = 2700/2119/864/220/361`
  - hot frame `total/layout/solve/prepaint/paint = 2446/1900/864/220/361`
- Node attribution still keeps the outer `ui-gallery-content-viewport` as the dominant owner, so
  this is a useful shell shrink but not yet an owner shift.
- I also tried removing the generic `ui-gallery-page-preview` semantics wrapper. That rerun
  regressed to `2700us`, so the change was reverted and the retained direct return is the only
  short-shell win.
