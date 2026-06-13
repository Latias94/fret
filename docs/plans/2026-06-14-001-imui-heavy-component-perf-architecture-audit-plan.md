---
title: IMUI heavy-component perf and architecture audit
type: audit
date: 2026-06-14
execution: code
---

# IMUI Heavy-Component Perf and Architecture Audit

## Summary
This plan tracks the ongoing effort to push Fret's immediate-mode surfaces toward stable 120Hz behavior under editor-grade composition. The first confirmed slice is `select`; broader menu, overlay, and combobox families remain secondary candidates until the first slice shows whether the dominant cost is shell depth, layout churn, or shared primitive policy.

## Problem Frame
The current IMUI surface is good enough for small demos, but it is not yet consistently boring under heavy composition. The failures seen in `imui_action_basics`, `imui_editor_controls_basics`, and `imui_plot_basics` are not one bug class: they mix stack pressure, layout instability, and contract gaps.

The working question is not "does the UI function at all". The question is whether the current module shapes can sustain dense editor-like usage without visible jank, overflow, or panic-level contract misses.

## Current Findings
- `ecosystem/fret-ui-shadcn/src/select.rs` is the largest recipe surface and currently mixes state transitions, sizing, scroll affordances, positioning, item rendering, and modal/pointer behavior.
- `ecosystem/fret-ui-kit/src/primitives/select.rs` is also substantial, but it is still a mechanism/helper layer rather than a recipe shell.
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

## Open Questions
- How much of the cost is unavoidable component composition, and how much is avoidable shell depth?
- Which heavy surface gives the cleanest before/after perf signal after the first deep slice?
- Should this plan remain a single audit lane, or split into a narrower follow-on once `select` proves the pattern?

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
