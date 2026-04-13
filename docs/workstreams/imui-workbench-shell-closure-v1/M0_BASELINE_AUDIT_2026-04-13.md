# M0 Baseline Audit - 2026-04-13

Status: active baseline audit

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/WORKSPACE_SHELL_STARTER_SET.md`

## Assumptions-first baseline

### 1) This lane is a real follow-on, not a duplicate umbrella note

- Area: lane state
- Assumption: the active immediate-mode umbrella already froze the P1 proof order and the promoted
  shell diagnostics floor, so implementation-heavy shell work now belongs in a narrower lane.
- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md`
- Confidence: Confident
- Consequence if wrong: this folder would just restate umbrella strategy instead of narrowing the
  next executable P1 slice.

### 2) The current gap is not missing generic immediate-mode mechanics

- Area: owner split
- Assumption: focused shortcut seams and other recent `imui` parity work are no longer the primary
  blocker for the default editor workbench path.
- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence: Confident
- Consequence if wrong: this lane would drift back into generic `imui` helper growth.

### 3) `workspace_shell_demo` is still the broadest P1 product proof, even though it is not the only shell reference

- Area: proof roster
- Assumption: for the immediate-mode product-closure question, `workspace_shell_demo` remains the
  broadest first-open shell proof because it combines pane tree composition, shell command scope,
  dirty-close behavior, file-tree liveness, and shell-mounted editor rails in one surface.
- Evidence:
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `tools/diag-scripts/suites/diag-hardening-smoke-workspace/suite.json`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- Confidence: Confident
- Consequence if wrong: this lane would start from a proof surface that is too narrow to expose the
  real shell gap.

### 4) There is a useful distinction between shell-chrome baseline and P1 product proof

- Area: reference interpretation
- Assumption: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/WORKSPACE_SHELL_STARTER_SET.md`
  is still valid as a shell starter-set boundary note, but it does not replace the immediate-mode
  umbrella's decision to use `workspace_shell_demo` as the main P1 product proof.
- Evidence:
  - `docs/workstreams/editor-ecosystem-fearless-refactor-v1/WORKSPACE_SHELL_STARTER_SET.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- Confidence: Likely
- Consequence if wrong: the lane could confuse a reusable shell-chrome starter-set inventory with
  the broader workbench product-closure proof order.

### 5) The smallest current P1 gap is shell assembly posture, not another tabstrip behavior slice

- Area: next executable slice
- Assumption: the remaining product friction is that the default workbench answer is still largely
  assembled in app-local proof code, not that tabstrip/kernel behavior lacks coverage.
- Evidence:
  - `apps/fret-examples/src/workspace_shell_demo.rs`
    - app-local composition currently wires `WorkspaceFrame`, shell-mounted rails,
      `workspace_pane_tree_element_with_resize`, and `WorkspaceCommandScope` together directly
    - shell-level affordances such as dirty-close prompt and file-tree liveness are also owned in
      the same demo surface
  - `apps/fret-examples/src/editor_notes_demo.rs`
    - app-local composition proves `WorkspaceFrame.left/right(...)` rails, but does not cover
      pane tree or shell command scope
  - `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`
  - `docs/workstreams/workspace-shell-tabstrip-fearless-refactor-v1/DESIGN.md`
- Confidence: Likely
- Consequence if wrong: the lane would spend time on shell assembly/docs while a deeper missing
  behavioral primitive remains unresolved elsewhere.

## Findings

### 1) The broadest current workbench proof is still example-local assembly

`workspace_shell_demo` is valuable because it already demonstrates the whole P1 posture in one
surface:

- shell frame,
- left and right shell slots,
- pane tree with resize,
- shell command scope,
- dirty-close behavior,
- file-tree liveness,
- and shell-mounted editor rails.

But that same strength exposes the current maturity gap:

- the default workbench answer is still assembled directly inside the demo,
- and the repo does not yet leave behind one narrower executable note that says which part of that
  assembly should remain app-local versus which part is the durable first-party shell baseline.

### 2) `editor_notes_demo` is a second consumer, but not the full-shell answer

`editor_notes_demo` proves that `WorkspaceFrame.left/right(...)` is a real reusable shell seam for
editor rails.

It intentionally does not prove:

- pane-tree composition,
- shell command routing,
- dirty-close shell policy,
- or tabstrip/pane behavior.

That means it is important as a second consumer, but it cannot close P1 by itself.

### 3) The promoted launched shell suite already proves the right minimum behavior floor

`diag-hardening-smoke-workspace` already locks:

- tab close / reorder / split preview,
- dirty-close prompt,
- Escape focus restore,
- file-tree keep-alive.

So the next P1 question is not "which behavior should we gate?"
The next question is "which shell assembly rule should those gates be protecting?"

### 4) The tabstrip and multi-window lanes should remain references, not default owners here

The current evidence still supports this split:

- tabstrip parity and kernel behavior remain in the existing `workspace-tabstrip*` lanes,
- runner/backend multi-window hand-feel remains in docking parity,
- this lane should only own the default workbench shell closure question that sits above them.

## M0 verdict

M0 can be treated as closed for this lane.

The first executable P1 problem is now narrow enough:

- keep `workspace_shell_demo` as the primary product proof,
- keep `editor_notes_demo` as the second shell-mounted rail consumer,
- keep the promoted launched shell suite as the minimum shell behavior floor,
- and treat the remaining gap as a shell-assembly / first-party default-path question above the
  existing tabstrip and runner parity lanes.

## Immediate execution consequence

From this point forward:

1. do not reopen generic `imui` helper growth from this lane,
2. do not reopen tabstrip kernel parity here unless a concrete P1 issue reduces fully to tabstrip
   behavior,
3. do not reopen runner/backend multi-window closure here,
4. use this lane to decide the next bounded shell-assembly slice under one owner,
5. and make that next slice land with one source gate, one launched gate, and one evidence note.

## Recommended first M1 candidate

The strongest first candidate is:

- freeze whether the current default workbench answer should remain "example-local assembly over the
  frozen starter set" or whether one thinner first-party shell recipe/helper is now warranted.

Why this candidate first:

- it is directly implied by the current proof surfaces,
- it does not reopen tabstrip or runner work,
- and it will determine whether the next code slice belongs in `fret-workspace` or remains a
  documentation/proof-surface tightening pass.
