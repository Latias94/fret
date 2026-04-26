# M0 Baseline Audit - 2026-04-13

Status: historical baseline audit; mixed-DPI open-blocker state superseded by
`M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`

Status note (2026-04-25): this baseline still explains why the docking lane owned the P3
multi-window work, but references below to `DW-P0-dpi-006` as the current open blocker are historical.
The real-host mixed-DPI acceptance pair is now recorded in
`M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`.

Related:

- `WORKSTREAM.json`
- `docking-multiwindow-imgui-parity.md`
- `docking-multiwindow-imgui-parity-todo.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
- `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`

## Assumptions-first baseline

### 1) This lane is the current active execution lane for the remaining P3 multi-window hand-feel problem

- Area: lane state
- Assumption: after the P1 shell closeout, the next implementation-heavy work should continue in the
  existing docking parity lane rather than creating another umbrella or reopening a closed shell lane.
- Evidence:
  - `docs/roadmap.md`
  - `docs/todo-tracker.md`
  - `docs/workstreams/README.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/MILESTONES.md`
- Confidence: Confident
- Consequence if wrong: we would split the next runner/backend slice across redundant notes and lose
  a single execution owner.

### 2) The owner split is already frozen: keep P3 in runner/backend and docking policy, not in `crates/fret-ui`

- Area: source policy
- Assumption: the remaining parity work still belongs by default in `crates/fret-launch`,
  runner/backend integrations, and `ecosystem/fret-docking`.
- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence: Confident
- Consequence if wrong: we would reopen generic `imui` helper growth or widen `crates/fret-ui`
  to compensate for platform behavior gaps.

### 3) The bounded P3 package already exists and should stay the default regression envelope for this lane

- Area: gate posture
- Assumption: this lane should resume from the existing campaign and source-policy notes instead of
  inventing a second package shape.
- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
  - `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
  - `apps/fret-examples/src/lib.rs`
- Confidence: Confident
- Consequence if wrong: we would create competing P3 gate entry points and make runner regressions
  harder to track.

### 4) `DW-P0-dpi-006` is the smallest real open blocker in this lane

- Area: next execution slice
- Assumption: mixed-DPI follow-drag closure is the right default next step because it is the only
  remaining P0 TODO that is still marked in progress after the recent docking fixes and diag hardening.
- Evidence:
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- Confidence: Likely
- Consequence if wrong: the lane would drift into lower-priority platform cleanup while its last
  P0 hand-feel blocker stayed unresolved.

### 5) Windows placement and Wayland degradation remain open, but they are follow-up slices after mixed-DPI posture is explicit

- Area: backlog ordering
- Assumption: `DW-P1-win-002` and `DW-P1-linux-003` should stay open references, not replace the
  current first slot.
- Evidence:
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- Confidence: Likely
- Consequence if wrong: the lane would lose a bounded next slice and reopen a wider platform matrix
  too early.

## Findings

### 1) The lane was active, but its first-open state was still implicit

Before this audit, the lane had a narrative overview and a task tracker, but no first-open
state index and no dated current-status note. That made the correct resume order obvious only to
someone who already knew the recent umbrella P3 decisions.

### 2) The source-policy split and bounded package are already settled elsewhere

The umbrella P3 notes already freeze:

- runner/backend ownership for hovered-window, peek-behind, transparent payload, and mixed-DPI,
- the rejection of `crates/fret-ui` widening for runner gaps,
- and the bounded campaign entry over `docking_arbitration_demo`.

This lane should reference those decisions directly instead of re-arguing them in ad hoc status
messages.

### 3) `DW-P0-dpi-006` is the current execution bottleneck

The `docking-multiwindow-imgui-parity-todo.md` tracker already shows that the earlier user-visible
P0 items are closed, while `DW-P0-dpi-006` still has two real open questions:

- capture the real-host mixed-DPI acceptance pair,
- and decide whether mixed-DPI environment detection is reliable enough for an automated gate.

That makes it the best default next slice for this lane.

### 4) The launched proof surface is already stable enough to resume from immediately

This lane does not need a new demo or a new broad suite:

- `cargo run -p fret-demo --bin docking_arbitration_demo`
- `cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json`
- `cargo run -p fretboard-dev -- diag campaign run imui-p3-multiwindow-parity --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release`

already provide the small reopen surface and the bounded launched gate package.

## M0 verdict

M0 can be treated as closed for this lane.

The current execution posture is now explicit:

- this lane is the active owner for the remaining P3 multi-window hand-feel work,
- it resumes from the existing bounded P3 package rather than a new umbrella,
- it keeps runner/backend and docking policy as the default owners,
- and it treats `DW-P0-dpi-006` as the next bounded execution slice.

## Immediate execution consequence

From this point forward:

1. open `WORKSTREAM.json` first, then this baseline audit, then the TODO tracker,
2. use `docking_arbitration_demo` as the first runnable proof surface,
3. use `tools/diag-campaigns/imui-p3-multiwindow-parity.json` as the bounded P3 regression entry,
4. do not reopen generic `imui` helper growth or widen `crates/fret-ui` for runner/backend gaps,
5. keep `DW-P0-dpi-006` as the default next slice until the real-host acceptance pair and
   automation posture are explicit.

## Recommended next slice

The next landable slice should close the remaining `DW-P0-dpi-006` evidence gap:

- capture one real mixed-DPI acceptance pair with pre-crossing and post-crossing bundles,
- record whether the host can be identified as genuinely mixed-DPI without fragile heuristics,
- and then either promote an automated gate or freeze the reason it remains manual.

After that, keep `DW-P1-win-002` and `DW-P1-linux-003` as the next follow-up slices rather than
reopening broader `imui` or runtime scope.
