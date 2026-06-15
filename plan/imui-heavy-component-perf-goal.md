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

## 2026-06-15 Eighth Slice Findings

- Investigated the residual renderer tail after the closed command-palette slice. The latest green
  baseline bundle still had renderer text preparation around `418-582us`, mostly
  `collect_pin_keys` and `bucket_delta`, even when UI paint had no text re-shaping and paint cache
  was replaying stable subtrees.
- Added a renderer-level retained text pin bucket fast path in `fret-render-wgpu`. Each swapchain
  ring bucket records the exact visible `TextBlobId` sequence after a successful full pin pass. If
  the same bucket sees the exact same live text blob sequence again, it skips glyph-bucket rebuild,
  bucket delta, prewarm, and pin ref-count updates for that frame.
- The fast path is intentionally conservative:
  - it stores an exact `TextBlobId` list rather than a hash-only signature,
  - atlas reset clears bucket signatures,
  - missing/evicted text blobs disable reuse,
  - incomplete prewarm does not record a reusable signature,
  - scene changes fall back to the original full path.
- Added diagnostics visibility through
  `renderer_prepare_text_fast_scene_bucket_reuses`, including `fret-diag stats` top-row output as
  `renderer.text_prepare.counts(blobs/fast_reuse/pinned/prewarm/retained/added/removed)`.
- Focused validation passed:
  `cargo test -p fret-render-wgpu --lib prepare_for_scene --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-diag --lib renderer_prepare_text --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-render-wgpu --profile dev-fast -j 1`,
  `cargo check -p fret-bootstrap --profile dev-fast -j 1`,
  and `cargo check -p fret-diag --profile dev-fast -j 1`.
- The correct combobox dev-fast perf gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-text-pin-bucket-reuse-vc/1781457750260/bundle.schema2.json`.
  Top frame was `10103us` with `layout=5943us`, `solve=1155us`, `prepaint=629us`,
  `paint=3531us`, `dispatch=0us`, `hit_test=49us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `cache.replayed_ops=203`.
- Compared with the previous accepted bundle
  `target/fret-diag/gate-combobox-filter-select-devfast-command-palette-closed-vc/1781455378046/bundle.schema2.json`,
  the top frame moved from `11197us` to `10103us`, total considered time moved from `43755us` to
  `40004us`, and renderer text p95/max moved from `582us` to `446us`.
- The fast path does not hit the slowest filter/select mutation frames (`fast_reuse=0`) because
  their visible text blob sequence changes. It does hit stable frames (`fast_reuse=1`), reducing
  their text prepare path to roughly `12-16us`.
- Current decision: keep this slice as a shared renderer infrastructure win, but do not treat it as
  the final answer for the dense combobox path. The remaining worst frames are still dominated by
  popover/root layout request/apply plus renderer upload/finish/encode work and command availability
  tails around focus/text paste routing.

## 2026-06-15 Ninth Slice Findings

- Investigated the residual action availability tail after the renderer text-pin slice. The key
  design issue was not that shadcn-style component nesting is inherently too expensive; it was that
  owner-scoped action availability hooks could only declare "all commands" interest, so a runtime
  snapshot could route unrelated widget commands into policy handlers.
- Added command-specific availability interest to the action-route mechanism. The existing
  `action_on_command_availability_for_owner` and `action_add_on_command_availability_for_owner`
  APIs keep their conservative `All` behavior, while new command-specific APIs let strategy/app
  layers declare a precise `CommandId` interest.
- Refined the implementation from owner-level aggregation to entry-level filtering. This matters for
  app render action hooks: a single owner can register many typed action availability handlers, and
  the runtime should only invoke the entry whose declared command matches the command being queried.
- Reworked declarative command-interest metadata into a small composable structure. Built-in
  interests such as text editing, selectable text editing, and focus traversal can now union with
  command-specific action interests without widening to `All`. This also fixes the semantic hazard
  where an early built-in interest return on a `TextInput` node could hide a custom command-specific
  action availability hook on the same element.
- Updated `ecosystem/fret` app-render action availability to use the command-specific API. This
  keeps typed action availability discoverable without forcing unrelated command probes through the
  same app action owner.
- Focused validation passed:
  `cargo test -p fret-ui --lib owner_scoped_action_availability_for_command --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui --lib window_command_action_availability_snapshot --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui --profile dev-fast -j 1`,
  `cargo check -p fret --profile dev-fast -j 1`,
  `cargo check -p fret-ui-gallery --profile dev-fast -j 1`,
  `cargo fmt -p fret-ui -p fret`, and `git diff --check`.
- The correct combobox dev-fast perf gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-action-interest-entries-vc/1781460864667/bundle.schema2.json`.
  Top frame was `9885us` with `layout=5872us`, `solve=1084us`, `prepaint=624us`,
  `paint=3389us`, `dispatch=0us`, `hit_test=47us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `cache.replayed_ops=203`.
- This slice is primarily a mechanism-correctness and future scaling improvement, not a dramatic
  win for the current gate. The current run remains inside the green noise band around the previous
  `10103us` text-pin bundle, but `ui_gallery.switch.command_gate.action` still appears when that
  command itself is being evaluated. Command-interest filtering can prevent unrelated handlers from
  running; it cannot remove a command from a full-window snapshot command set.
- Current decision: keep the command-specific hook API and entry-level filtering. The next command
  availability question is publisher-level command grouping/surfaces, not more per-handler
  filtering.

## 2026-06-15 Layout Profile Follow-up

- Re-ran the correct combobox dev-fast gate with layout node profiling after the command-specific
  action-interest slice:
  `target/fret-diag/gate-combobox-filter-select-devfast-layout-profile-after-action-interest/1781461140853/bundle.schema2.json`.
- The gate remained green. Top frame was `10452us` with `layout=6134us`, `solve=1093us`,
  `prepaint=644us`, `paint=3674us`, `dispatch=0us`, `hit_test=56us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `cache.replayed_ops=203`.
- Root-level layout attribution showed the popover overlay root as the meaningful dirty root:
  `root[window-overlays.popover.29fc70edc4575465]` rebuilt a `DismissibleLayer` subtree with
  `subtree_layout_dirty_count=37` in about `285us`. The primary dirty source was model observation
  flowing through the overlay `ViewCache` and `InteractivityGate`.
- The main window root only performed a `mark_seen` pass over `1073` nodes in about `112us`. That is
  not currently a high-leverage or low-risk target compared with the remaining layout request/apply
  and renderer tails.
- Query/filter changes in `CommandPalette` and combobox remain layout-sensitive because they can
  change row materialization and content height. Treating those model observations as paint-only
  would be a contract bug unless the runtime gains a deeper virtual row layout contract.
- Renderer text preparation is now the cleaner next target for mutation frames: the latest profile
  still shows `renderer.text_prepare p95/max ~= 440us`, with `collect_pin_keys ~= 251us` and
  `bucket_delta ~= 171us` when `fast_reuse=0`. Stable frames are already covered by the retained
  text pin bucket fast path, so the open question is whether changed-blob keyed glyph bucket deltas
  can avoid rebuilding the full `GlyphKeyBuckets` on filter/select mutation frames.

## 2026-06-15 Tenth Slice Findings

- Implemented a renderer text pin-state delta path for mutation frames. `TextPinState` now keeps
  per-ring-bucket glyph membership sets, collects a lightweight scene pin snapshot, and computes
  retained/added/removed glyph keys directly from the current ref-count maps instead of rebuilding a
  full `GlyphKeyBuckets` and diffing it against the old bucket.
- The bucket update is in-place: removed glyph keys are deleted from the current ring bucket, added
  glyph keys are appended after successful atlas prewarm, and the exact scene signature is recorded
  only when the bucket is complete. This keeps atlas pin correctness aligned with the previous
  full-diff path while reducing allocation and retained-key movement.
- Switched the pin-state hot maps/sets to `rustc_hash::FxHashMap/FxHashSet`. These are internal
  glyph/blob id maps, not attacker-controlled lookup tables, and match the existing choice in the
  layout hot path.
- Removed the old full-bucket diff helper from `atlas.rs`; the regression surface now uses the real
  `prepare_for_scene` path instead of a detached helper test.
- Focused validation passed:
  `cargo test -p fret-render-wgpu --lib prepare_for_scene --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-render-wgpu --profile dev-fast -j 1`,
  `cargo build -p fret-ui-gallery --profile dev-fast -j 1`,
  `cargo fmt -p fret-render-wgpu`, and `git diff --check`.
- The correct combobox dev-fast perf gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-text-pin-fx-delta-vc/1781463556043/bundle.schema2.json`.
  Top frame was `9932us` with `layout=5635us`, `solve=1034us`, `prepaint=606us`,
  `paint=3691us`, `dispatch=0us`, `hit_test=47us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `cache.replayed_ops=203`.
- Renderer text preparation improved materially on the mutation frames:
  previous layout-profile bundle showed `renderer.text_prepare p95/max ~= 440us` with
  `collect_pin_keys=251us` and `bucket_delta=171us`; the accepted bundle shows
  `renderer.text_prepare p95/max=179us`, `collect_pin_keys=76us`, and `bucket_delta=80us`.
- Current decision: keep this slice as a shared renderer infrastructure win. The combobox gate is
  now again dominated by overlay/root layout request/apply, command-availability tails, and renderer
  upload/finish rather than text pin bucket reconstruction.

## 2026-06-15 Eleventh Slice Findings

- Re-profiled the correct combobox dev-fast gate with layout node profiling after the text pin-state
  delta slice:
  `target/fret-diag/gate-combobox-filter-select-devfast-layout-profile-after-text-pin-fx/1781463914238/bundle.schema2.json`.
  The top frame was `9655us` with `layout=5711us`, `solve=1054us`, `paint=3353us`,
  and renderer text preparation down to about `186us`.
- Layout attribution showed the remaining high-cost dirty root was the popover overlay
  `DismissibleLayer -> ViewCache -> InteractivityGate` chain. The important issue was that the
  keep-alive overlay `ViewCache` remained parent-dependent even though its root bounds are known and
  fill the overlay surface.
- Added a shared `overlay_keep_alive_view_cache_props()` helper in `fret-ui-kit` and applied it to
  modal, popover, hover overlay, and tooltip keep-alive caches. The helper gives the cache root
  `width: fill`, `height: fill`, and
  `ViewBoundaryHints::contain_layout_when_bounds_known(true)`.
- The corrected all-overlay probe passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-overlay-contained-all-vc/1781464820646/bundle.schema2.json`.
  Top frame was `9468us` with `layout=5682us`, `solve=1045us`, `paint=3205us`,
  and `contained_relayouts=1`.
- The main win is not a lower solve number yet; it is locality. `request_build` roots dropped from
  about `1187us` to about `144us`, roots apply dropped from about `925us` to about `4us`, and the
  parent `DismissibleLayer` became a cheap `mark_seen` pass of about `25us`.
- The remaining layout solve is now isolated inside the contained `ViewCache` relayout root
  (`layout_dependency = contained_when_bounds_known`), around `0.8-1.0ms` on the worst mutation
  frame. This is the next high-leverage target if fresh evidence keeps pointing at layout.
- While validating the shadcn overlay path, the focused `popover` test filter exposed an adjacent
  existing layout-contract issue: `PopoverHeader` had drifted onto the shrink-wrapped stack helper
  while its wrapping-text test still expected a fill-width inner stack. Restored the fill-width
  helper and changed the test to assert the actual layout contract (`Fill` + `min-width: 0`) instead
  of depending on whether the transparent helper materializes as `Flex` or `Container`.
- Focused validation passed:
  `cargo test -p fret-ui --lib try_with_state_mut_only_records_existing_state_keys_for_view_cache --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui --lib view_cache --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui-kit --profile dev-fast -j 1`,
  `cargo build -p fret-ui-gallery --profile dev-fast -j 1`,
  `cargo test -p fret-ui-shadcn --lib popover --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui-shadcn --lib combobox --profile dev-fast -j 1 -- --test-threads=1`,
  and `cargo fmt -p fret-ui -p fret-ui-kit -p fret-ui-shadcn`.
- Current decision: keep this slice. It is a mechanism-level containment fix with direct benefit to
  shadcn popover/combobox surfaces and likely to modal, hover-card, and tooltip surfaces. It also
  makes the next optimization clearer: optimize the contained overlay cache relayout itself, not the
  parent overlay root.

## 2026-06-15 Twelfth Slice Exploration

- Re-opened the latest contained-overlay bundle:
  `target/fret-diag/gate-combobox-filter-select-devfast-overlay-contained-all-vc/1781464820646/bundle.schema2.json`.
  The worst considered frame stayed green but still exceeded a strict 120Hz budget:
  `total=9468us`, `layout=5682us`, `layout.engine_solve=1045us`, `paint=3205us`,
  and `contained_relayouts=1`.
- The important distinction is that the popover overlay cache root was not merely a clean reused
  cache needing a root-only geometry update. On the worst mutation frame it was
  `reuse_reason = needs_rerender`, with the filtered command/list subtree changing. A root-only
  ViewCache relayout fast path would therefore not address the immediate gate unless it also had a
  correct way to prove child layout dependencies were unchanged.
- Root-only contained ViewCache relayout remains a future mechanism candidate, but it needs an
  authoritative dirty-cause contract. Debug invalidation details are useful evidence, not a safe
  behavior input. The runtime must distinguish root-only scheduling/geometric dirty from subtree
  dependency dirty before it can skip descendant expansion globally.
- The current residual cost is distributed across three regions rather than one obvious full rewrite
  target: contained overlay subtree layout, semantics refresh on dirty frames, and renderer
  upload/finish/text tail. This reinforces the current strategy: keep landing narrow mechanism and
  strategy fixes with perf evidence instead of declaring shadcn-style nested composition inherently
  too expensive.
- Negative experiment: keeping a large plain command source on the virtual-list lane after filtering
  to a tiny result set did remove the previous full-rows branch hotspot around `command.rs:3543`, but
  it did not improve the gate. The measured run at
  `target/fret-diag/gate-combobox-filter-select-devfast-large-source-virtual-vc/1781467734298/bundle.schema2.json`
  regressed to `total=10364us`, `layout=6339us`, `layout.engine_solve=1339us`, `paint=3391us`.
- Node-level profiling confirmed the reason. With forced virtualization on the one-row filtered
  state, the hot overlay nodes became the virtual-list path itself:
  `VirtualList self=640us total=796us`, `ScrollArea/Stack total=~0.95-1.18ms` at frame 160 in
  `target/fret-diag/gate-combobox-filter-select-devfast-large-source-virtual-node-profile/1781467878354/bundle.schema2.json`.
  Decision: do not land this strategy. Small filtered results should stay on the simple full-row
  layout path until the virtual-list fixed cost is substantially lower.
- The useful learning is architectural: stable layout shape is not automatically cheaper than the
  simplest shape for the current result set. The next optimization should target shared container
  fixed costs (`ScrollArea`, contained overlay relayout, virtual-list measurement/update) rather than
  forcing every command state through virtualization.
- Control experiment on the original non-forced strategy:
  `target/fret-diag/gate-combobox-filter-select-devfast-original-node-profile/1781468169506/bundle.schema2.json`
  passed with `total=10236us`, `layout=5985us`, `layout.engine_solve=1018us`,
  `paint=3632us`. The one-row full-row path showed the overlay `ScrollArea` around `502us`, while
  the forced virtual path added `VirtualList` work on top. This confirms the right immediate policy:
  keep virtualization thresholded by rendered row count.
- Landed the next narrow policy/mechanism split: the compact `ScrollArea` surface can now forward
  the existing low-level `ScrollAreaViewport::focus_ring(false)` knob, and `CommandPalette` disables
  the viewport focus-ring wrapper for its listbox scroll areas. Standalone `ScrollArea` parity stays
  unchanged by default; command/combobox listboxes keep focus in the input and expose highlight via
  `active_descendant`, so the viewport focus wrapper was duplicated strategy cost.
- Validation passed:
  `cargo test -p fret-ui-shadcn --lib scroll_area --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui-shadcn --lib command_palette --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui-shadcn --lib combobox --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui-shadcn --profile dev-fast -j 1`, and
  `cargo fmt -p fret-ui-shadcn`.
- The corrected combobox gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-command-list-no-viewport-focus-ring/1781468669211/bundle.schema2.json`.
  Top frame was `9827us` with `layout=5766us`, `layout.engine_solve=763us`,
  `paint=3403us`, `dispatch=90us`, `hit_test=16us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `contained_relayouts=1`. Dirty invalidation nodes on the mutation frame
  dropped to about `120-130`, and pointer tails stayed inside the checked-in thresholds.
- Decision: keep this slice. It is not a broad architecture rewrite, but it is the right kind of
  component-ecosystem optimization: expose an existing mechanism knob at the recipe surface and let
  the CommandPalette strategy avoid unnecessary focus/animation/semantics wrapper nodes.

## 2026-06-15 Full-Row Bounded Probe Follow-up

- Aligned the non-virtualized `CommandPalette` full-row listbox path with the virtualized path by
  disabling unbounded scroll viewport probing. Command/listbox surfaces already receive explicit
  strategy sizing and max-height constraints, so the shrink-wrap intrinsic probe is unnecessary for
  this recipe path.
- Added `command_palette_full_rows_use_bounded_scroll_viewport_probe` to lock this strategy at the
  element-tree level. The test deliberately avoids timing assertions; perf gates remain responsible
  for measuring whether the policy matters on a given scenario.
- Focused validation passed:
  `cargo test -p fret-ui-shadcn --lib command_palette --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui-shadcn --lib combobox --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui-shadcn --profile dev-fast -j 1`, and
  `cargo fmt -p fret-ui-shadcn`.
- The corrected combobox gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-full-row-bounded-probe/1781469475271/bundle.schema2.json`.
  Top frame was `9831us` with `layout=5837us`, `layout.engine_solve=1089us`,
  `paint=3383us`, `dispatch=0us`, `hit_test=47us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `contained_relayouts=1`.
- Decision: keep this as a small strategy-consistency cleanup, not as a claimed major win. It keeps
  the current green band stable and removes another avoidable listbox fixed-cost path. The dominant
  residual work is still contained overlay relayout, command-availability publication breadth, and
  renderer upload/finish/text tail.

## Next Verification

1. Revisit the contained popover `ViewCache` relayout solve tail now that parent-root request/apply
   is no longer the dominant overlay cost.
2. Keep reducing the remaining contained overlay listbox cost only where evidence points to shared
   fixed work. The latest slices removed the duplicated viewport focus wrapper, unbounded full-row
   probing, and hidden scrollbar chrome; remaining hot areas are contained overlay relayout solve,
   the core `Scroll` layout path, and renderer upload/finish/text.
3. Design a publisher-level command surface/group mechanism only if fresh evidence shows full-window
   snapshot command sets are still taxing dense component interactions. Per-handler filtering is now
   in place; the remaining lever is deciding which commands the publisher should evaluate at all.
4. Keep watching renderer upload/finish/encode p95, which is now often comparable to the remaining
   UI-side work in green combobox probes.

## 2026-06-15 Thirteenth Slice Findings

- Added a frame-local command action-availability demand contract in `fret-ui`. The default remains
  conservative: if no surface declares demand, `publish_window_runtime_snapshots(...)` still
  publishes all registered widget commands. Declared surfaces can now request either all widget
  commands or a specific command set; omitted commands remain unknown, not disabled.
- Kept the demand in `WindowElementState` and cleared it at frame boundaries. `ElementContext` now
  exposes narrow request APIs so ecosystem surfaces can declare their command-gating consumption
  without turning this into a global app setting.
- Wired the two immediate consumers:
  `command_catalog_entries_from_host_commands_with_options(...)` requests the full host catalog,
  while `menubar_from_runtime_with_focus_handle(...)` requests only command ids found in the
  normalized menu bar. This keeps OS/native menu compatibility conservative when no surface has
  declared demand, and lets the in-window menu avoid unrelated widget-command probes.
- Focused tests cover the compatibility boundary:
  no demand keeps full publication, filtered demand publishes only requested commands, and full
  demand wins over filtered demand.
- Validation passed:
  `cargo test -p fret-ui --lib window_command_action_availability_snapshot --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui-gallery --profile dev-fast -j 1`,
  `cargo check -p fret-ui-shadcn --profile dev-fast -j 1`,
  `cargo fmt -p fret-ui -p fret-ui-kit -p fret -p fret-ui-gallery`, and `git diff --check`.
- The corrected combobox dev-fast perf gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-command-demand-v1/1781472736858/bundle.schema2.json`.
  Top frame was `10032us` with `layout=5946us`, `layout.engine_solve=1141us`,
  `paint=3491us`, `dispatch=0us`, `hit_test=52us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `contained_relayouts=1`.
- `diag stats` confirmed the demand contract is active in the dense path:
  `window_runtime_snapshot.command_availability(widget_count/collect_us/eval_us)=4/34/112` on the
  top frame, with the slower observed frame at `4/55/437`. This removes unrelated full-registry
  widget-command publication from the closed command-palette + in-window-menu steady path, but it is
  not the dominant remaining cost.
- Current decision: keep this slice as the publisher-level command surface mechanism. The remaining
  strict-120Hz gap is still distributed across contained overlay relayout, renderer upload/finish,
  and smaller command/text tails rather than a single shadcn nesting tax.

## 2026-06-15 Fourteenth Slice Findings

- Converted `ScrollArea::show_scrollbar(false)` into a real viewport-only chrome path. It keeps the
  layout `Stack` root and `Scroll` viewport, but skips the `HoverRegion`, scrollbar visibility
  state, hidden `Scrollbar` primitives, interactivity gates, opacity wrappers, and corner chrome.
  The default `ScrollArea` path remains Radix/shadcn-aligned and still mounts scrollbar chrome.
- Factored the shared viewport construction so the default chrome path and the viewport-only path
  both use the same `Scroll` + optional focus-ring/semantics wrapper logic. This avoids a second
  focus-ring implementation while still returning the inner `Scroll` element id for scrollbar
  targeting.
- `CommandPalette` now declares `.show_scrollbar(false)` for both the virtualized and full-row
  listbox paths. This matches the recipe strategy: focus stays in the search input, highlight is
  exposed via `active_descendant`, and the listbox does not need hover-gated scrollbar chrome.
- Added structure tests:
  `scroll_area_show_scrollbar_false_uses_viewport_only_chrome` locks the standalone
  `show_scrollbar(false)` contract, and
  `command_palette_listboxes_use_scrollbarless_viewport_chrome` locks the CommandPalette recipe
  policy. These are element-tree tests rather than timing assertions.
- Validation passed:
  `cargo test -p fret-ui-shadcn --lib scroll_area --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui-shadcn --lib command_palette --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui-shadcn --lib combobox --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui-shadcn --profile dev-fast -j 1`,
  `cargo check -p fret-ui-gallery --profile dev-fast -j 1`,
  `cargo fmt -p fret-ui-shadcn`, and `git diff --check`.
- The first corrected combobox gate was a narrow solve-threshold miss:
  `target/fret-diag/gate-combobox-filter-select-devfast-scrollbarless-listbox/1781474462029/bundle.schema2.json`
  had `total=10814us`, `layout=6627us`, `layout.engine_solve=1469us`,
  `paint=3543us`, and failed only `top_layout_engine_solve_time_us > 1389us`.
  The run still showed the intended structural reduction: layout nodes around `32` and no hover
  invalidations, so this looked like a solve-tail sample rather than a clear regression.
- The rerun passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-scrollbarless-listbox-rerun/1781474598799/bundle.schema2.json`.
  Top frame was `9710us` with `layout=5587us`, `layout.engine_solve=1085us`,
  `paint=3516us`, `dispatch=0us`, `hit_test=48us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `contained_relayouts=1`.
- Node profiling after the change also passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-scrollbarless-listbox-node-profile/1781474698584/bundle.schema2.json`.
  Top frame was `10108us` with `layout=5848us`, `layout.engine_solve=1124us`,
  and `paint=3588us`. The overlay listbox `Scroll` node measured around
  `350us self / 517us total` on the profiled top frame, compared with the previous
  `~500-1200us total` band seen before removing hidden chrome.
- Decision: keep this slice. It is a policy-level trim rather than a broad framework rewrite, but
  it removes avoidable recipe chrome and confirms that shadcn-style composition does not inherently
  require hidden wrappers for every scroll surface. The remaining strict-120Hz gap is now dominated
  by contained overlay relayout solve, the core `Scroll` layout cost, and renderer upload/finish.

## 2026-06-15 Fifteenth Slice Findings

- Added a narrow static listbox surface for non-virtualized `CommandPalette` full-row results when
  filtering produces exactly one `Item` row. The static surface keeps the same outer list sizing
  contract as the viewport-only `ScrollArea` stack (`width: fill`, `min-width: 0`, `min-height: 0`,
  plus caller `refine_scroll_layout(...)` sizing), but skips `Scroll`, scroll handles, focus-ring
  wrappers, hidden scrollbar chrome, and scroll-to-active work.
- Kept the strategy intentionally narrow:
  grouped rows, headings, separators, loading rows, empty states, and multi-row results still use
  the existing full-row `ScrollArea` path; large plain item sets still use virtualization. The
  static path only applies to the one-row filtered state that was showing avoidable fixed scroll
  cost in the combobox gate.
- Added structure and semantics tests:
  `command_palette_single_item_full_rows_use_static_list_surface` proves the single-row path mounts
  no `Scroll`, `Scrollbar`, or `HoverRegion`; `command_palette_listboxes_use_scrollbarless_viewport_chrome`
  now also proves multi-row listboxes still mount one `Scroll`; and
  `command_palette_single_item_static_list_surface_preserves_active_descendant_semantics` proves
  the input still controls the listbox and its `active_descendant` points at the selected option.
- Validation passed:
  `cargo test -p fret-ui-shadcn --lib command_palette --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui-shadcn --lib combobox --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui-shadcn --profile dev-fast -j 1`,
  `cargo check -p fret-ui-gallery --profile dev-fast -j 1`,
  `cargo fmt -p fret-ui-shadcn`, and `git diff --check`.
- The corrected combobox dev-fast perf gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-single-item-static-listbox/1781477402294/bundle.schema2.json`.
  Top frame was `9808us` with `layout=5560us`, `layout.engine_solve=954us`,
  `prepaint=654us`, `paint=3594us`, `dispatch=0us`, `hit_test=44us`,
  `paint.cache_misses=0`, `cache.reused=1`, and `contained_relayouts=1`.
- `diag stats` confirmed the remaining tail is no longer the single-row scroll surface:
  `layout.nodes=31`, renderer p95/max `upload=688us`, `finish=1541us`, `encode=867us`,
  `text=200us`, and command availability stayed bounded at
  `widget_count/collect_us/eval_us=4/29/105` on the top frame. The strict 120Hz gap remains
  distributed across contained overlay relayout, paint traversal, and renderer upload/finish.
- Current decision: keep this slice. It is a component-policy optimization, not a core
  architecture rewrite. It reinforces the current architecture call: shadcn-style nesting is not
  inherently the blocker; avoidable fixed-cost recipe surfaces are. Continue optimizing where the
  gate shows unnecessary mechanism costs, and reserve a `ViewCache`/layout-engine rethink for a
  stronger root-only dirty-cause contract.

## 2026-06-15 Sixteenth Slice Findings

- The first broad incremental semantics-reuse pass was too permissive. It let clean descendants
  reuse old semantics records even when an ancestor had a semantic invalidation that changed the
  ancestor's children transform, which left scroll/visibility scripts observing stale descendant
  bounds.
- Narrowed the reuse contract so descendant replay is only allowed when no ancestor on that path was
  rebuilt in the same traversal. This keeps the useful sibling-subtree reuse win while forcing
  descendants to rebuild after scroll/transform changes.
- Added a regression test that mutates a parent's children transform and verifies the child rebuilds
  its semantics bounds on the next refresh.
- Validation passed:
  `cargo test -p fret-ui --lib semantics_focus_shortcuts --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui --profile dev-fast -j 1`, and `cargo fmt -p fret-ui`.
- The corrected perf gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-semantics-incremental-rerun/1781481180728/bundle.schema2.json`.
  `check.perf_thresholds.json` reports `failures=[]`; top frame was `9010us` with
  `layout=5222us`, `solve=860us`, `paint=3202us`, `dispatch=0us`, and `hit_test=46us`.
- The useful conclusion is narrower than the original experiment: semantics reuse is still a valid
  lever, but only below an ancestor-sensitivity line. Do not broaden it again without a stronger
  freshness marker for layout-affecting ancestors.
- The next obvious follow-up is not more semantics reuse. The remaining dense-component tail is now
  back where the earlier evidence pointed: overlay root solve/apply and renderer upload/finish.

## 2026-06-15 Seventeenth Slice Findings

- The runtime `Popover` path in `ecosystem/fret-ui-shadcn/src/popover.rs` still mounted
  `radix_popover::popover_dialog_wrapper(...)` around `PopoverContent`, and that wrapper was the
  remaining dialog-shaped `Semantics` hotspot in the combobox/popover path.
- Attempted to collapse the common popover trigger-controls target onto the existing
  `PopoverContent` root instead of a separate dialog wrapper, and changed `PopoverContent` itself
  to carry `SemanticsRole::Dialog` directly.
- The experiment was later rolled back after the combobox perf gate regressed, so this is a
  negative result rather than a landed optimization.
- Keep this as a recorded dead end: the next safe slice should start from the current wrapper-based
  shape and only move if a smaller, better-attributed overlay or command path appears.

## 2026-06-15 Seventeenth Slice Reversal

- The first attempt to collapse the popover dialog wrapper onto the `PopoverContent` root regressed
  the combobox perf gate badly: `top_total_time_us` rose to `20701us` and
  `top_layout_engine_solve_time_us` rose to `2049us`.
- The bad run also showed `window_runtime_snapshot.command_availability` and `focus_repair` tails
  rising on the same frame, which means the wrapper change was not a harmless structural trim.
- The popover code was reverted to the wrapper-based shape. Keep this as negative evidence: the
  `Dialog`-role root is not the next safe optimization lever on this path.
- Next direction should return to a lower-risk overlay/mechanism target or a command/layout path
  with clearer attribution, rather than trying to merge the popover semantic root again.

## 2026-06-15 Eighteenth Slice Diagnosis

- The failed popover-root experiment is not a clean framework signal by itself. On the bad run,
  the frame still spent most of its time in layout and paint, with renderer tail cost remaining
  visible: `layout=11539us`, `paint=8039us`, `renderer.finish=3724us`, `renderer.upload=585us`.
- The same run showed `window_runtime_snapshot.focus_repair=908us` and
  `window_runtime_snapshot.command_availability=410us`, but the command set was still tiny
  (`widget_count=4`). That makes command publication a real cost, not the dominant one.
- `repair_focus_node_from_focused_element_if_needed(...)` is already gated to final layout passes,
  and `revalidate_focus_for_dispatch_snapshot(...)` is a bounded reachability check. The hotspot
  is the repeated authoritative snapshot boundary around the overlay, not an obviously shallow
  focus algorithm.
- `publish_window_command_action_availability_snapshot_for_command_set(...)` is also already
  filtered and signature-cached. Its remaining cost matters, but the current evidence does not
  justify treating it as the main blocker for 120Hz on dense component surfaces.
- The stable recurring tail is renderer-side: `finish`, `upload`, and `text_prepare` remain the
  visible p95/max costs even on the better rerun. That points back to surface complexity and text
  churn rather than a single renderer bug.
- Keep the current wrapper-based popover shape. Do not revisit the dialog-root merge.
- The next verification should stay on a lower-risk component/recipe candidate or a heavier probe
  with clearer text/upload attribution before touching core focus or command machinery again.

## 2026-06-15 Nineteenth Slice Probe Triage

- Ran four more probe surfaces: `data-table`, `virtual-list`, `inspector`, and `code-editor`.
- `ce-data-table-probe` is not yet a stable perf gate. It failed at step 27 on the row-selection
  assertion after clicking `ui-gallery-data-table-row-0`, so the bundle is useful for diagnosis but
  not for a durable perf comparison until that assertion path is stabilized or replaced.
- The collected profiles still show the important shape:
  - `ce-data-table-probe`: `total=8353us`, `layout=7725us`, `paint=492us`,
    `command_availability=2422us`, `widget_count=4`.
  - `ce-virtual-list-probe`: `total=7345us`, `layout=6866us`, `solve=1896us`, `prepaint=87us`,
    `paint=392us`, `dispatch=145us`, `hit_test=17us`.
  - `ce-inspector-probe`: `total=4936us`, `layout=4270us`, `solve=1350us`, `prepaint=189us`,
    `paint=477us`, `dispatch=149us`, `hit_test=15us`.
  - `ce-code-editor-probe`: `total=789us`, `layout=125us`, `prepaint=399us`, `paint=265us`,
    `dispatch=0us`, `hit_test=0us`.
- The conclusion is still the same at a broader sample set: there is no single "shadcn nesting tax"
  to remove. The heavier table and inspector surfaces are still layout-dominant, command
  availability can become a visible secondary cost, and the current code-editor probe is too light
  to stand in for a real heavy editor path.
- Next action is to prefer the stable retained / view-cache data-table scripts already in the repo
  and treat the current `ce-data-table-probe` as a diagnosis-only artifact until its step-27
  selection assertion is stabilized or removed. The next optimization target should come from a
  probe that is both dense and repeatable.

## 2026-06-15 Twentieth Slice Stable Probe Selection

- Kept `ce-data-table-probe` in diagnosis-only mode. The step-27 row-selection assertion still
  makes it too fragile to serve as a durable gate.
- The stable next probes are the retained/view-cache data-table suites and the inspector torture
  suite, because they are dense enough to keep layout-dominant evidence while still being repeatable
  across runs.
- The collected profiles still separate the heavy surfaces from the light one: `ce-data-table-probe`
  at `8353us`, `ce-virtual-list-probe` at `7345us`, `ce-inspector-probe` at `4936us`, and
  `ce-code-editor-probe` at `789us`. The last one is too light to stand in for a real heavy editor
  path.
- The strongest remaining leverage point is still the table row/cell policy plus the
  `VirtualList` retained reconciliation seam, with `ecosystem/fret-ui-kit/src/declarative/table.rs`
  as the likely next focus and `window-command-availability-snapshot-v2` remaining secondary.
- The broader architecture conclusion has not changed: there is no single shadcn nesting tax to
  remove. The optimization path is component- and surface-specific, and the probe choice matters as
  much as the code change.

## 2026-06-15 Twenty-First Slice Architecture Split

- A focused architecture review of `table.rs` found that the data-table row/cell layer is still a
  wide adapter rather than the deepest seam. It owns sorting, grouping, pinning, selection, debug
  ids, paint order, grid lines, measured rows, and keep-alive policy in one broad surface.
- The higher-leverage next seam is `VirtualList` retained reconciliation: mount-time keep-alive /
  attach-detach / reuse logic, element `items_revision` / key-cache adapters, and the prepaint
  window-shift classifier. That seam crosses table, list, inspector, and editor-grade surfaces.
- Do not spend the next slice on local table knobs such as grid-line switches, paint-order flags,
  wrapper test-id plumbing, or header/body wrapper symmetry unless a stable probe makes one of them
  a primary owner. These knobs may clean code, but they do not deepen the architecture boundary.
- `focus_repair` and `command_availability` are now treated as an independent runtime owner named
  `Dispatch Snapshot`. Data-table, inspector, and virtual-list surfaces amplify this cost, but the
  owner is the window runtime snapshot path, not the component surface.
- The performance tracks for the next work should therefore be separated as:
  layout / virtual-list reconciliation, dispatch snapshot, and renderer tail. This prevents every
  heavy-surface spike from being misattributed to "shadcn nesting" or to data-table row/cell shape.

## 2026-06-15 Twenty-Second Slice Retained Reconcile Fast Path

- Implemented the first mechanism-layer follow-up in `crates/fret-ui/src/declarative/mount.rs`.
- Retained `VirtualList` reconcile no longer constructs the desired-key `HashSet` when
  `keep_alive == 0`; that set only exists to identify detached rows for the keep-alive pool.
- Added a conservative ordered-overlap fast path for contiguous retained windows. When the current
  and desired visible windows overlap in the same index/key order, preserved children are copied by
  slice position instead of using the generic `existing_by_key` map.
- The fast path deliberately rejects non-contiguous windows so custom range extractors, sticky rows,
  anchor rows, or reorders stay on the generic keyed reconcile path.
- Focused validation passed:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui retained_virtual_list_ordered_overlap mechanism_harness_retained_virtual_list_reconcile_matches_oracles --no-fail-fast --no-capture`,
  `cargo fmt -p fret-ui`, and `git diff --check`.
- Extended focused validation also passed:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui retained_virtual_list_ --no-fail-fast --no-capture`
  covered the retained reconcile harness plus retained VirtualList view-cache, prefetch, keep-alive,
  and viewport-authority tests.
- Runtime correctness gate passed:
  `target/release/fretboard-dev.exe diag suite ui-gallery-data-table-retained ... --launch -- cargo run -p fret-ui-gallery --release --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness`
  passed 12/12 scripts with evidence in
  `target/fret-diag/vlist-retained-fastpath-v1-cargo/suite.summary.json`.
- The first attempted prebuilt-exe suite failed before launch because the diagnostics preflight
  could not prove required gallery cargo features from a prebuilt binary; it was not a runtime or
  code failure.
- This slice should reduce allocation/hash work in the common retained scroll-window shift path, but
  it is not yet claimed as a user-visible perf win until a stable data-table or inspector probe is
  rerun.

## 2026-06-15 Twenty-Third Slice Semantics Translation Correctness

- Investigated a possible view-cache layout dirty expansion shortcut: stop walking into a clean
  nested cache root when an outer contained cache root was dirty. That shortcut is unsafe. A clean
  nested cache root may still need descendant geometry refreshed when the cached subtree moves, so
  pruning it can leave hit-test or semantics bounds stale.
- Kept layout dirty expansion conservative and added regression tests proving dirty expansion must
  pass through clean nested cache roots, dirty nested cache roots, and non-contained nested roots.
- Found a separate correctness bug in incremental semantics snapshot reuse. A clean subtree was
  reused only from `subtree_semantics_dirty_count == 0`; it did not compare the current semantic
  root bounds with the previous snapshot root bounds. When a cache-hit subtree moved, reused
  descendants could keep their old absolute bounds.
- Fixed the reuse contract in `crates/fret-ui/src/tree/ui_tree_semantics.rs`:
  identical parent/bounds reuse the previous range unchanged, origin-only movement with the same
  size translates the reused range, and other root-bound changes rebuild the subtree.
- Focused validation passed:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui view_cache_semantics_moving_relative_inset_updates_bounds_without_rerender --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui semantics_snapshot_rebuilds_clean_descendants_when_dirty_ancestor_transform_changes semantics_snapshot_reuses_clean_subtrees_between_dirty_refreshes --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui tree::tests::view_cache --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui view_cache_ --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui semantics_snapshot_ --no-fail-fast --no-capture`,
  `cargo fmt -p fret-ui`, `cargo check -p fret-ui --profile dev-fast -j 1`, and
  `git diff --check`.
- The stable data-table view-cache torture gate passed:
  `target/release/fretboard-dev.exe diag suite ui-gallery-data-table-view-cache-torture --dir target/fret-diag/vlist-view-cache-semantics-translate-v1 --session-auto --timeout-ms 900000 --ai-packet --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness`.
  Evidence is in
  `target/fret-diag/vlist-view-cache-semantics-translate-v1/sessions/1781497567429-36420/suite.summary.json`.
- The final data-table bundle remained layout-dominant rather than semantics-dominant:
  `total=14432us`, `layout=13507us`, `layout.engine_solve=3969us`, `layout.nodes=813`, and
  `paint=751us`.
- Decision: do not optimize by pruning nested view-cache layout dirty expansion. The next
  performance lane should target the data-table/view-cache layout root apply and row/cell layout
  policy, not stale geometry shortcuts.

## 2026-06-15 Twenty-Fourth Slice Known Scroll Extents

- Followed the data-table view-cache torture node-profile evidence: the worst frame contained many
  per-row horizontal `Scroll` nodes around `ui-gallery-data-table-row-123xx`, each spending roughly
  `~200us` self to rediscover content extents even though the table strategy already knows fixed
  column widths.
- Added `ScrollProps::known_content_size` as a mechanism-layer contract. When set, `Scroll` treats
  the supplied extent as authoritative for the scroll axis, skips unbounded child extent probing and
  post-layout overflow rediscovery, but still lays out children, clips, transforms, hit-tests, and
  synchronizes the scroll handle.
- Kept policy ownership in `fret-ui-kit`: declarative table center-column horizontal scroll wrappers
  now pass the summed center-column width. Generic shadcn `ScrollArea`, AI suggestions, and tests use
  `known_content_size: None` so auto-size scroll surfaces keep their old content-probing behavior.
- Focused validation passed:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui scroll_known_content_size_skips_extent_probe_but_updates_handle_extent --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui scroll_known_content_size_ scroll_intrinsic_content_mode_measures_children scroll_intrinsic_viewport_mode_skips_children --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui tree::tests::view_cache --no-fail-fast --no-capture`,
  `cargo check -p fret-ui-shadcn --tests --profile dev-fast -j 1`,
  `cargo check -p fret-ui-ai --tests --profile dev-fast -j 1`,
  `cargo check -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness -j 1`,
  `cargo fmt -p fret-ui -p fret-ui-kit -p fret-ui-shadcn -p fret-ui-ai`, and `git diff --check`.
- A broad `cargo check --workspace --all-targets --profile dev-fast -j 1` was run after most fixes.
  It progressed through the workspace and failed only on a test-only `ScrollProps` initializer in
  `ecosystem/fret-ui-ai/src/elements/checkpoint.rs`; that initializer was fixed and rechecked with
  `cargo check -p fret-ui-ai --tests --profile dev-fast -j 1`.
- The stable data-table view-cache torture gate passed:
  `target/release/fretboard-dev.exe diag suite ui-gallery-data-table-view-cache-torture --dir target/fret-diag/vlist-view-cache-known-scroll-extent-v1-rerun --session-auto --timeout-ms 900000 --ai-packet --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness`.
  Evidence is in
  `target/fret-diag/vlist-view-cache-known-scroll-extent-v1-rerun/sessions/1781503051796-66032/suite.summary.json`.
- `diag stats` on that bundle stayed layout/root-apply dominated:
  `total=14564us`, `layout=13519us`, `layout.engine_solve=6772us`, `layout.root apply=11209us`,
  `layout.nodes=813`, `paint=851us`, and command availability stayed small at
  `widget_count/collect_us/eval_us=4/7/13`.
- A node-profile rerun also passed:
  `target/fret-diag/vlist-view-cache-known-scroll-extent-node-profile-v1/sessions/1781503961156-129796/suite.summary.json`.
  The row-level horizontal `Scroll` nodes still appear in the top list around
  `172-178us self / 205-210us total`; this confirms the mechanism removes extent discovery
  duplication without eliminating the remaining per-row scroll layout baseline.
- Decision: keep this slice because it moves known table geometry out of repeated Scroll probing and
  records the policy/mechanism boundary cleanly. Do not keep grinding this path as the next main
  performance lever. The next serious target is `layout.root apply` / dirty-root application for the
  table view-cache torture frame, followed by row/cell layout policy if root apply attribution points
  back to the table layer.

## Next Verification

1. Use `tools/diag-scripts/suites/ui-gallery-data-table-retained/suite.json` and
   `tools/diag-scripts/suites/ui-gallery-data-table-view-cache-torture/suite.json` as the main
   repeatable gates.
2. Keep `ce-data-table-probe` as diagnosis-only until the row-selection assertion is stabilized or
   replaced.
3. Prioritize `layout.root apply` / dirty-root application attribution for
   `ui-gallery-data-table-view-cache-torture`; only return to per-row `Scroll` if node profiles show
   it growing beyond the current `~170-180us self` baseline.
4. Revisit `window-command-availability-snapshot-v2` as the `Dispatch Snapshot` lane only if stable
   probes show command publication or focus repair moving from secondary cost to primary blocker.

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

## 2026-06-15 Twenty-Fifth Slice Plan

- New evidence from the known-scroll-extent rerun says the current blocker is no longer scroll
  extent discovery alone. The worst view-cache torture frame is dominated by `layout.root apply`
  and `layout.engine_solve`, while per-row horizontal `Scroll` nodes remain as a repeated baseline
  cost.
- Hypothesis: the ordinary `table_virtualized` common path still models a single unpinned table as
  "one horizontal scroll viewport per visible row". That is structurally too expensive for dense
  shadcn/data-table surfaces because header/body share the same horizontal offset and fixed column
  widths are already known.
- Slice boundary: first optimize only the single center-column group path
  (`left_cols == 0 && center_cols > 0 && right_cols == 0`). Pinned columns and mixed groups keep the
  old per-group structure until a separate alignment gate proves an outer-scroll representation for
  pinned sections.
- Intended shape: keep one shared `ScrollHandle`, keep header/body horizontal alignment, preserve
  row pressable semantics and cell debug ids, but remove the repeated row-level horizontal `Scroll`
  wrappers from the unpinned body path. If the layout engine needs a definite content width, add a
  fixed-width content shell around header/body rather than forcing every row to be a scroll viewport.
- Gates for this slice: focused table tests around overflow/alignment/selection first, then
  `ui-gallery-data-table-view-cache-torture` with node profiling if the compile and correctness
  gates pass.

## 2026-06-15 Twenty-Fifth Slice Findings

- Added `ScrollContentTransform` as a `fret-ui` mechanism primitive. It reads an existing
  `ScrollHandle` offset and applies a children-only render/input transform, but it does not own
  viewport/content extent, does not handle wheel input, and does not publish scroll semantics.
- Updated the ordinary unpinned `table_virtualized` body path to replace each row's horizontal
  `Scroll` wrapper with `ScrollContentTransform` plus a fixed-width content shell. The table header
  keeps the single real horizontal `Scroll` owner, and the body gets one shared X-axis
  `WheelRegion` so rows continue to follow the same `ScrollHandle`.
- Kept pinned/mixed/grouped and retained-table paths unchanged in this slice. That avoids changing
  sticky-column semantics or grouped paint order without a separate alignment gate.
- Focused validation passed:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui scroll_content_transform_moves_children_without_owning_scroll_extent --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_unpinned_body_uses_shared_horizontal_transform table_virtualized_alignment_gate_header_matches_rows_under_overflow_and_variable_height table_virtualized_pointer_select_does_not_shift_row_bounds --no-fail-fast --no-capture`,
  `cargo check -p fret-ui-kit --tests --profile dev-fast -j 1`,
  `cargo check -p fret-ui --tests --profile dev-fast -j 1`,
  `cargo fmt -p fret-ui -p fret-ui-kit`, and `git diff --check`.
- The stable data-table view-cache torture suite with layout node profiling passed:
  `target/release/fretboard-dev.exe diag suite ui-gallery-data-table-view-cache-torture --dir target/fret-diag/vlist-view-cache-shared-row-xform-v1 --session-auto --timeout-ms 900000 --ai-packet --env FRET_LAYOUT_NODE_PROFILE=1 --env FRET_LAYOUT_NODE_PROFILE_TOP=30 --env FRET_LAYOUT_NODE_PROFILE_MIN_US=80 --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness`.
  Evidence is in
  `target/fret-diag/vlist-view-cache-shared-row-xform-v1/sessions/1781511962931-133156/suite.summary.json`
  and
  `target/fret-diag/vlist-view-cache-shared-row-xform-v1/sessions/1781511962931-133156/1781512647087/bundle.schema2.json`.
- `diag stats` on the final bundle: `total=13260us`, `layout=12236us`,
  `layout.engine_solve=6965us`, `layout.root apply=10839us`, and command availability stayed small
  at roughly `widget_count/collect_us/eval_us=4/9/12`.
- The node-profile shape changed in the intended direction: row-level top nodes now show `Flex` for
  `ui-gallery-data-table-row-123xx` at about `104-106us self`, replacing the previous repeated
  row-level horizontal `Scroll` entries around `172-178us self / 205-210us total`.
- Decision: keep this slice. It removes a structurally unnecessary per-row horizontal scroll
  viewport from the common unpinned table body and preserves shared header/body alignment.
- Next target remains `layout.root apply` / dirty-root application attribution and table row/cell
  layout policy. More scroll extent probing is now lower leverage unless future profiles show it
  becoming hot again.
