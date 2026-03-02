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

- [ ] WTS-contract-001 Write down the Workspace TabStrip contract surface:
  - Surface classification vocabulary (header space vs tabs viewport vs controls).
  - Hit targets (tab content vs close, overflow menu row content vs close).
  - Insert index semantics (canonical order, under overflow).
- [ ] WTS-contract-002 Decide how Workspace TabStrip relates to Docking TabBar:
  - share a single kernel,
  - share only math helpers, or
  - keep fully separate.
- [ ] WTS-contract-003 Decide the minimum stable diagnostic anchors needed for scripted gates.

## B. Kernelization (Mechanism vs Policy)

- [ ] WTS-kernel-010 Ensure Workspace TabStrip uses `fret-ui-headless` for:
  - surface classification (`TabStripSurface`),
  - overflow membership / geometry helpers (if needed).
- [ ] WTS-kernel-011 Keep click arbitration policy in `ecosystem/fret-ui-kit`:
  - Evidence: `ecosystem/fret-ui-kit/src/headless/tab_strip_controller.rs`.
- [ ] WTS-kernel-012 Add a small workspace-specific "tab strip kernel" module for:
  - mapping pointer → hit target,
  - mapping hit target → insert index (including end-drop),
  - mapping tab rects → drag preview banding.

## C. Diagnostics + Gates

- [ ] WTS-gates-020 Add a diag script gate:
  - drop-at-end resolves `insert_index == tab_count` for workspace tabs.
- [ ] WTS-gates-021 Add a diag script gate:
  - active tab stays visible (no hidden-active regression under overflow).
- [ ] WTS-gates-022 Add at least 2 nextest tests that validate:
  - end-drop insert index (no overflow + overflow),
  - reorder in-place maintains canonical ordering invariants.

## D. Editor Semantics (Policy Layer)

- [ ] WTS-editor-030 Pinned tabs (policy) in workspace layer:
  - pinned region model, reorder rules, close affordances.
- [ ] WTS-editor-031 Preview tab slot (Zed-style):
  - activate/commit rules, replacement rules.
- [ ] WTS-editor-032 Dirty close confirmation hooks (workspace-level, not tab mechanism).

## E. Cleanup + Convergence

- [ ] WTS-cleanup-040 Remove legacy ad-hoc hit-tests once kernel is in use.
- [ ] WTS-cleanup-041 Converge styling recipes (shadcn/material) without affecting mechanism tests.
