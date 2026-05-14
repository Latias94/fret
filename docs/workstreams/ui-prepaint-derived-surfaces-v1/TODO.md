# UI Prepaint Derived Surfaces v1 - TODO

Date: 2026-05-15
Status: Closed (M0/M1/M2/M3 complete; retained mechanisms and follow-ons recorded in closeout)

## M0 - Baseline And Contract Setup

- [x] Create the workstream doc set.
- [x] Record that this is a follow-on to the closed Frame Pipeline v2 lane, not a reopening.
- [x] Record the initial ADR decision: no new ADR until the lane changes a hard runtime or renderer
  contract.
- [x] Capture the first baseline bundle for `ui-gallery-virtual-list-torture-steady` on the current
  machine profile.
- [x] Run worst-bundle attribution for the baseline and record the dominant phase: layout,
  prepaint, paint, renderer payload, or dispatch.
- [x] Decide the first code slice from measured evidence rather than from assumed architecture
  cleanup.

Evidence:

- baseline/attribution bundle:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-virtual-list-attrib/1778755322233/bundle.schema2.json`
- dominant phase: layout (`12011us` of `14396us` total in the sampled bundle)

## M1 - Retained Virtual List Derived Surface

Current implementation note (2026-05-14): `VirtualListPrepaintWindowOutput` is now stored in
`ViewBoundaryState` and retained reconcile reads that boundary-owned output first. `VirtualListState`
remains as the declarative build/layout bridge while the lane proves whether more local state can
move to the boundary owner.

- [x] Audit current retained virtual-list derived state:
  - window range,
  - render window range,
  - retained reconcile metadata,
  - keep-alive membership,
  - prepaint virtual-list window records,
  - boundary diagnostics.
- [x] Identify any state that is still surface-local but should be boundary-owned.
- [x] Add or tighten a focused mechanism test before changing the runtime path.
- [x] Move one measured derived state path into the canonical boundary owner, or document why it
  must remain outside `ViewBoundaryState`.
- [x] Delete or narrow the replaced local path.
- [x] Prove the mechanism slice with focused `cargo nextest` tests.
- [x] Prove the mechanism slice compiles with `cargo check -p fret-ui --all-targets`.
- [x] Close the M1 performance/stats evidence with:
  - `diag stats` reuse/rejection checks,
  - perf gate against the selected baseline or an explicitly attribution-only macOS note.

M1 ownership audit:

- Moved to boundary owner:
  `VirtualListPrepaintWindowOutput` now carries the prepaint-derived visible/window range plus the
  policy/input key fields required to reject stale output. Retained reconcile validates this output
  against current props, viewport, offset, and visible range before using it.
- Narrowed old path:
  retained reconcile now reads boundary-owned output first, and only falls back to
  `VirtualListState.window_range` / `render_window_range` when no valid boundary output exists.
  The focused test intentionally poisons `VirtualListState` with a valid but overbroad stale window
  and proves retained reconcile still consumes the precise boundary-owned prepaint window.
- Explicitly retained:
  `VirtualListState` remains element-local because it bridges declarative render, layout metrics,
  scroll offset/viewport bookkeeping, measured/known row metrics, key cache, and layout scratch.
  `RetainedVirtualListKeepAliveState` remains in `WindowElementState` because it owns detached live
  `NodeId`s and LRU-like reuse order, not geometry-derived prepaint output.
  `UiDebugRetainedVirtualListReconcile` remains diagnostics-only.

M1 focused evidence:

- `cargo nextest run -p fret-ui retained_virtual_list_host_updates_window_without_rerendering_view_cache_root --no-fail-fast`
  passed on 2026-05-15.
- `cargo nextest run -p fret-ui retained_virtual_list_keep_alive_reuses_detached_items_when_scrolling_back mechanism_harness_retained_virtual_list_reconcile_matches_oracles mechanism_harness_prepaint_virtual_list_window_update_matches_oracles --no-fail-fast`
  passed on 2026-05-15.
- `cargo check -p fret-ui --all-targets` passed on 2026-05-15.

M1 perf/stats closeout:

- macOS attribution-only perf run:
  `target/release/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-virtual-list-torture-steady.json --repeat 7 --warmup-frames 5 --reuse-launch --sort time --top 15 --json --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_DIAG_MAX_SNAPSHOTS=240 --launch -- target/release/fret-ui-gallery`
  passed on 2026-05-15.
- Worst bundle:
  `target/fret-diag/1778777905741/bundle.schema2.json`.
- `diag stats --sort cpu_cycles --top 30` on the worst bundle reported
  total/layout/prepaint/paint = `14833/11423/315/3095us`, p50/p95 total =
  `4374/5555us`, p50/p95 layout = `2887/4729us`, and p50/p95 prepaint =
  `62/139us`.
- Interpretation: the virtual-list proof surface is still layout-dominant; the boundary-owned
  prepaint output is the correct ownership migration, not a claim that prepaint was the dominant
  cost.

## M2 - Retained Data Table / View-Cache Torture

- [x] Run the first view-cache filter-shrink torture repro and record the current blocker.
- [x] Rerun the view-cache filter-shrink repro with a `gallery-dev` build and record that it passes.
- [x] Run the retained data-table suite as the next correctness repro.
- [x] Triage the retained data-table step-22 assertion:
  decide why the retained path shows `GlobalFilter: Process 123` but still reports a
  pre-filter virtual-list `items_len=50000` and no retained reconcile input-change record.
- [x] Attribute the worst data-table frame and compare it with the virtual-list M1 findings.
- [x] Confirm whether the remaining cost is boundary ownership, component policy churn, or renderer
  payload.
- [x] Move only mechanism-level reusable state into `crates/fret-ui`; keep table policy in
  `ecosystem/fret-ui-kit`.
- [x] Add or update a data-table diag/perf gate only after the surface is stable enough to avoid
  broad-suite noise.
- [x] Rerun the view-cache comparison suite after the retained-table fixes so M2 has both retained
  and non-retained proof-surface evidence from the same implementation state.

Current M2 evidence:

- historical first failing view-cache bundle:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-torture/1778759907796-script-step-0023-assert-failed/bundle.schema2.json`
- current passing view-cache bundle with `gallery-dev`:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-torture-current-dev/1778762426810-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- same-state passing view-cache suite after script stabilization:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-2026-05-15-after-scroll-wait/suite.summary.json`
- same-state view-cache filter-shrink bundle:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-view-cache-2026-05-15-after-scroll-wait/1778777531409-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`
- historical retained-suite failing bundle:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-current-dev-cargo-run/1778762568416-script-step-0022-assert-failed/bundle.schema2.json`
- retained-suite passing summary after the row-order, idempotent sync, reset-epoch, and sort-anchor
  split fixes:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-after-sort-anchor-split-suite/suite.summary.json`
- same-state retained-suite passing summary after script stabilization:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/suite.summary.json`
- current view-cache `diag stats`: total/layout/prepaint/paint =
  `185149/168294/722/16133us`; p50/p95 total = `14092/16867us`; p50/p95 layout =
  `12793/15541us`. The view-cache surface is still layout-dominant and cache roots are not reused
  in the last snapshots because of `layout_invalidated`.
- retained filter-shrink passing bundle:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-after-sort-anchor-split-suite/1778776884214-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
  with total/layout/prepaint/paint = `20432/16832/1125/2475us` and p50/p95 total =
  `180/14752us`.
- same-state retained filter-shrink bundle:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/1778777573643-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`
  with total/layout/prepaint/paint = `19351/15941/1027/2383us` and p50/p95 total =
  `163/13948us`.
- retained multi-sort passing bundle:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-after-sort-anchor-split-suite/1778776885328-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json`
  with total/layout/prepaint/paint = `40318/33192/1707/5419us` and p50/p95 total =
  `484/10481us`.
- same-state retained multi-sort bundle:
  `target/fret-diag/ui-prepaint-derived-surfaces-v1-data-table-retained-2026-05-15-after-scroll-wait/1778777574739-ui-gallery-data-table-retained-multi-sort-shift-click/bundle.schema2.json`
  with total/layout/prepaint/paint = `39836/32667/1862/5307us` and p50/p95 total =
  `523/10077us`.

M2 root-cause notes:

- The retained table row-order path used constrained sizing columns as the filter/sort source, which
  dropped filter and sort closures. It now indexes the original column definitions for layout-time
  row-order work.
- Toolbar and recipe sync paths could perform non-idempotent model updates, which made retained
  data-table scripts observe avoidable revision churn. These sync paths now update only on real
  changes.
- The reset harness needed a keyed `reset_epoch` to clear toolbar-local retained state between
  suite scripts.
- Header debug IDs overlapped the column-actions accessory trigger. The retained header now splits
  the sort pressable and accessory menu into sibling nodes so scripted clicks on the header sort
  anchor do not open the column-actions menu.

M2 interpretation:

- The retained table surface is now correctness-green enough to serve as a proof surface.
- The measured retained-table cost remains layout-dominant on the interaction frames. Some snapshots
  also show high `command_availability` evaluation time; that is worth a follow-on if it persists,
  but it is not a blocker for this boundary-derived-state lane.
- The M2 code changes stayed in the policy/component layers except for the existing M1 mechanism
  path in `crates/fret-ui`.

## M3 - Cross-Surface Contract Cleanup

- [x] Remove obsolete feature flags, env knobs, or diagnostics branches introduced only to compare
  old and new paths.
- [x] Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` if any hard contract changed.
- [x] Record retained mechanisms that remain after M1/M2 and why they are not migration leftovers.
- [x] Add closeout audit when both proof surfaces pass and old paths are deleted or explicitly
  retained.

M3 closeout notes:

- No obsolete comparison env knob or feature flag was introduced by this lane. The existing gallery
  env knobs remain proof-surface selectors.
- No hard runtime, renderer, public boundary API, retained cache ownership, or external diagnostics
  schema changed, so no ADR/alignment update is required.
- Retained mechanisms and follow-ons are recorded in
  `docs/workstreams/ui-prepaint-derived-surfaces-v1/CLOSEOUT_AUDIT_2026-05-15.md`.

## Guardrails

- [x] Do not start renderer `Scene` per-boundary recording in this lane.
- [x] Do not widen `crates/fret-ui` into component policy.
- [x] Do not loosen perf thresholds to make a refactor pass; re-seed only with an explicit policy
  reason and before/after evidence.
- [x] Do not reopen `ui-frame-pipeline-v2-fearless-refactor-v1`; link to it as historical evidence.
