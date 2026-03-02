# Workspace TabStrip (Fearless Refactor v1) — TODO

Status: Active
Last updated: 2026-03-02

Related:

- Design: `docs/workstreams/workspace-tabstrip-fearless-refactor-v1/DESIGN.md`
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

- [~] WTS-contract-001 Write down the Workspace TabStrip contract surface:
  - Surface classification vocabulary (header space vs tabs viewport vs controls).
  - Hit targets (tab content vs close, overflow menu row content vs close).
  - Insert index semantics (canonical order, under overflow).
- [ ] WTS-contract-002 Decide how Workspace TabStrip relates to Docking TabBar:
  - share a single kernel,
  - share only math helpers, or
  - keep fully separate.
- [x] WTS-contract-003 Minimum stable diagnostic anchors exist:
  - end-drop: `{root}.drop_end`
  - overflow button: `{root}.overflow_button`
  - overflow entry + close: `{root}.overflow_entry.{tab_id}[.close]`
  - pinned boundary: `{root}.drop_pinned_boundary`

## B. Kernelization (Mechanism vs Policy)

- [x] WTS-kernel-010 Workspace TabStrip uses `fret-ui-headless` for:
  - surface classification (`TabStripSurface`),
  - overflow membership / geometry helpers (if needed).
- [x] WTS-kernel-011 Click arbitration policy lives in `ecosystem/fret-ui-kit`:
  - Evidence: `ecosystem/fret-ui-kit/src/headless/tab_strip_controller.rs`.
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

- [ ] WTS-editor-030 Pinned tabs (policy) in workspace layer:
  - pinned region model, reorder rules, close affordances.
  - Diag gates:
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-pinned-boundary-toggle-smoke.json`
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-pinned-cross-boundary-drop-does-not-pin-smoke.json`
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-pin-commits-preview-smoke.json`
- [x] WTS-editor-031 Preview tab slot (Zed-style):
  - activate/commit rules, replacement rules.
  - Diag gates:
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-preview-replaces-existing-smoke.json`
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-preview-commit-keeps-old-tab-smoke.json`
- [~] WTS-editor-033 Bulk-close commands keep pinned tabs:
  - Evidence: `ecosystem/fret-workspace/src/tabs.rs` (`close_left_of_active`, `close_right_of_active`, `close_others`)
  - Diag gates:
    - [x] `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-left-keeps-pinned-smoke.json`
    - [x] `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-others-keeps-pinned-smoke.json`
    - [ ] (TODO) close right keeps pinned
- [ ] WTS-editor-032 Dirty close confirmation hooks (workspace-level, not tab mechanism).

## E. Cleanup + Convergence

- [~] WTS-cleanup-040 Remove legacy ad-hoc hit-tests once kernel is in use.
- [ ] WTS-cleanup-041 Converge styling recipes (shadcn/material) without affecting mechanism tests.
