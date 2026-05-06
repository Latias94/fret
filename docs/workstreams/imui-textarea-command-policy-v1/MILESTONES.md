# ImUi Textarea Command Policy v1 Milestones

Status: Closed
Last updated: 2026-05-06

## M0 - Source Mapping

Exit criteria:

- Multiline submit/cancel is classified as IMUI policy, not runtime textarea mechanism.
- Dear ImGui mutable-buffer callbacks remain a non-goal.

State: Complete.

## M1 - Command-Oriented Option Surface

Exit criteria:

- `TextAreaOptions` exposes submit/cancel command fields, submit key policy, and repeat opt-in.
- Defaults are inert except for preserving existing multiline text insertion behavior.

State: Complete.

## M2 - Focused Key Dispatch

Exit criteria:

- Focused IMUI textarea dispatches app-owned submit/cancel commands on the configured keys.
- Capture-phase Enter-submit prevents accidental newline insertion when that policy is selected.
- IME composition, unsupported modifiers, and repeated keydown stay guarded.

State: Complete.

## M3 - Closeout

Exit criteria:

- Focused tests and workstream/audit docs explain the bounded command policy.
- Remaining editor-heavy multiline behavior is left to future narrow follow-ons.

State: Complete.
