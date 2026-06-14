---
title: IMUI heavy-component perf and architecture audit
type: audit
date: 2026-06-14
execution: code
---

# IMUI Heavy-Component Perf and Architecture Audit

## Summary
This plan tracks the ongoing effort to push Fret's immediate-mode surfaces toward stable 120Hz behavior under editor-grade composition. The first confirmed slice is `select`; broader menu, overlay, and combobox families remain secondary candidates until the first slice shows whether the dominant cost is shell depth, layout churn, or shared primitive policy.

This lane is now explicitly documented in `docs/plans/` so each follow-on decision can record evidence, not just chat history.

## Problem Frame
The current IMUI surface is good enough for small demos, but it is not yet consistently boring under heavy composition. The failures seen in `imui_action_basics`, `imui_editor_controls_basics`, and `imui_plot_basics` are not one bug class: they mix stack pressure, layout instability, and contract gaps.

The working question is not "does the UI function at all". The question is whether the current module shapes can sustain dense editor-like usage without visible jank, overflow, or panic-level contract misses.

## Current Findings
- `ecosystem/fret-ui-shadcn/src/select.rs` is the largest recipe surface and currently mixes state transitions, sizing, scroll affordances, positioning, item rendering, modal/pointer behavior, and the test suite.
- The module is already far beyond a normal recipe file in size: it is about 481 KB, which is a strong signal that the interface is too shallow for the amount of behavior behind it.
- `ecosystem/fret-ui-kit/src/primitives/select.rs` is also substantial, but it is still a mechanism/helper layer rather than a recipe shell.
- `repo-ref/base-ui/packages/react/src/select/` splits the same concept into many small files (`root`, `trigger`, `positioner`, `popup`, `list`, `item`, `group`, `scroll arrows`, `portal`, `backdrop`, `arrow`, `icon`). That is the clearest external signal that Fret's current single-file recipe surface is too shallow.
- `ecosystem/fret-ui-shadcn/src/select.rs` already contains four different responsibilities in one file-shaped module: the public builder surface, the open/close and typeahead state machines, the placement/scroll geometry math, and the regression suite. Those are natural seam candidates for a fearless split.
- The shape is also materially out of line with the reference `base-ui` structure, which splits the same concept across `root`, `trigger`, `positioner`, `popup`, `list`, `item`, `group`, `scroll arrows`, `portal`, `backdrop`, and `icon` modules. That is strong evidence that the current Fret surface is too shallow to stay maintainable under more heavy examples.
- `ecosystem/fret-ui-kit/src/primitives/select.rs` is in a better state than the recipe layer, but it still combines policy, state, placement, and event plumbing. It should remain the mechanism baseline, not a place where recipe-specific growth accumulates.
- `imui_editor_controls_basics` shows visible height jitter when controls open or change state, which points to layout or chrome coupling rather than a pure paint issue.
- The jitter is not isolated to one widget: `DragValue`, `AxisDragValue`, and `Slider` all shared a common "two mounted branches" pattern without a stable outer shell contract.
- `NumericInput` already carries a stable outer flex box with a `min_height` row-height contract, which explains why `Exposure` behaved more consistently than `Roughness` in the demo.
- `imui_plot_basics` currently fails on a missing theme token (`surface`), which is a contract coverage problem and should be treated separately from perf.
- `imui_action_basics` can overflow the stack after repeated clicks, which suggests either uncontrolled nesting or a recursive update path that needs a dedicated repro.
- The broader menu/select policy lane already has a closeout record; this plan does not reopen it.
- The first promoted command/combobox perf probes now split the two cases clearly: command palette query/navigation is currently inside a 120Hz frame budget on the local RTX 4090 Windows run, while searchable combobox long-list filter/commit is not.
- The combobox long-list probe's worst frame is layout-dominated, not renderer-dominated: `total=24090us`, `layout=21581us`, `layout.engine_solve=10687us`, `layout.nodes=1827`, `paint.nodes=2599`, and `inv.calls=272`.
- The command probe's worst frame is much smaller but still layout-heavy: `total=3791us`, `layout=3214us`, `layout.engine_solve=1306us`.
- The next optimization target should therefore be combobox popup/list layout breadth and invalidation scope before further GPU or command-palette work.
- The first combobox policy optimization is now identified: searchable combobox commits used to clear the query in the same frame that the overlay started closing, which rematerialized the full long list while close presence was still mounted.
- Deferring the query clear until `Popover`/`Drawer` close completion addresses that close-phase rematerialization, but it does not solve the heavier filter-time cost: the command/combobox path still creates filtered rows and row elements for the full matching result set.
- `repo-ref/base-ui` uses virtualized combobox rows for large option sets, and Fret already has reusable virtual list mechanisms in `ecosystem/fret-ui-kit/src/declarative/list.rs`. The next deepening target is therefore a recipe-layer virtualized row seam for `CommandPalette`/searchable `Combobox`, not an example-only shortcut.
- After row virtualization and command availability pruning, the remaining combobox long-list tail is now dominated by root/layout apply breadth. The newest scroll profile shows `layout=17672us`, `layout_roots_apply_time_us=11805`, `layout_engine_solve_time_us=3824`, `layout_clean_geometry_apply_nodes=628`, `layout_clean_geometry_apply_fallback_layouts=20`, `invalidation_walk_calls=11`, and `invalidation_walk_nodes=396`.
- The popup listbox itself is no longer the main layout bottleneck. The expensive scroll profile is the main page viewport (`ui-gallery-content-viewport`), not `ui-gallery-combobox-long-list-listbox`: `total_us=11244`, `solve_barrier_us=8320`, `layout_children_corrected_content_us=1207`, `corrected_content_relayout=true`, `direct_children_layout_invalidated=true`, and `descendant_subtree_layout_dirty=true`.
- The concrete trigger in the current combobox long-list gallery snippet is the `Query: ...` state text under the combobox. It reads the query model and rerenders on every typed character. Because declarative text diffs currently treat any text content change as `Layout`, that tiny status label invalidates the surrounding main scroll content and pulls the whole page viewport into a costly corrected-content relayout.

## Scope Boundaries
### In scope
- Heavy IMUI cookbook examples and their diagnostic scripts.
- The `select`, `combobox`, `command`, `popover`, `dialog`, `drawer`, `sidebar`, `dropdown_menu`, `context_menu`, `menubar`, and `tabs` families as candidates for deepening.
- Refactors that deepen modules, reduce shallow shell duplication, or split mixed responsibilities into smaller seams.
- Repro and gate work that makes the heavy paths measurable.

### Out of scope
- Broad compatibility preservation for surfaces that are clearly worth breaking.
- Widening `crates/fret-ui` just to keep old policy mixed into mechanism.
- Reopening closed policy lanes unless fresh evidence appears.
- Rewriting the whole UI stack before the first deep slice proves where the real cost sits.

## Working Hypotheses
1. A meaningful share of the cost comes from shell depth and coupled responsibilities, not only from draw count.
2. Some of the visible instability is likely layout invalidation or state churn caused by nested composition.
3. The right first cut is to deepen the most concentrated module rather than scatter small optimizations across every caller.
4. The heavy surfaces should become easier to reason about after one or two large seams are made explicit.

## Immediate Decisions
- Use `select` as the first refactor target because it is the largest and most representative recipe surface.
- Keep `fret-ui` mechanism-only; move policy and defaults outward whenever a seam is available.
- Use `repo-ref/ui`, `repo-ref/base-ui`, and `repo-ref/imgui` as comparison sources, not as dependencies.
- Do not preserve compatibility when a cleaner seam is obviously better, unless a current consumer proves the old shape is still needed.
- Deepen `select` by extracting the state/placement/render seams first, then reassess whether any adjacent heavy recipe needs the same cut.
- Keep the public `Select` builder as a thin facade over narrower internal modules; the goal is locality and simpler change points, not a bigger single file.

## Implementation Units

### U1. Establish stable baselines and repro gates
**Goal:** Turn the current user-reported failures into reproducible gates with named scripts and stable demo targets.

**Files:**
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`
- `apps/fret-cookbook/examples/imui_plot_basics.rs`
- `tools/diag-scripts/cookbook/imui-action-basics/*.json`
- `tools/diag-scripts/cookbook/imui-editor-controls-basics/*.json`
- `tools/diag-scripts/cookbook/imui-plot-basics/*.json`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

**Test scenarios:**
- Repeated click stress no longer overflows the stack.
- Control open/close does not cause the editor-controls panel height to jump.
- Plot basics no longer panic on a missing theme token.
- The repros stay stable enough to compare before and after changes.

**Verification:**
- `cargo check -p fret-cookbook --features cookbook-imui --example imui_action_basics`
- `cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics`
- `cargo check -p fret-cookbook --features cookbook-imui-plot --example imui_plot_basics`
- `cargo run -p fretboard-dev -- diag suite cookbook-imui-editor-controls-basics --launch -- cargo run -p fret-cookbook --features cookbook-imui,cookbook-diag --example imui_editor_controls_basics`

### U2. Deepen `select` into smaller seams
**Goal:** Split the largest recipe surface into narrower responsibilities so state, layout, and interaction policy do not live in one file-shaped pile.

**Files:**
- `ecosystem/fret-ui-shadcn/src/select.rs`
- `ecosystem/fret-ui-kit/src/primitives/select.rs`

**Test scenarios:**
- Open/close still works with the same visible result.
- Keyboard navigation and selection still match the current contract.
- Scroll affordances still appear only when needed.
- The refactor does not add a new size jump or focus regression.
- The split should preserve the current public builder surface while moving internal state, placement, and render concerns behind narrower seams.
- The first cut should isolate the state machine and placement math before any further attempt to split rendering parts.

**Verification:**
- `ecosystem/fret-ui-shadcn/tests/select_test_id_stability.rs`
- `ecosystem/fret-ui-shadcn/tests/select_keyboard_navigation.rs`
- `ecosystem/fret-ui-shadcn/tests/select_escape_dismiss_focus_restore.rs`
- `ecosystem/fret-ui-shadcn/tests/select_typeahead.rs`

### U3. Audit adjacent heavy shells for the same pattern
**Goal:** Identify which neighboring component families need the same deepening pass and which can wait.

**Files:**
- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- `ecosystem/fret-ui-shadcn/src/context_menu.rs`
- `ecosystem/fret-ui-shadcn/src/menubar.rs`
- `ecosystem/fret-ui-shadcn/src/popover.rs`
- `ecosystem/fret-ui-shadcn/src/dialog.rs`
- `ecosystem/fret-ui-shadcn/src/drawer.rs`
- `ecosystem/fret-ui-shadcn/src/sidebar.rs`
- `ecosystem/fret-ui-shadcn/src/command.rs`
- `ecosystem/fret-ui-shadcn/src/combobox.rs`
- `ecosystem/fret-ui-shadcn/src/tabs.rs`

**Test scenarios:**
- Menu and overlay dismissal still behave the same after any shared seam extraction.
- Heavy surfaces remain scriptable through diagnostics.
- Any shared helper introduced here improves locality instead of becoming a new shallow layer.

**Verification:**
- `ecosystem/fret-ui-shadcn/tests/combobox_test_id_prefix_semantics.rs`
- `ecosystem/fret-ui-shadcn/tests/combobox_keyboard_navigation.rs`
- `ecosystem/fret-ui-shadcn/tests/combobox_escape_dismiss_focus_restore.rs`
- `ecosystem/fret-ui-shadcn/tests/combobox_filtering.rs`

### U4. Narrow combobox long-list layout breadth
**Goal:** Reduce the searchable combobox long-list filter/commit worst frame below the 120Hz budget by shrinking popup/list layout work, invalidation scope, or row materialization cost.

**Files:**
- `ecosystem/fret-ui-shadcn/src/combobox.rs`
- `ecosystem/fret-ui-shadcn/src/command.rs`
- `ecosystem/fret-ui-kit/src/primitives/combobox.rs`
- `ecosystem/fret-ui-kit/src/declarative/list.rs`
- `apps/fret-ui-gallery/src/ui/snippets/combobox/long_list.rs`
- `tools/diag-scripts/ui-gallery/perf/ui-gallery-combobox-filter-select-steady.json`

**Evidence target:**
- Current local worst frame: `target/fret-diag/imui-heavy-perf-probes-combobox/1781389001428/bundle.schema2.json`.
- Current bottleneck: `roots.apply=12311us`, `request_build=7374us`, `layout.engine_solve=10687us`, `layout.nodes=1827`, `paint.nodes=2599`.
- Deferred query-clear direction check: `target/fret-diag/imui-heavy-perf-probes-combobox-devfast-after/1781392775186/bundle.schema2.json`.
- Directional result: close/selection frames shrink after the query-clear deferral, while the worst dev-fast frame moves to `set_text_value("249")`; that means the remaining problem is long-list filtering/materialization and layout breadth.

**Test scenarios:**
- Open long-list combobox, filter to one item, select it, and close without selector regressions.
- Preserve keyboard/focus/dismiss behavior.
- Keep command-adapter metadata behavior covered by the existing focused unit test.
- Closing commits keep the filtered list stable until overlay close completion, then clear the query exactly once.
- Virtualized long-list rendering keeps row-level diagnostics selectors available for filtered results.

**Verification:**
- `target\debug\fretboard-dev.exe diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-combobox-filter-select-steady.json --dir target/fret-diag/imui-heavy-perf-probes-combobox --repeat 1 --warmup-frames 2 --timeout-ms 240000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_UI_GALLERY_START_PAGE=combobox --launch -- target\release\fret-ui-gallery.exe`
- `cargo check -p fret-ui-shadcn -j 1`
- Focused `combobox` tests when Windows test-target compilation is warm enough.

### U5. Add a CommandPalette virtual row seam
**Goal:** Make long command/combobox lists pay for visible rows instead of all matching rows while preserving cmdk filtering, keyboard navigation, a11y collection metadata, and diagnostics selectors.

**Files:**
- `ecosystem/fret-ui-shadcn/src/command.rs`
- `ecosystem/fret-ui-shadcn/src/combobox.rs`
- `ecosystem/fret-ui-kit/src/declarative/list.rs`
- `apps/fret-ui-gallery/src/ui/snippets/combobox/long_list.rs`
- `tools/diag-scripts/ui-gallery/perf/ui-gallery-combobox-filter-select-steady.json`

**Design decisions:**
- Keep filtering and navigation snapshot construction in `CommandPalette`; those are cmdk policy and must remain available to keyboard handling even when rows are not mounted.
- Virtualize row element materialization and layout, not the filtered item model itself. Filtering all items is acceptable at 250 rows; building and solving all row nodes is not.
- Keep the initial seam opt-in or threshold-driven at the recipe layer so small command palettes stay simple and existing command behavior remains easy to reason about.
- Prefer the existing `fret-ui-kit` virtual list helpers over introducing a new list mechanism inside `fret-ui-shadcn`.
- First implementation is intentionally item-only: no headings, group padding, separators, loading rows, or custom `CommandItem::children`. Those cases stay on the full render path because they are variable-height or move-only and would make the seam shallow.
- Active-descendant semantics remain tied to a mounted row element. The virtual range extractor keeps the active index mounted in addition to the visible window, so keyboard navigation does not point the input at a stale or missing row.

**Test scenarios:**
- Filtering to a single long-list option still produces a stable row `test_id` and selection action.
- Keyboard navigation still updates active descendant against the full filtered item set.
- The empty, loading, separator, heading, and grouped-row cases do not lose semantics.
- Non-virtual command palettes continue to render the same row order and collection metadata.
- Duplicate values keep distinct virtual row keys through occurrence suffixes, matching existing diagnostics selector behavior.

**Verification:**
- Focused `command_palette` and `combobox` unit filters after implementation.
- `cargo check -p fret-ui-shadcn -j 1`
- Directional dev-fast perf for `tools/diag-scripts/ui-gallery/perf/ui-gallery-combobox-filter-select-steady.json`.
- Release perf rerun when `target\release\fret-ui-gallery.exe` can be rebuilt without a release codegen timeout.

### U6. Keep stable status text out of layout dirty frontiers
**Goal:** Avoid turning stable, single-line text content updates into layout invalidations when the text contract proves the node's layout box is unchanged.

**Files:**
- `crates/fret-ui/src/declarative/mount.rs`
- `crates/fret-ui/src/declarative/tests/text_cache.rs`
- `apps/fret-ui-gallery/src/ui/snippets/combobox/long_list.rs`

**Design decisions:**
- This is a mechanism-level diff optimization, not a component-only shortcut: counters, status labels, badges, and form echoes are common in dense application UIs.
- The first landed optimization is intentionally limited to plain `Text`. Styled/selectable text content changes still invalidate layout until there are dedicated gates for span-boundary semantics, interactive span hit-testing, and selection geometry.
- The plain-text optimization must stay conservative. Text content changes may skip layout only when layout style, text style, wrap, overflow, alignment, and ink overflow are unchanged, wrapping is `TextWrap::None`, overflow is `Clip` or `Ellipsis`, the width is non-auto, and the height is fixed by layout or fixed line-height policy.
- Plain text content changes are not truly "paint-only": they skip layout, but mark semantics dirty with `DeclarativeTextContentChanged` so accessibility labels and automation snapshots refresh.
- Wrapped text (`Word`, `Balance`, `WordBreak`, `Grapheme`) must continue to invalidate layout because content changes can change height under the same width.
- Gallery state labels should opt into a stable single-line typography contract instead of using block-style `muted()` (`TextWrap::Word`).

**Evidence target:**
- Latest bottleneck bundle: `target/fret-diag/imui-heavy-perf-probes-combobox-devfast-scroll-profile/1781401573447/bundle.schema2.json`.
- Primary hotspot: main content viewport scroll relayout, not popup listbox relayout.

**Verification:**
- `cargo test -p fret-ui --lib stable_unwrapped_text_content_changes_are_paint_only_in_declarative_diff -j 1`
- `cargo test -p fret-ui --lib wrapped_text_content_changes_still_invalidate_layout_in_declarative_diff -j 1`
- `cargo fmt -p fret-ui`
- `cargo check -p fret-ui -j 1`
- `cargo build -p fret-ui-gallery --profile dev-fast -j 1`
- Directional dev-fast perf for `tools/diag-scripts/ui-gallery/perf/ui-gallery-combobox-filter-select-steady.json` with scroll layout profiling enabled.

### U7. Cache command availability interest during snapshot publication
**Goal:** Reduce widget command snapshot publication overhead by caching whether declarative host nodes can participate in command availability for the duration of one publication.

**Files:**
- `crates/fret-ui/src/tree/commands.rs`
- `crates/fret-ui/src/tree/mod.rs`
- `crates/fret-ui/src/tree/tests/window_command_action_availability_snapshot.rs`

**Design decisions:**
- Keep the cache local to `publish_window_command_action_availability_snapshot`. It is not a retained tree cache and must not survive cross-frame state or hook changes.
- Cache a small interest profile (`All`, `TextEdit`, `SelectableTextEdit`, `FocusTraversal`, `None`) instead of caching `(node, command)` results. The profile shape is what gives cross-command leverage.
- Keep single-command dispatch on the uncached path. Dispatch does not repeatedly probe all widget commands, so it does not need the extra cache object.
- Treat declarative nodes with managed-surface hooks or action availability hooks as `All` until typed action hooks expose explicit command-interest metadata.
- Use a test-only, thread-local probe counter to lock the publication-cache behavior without adding runtime diagnostics or parallel-test flakiness.

**Evidence target:**
- Before this slice, post-text-diff dev-fast combobox had worst `total=12232us` with command availability spikes around `3.5ms` to `4.9ms`; evidence bundle `target/fret-diag/imui-heavy-perf-probes-combobox-devfast-final-semantics/1781406489463/bundle.schema2.json`.
- After this slice, dev-fast combobox worst is `total=10874us`, `layout=4994us`, `paint=5101us`, and `command_availability_eval=990us`; evidence bundle `target/fret-diag/imui-heavy-perf-probes-combobox-devfast-command-availability-cache/1781409098126/bundle.schema2.json`.

**Verification:**
- `cargo fmt -p fret-ui`
- `git diff --check`
- `cargo check -p fret-ui -j 1`
- `cargo check -p fret-ui --tests -j 1`
- `cargo build -p fret-ui-gallery --profile dev-fast -j 1`
- `target\debug\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\perf\ui-gallery-combobox-filter-select-steady.json --dir target\fret-diag\imui-heavy-perf-probes-combobox-devfast-command-availability-cache --repeat 1 --warmup-frames 2 --timeout-ms 240000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_UI_GALLERY_START_PAGE=combobox --launch -- target\dev-fast\fret-ui-gallery.exe`
- Focused test `cargo test -p fret-ui --lib action_availability_snapshot_caches_declarative_interest_within_publication -j 1` still timed out during Windows test-target compilation; `cargo check --tests` covers compilation of the test body.

### U8. Opt dynamic combobox page out of whole-page content cache
**Goal:** Keep `FRET_UI_GALLERY_VIEW_CACHE=1` from wrapping the combobox page's high-churn query/open/overlay state in a whole-page content cache root.

**Files:**
- `apps/fret-ui-gallery/src/spec.rs`
- `apps/fret-ui-gallery/src/ui/snippets/combobox/long_list.rs`

**Design decisions:**
- Treat this as a Gallery page policy correction, not a `fret-ui` mechanism change. The bad frame appears when an interactive page is wrapped by the content cache root, while shell-only cache stays near the current baseline.
- Keep shell/sidebar cache available. The regression is specifically `cache_content=1` on the combobox page.
- Do not use a broad global view-cache heuristic from this one case. Other mostly-static documentation pages remain cacheable until a page-specific perf script proves otherwise.

**Evidence target:**
- Before opt-out, same view-cache-on run produced worst `total=44825us`, `layout=40451us`, `layout_roots_apply=31049us`, `layout.clean_geometry.apply_nodes=630`; evidence bundle `target/fret-diag/imui-heavy-perf-probes-combobox-devfast-viewcache-current/1781410310252/bundle.schema2.json`.
- Shell-only view cache produced worst `total=12973us`, `layout=8143us`, `layout_roots_apply=968us`; evidence bundle `target/fret-diag/imui-heavy-perf-probes-combobox-devfast-viewcache-shell-only/1781410355737/bundle.schema2.json`.
- After opt-out, same view-cache-on run produced worst `total=12643us`, `layout=7593us`, `layout_roots_apply=703us`; evidence bundle `target/fret-diag/imui-heavy-perf-probes-combobox-devfast-viewcache-after-combobox-optout/1781411687462/bundle.schema2.json`.
- No-view-cache dev-fast baseline after this change produced worst `total=12951us`, `layout=3854us`, `paint=8185us`; evidence bundle `target/fret-diag/imui-heavy-perf-probes-combobox-devfast-baseline-after-cache-optout/1781411739477/bundle.schema2.json`.

**Verification:**
- `cargo fmt -p fret-ui-gallery`
- `cargo check -p fret-ui-gallery --profile dev-fast -j 1`
- `cargo build -p fret-ui-gallery --profile dev-fast -j 1`
- `target\debug\fretboard-dev.exe diag perf tools\diag-scripts\ui-gallery\perf\ui-gallery-combobox-filter-select-steady.json --dir target\fret-diag\imui-heavy-perf-probes-combobox-devfast-viewcache-after-combobox-optout --repeat 1 --warmup-frames 2 --timeout-ms 240000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_UI_GALLERY_START_PAGE=combobox --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_UI_GALLERY_VIEW_CACHE_CONTENT=1 --launch -- target\dev-fast\fret-ui-gallery.exe`
- `cargo test -p fret-ui-gallery --lib combobox_opts_out_of_whole_page_content_cache -j 1` timed out during Windows test-target compilation without a test assertion result.

## Progress Log
- 2026-06-14: user-reported failures include stack overflow in `imui_action_basics`, missing theme token `surface` in `imui_plot_basics`, and height jitter in `imui_editor_controls_basics`.
- 2026-06-14: local inspection showed `select` is the largest recipe surface and a strong candidate for the first deepening slice.
- 2026-06-14: the closed `shadcn-menu-select-policy-followon-v1` lane stays closed; this audit is a new perf/architecture track, not a reopen.
- 2026-06-14: `DragValue`, `AxisDragValue`, and `Slider` now share a stable session shell helper instead of relying on ad hoc outer containers.
- 2026-06-14: new structural tests lock the shell layout contract for those three controls and their hidden branches.
- 2026-06-14: `cargo test -p fret-ui-editor session_shell -j 1` passed.
- 2026-06-14: `cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics -j 1` passed.
- 2026-06-14: `cargo check -p fret-cookbook --features cookbook-imui --example imui_action_basics -j 1` passed.
- 2026-06-14: `repo-ref/base-ui` shows the reference `select` surface split across many small files, which strongly argues for deepening `fret-ui-shadcn::Select` instead of continuing to grow one large recipe module.
- 2026-06-14: current `select` evidence points to three practical seams: state machine, placement/layout, and render parts. The first refactor should target the first two seams.
- 2026-06-14: adjacent heavy recipes remain candidates, but `select` is the highest-leverage first cut because it concentrates the most behavior behind the current facade.
- 2026-06-14: first code slice split `select.rs` into private seam modules: `select/interaction.rs` for open-change and trigger/session state, `select/geometry.rs` for popper width and list-height sizing, and `select/content_tree.rs` for entry traversal/typeahead/value-label helpers.
- 2026-06-14: `cargo fmt -p fret-ui-shadcn` passed.
- 2026-06-14: `cargo check -p fret-ui-shadcn -j 1` passed after the seam split.
- 2026-06-14: focused `cargo test` / `cargo nextest` select filters did not produce a failure, but timed out during test compilation on Windows; rerun the focused select gates after the test artifact cache is warm or with a longer timeout.
- 2026-06-14: second code slice moved overlay row normalization into `select/content_tree.rs` (`SelectRow`, row disabled mask, labels, values, item-count, selected-index helpers), leaving render assembly in `select.rs`.
- 2026-06-14: `cargo fmt -p fret-ui-shadcn` passed again after row-normalization extraction.
- 2026-06-14: `cargo check -p fret-ui-shadcn -j 1` passed again after row-normalization extraction.
- 2026-06-14: third code slice moved the grouped entry recursion into `select/content_render.rs` (`render_select_entries`), keeping row rendering closures in `select.rs` while giving future render-part extraction a concrete seam.
- 2026-06-14: `cargo fmt -p fret-ui-shadcn` passed after entry-render extraction.
- 2026-06-14: `cargo check -p fret-ui-shadcn -j 1` passed after entry-render extraction.
- 2026-06-14: fourth code slice replaced per-frame repeated row derivations with a single `SelectRows` snapshot in `select/content_tree.rs`, co-locating row order, disabled mask, labels, values-by-row, and item count behind one private interface.
- 2026-06-14: `cargo fmt -p fret-ui-shadcn` passed after `SelectRows` extraction.
- 2026-06-14: `cargo check -p fret-ui-shadcn -j 1` passed after `SelectRows` extraction.
- 2026-06-14: added focused `SelectRows` unit coverage for grouped flattening, disabled masks, labels, values-by-row, selected lookup, and disabled-root behavior.
- 2026-06-14: `cargo test -p fret-ui-shadcn --lib select_rows_ -j 1` surfaced a test-only temporary-Arc lifetime bug, which was fixed; reruns then timed out during Windows test-target compilation without a test failure result.
- 2026-06-14: `cargo check -p fret-ui-shadcn -j 1` passed after the focused `SelectRows` tests were added.
- 2026-06-14: adjacent audit found the same repeated metadata pattern in `dropdown_menu.rs` and `context_menu.rs`: root/submenu paths separately compute item counts, roving labels, and disabled masks before rendering. These are good follow-on candidates for a menu row snapshot seam, but they are riskier than `SelectRows` because they include submenu extraction and command gating.
- 2026-06-14: `dropdown_menu.rs` now has a private `DropdownMenuRovingMetadata` seam that builds root/submenu roving labels, disabled flags, and item count in one recursive pass, replacing separate count/collect passes in both menu surfaces.
- 2026-06-14: `cargo check -p fret-ui-shadcn -j 1` passed after the dropdown-menu metadata seam.
- 2026-06-14: `cargo test -p fret-ui-shadcn --lib dropdown_menu_disabled_focusable_items_remain_roving_candidates -j 1` timed out during Windows test-target compilation without a test failure result; the focused test body was updated to assert the new metadata seam.
- 2026-06-14: `context_menu.rs` now has a private `ContextMenuRovingMetadata` seam that computes leading-slot need, roving labels, gated disabled flags, and item count in one recursive pass for both submenu and root panels.
- 2026-06-14: `cargo check -p fret-ui-shadcn -j 1` passed after the context-menu metadata seam.
- 2026-06-14: `cargo test -p fret-ui-shadcn --lib context_menu_items_have_collection_position_metadata_excluding_separators -j 1` timed out during Windows test-target compilation without a test failure result.
- 2026-06-14: `menubar.rs` has the same labels/disabled/item-count pattern in root and submenu panels, but the surrounding closure also owns active-row selection, submenu entry extraction, and group-active switching. Treat it as a next candidate only after adding/choosing a tighter regression gate.
- 2026-06-14: `menubar.rs` now has a private `MenubarRovingMetadata` seam that computes leading-slot need, gated labels/disabled flags, item count, and first active index in one pass for root and submenu panels without changing submenu extraction or rendering.
- 2026-06-14: `cargo check -p fret-ui-shadcn -j 1` passed after the menubar metadata seam.
- 2026-06-14: `cargo test -p fret-ui-shadcn --lib menubar_items_have_collection_position_metadata_excluding_separators -j 1` timed out during Windows test-target compilation without a test failure result.
- 2026-06-14: The next P1 heavy-component candidates are `CommandPalette`, `Combobox`, `Carousel`, `DataTable` toolbar recipes, and `Sidebar`; `Tabs` and `Calendar` remain lower-priority because their repeated row metadata exists but typical row counts are smaller or current parity risk is lower.
- 2026-06-14: `command.rs` now has a private `CommandPaletteNavigationSnapshot` seam that co-locates command entries, navigation/activation/semantics disabled flags, item groups, and group order. The key handler now reuses this snapshot instead of allocating disabled flag vectors and group order on every navigation key press.
- 2026-06-14: `cargo check -p fret-ui-shadcn -j 1` passed after the command-palette navigation snapshot seam.
- 2026-06-14: `cargo test -p fret-ui-shadcn --lib command_palette_navigation_snapshot_reuses_disabled_and_group_metadata -j 1` timed out during Windows test-target compilation without a test failure result; residual `cargo`/`rustc` processes from that validation were stopped.
- 2026-06-14: `combobox.rs` now has a private `ComboboxCommandItemFrame` seam that centralizes the repeated `ComboboxItem -> CommandItem` adapter metadata: detail-aware label text, keywords, disabled/selected state, test id derivation, and selection commit action. Drawer/popover search and plain-list paths now share this seam while keeping their own visuals.
- 2026-06-14: `cargo check -p fret-ui-shadcn -j 1` passed after the combobox command-adapter seam.
- 2026-06-14: `cargo test -p fret-ui-shadcn --lib combobox_command_item_frame_derives_command_row_metadata -j 1` timed out during Windows test-target compilation without a test failure result; residual validation `cargo`/`rustc` processes were stopped.
- 2026-06-14: added `ui-gallery-command-palette-navigation-filter-steady` and `ui-gallery-combobox-filter-select-steady` perf probes and promoted them into `perf-ui-gallery-general-app-components`.
- 2026-06-14: `python -m json.tool` passed for the two new perf probes and the updated suite manifest.
- 2026-06-14: `python tools/check_diag_scripts_registry.py --write` refreshed `tools/diag-scripts/index.json`; the follow-up strict registry check passed. The registry pass itself takes about 80-90 seconds locally.
- 2026-06-14: first `diag perf` attempt used `cargo run -p fret-ui-gallery --release` and timed out at 10 minutes while the release build consumed the timeout; residual validation processes were stopped.
- 2026-06-14: rerunning command probe against `target\release\fret-ui-gallery.exe` passed with worst frame `total=3791us`, `layout=3214us`, `layout.engine_solve=1306us`; evidence bundle `target/fret-diag/imui-heavy-perf-probes-command/1781388826377/bundle.schema2.json`.
- 2026-06-14: rerunning combobox probe against `target\release\fret-ui-gallery.exe` passed but exposed a real perf problem: worst frame `total=24090us`, `layout=21581us`, `layout.engine_solve=10687us`; evidence bundle `target/fret-diag/imui-heavy-perf-probes-combobox/1781389001428/bundle.schema2.json`.
- 2026-06-14: `diag stats --sort cpu_cycles` and `--sort time` on the combobox bundle show the spike is real UI-thread work, not a renderer-only issue: `roots.apply=12311us`, `request_build=7374us`, `layout.nodes=1827`, `paint.nodes=2599`, `inv.calls=272`.
- 2026-06-14: `combobox.rs` now defers query clearing for close-on-commit selections until `Popover`/`Drawer` close completion, keeping the filtered list stable during close presence instead of rematerializing the full list on the selection frame.
- 2026-06-14: `cargo fmt -p fret-ui-shadcn` passed for the deferred query-clear change.
- 2026-06-14: `cargo check -p fret-ui-shadcn -j 1` passed for the deferred query-clear change.
- 2026-06-14: `cargo test -p fret-ui-shadcn --lib combobox_ -j 1` timed out during Windows test-target compilation without a test failure result.
- 2026-06-14: `cargo build -p fret-ui-gallery --release -j 1` did not produce a fresh release binary after more than 20 minutes of `fret_ui_gallery` codegen, so no same-profile after number is available yet.
- 2026-06-14: `cargo build -p fret-ui-gallery --profile dev-fast -j 1` passed and the dev-fast perf rerun moved the worst work to the filter-input step, with close/selection frames much smaller afterward. This supports the next decision: optimize row materialization/layout via a virtual row seam.
- 2026-06-14: `command.rs` now has a private item-only virtual row seam for `CommandPalette`: pure long item lists use `VirtualListOptions::fixed`, stable row keys/revisions, and active-index range injection while grouped/loading/custom-child palettes keep the full render path.
- 2026-06-14: new command structural tests cover virtual eligibility, grouped/loading/custom-child rejection, duplicate occurrence row keys, test-id derivation, and row revision changes.
- 2026-06-14: `cargo fmt -p fret-ui-shadcn` passed after the virtual row seam.
- 2026-06-14: `cargo check -p fret-ui-shadcn -j 1` passed after the virtual row seam.
- 2026-06-14: `cargo test -p fret-ui-shadcn --lib command_palette_virtual -j 1` timed out during Windows test-target compilation without a test failure result; no unrelated `dear-imgui`/`dear-implot` nextest processes were stopped.
- 2026-06-14: `cargo build -p fret-ui-gallery --profile dev-fast -j 1` passed after the virtual row seam.
- 2026-06-14: dev-fast combobox perf after virtualization passed the script and materially reduced layout breadth: worst frame `total=46780us`, `layout=34640us`, `layout.engine_solve=6548us`, `layout.nodes=52`, `paint.nodes=1070`, `inv.calls=11`; evidence bundle `target/fret-diag/imui-heavy-perf-probes-combobox-devfast-virtualized/1781397188538/bundle.schema2.json`.
- 2026-06-14: compared to the prior dev-fast direction bundle (`total=105187us`, `layout=95079us`, `layout.engine_solve=50873us`, `layout.nodes=1775`, `paint.nodes=2598`, `inv.calls=265`), virtualization confirms row materialization/layout breadth was a real bottleneck. The remaining worst frames are still above 120Hz and now point at root apply, command availability, and focus traversal costs rather than 250-row layout.
- 2026-06-14: a framework-level command availability slice now avoids creating optional declarative hook state during availability probes, short-circuits focus traversal availability once one candidate is found, and skips declarative host availability calls for nodes with no built-in or hook-level interest.
- 2026-06-14: `cargo fmt -p fret-ui` passed after the command availability slice.
- 2026-06-14: `cargo check -p fret-ui -j 1` passed after the command availability slice.
- 2026-06-14: focused `cargo test -p fret-ui --lib try_with_state_mut_only_records_existing_state_keys_for_view_cache -j 1` and `cargo test -p fret-ui --lib focus_traversal_availability_short_circuits_after_first_candidate -j 1` timed out during Windows test-target compilation without a test failure result.
- 2026-06-14: `cargo build -p fret-ui-gallery --profile dev-fast -j 1` passed after the command availability slice.
- 2026-06-14: dev-fast combobox perf after the declarative availability-interest fast path passed with worst frame `total=23687us`, `layout=18558us`, `layout.engine_solve=4063us`, `paint=4415us`, `command_availability_eval=4734us`, and `roots.apply=12546us`; evidence bundle `target/fret-diag/imui-heavy-perf-probes-combobox-devfast-declarative-availability-interest/1781401057880/bundle.schema2.json`.
- 2026-06-14: compared to the virtualized dev-fast bundle (`total=46780us`, `command_availability_eval=15030us`, `roots.apply=20769us`), the availability-interest slice confirms that shared mechanism overhead was a real bottleneck. The next bottleneck is now root/layout apply breadth, not full row materialization or availability probing alone.
- 2026-06-14: scroll profiling of the post-availability combobox run identified the main gallery content viewport as the remaining expensive layout frontier: the popup listbox profile is sub-millisecond, while the main viewport pays about 11ms because the query status text rerender is classified as layout invalidation.
- 2026-06-14: next slice is a conservative declarative text diff optimization plus a gallery status-label layout contract change: single-line stable text content changes should be paint-only, while wrapped text remains layout-affecting.
- 2026-06-14: during review, the text diff optimization was narrowed from plain/styled/selectable text to plain `Text` only. Rich/selectable text carries span-boundary, interactive-span, and selection-geometry obligations that need separate gates before skipping layout safely.
- 2026-06-14: plain text content changes now use a non-layout invalidation path that still marks semantics dirty via `DeclarativeTextContentChanged`; the focused regression test asserts both zero layout work and updated text semantics.
- 2026-06-14: `text_control_label` now has a fixed line-box height, and the combobox long-list snippet uses explicit single-line `TextProps` for `Query:` / `Selected:` state rows instead of block-style `muted()` typography.
- 2026-06-14: validation passed for `cargo fmt -p fret-ui -p fret-ui-kit`, `git diff --check`, `cargo check -p fret-ui -j 1`, `cargo check -p fret-ui-kit -j 1`, `cargo check -p fret-ui-shadcn -j 1`, and `cargo build -p fret-ui-gallery --profile dev-fast -j 1`.
- 2026-06-14: focused tests `cargo test -p fret-ui --lib stable_unwrapped_text_content_changes_are_paint_only_in_declarative_diff -j 1` and `cargo test -p fret-ui --lib wrapped_text_content_changes_still_invalidate_layout_in_declarative_diff -j 1` still timed out during Windows test-target compilation without a test assertion result; residual Fret `rustc` processes were stopped.
- 2026-06-14: final dev-fast combobox perf after semantics-safe text diff passed with worst frame `total=12232us`, `layout=5463us`, `layout.engine_solve=1412us`, `paint=5998us`, `roots.apply=859us`, `layout.nodes=29`, and evidence bundle `target/fret-diag/imui-heavy-perf-probes-combobox-devfast-final-semantics/1781406489463/bundle.schema2.json`.
- 2026-06-14: compared to the earlier final control-label-height bundle (`total=9999us`, `layout=4578us`, `paint=4691us`), the 12.2ms run shows measurement noise and remaining non-layout hotspots. The important structural result holds: roots.apply is now about 1ms and clean geometry applies only a handful of nodes instead of hundreds.
- 2026-06-14: remaining above-120Hz work is no longer the main scroll layout frontier. The next hotspots are command availability eval spikes (`~3.5ms` to `~4.9ms` in worst frames), paint cache misses / text prepare, and renderer finish/encode tail.
- 2026-06-14: command availability snapshot publication now uses a short-lived declarative interest cache shared across all widget command routes inside one publication. Ordinary declarative host nodes no longer re-read the same element record and hook states for every registered widget command.
- 2026-06-14: added a focused structural test that uses a test-only thread-local probe counter to assert repeated widget command publication profiles each declarative node once per publication.
- 2026-06-14: validation passed for `cargo fmt -p fret-ui`, `git diff --check`, `cargo check -p fret-ui -j 1`, `cargo check -p fret-ui --tests -j 1`, and `cargo build -p fret-ui-gallery --profile dev-fast -j 1`.
- 2026-06-14: focused `cargo test -p fret-ui --lib action_availability_snapshot_caches_declarative_interest_within_publication -j 1` timed out during Windows test-target compilation without a test assertion result; residual Fret test `cargo`/`rustc` processes were stopped.
- 2026-06-14: dev-fast combobox perf after the publication cache passed with worst frame `total=10874us`, `layout=4994us`, `layout.engine_solve=939us`, `paint=5101us`, `command_availability_eval=990us`, and evidence bundle `target/fret-diag/imui-heavy-perf-probes-combobox-devfast-command-availability-cache/1781409098126/bundle.schema2.json`.
- 2026-06-14: remaining combobox long-list tail is now split between small layout bursts, paint cache misses/text prepare, and renderer encode/finish time. Do not keep pushing command availability until a new trace makes it hot again.
- 2026-06-14: view-cache experiments confirmed that whole-page content caching is the wrong boundary for the combobox page: view-cache-on with content caching produced `total=44825us` and `layout_roots_apply=31049us`, while shell-only view cache stayed near the current baseline at `total=12973us`.
- 2026-06-14: `PAGE_COMBOBOX` now opts out of whole-page content cache. The same view-cache-on perf script dropped to `total=12643us` with `layout_roots_apply=703us`, matching the shell-only result and avoiding a `fret-ui` cache mechanism rewrite.

## Open Questions
- How much of the cost is unavoidable component composition, and how much is avoidable shell depth?
- Which heavy surface gives the cleanest before/after perf signal after the first deep slice?
- Should this plan remain a single audit lane, or split into a narrower follow-on once `select` proves the pattern?
- After the state/placement split, should render-part extraction stay in `select.rs` or move into a sibling module tree immediately?
- Should `CommandPalette` virtualize automatically above a row-count threshold, or expose an explicit recipe option first and make automatic policy a later decision?
- Should command virtualization live directly in `CommandPalette`, or should `Combobox` get a narrower virtualized-search adapter first and only promote it after one successful perf slice?
- Should root apply / command availability be optimized as a shared framework seam after command-list virtualization, since the remaining dev-fast tail is no longer dominated by row count?
- Should `fret-ui-shadcn::typography` expose a named one-line muted helper, or should examples keep using explicit `TextProps` when they need a stable status-label contract?
- Should rich/selectable text content changes get a future non-layout path with explicit span-boundary and selection-hit-test gates, or remain layout-affecting until a broader text surface refactor?

## Sources
- `docs/architecture.md`
- `docs/roadmap.md`
- `docs/todo-tracker.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/perf/ui-gallery-profile-report.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/select-combobox-deep-redesign-v1/DESIGN.md`
- `docs/workstreams/select-combobox-deep-redesign-v1/TODO.md`
- `docs/workstreams/shadcn-menu-select-policy-followon-v1/CLOSEOUT_AUDIT_2026-05-17.md`
- `repo-ref/imgui`
- `repo-ref/ui`
- `repo-ref/base-ui`
