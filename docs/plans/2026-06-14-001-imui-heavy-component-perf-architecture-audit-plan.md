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

## Open Questions
- How much of the cost is unavoidable component composition, and how much is avoidable shell depth?
- Which heavy surface gives the cleanest before/after perf signal after the first deep slice?
- Should this plan remain a single audit lane, or split into a narrower follow-on once `select` proves the pattern?
- After the state/placement split, should render-part extraction stay in `select.rs` or move into a sibling module tree immediately?

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
