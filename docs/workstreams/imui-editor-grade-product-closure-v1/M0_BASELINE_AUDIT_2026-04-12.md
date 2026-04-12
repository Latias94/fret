# M0 Baseline Audit - 2026-04-12

Status: baseline audit

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

## Assumptions-first read

### 1) The runtime-vs-policy split is already frozen enough to guide this lane

- Evidence:
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `docs/architecture.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would accidentally reopen runtime-surface design instead of closing maturity gaps
    above the runtime.

### 2) The `imui` stack ownership reset is closed and should stay closed by default

- Evidence:
  - `docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`
  - `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would blur into another generic stack rewrite instead of a focused follow-on.

### 3) The remaining gap relative to Dear ImGui is closure, not primitive scarcity

- Evidence:
  - `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-editor/src/imui.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - we would incorrectly optimize for more helper growth instead of fixing the developer-facing
    system shape.

### 4) Diagnostics infrastructure already exists; the missing step is productization

- Evidence:
  - `docs/ui-diagnostics-and-scripted-tests.md`
  - `docs/workstreams/diag-fearless-refactor-v2/README.md`
  - `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would overstate current tooling readiness and under-budget foundational diagnostics
    work.

### 5) Multi-window parity remains a runner/backend closure problem

- Evidence:
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the repo might try to patch platform hand-feel through `imui` helpers or runtime leakage.

## Audited evidence

### 1) The old `imui` lanes already closed the stack and helper-closure questions

The existing closeout records already answer the questions that normally justify a broad immediate
mode lane:

- who owns the minimal shared contract,
- who owns generic immediate vocabulary,
- who owns editor composites,
- which retained compatibility surfaces were explicitly kept or deleted.

This matters because the new work should not reopen those questions casually.

### 2) The remaining gaps already appear in four real proof families

The surviving maturity pressure is visible today in four distinct proof families:

- immediate/editor proof surfaces (`imui_editor_proof_demo`, `fret-ui-kit::imui`,
  `fret-ui-editor::imui`),
- workspace/editor shell proof surfaces (`workspace_shell_demo`, `editor_notes_demo`),
- diagnostics/devtools surfaces (bundles, scripts, `fret-devtools`, CLI/MCP docs),
- and docking multi-window parity docs and scripts.

The problem is not absence. The problem is lack of one coherent first-open path across them.

### 3) Reopening closed `imui` helper lanes would miss the actual problem

The editor-grade helper closure lane already concluded that the surviving backlog belongs to recipe,
shell, or platform layers rather than more generic `imui` helpers.

That finding still holds:

- authoring friction should be solved by a clearer golden path and better proof selection,
- shell gaps should be solved in workspace/docking/editor lanes,
- diagnostics gaps should be solved in tooling/devtools lanes,
- and multi-window hand-feel should stay runner-owned.

### 4) A new follow-on is justified, but it should stay narrow in purpose

The right next step is not "another stack reset." The right next step is a follow-on that:

- freezes the maturity-gap taxonomy,
- names owners,
- defines the phase order,
- and identifies the proof/gate package for each phase.

Implementation-heavy work should then split into narrower follow-ons once the owner split is
stable.

## Baseline verdict

Create a new active follow-on lane for editor-grade product closure.

Keep the old `imui` stack and helper lanes closed as evidence.

Use this new lane to drive four ordered phases:

1. P0 - default authoring lane closure,
2. P1 - editor shell/workbench closure,
3. P2 - diagnostics/devtools unification,
4. P3 - multi-window hand-feel closure.

## Immediate execution consequence

From this point forward:

1. treat `imui-stack-fearless-refactor-v2` as closed evidence, not an active execution surface,
2. treat `imui-editor-grade-surface-closure-v1` as helper-closure evidence, not a generic backlog,
3. keep `diag-fearless-refactor-v2` and `docking-multiwindow-imgui-parity` as active reference
   lanes for P2 and P3,
4. keep `crates/fret-ui` contract widening out of this lane unless ADR-backed evidence shows the
   runtime itself is the blocker,
5. start new narrow follow-ons once P0 or P1 turns from taxonomy into concrete implementation
   work.
