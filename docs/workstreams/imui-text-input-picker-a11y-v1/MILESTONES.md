# ImUi Text Input Picker Accessibility v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Baseline

Exit criteria:

- Confirmed `TextInputProps` already supports active-descendant and controls relationships.
- Confirmed editor `TextAssistField` has a richer owner-owned assistive pattern, but generic IMUI
  picker can close the first semantics gap without adopting editor storage policy.

State: Complete.

## M1 - Runtime-Neutral Semantics Wiring

Exit criteria:

- Internal IMUI text input builder accepts assistive semantics.
- Picker input passes combobox role, expanded state, controls relation, and active-descendant element
  without widening `InputTextOptions`.
- Focused semantics test proves the relationship through a real picker render.

State: Complete.

## M2 - Closeout

Exit criteria:

- Workstream docs and gap audit point to the shipped generic picker a11y slice.
- Remaining editor-owned and platform-bridge work is explicitly split out.

State: Complete.
