# ImUi Text Input Picker Accessibility v1

Status: Closed narrow follow-on
Last updated: 2026-05-04

This lane closes the first generic accessibility gap left after the completion/history picker
keyboard navigation slice.

## Ownership

- `fret-ui` already owns the mechanism fields on `TextInputProps`:
  `active_descendant`, `active_descendant_element`, `controls_element`, and `expanded`.
- `fret-ui-kit::imui` owns picker policy: when the helper composes a text input and popup
  candidates, it decides that the input should behave like a combobox owner.
- `fret-imui` remains the proof surface and does not gain widget policy.

## Must-Be-True Outcomes

- Picker inputs default to `SemanticsRole::ComboBox` unless the caller has intentionally chosen a
  non-default role.
- When the popup is mounted, the input exposes `expanded=true`.
- The input controls the popup panel node when the popup panel is known.
- After keyboard navigation stabilizes an active option element, the input exposes
  `active_descendant` for that option.
- Candidate storage, ranking, popup dismissal policy, and editor history persistence remain outside
  `crates/fret-ui`.

## Non-Goals

- No new runtime accessibility contract.
- No editor-owned completion ranking/storage.
- No AccessKit platform bridge assertion in this slice.
- No conversion of the generic popup menu panel into a full dedicated listbox overlay primitive.
