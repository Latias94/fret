# ImUi Text Input History Completion Policy v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Source Mapping

Exit criteria:

- Dear ImGui completion/history keys are identified.
- Fret ownership is explicit.

State: Complete.

## M1 - Command-Oriented Option Surface

Exit criteria:

- Single-line `InputTextOptions` exposes completion/history command fields and repeat opt-ins.
- Defaults remain inert and backward compatible.

State: Complete.

## M2 - Focused Key Dispatch

Exit criteria:

- Focused IMUI input text dispatches completion/history commands on the intended unmodified keys.
- IME composition, modified keys, and repeated keydown do not accidentally trigger commands.

State: Complete.

## M3 - Closeout

Exit criteria:

- Focused tests and workstream/audit docs explain the bounded command policy.
- Remaining callback-heavy gaps are left as separate follow-ons.

State: Complete.
