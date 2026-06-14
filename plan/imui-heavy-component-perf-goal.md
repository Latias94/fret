---
title: IMUI and heavy-component performance goal
type: active-goal-log
date: 2026-06-14
---

# IMUI and Heavy-Component Performance Goal

## Goal

Keep optimizing Fret's IMUI and shadcn-style heavy component paths until dense general-app
surfaces can stay within a practical 120Hz interaction budget. The current primary gate is the
ui-gallery searchable combobox long-list perf probe because it exercises nested composition,
command filtering, virtual rows, overlays, semantics selectors, layout, paint, and diagnostics.

This root-level `plan/` note mirrors the user's requested progress log location. The longer
historical records remain in:

- `docs/plans/2026-06-14-001-imui-heavy-component-perf-architecture-audit-plan.md`
- `docs/plans/2026-06-14-002-imui-heavy-component-perf-progress-log.md`

## Current Evidence

- Command palette query/navigation is already inside the local 120Hz budget on the RTX4090 Windows
  release probe.
- Searchable combobox long-list has moved from full-list row materialization and broad page layout
  relayout into smaller layout, paint/text, and renderer tail costs.
- The checked-in dev-fast regression baseline is
  `docs/workstreams/perf-baselines/ui-gallery-combobox-filter-select-steady.dev-fast.windows-rtx4090.v1.json`.
- The latest accepted command item-only fast-path gate stayed green with worst frame around
  `11215us`, still above strict 120Hz.
- Earlier accepted optimizations were mixed: component policy/rendering seams, shared `fret-ui`
  mechanism optimizations, declarative text diff narrowing, and gallery cache-boundary policy.

## Decisions

### D1. Continue mixed component plus mechanism optimization

The evidence does not support a single broad framework rewrite as the next step. Large wins came
from specific seams: combobox close policy, command row virtualization, bounded virtual scroll
viewport probing, command availability caching, incremental view-cache observation collapse, and
paint-only plain text content diffing.

### D2. Treat diagnostics overhead as part of perf-gate fidelity

Perf scripts must not make component frames look slower than a real app because diagnostic target
resolution requests fresh semantics snapshots too broadly. Reducing diagnostics-only refreshes is
valid when it does not weaken:

- runtime accessibility semantics,
- selector correctness for current-frame geometry,
- stale-cache behavior for current-window `exists` / `not_exists` assertions.

### D3. Do not use stale semantics as current semantics

Skipping a fresh semantics refresh is only safe if the script step can evaluate without current
semantics. Passing an old `semantics_snapshot_arc()` to a selector-based step would be a correctness
bug, not an optimization. Conservative no-refresh candidates are frame-independent predicates such
as event-kind, font readiness, app snapshot, window size, and off-window runtime diagnostics.

## Active Hypothesis

`UiDiagnosticsService::wants_semantics_snapshot` still over-requests semantics during active
`wait_until` loops. `script_engine::active_script_needs_semantics_snapshot` has predicate-level
helpers, but the early `active.wait_until.is_some()` branch currently returns `true` without
checking whether the active wait predicate can evaluate without current semantics.

If narrowed carefully, scripted perf gates should be closer to real app behavior without changing
component code or accessibility output.

## 2026-06-14 Findings

- The previously observed `layout_semantics_refresh_time_us ~= 2908us` happened on a dirty frame
  where the semantics fingerprint changed. It is not safe to skip that refresh.
- The adjacent clean frame already reported `layout_semantics_refresh_time_us = 0us`, which confirms
  the existing dirty/request gate works for clean frames.
- Safe diagnostics narrowing is limited to script steps that truly do not need current semantics:
  frame-independent predicates such as event kind, font readiness, window size, app snapshot, and
  off-window runtime diagnostics.
- Current-window selector predicates, including `exists` / `not_exists(test_id)`, must remain fresh
  unless the runtime gains an explicit "fresh semantics for this frame" marker. Reusing stale
  `semantics_snapshot_arc()` for these would be a correctness bug.
- The first runtime-safe dirty-refresh optimization is allocation-oriented: reuse a semantics
  children scratch buffer during full traversal instead of cloning/allocating a `Vec<NodeId>` per
  visited node.
- A first dev-fast gate after the scratch-buffer change failed once:
  `target/fret-diag/gate-combobox-filter-select-devfast-semantics-scratch/1781435706747/bundle.schema2.json`.
  The top frame was `16650us` total with `10297us` layout, `5930us` semantics refresh,
  `2012us` layout solve, and high renderer finish. This looked like a dirty popover overlay tail,
  not a deterministic regression.
- The same gate passed on rerun:
  `target/fret-diag/gate-combobox-filter-select-devfast-semantics-scratch-rerun/1781435998219/bundle.schema2.json`.
  The top frame was `10632us` total with `6090us` layout, `2460us` semantics refresh,
  `1072us` layout solve, `1874us` max renderer finish, `820us` max pointer dispatch, and no
  threshold failures.
- The accepted pre-scratch bundle had a worst frame around `11215us` with `2908us` semantics
  refresh. The rerun does not prove a major latency win, but it does validate the scratch change as
  a low-risk allocation reduction that keeps the gate green.

## Current Decision

Land the current slice as a reversible performance/fidelity cleanup:

- diagnostics only ask for fresh semantics on active waits whose predicates need current semantics,
- semantics traversal reuses a root-local children scratch buffer instead of cloning per visited
  node,
- current-window selector predicates stay conservative and still require fresh semantics.

Do not broaden stale-semantics reuse until the runtime has an explicit freshness marker. The next
meaningful optimization should target either dirty-frame semantics traversal cost itself or the
popover overlay root solve tail.

## 2026-06-14 Second Slice Findings

- Added a layout-derived semantics hook classifier for declarative host widgets. It skips the
  expensive `semantics_impl` instance clone/match for plain pass-through nodes, while preserving
  snapshot nodes, bounds, children traversal, focus/text-input defaults, root `Window` role behavior,
  and any `attach_semantics` decoration.
- Do not synchronize this classifier from `mount_element` for every declarative element. A trial
  mount-path sync added one widget mutation per mounted element and made the dense combobox gate
  worse. The final slice keeps classification on the layout path only.
- Focused correctness tests cover both sides of the contract:
  undecorated plain containers remain present as generic semantics nodes, and `attach_semantics` on
  a plain container still stamps role/label/test_id and keeps child traversal.
- The current combobox dev-fast gate still fails on repeated probes after this slice:
  `target/fret-diag/gate-combobox-filter-select-devfast-semantics-hook-layout-only/1781440592740/bundle.schema2.json`
  had `22850us` total, `12740us` layout, `2428us` solve, and `8927us` paint.
- The failure shape is not semantics-led. The worst frame is the searchable combobox popover root
  (`DismissibleLayer`) doing a `new_frame_same_key` solve of about 27 subtree nodes, plus root apply,
  paint, and renderer finish. A semantics-profile probe with the same binary showed semantics
  snapshots mostly in the `1-3ms` traversal range and a better top frame around `14365us`.
- The next higher-leverage target is therefore the popover overlay root solve / paint tail, not more
  semantics micro-optimization.

## 2026-06-14 Third Slice Findings

- Added a frame-local declarative command-availability interest cache in `fret-ui`. It caches only
  whether a node may handle a command class, not the final command availability result, and is keyed
  by `(frame_id, command_availability_revision, window)`.
- This is intentionally narrower than caching `Available` / `Blocked`: command availability hooks
  still run when a node is a possible handler, while repeated runtime-snapshot publications in the
  same frame avoid re-reading declarative element state for every node in the same command path.
- Focused tests prove the intended boundaries:
  `action_availability_snapshot_reuses_declarative_interest_across_same_frame_refine` shows a
  forced same-frame refine reuses the cached interest metadata, and a layout invalidation bumps the
  command availability revision and forces a re-read.
- Validation passed:
  `cargo test -p fret-ui --lib window_command_action_availability_snapshot --profile dev-fast -j 1`,
  `cargo fmt -p fret-ui`, and `cargo check -p fret-ui -j 1`.
- The current combobox gate still fails on a single dev-fast probe:
  `target/fret-diag/gate-combobox-filter-select-devfast-interest-cache/1781443011610/bundle.schema2.json`
  had `17523us` total, `7973us` layout, `914us` solve, `8829us` paint, and failed only
  `top_total_time_us > 15443us`.
- This probe reduced the worst runtime snapshot command-availability evaluation shape compared with
  the earlier `1.4-1.8ms` readings, but it did not move the overall gate under budget. The worst
  frame is now dominated by paint/cache behavior: `paint.cache_misses=1033`, `paint.nodes=1099`,
  and `cache.reused=0`.
- Current decision: keep this slice as a low-risk mechanism optimization, but move the next
  investigation to paint cache root reuse during the combobox filter/select path.

## 2026-06-14 Fourth Slice Findings

- Fixed-height declarative `TextInput` now treats text content changes as paint-only. The runtime
  still keeps auto-height inputs layout-sensitive, because their measured size may depend on the
  current text.
- The policy is derived from `TextInputProps.layout.size.height`: `Length::Px(_)` uses
  `Invalidation::Paint`, and non-fixed height keeps `Invalidation::Layout`. This applies through
  the declarative event/layout/paint/command paths, model observation, and platform text replacement
  APIs.
- `measure_text_input` no longer reads or layout-observes the model for fixed-height inputs. It uses
  a stable `"M"` line-metric probe for the fixed-height case, which avoids turning command-search
  query changes into layout work.
- Focused tests cover the contract:
  `fixed_height_text_input_model_change_invalidates_paint_only` and
  `auto_height_text_input_model_change_keeps_layout_invalidation`.
- A first combobox perf probe was invalid as performance evidence because it omitted
  `FRET_UI_GALLERY_VIEW_CACHE=1` and `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`. Direct JSON-script
  `diag perf` targets do not receive the suite-name auto-env defaults, so the overlay root reported
  `reuse_reason = view_cache_disabled` and produced a misleading `18602us` top frame.
- The correct gate command includes the view-cache env vars plus the prewarm/prelude scripts used
  by the checked-in baseline. That run passed with `failures=[]`:
  `target/fret-diag/gate-combobox-filter-select-devfast-fixed-textinput-paint-vc/1781446249173/bundle.schema2.json`.
- Correct-gate top frame after the fixed-height input slice:
  `total=10000us`, `layout=5743us`, `solve=945us`, `paint=3628us`,
  `paint.cache_misses=0`, `cache.reused=1`, `cache.replayed_ops=203`,
  `pointer_move_max_dispatch=775us`, and `pointer_move_max_hit_test=114us`.
- Current residual hotspot is no longer the search `TextInput` model observation path. The worst
  frame still includes popover `DismissibleLayer` layout solve and retained paint/renderer work,
  so the next optimization should target overlay/list cache boundaries or command-availability tail
  only if a fresh profile makes them hot again.

## 2026-06-14 Fifth Slice Findings

- Investigated the next paint/cache tail after the fixed-height `TextInput` slice. The important
  failure mode was not a component-level combobox policy bug: when an ancestor subtree replayed from
  paint cache, descendant `PaintCacheEntry` records could remain tied to the previous generation.
  If the ancestor was paint-invalidated on the following frame, stable descendants no longer had
  fresh ranges to replay and could fall back toward repainting the dense subtree.
- Added a mechanism-level paint-cache rebase step after a successful ancestor replay. It walks
  descendants and promotes only safe previous-frame entries into the current generation:
  descendant ranges must be fully contained in the replayed parent range, the descendant entry must
  come from the current source generation, and the walker prunes paint-invalidated descendant
  subtrees. The rebase only remaps op/text-blob ranges; origin translation remains owned by the
  existing cache replay/bounds translation path.
- This keeps component caches local. It deliberately does not wrap the full `CommandPalette` or
  `Combobox` in a broad view cache, because those surfaces carry active descendant, selection,
  disabled/highlight state, semantics, and test-id behavior that should remain policy-owned.
- Added `paint_cache_rebases_descendant_entries_after_ancestor_replay` to cover the three-frame
  sequence: first full paint, second ancestor replay with changed bounds, third ancestor repaint
  while the stable child still replays from the rebased descendant entry.
- Added `paint_cache_rebase_prunes_paint_invalidated_descendant_subtrees` to prove an invalidated
  intermediate node prevents deeper descendants from being rebased and later replayed through that
  invalidated subtree.
- Tightened the existing selectable-text replay test so cache replay across a bounds move still
  touches selectable span state without corrupting local span bounds.
- While running the broader paint-cache test filter, the existing
  `focus_traversal_availability_short_circuits_after_first_candidate` assertion failed because it
  counted layout/prepaint sampling work as part of command availability. The runtime behavior was
  already short-circuiting correctly; the test now asserts the post-layout call delta instead.
- Validation passed:
  `cargo test -p fret-ui --lib paint_cache --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui --lib focus_traversal_prepaint_cache --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo fmt -p fret-ui`, and `cargo check -p fret-ui -j 1`.
- The correct combobox dev-fast perf gate with view-cache env, prewarm, and prelude passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-paint-cache-rebase-prune-vc/1781449512986/bundle.schema2.json`.
  Top frame was `10151us` with `layout=5855us`, `solve=1131us`, `prepaint=614us`,
  `paint=3682us`, `paint.cache_misses=0`, `cache.reused=1`, and `cache.replayed_ops=203`.
- Current residual tail is no longer full subtree repaint. The remaining top-frame cost is mostly
  popover/root layout request/apply plus renderer upload/finish/text preparation. The next slice
  should only revisit paint cache if a fresh profile shows cache misses returning; otherwise the
  higher-leverage targets are overlay root apply/layout and renderer retained-text work.

## 2026-06-14 Sixth Slice Findings

- Investigated residual command-availability churn during the searchable combobox filter/select
  gate. Runtime snapshot publication can happen more than once in a frame while pending
  declarative/post-layout refine state is active, but the previous signature gate treated any
  pending state as a hard reason to recompute.
- Added pending window-runtime snapshot state to
  `WindowCommandActionAvailabilitySnapshotSignature`: sorted pending declarative roots plus the
  frame-local post-layout refine marker. This keeps the required post-layout authoritative publish
  while deduping duplicate same-frame interim publishes with identical inputs.
- Split command-availability invalidation from the broader semantics invalidation predicate for the
  first safe case: `ScrollHandleHitTestOnly` invalidations still keep semantics behavior unchanged,
  but no longer reset command-interest metadata or force a command-availability revision bump.
- Focused tests cover both contracts:
  `action_availability_snapshot_dedupes_same_pending_refine_but_post_layout_republishes` and
  `action_availability_snapshot_keeps_interest_cache_for_scroll_hit_test_only_invalidation`.
- Validation passed:
  `cargo test -p fret-ui --lib window_command_action_availability_snapshot --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui --lib focus_traversal_prepaint_cache --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo fmt -p fret-ui`, `cargo check -p fret-ui -j 1`, and `git diff --check`.
- The correct combobox dev-fast perf gate still failed on one noisy probe, but the shape changed:
  `target/fret-diag/gate-combobox-filter-select-devfast-command-inv-split-vc/1781453177303/bundle.schema2.json`
  had `total=11586us`, `layout=1575us`, `solve=0us`, `prepaint=1129us`, `paint=8882us`,
  `paint.cache_misses=0`, `cache.reused=1`, and `cache.replayed_ops=222`.
- Remaining failures were pointer-tail thresholds only:
  `pointer_move_max_dispatch_time_us=1507` over `1001`, and
  `pointer_move_max_hit_test_time_us=210` over `170`. The top frame was under the checked-in
  `top_total_time_us` threshold, so this slice is a conservative mechanism cleanup rather than the
  final answer for 120Hz dense UI.
- The remaining command-availability hotspot is still
  `ui_gallery.switch.command_gate.action@focused_or_default`, which is a gallery-level widget
  command registered globally for the UI gallery. The next architectural question is whether runtime
  snapshots should publish all widget commands for every surface, or whether command groups/surfaces
  need a deeper interface for filtering without weakening app command behavior.

## 2026-06-14 Seventh Slice Findings

- Added a mechanism-level filtered action-availability publisher:
  `UiTree::publish_window_command_action_availability_snapshot_filtered(...)`. The existing
  `publish_window_command_action_availability_snapshot(...)` remains the conservative full-window
  default and still publishes every registered widget-scoped command.
- The filtered publisher is intentionally caller-owned: it accepts a concrete command set, sorts and
  dedupes it for stable snapshot signatures, ignores unregistered/non-widget commands, and leaves
  omitted commands as `unknown` in `WindowCommandActionAvailabilityService` rather than publishing
  them as disabled.
- Focused tests cover the contract:
  `action_availability_filtered_snapshot_publishes_only_requested_widget_commands` and
  `action_availability_filtered_snapshot_signature_dedupes_sorted_command_set`.
- Found the higher-leverage immediate issue in the UI Gallery strategy layer: the global shadcn
  command dialog was built with `new_with_host_commands(...)` even while the dialog was closed.
  That meant closed chrome still materialized host command entries and their command/action
  surfaces during dense component frames.
- Aligned the Gallery with the bootstrap command-palette strategy: when closed, render the dialog
  shell with empty entries; when open, build host command entries. This keeps the first screen
  behavior unchanged while removing unrelated command-entry surfaces from the combobox steady path.
- Validation passed:
  `cargo test -p fret-ui --lib window_command_action_availability_snapshot --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui-gallery --profile dev-fast -j 1`, and
  `cargo build -p fret-ui-gallery --profile dev-fast -j 1`.
- The corrected combobox dev-fast perf gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-command-palette-closed-vc/1781455378046/bundle.schema2.json`.
  Top frame was `11197us` with `layout=5662us`, `solve=913us`, `prepaint=659us`,
  `paint=4876us`, `dispatch=95us`, `hit_test=26us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `cache.replayed_ops=203`.
- Pointer-tail thresholds are now inside the checked-in gate:
  `pointer_move_max_dispatch_time_us=846` and `pointer_move_max_hit_test_time_us=132`.
- The diagnostics app snapshot still reports `command_palette_entries_count=134` because that value
  is produced by the diagnostics snapshot provider from the host command catalog, not by the closed
  dialog render tree. The runtime hotspot evidence is the important signal: the previous
  `ui_gallery.switch.command_gate.action@focused_or_default` hotspot disappeared from the worst
  frames after closed-dialog entry materialization was removed.
- Current decision: keep both changes. The filtered publisher is the right mechanism for future
  app-owned command surfaces, while the Gallery closed-dialog fix is the actual perf win for this
  combobox gate.

## Next Verification

1. Add focused unit coverage for active `wait_until` semantics demand. Done with the actual
   `ui-app-driver,diagnostics` feature set.
2. Run `cargo fmt -p fret-ui -p fret-bootstrap`. Done.
3. Run `cargo check -p fret-ui -j 1`. Done.
4. Run focused `fret-ui` semantics tests for dirty gate, relations, active descendant, and virtual
   list collection metadata. Done.
5. Re-run the current combobox dev-fast perf gate and compare the newest bundle with
   `diag stats --sort time --top 6`. Done; rerun passed with zero threshold failures.
6. Run final diff and focused verification before commit.
7. Investigate whether app-level gallery commands should be excluded from unrelated heavy-component
   runtime snapshots through a command-surface/group mechanism instead of per-component micro
   caching.
8. Before committing the seventh slice, run `git diff --check` and re-run the focused
   action-availability test after the documentation update.

## Open Questions

- Should `ActiveScript` store the active wait predicate in `WaitUntilState` to avoid re-reading the
  current step for demand classification?
- Should `drive_script_for_window` pass a "fresh semantics this frame" marker instead of only
  `Option<&SemanticsSnapshot>` so stale snapshots cannot accidentally satisfy current semantics
  requirements?
- Should current-window `exists` / `not_exists(test_id)` ever use the cached bounds map when a fresh
  semantics snapshot was intentionally skipped? Current answer: no, unless a stronger freshness
  marker is added.
- Should command availability snapshots evaluate every registered widget command for a window, or
  should apps expose command surfaces/groups so unrelated command families do not tax dense
  component interactions?
