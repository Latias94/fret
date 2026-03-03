# Workspace TabStrip (Fearless Refactor v1) — TODO

Status: Active
Last updated: 2026-03-03

Related:

- Design: `docs/workstreams/workspace-tabstrip-fearless-refactor-v1/DESIGN.md`
- Zed parity checklist: `docs/workstreams/workspace-tabstrip-fearless-refactor-v1/ZED_PARITY_CHECKLIST.md`
- Milestones:
  - `docs/workstreams/workspace-tabstrip-fearless-refactor-v1/M1_FOUNDATION.md`
  - `docs/workstreams/workspace-tabstrip-fearless-refactor-v1/M2_DRAG_AND_DROP.md`
  - `docs/workstreams/workspace-tabstrip-fearless-refactor-v1/M3_EDITOR_SEMANTICS.md`
- Evidence/gates: `docs/workstreams/workspace-tabstrip-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- Open questions: `docs/workstreams/workspace-tabstrip-fearless-refactor-v1/OPEN_QUESTIONS.md`

Tracking legend:

- `[ ]` open
- `[~]` in progress
- `[x]` done
- `[!]` blocked

ID format:

- `WTS-{area}-{nnn}`

---

## A. Contract + Boundaries

- [x] WTS-contract-001 Write down the Workspace TabStrip contract surface:
  - Surface classification vocabulary (header space vs tabs viewport vs controls).
  - Hit targets (tab content vs close, overflow menu row content vs close).
  - Insert index semantics (canonical order, under overflow).
- [x] WTS-contract-002 Decide how Workspace TabStrip relates to Docking TabBar:
  - Shared primitives in `ecosystem/fret-ui-headless` (surface classification, overflow membership,
    click arbitration, midpoint drop target resolution).
  - Adapter-specific policy remains separate (workspace pinned/preview, docking float/tear-off).
- [x] WTS-contract-003 Minimum stable diagnostic anchors exist:
  - end-drop: `{root}.drop_end`
  - overflow button: `{root}.overflow_button`
  - overflow entry + close: `{root}.overflow_entry.{tab_id}[.close]`
  - per-tab dirty marker: `{tab_test_id}.dirty`
  - pinned boundary: `{root}.drop_pinned_boundary`

## B. Kernelization (Mechanism vs Policy)

- [x] WTS-kernel-010 Workspace TabStrip uses `fret-ui-headless` for:
  - surface classification (`TabStripSurface`),
  - midpoint drop target kernel (`compute_tab_strip_drop_target_midpoint`),
  - overflow membership / geometry helpers (if needed).
- [x] WTS-kernel-011 Click arbitration policy lives in `ecosystem/fret-ui-kit`:
  - Evidence: `ecosystem/fret-ui-headless/src/tab_strip_controller.rs` (re-exported via `ecosystem/fret-ui-kit/src/headless/tab_strip_controller.rs`).
- [x] WTS-kernel-012 Workspace-specific kernel exists:
  - mapping pointer → hit target,
  - mapping hit target → insert index (including end-drop),
  - mapping tab rects → drag preview banding.

## C. Diagnostics + Gates

- [x] WTS-gates-020 Drop-at-end gates exist (via `test_id` + semantics predicates):
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-first-to-end-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-first-to-end-overflow-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-cross-pane-move-to-end.json`
- [x] WTS-gates-021 Overflow activation and "close does not activate" gates exist:
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-overflow-activate-hidden-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-overflow-close-does-not-activate.json`
- [x] WTS-gates-022 Nextest coverage exists in:
  - `ecosystem/fret-workspace/src/tab_strip/kernel.rs`
  - `ecosystem/fret-workspace/src/tab_strip/utils.rs`
  - `ecosystem/fret-workspace/src/tab_strip/overflow.rs`

## D. Editor Semantics (Policy Layer)

- [x] WTS-editor-030 Pinned tabs (policy) in workspace layer:
  - pinned region model, reorder rules, close affordances.
  - Diag gates:
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-pinned-boundary-toggle-smoke.json`
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-within-pinned-smoke.json`
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-pinned-cross-boundary-drop-does-not-pin-smoke.json`
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-pin-commits-preview-smoke.json`
- [x] WTS-editor-031 Preview tab slot (Zed-style):
  - activate/commit rules, replacement rules.
  - Diag gates:
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-preview-replaces-existing-smoke.json`
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-preview-commit-keeps-old-tab-smoke.json`
- [x] WTS-editor-033 Bulk-close commands keep pinned tabs:
  - Evidence: `ecosystem/fret-workspace/src/tabs.rs` (`close_left_of_active`, `close_right_of_active`, `close_others`)
  - Diag gates:
    - [x] `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-left-keeps-pinned-smoke.json`
    - [x] `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-others-keeps-pinned-smoke.json`
    - [x] `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-right-keeps-pinned-smoke.json`
- [x] WTS-editor-032 Dirty close confirmation hooks (workspace-level, not tab mechanism).
  - Evidence: `ecosystem/fret-workspace/src/close_policy.rs` and `ecosystem/fret-workspace/src/tabs.rs` (`apply_command_with_close_policy`)
  - Diag gates:
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-dirty-is-blocked-smoke.json`
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-dirty-shows-prompt-and-discard-closes-smoke.json`
- [x] WTS-editor-034 Tabstrip focus restore on close (editor-grade):
  - Evidence: `ecosystem/fret-workspace/src/tab_strip/utils.rs` (`predict_next_active_tab_after_close`) and `ecosystem/fret-workspace/src/tab_strip/mod.rs` (close-focus hook)
  - Diag gate: `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tabstrip-close-keeps-focus-smoke.json`

## E. Cleanup + Convergence

- [x] WTS-cleanup-040 Remove legacy ad-hoc hit-tests once kernel is in use:
  - Evidence: `ecosystem/fret-workspace/src/tab_strip/mod.rs` (cross-pane hover insertion now uses `compute_workspace_tab_strip_drop_target`)
- [ ] WTS-cleanup-041 Converge styling recipes (shadcn/material) without affecting mechanism tests.
- [x] WTS-converge-050 Docking TabBar drop excludes the dragged tab from midpoint candidates:
  - Evidence: `ecosystem/fret-docking/src/dock/space.rs` (passes dragged tab index to the tab-bar kernel)
  - Coverage: `ecosystem/fret-docking/src/dock/tab_bar_kernel.rs` (`resolve_tab_bar_drop_excludes_dragged_tab_from_candidates`)

## F. Parity tracking

- [x] WTS-parity-060 Add a Zed parity checklist with evidence anchors:
  - `docs/workstreams/workspace-tabstrip-fearless-refactor-v1/ZED_PARITY_CHECKLIST.md`

## G. Keyboard + A11y

- [x] WTS-a11y-070 Tabstrip keyboard roving and Escape gates exist:
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tabstrip-keyboard-roving-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tabstrip-escape-restores-content-focus-smoke.json`
