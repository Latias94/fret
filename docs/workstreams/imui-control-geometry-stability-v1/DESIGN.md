# ImUi Control Geometry Stability v1

Status: active execution lane
Last updated: 2026-04-28

## Why this lane exists

`imui-text-control-chrome-stability-v1` closed the immediate bug where compact IMUI text inputs
appeared to grow when focused because they were still borrowing shadcn input recipe chrome. That
fix is intentionally narrow. It proves the policy boundary, but it does not yet give Fret one
lane-owned invariant for every base IMUI control.

This lane owns that invariant:

> Base IMUI controls must keep stable outer geometry across visual/interaction state changes.

The target is not "all controls look identical". The target is that hover, focus, pressed,
active/open, changed, and disabled styling must not change the layout footprint of a control in a
compact editor surface.

## Scope

In scope:

- `ecosystem/fret-ui-kit::imui` base controls:
  - button
  - checkbox / radio / switch
  - slider
  - combo trigger
  - selectable
  - input text / textarea inherited floor
  - menu / submenu trigger
  - tab trigger
- local unit/source-policy tests that can run on Windows without Linux compositor acceptance
- private fearless refactors that delete duplicated or stale state-specific chrome paths
- documentation updates to keep the old control-chrome and text-control closeouts closed

Out of scope:

- Linux Wayland compositor acceptance
- docking / OS-window tear-off hand-feel
- shadcn recipe focus-ring parity
- public `fret-imui` API widening
- `crates/fret-ui` runtime contract widening unless the audit proves an actual mechanism gap
- editor-grade tab overflow/reorder/close policy, which remains workspace-owned

## Owner Split

`ecosystem/fret-ui-kit::imui` owns compact immediate control chrome and helper policy.

`ecosystem/fret-imui` owns runnable IMUI authoring tests and proof surfaces that can verify control
bounds/state behavior through the existing immediate facade.

`apps/fret-examples` owns presentable demos and source-policy checks when a proof needs real demo
structure.

`crates/fret-ui` stays out of scope unless a focused repro proves the layout/runtime mechanism
cannot represent a stable control.

## Design Principles

1. State styles may change paint, not layout footprint.
2. Focus indication for IMUI controls should be inset/border-color/stable-chrome unless a specific
   control proves it needs reserved layout space.
3. Compact editor surfaces are the default proof target; controls must not require demo-local width
   workarounds to hide jumps.
4. Duplicated state-specific widget code should be deleted or routed through a shared geometry
   helper when the helper removes real drift.
5. Existing closed lanes stay closed. This lane references their evidence and carries the new
   cross-control invariant forward.

## Starting Assumptions

- Area: lane ownership
  - Assumption: this is a new narrow follow-on, not a continuation of
    `imui-control-chrome-fearless-refactor-v1`.
  - Evidence: `imui-control-chrome-fearless-refactor-v1/WORKSTREAM.json` is closed and says future
    pressure should start a narrower follow-on.
  - Confidence: Confident
  - Consequence if wrong: implementation would be recorded into a closed historical lane.

- Area: text controls
  - Assumption: text input and textarea are the inherited floor, not the new implementation target.
  - Evidence: `imui-text-control-chrome-stability-v1/CLOSEOUT_AUDIT_2026-04-28.md`.
  - Confidence: Confident
  - Consequence if wrong: this lane might duplicate already-closed text-control work.

- Area: Linux
  - Assumption: Linux compositor acceptance is not a blocker for this lane.
  - Evidence: user constraint for this turn and `docking-multiwindow-imgui-parity` already owning
    the active Wayland/runner acceptance path.
  - Confidence: Confident
  - Consequence if wrong: this lane's gate set would under-prove OS-window behavior, which is not
    its stated scope.

- Area: runtime mechanism
  - Assumption: most remaining geometry drift, if any, should be fixable in `fret-ui-kit::imui`
    chrome/options rather than `crates/fret-ui`.
  - Evidence: the text-control fix was a policy-layer chrome correction, and ADR 0066 keeps
    interaction policy out of `fret-ui`.
  - Confidence: Likely
  - Consequence if wrong: a focused repro must promote the issue into an ADR-backed runtime lane.

## Exit Criteria

- Every admitted base-control family has a focused test or source-policy gate proving state changes
  do not alter its outer geometry.
- Any unstable control found by the audit is refactored instead of papered over in demos.
- The final closeout names the controls audited, the controls changed, and any deferred control
  families with explicit owner rationale.
