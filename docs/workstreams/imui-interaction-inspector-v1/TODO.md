# ImUi Interaction Inspector v1 TODO

Status: closed execution checklist
Last updated: 2026-04-24

## P0 - Freeze Boundary

- [x] Create a narrow workstream rather than reopening closed behavior-kernel lanes.
- [x] Keep `imui_response_signals_demo` as the proof-first contract surface.
- [x] Keep runtime, public `fret-imui`, and public `fret-ui-kit::imui` APIs out of scope.

## P1 - Product Inspector Slice

- [x] Add inspector state and stable `test_id` anchors to `imui_interaction_showcase_demo`.
- [x] Record last meaningful response flags from pulse, drag, control, menu, tab, and context paths.
- [x] Render a compact response inspector card without adding nested app-card chrome.
- [x] Preserve responsive layout across stack, compact, and regular showcase layouts.

## P2 - Gates And Evidence

- [x] Update source-policy tests for the new inspector surface.
- [x] Run focused format/check/build/source-policy gates.
- [x] Run the workstream catalog and diff checks.
- [x] Close the lane or record the next narrower follow-on if automation is still needed.
