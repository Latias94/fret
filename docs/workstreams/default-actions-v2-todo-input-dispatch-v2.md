# Default Actions v2 — Mechanism vs Policy Boundary (worktree-local)

Status: Active (input-dispatch-v2 worktree-local tracker to avoid doc ownership conflicts)

This document exists to resolve `IDV2-def-006/007` without repeatedly reshaping core contracts.

## Context

In Input Dispatch v2, `prevent_default(DefaultAction)` must remain **orthogonal** to propagation
(`stop_propagation`) and to overlay/input policy (pointer occlusion, outside-press).

- Contract: `docs/adr/1157-input-dispatch-phases-prevent-default-and-action-availability-v2.md`
- Workstream: `docs/workstreams/input-dispatch-v2.md`
- Tracker: `docs/workstreams/input-dispatch-v2-todo.md` (`IDV2-def-006/007`)

## Decision Criteria (when a behavior qualifies as a mechanism-owned DefaultAction)

A behavior qualifies as a `DefaultAction` only if all are true:

1) **Cross-component invariant**: users expect the behavior across multiple unrelated widgets.
2) **Design-system agnostic**: the behavior does not depend on shadcn/Radix policy choices.
3) **Stable semantics**: the behavior can be specified precisely (no hidden ordering hacks).
4) **Composable suppression**: suppressing it should not require stopping propagation.
5) **Mechanism-shaped effect**: the action maps cleanly to a runtime “request” (focus/capture/etc),
   not to widget-private state machines.
6) **Testable**: we can lock it with a small conformance test in `crates/*` plus at least one
   ecosystem consumer test (if the action exists primarily to support a specific ecosystem rule).

Anti-criteria (keep out of `DefaultAction`):

- Anything that encodes a design system policy (pressed/toggle semantics, menu focus recipes).
- Anything that is naturally a widget private protocol (text selection/IME editing policies).
- Anything that requires implicit “capture-phase side effects” to work (should live in Capture).

## Current Inventory (v1)

- `DefaultAction::FocusOnPointerDown`
  - Rationale: universal, hard-to-retrofit, editor-grade ergonomic primitive.
  - Evidence: `crates/fret-runtime/src/input.rs`, `crates/fret-ui/src/tree/dispatch.rs`

## Candidate Actions (v2+; do not implement without justification)

| Candidate | Likely Owner | Why / Notes |
| --- | --- | --- |
| `ScrollWheelRouteToUnderlay` | Mechanism (maybe) | Could be a general routing rule, but must not encode “scroll area” policy. Prefer keeping it as routing logic, not a default action. |
| `BeginPointerTextSelection` | Policy | Highly editor/text-widget specific; likely belongs in `fret-ui-kit`/text ecosystem. |
| `PointerCaptureOnPress` | Policy | Press/click semantics vary; prefer widget-private behavior gated by `prevent_default(FocusOnPointerDown)` + explicit capture requests. |
| `RestoreFocusOnClose` | Policy | Overlay-specific; should remain in overlay policy hooks, not a global default action. |

## TODO

- [x] DA2-001 Decide whether any candidate belongs in `DefaultAction` beyond `FocusOnPointerDown`.
  - Decision: keep v2 limited to `FocusOnPointerDown` for now.
  - Rationale: all current candidates either encode design-system policy (Radix/shadcn), are
    widget-private state machines, or are better expressed as routing rules / overlay policy.
- [x] DA2-002 For each accepted `DefaultAction`, add:
  - a mechanism conformance test (near dispatch),
  - at least one ecosystem integration test proving composability.
- [x] DA2-003 Document “why not a DefaultAction” decisions to avoid drift.
  - Decision log: see “Candidate Actions (v2+)” table above.

## Evidence (DA2-002)

- Mechanism conformance:
  - `crates/fret-ui/src/tree/tests/prevent_default.rs` (`prevent_default_focus_on_pointer_down_suppresses_default_focus`)
- Ecosystem integration:
  - `ecosystem/fret-ui-shadcn/tests/default_action_focus_on_pointer_down.rs`
