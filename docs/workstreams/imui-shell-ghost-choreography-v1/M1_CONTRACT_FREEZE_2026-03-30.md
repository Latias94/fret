# M1 Contract Freeze — 2026-03-30

Status: accepted v1 decision

Related:

- `docs/workstreams/imui-shell-ghost-choreography-v1/DESIGN.md`
- `docs/workstreams/imui-shell-ghost-choreography-v1/TODO.md`
- `docs/workstreams/imui-shell-ghost-choreography-v1/MILESTONES.md`
- `docs/workstreams/imui-cross-window-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`
- `docs/workstreams/docking-hovered-window-contract-v1/docking-hovered-window-contract-v1.md`
- `docs/workstreams/docking-multiviewport-arbitration-v1/docking-multiviewport-arbitration-v1.md`
- `apps/fret-examples/src/docking_arbitration_demo.rs`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-under-moving-window-basic.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json`
- `crates/fret-runtime/src/drag.rs`
- `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`
- `ecosystem/fret-docking/src/dock/space.rs`
- `ecosystem/fret-workspace/src/tab_strip/mod.rs`

## Purpose

This note closes the Phase B question set for `imui-shell-ghost-choreography-v1`.

The goal is to freeze the smallest correct shell-aware contract before implementation drift starts:

- which layer owns the first shell-specific choreography slice,
- which proof surface is actually capable of exercising the problem,
- how transient no-hover gaps should behave,
- whether a new shared controller is justified,
- and what minimum diagnostics/gate package must exist before code is considered reviewable.

## Frozen decisions

### 1) The first landed shell-aware slice belongs to docking-aware layers, not to generic recipes

Decision:

- keep `ecosystem/fret-ui-kit::recipes` limited to the already-shipped generic cross-window
  transfer baseline,
- keep `fret-ui-kit::imui` observational only,
- let `ecosystem/fret-docking` own the first landed shell-aware choreography slice,
- and keep runner-owned hover selection / moving-window truth in `crates/fret-launch` and
  `crates/fret-runtime` unless proof shows a real mechanism gap.

This is intentionally not a workspace-first change.

Why:

- the runner already publishes `moving_window` and `window_under_moving_window`,
- docking diagnostics already capture and gate those fields,
- and workspace tab-strip behavior currently consumes only `current_window`, which is not enough to
  prove shell-specific moving-window choreography.

### 2) The first proof must start in the docking arbitration demo, not in the generic main/aux proof

Decision:

- use `apps/fret-examples/src/docking_arbitration_demo.rs` as the first proof surface,
- and treat the generic `imui_editor_proof_demo` main/aux pair as insufficient for this lane.

Why:

- the shell-aware question is not ordinary cross-window transfer,
- it is the interaction between `current_window`, `moving_window`, and
  `window_under_moving_window`,
- and the docking arbitration demo already exposes the relevant tear-off and overlap anchors such
  as `dock-arb-tab-drag-anchor-right`.

The generic main/aux proof remains valid evidence for the generic transfer baseline, but it cannot
answer the shell-aware choreography question.

### 3) Transient no-hover gaps remain runner-owned; shell layers must not invent a second hover timer

Decision:

- shell layers must continue to treat runner-owned `current_window` as the primary hover/drop
  truth,
- runner-owned latching/fallback behavior remains responsible for transient no-hover gaps,
- and shell layers must not add their own time-based grace period or independent hover heuristic.

Operational rule:

- when shell-aware follow/transparent-payload behavior is inactive, choreography follows
  `current_window` directly,
- when shell-aware follow/transparent-payload behavior is active and the runner reports
  `window_under_moving_window`, docking may use that as a shell input for choreography,
- but the shell still consumes runner-owned state rather than inventing a parallel hover model.

This keeps the owner split clean:

- runner owns hover selection truth,
- docking owns shell policy around that truth.

### 4) No new cross-shell controller is justified yet

Decision:

- do not add a new `fret-ui-kit` or runtime-level shell ghost controller in the first slice,
- do not widen the generic recipe API with docking/workspace flags,
- and keep the first shell-aware orchestration local to docking-aware layers.

Allowed first-slice shapes:

- a docking-local wrapper around the generic cross-window transfer helper,
- a docking-local suppression/remap policy when the moving window itself is already the visible
  payload,
- or another narrow docking-owned orchestration layer proven by the first proof surface.

What is explicitly rejected for M1:

- a new shared controller before a second shell family proves reuse,
- or a generic `recipes` API that absorbs moving-window policy just to make first-party docking
  easier.

### 5) The minimum regression package is diag-first, not screenshot-first

Decision:

- the minimum proof/gate package for this lane starts from existing docking diagnostics and scripted
  repros,
- the smallest gate is
  `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-under-moving-window-basic.json`,
- and the stronger overlap/transparent-payload follow-on gate is
  `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json`.

Minimum assertions that must stay observable:

- `dock_drag_current_window_is`
- `dock_drag_moving_window_is`
- `dock_drag_window_under_moving_window_is`
- `dock_drag_window_under_cursor_source_is`

If the first implementation adds non-trivial docking-local policy code, add focused unit coverage
in `ecosystem/fret-docking` as a secondary gate.
Screenshots are optional and should be added only if visible chrome semantics become the actual
review risk.

## Rejected alternatives

### Workspace-first choreography

Rejected because:

- current workspace tab-strip logic only proves `current_window`-scoped behavior,
- it does not exercise moving-window or under-moving-window semantics,
- and starting there would hide the real owner split behind a weaker proof surface.

### Generic recipe flags for moving-window behavior

Rejected because:

- moving-window choreography is shell-specific,
- the generic recipe lane just closed with a narrower owner split,
- and widening it immediately would reopen the wrong contract.

### Shell-local hover timers or heuristics

Rejected because:

- the runner already owns transient out-of-window fallback/latching behavior,
- duplicating that policy in shells would create two sources of truth,
- and diagnostics would become harder to reason about.

### A new shared controller before proof

Rejected for v1 because:

- there is not yet a second shell family proving reuse,
- and the first slice can be evaluated with docking-local orchestration over existing runtime
  signals.

## Immediate consequences

From this point forward:

1. treat `ecosystem/fret-docking` as the first owner for shell-aware ghost choreography,
2. treat `apps/fret-examples/src/docking_arbitration_demo.rs` as the first proof surface,
3. treat `current_window` as primary hover truth and `window_under_moving_window` as a docking
   input, not a generic recipe rule,
4. do not add a new cross-shell controller until a second shell family proves reuse,
5. use diag-first regression protection before adding screenshot-first or runtime-wide surfaces.

## What Phase C now needs to prove

The next proof/implementation slice should verify:

- whether docking should suppress, remap, or reuse the generic ghost while a real moving window is
  already visible,
- how the moving window and the final hovered/target window share ownership without duplicates,
- whether any docking-local wrapper around the generic transfer helper is actually enough,
- and whether any real mechanism gap remains after docking-local orchestration is attempted first.
