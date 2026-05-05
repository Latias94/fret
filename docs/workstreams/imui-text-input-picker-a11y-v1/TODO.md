# ImUi Text Input Picker Accessibility v1 TODO

Status: Closed
Last updated: 2026-05-04

## M1 - Generic Picker Semantics

- [x] Add an internal IMUI text-input builder path that can pass assistive semantics into
  `TextInputProps` without exposing a new public runtime or IMUI option.
- [x] Keep regular `input_text_model_with_options` behavior unchanged by defaulting the internal
  assistive semantics to empty.
- [x] Make completion/history picker inputs expose combobox-style semantics.
- [x] Record active picker option element IDs from rendered selectable candidates.
- [x] Wire `expanded`, `controls_element`, and `active_descendant_element` back to the owning input.
- [x] Add a focused `fret-imui` semantics regression test.

## Future Follow-Ons

- [ ] Add editor-owned completion/history ranking and persistence policy where a real editor field
  needs it.
- [ ] Add richer AccessKit/platform bridge checks once the generic semantics snapshot is not enough
  to prove user-facing announcements.
- [ ] Revisit popup role refinement if generic picker recipes need a dedicated listbox overlay
  instead of reusing the menu popup substrate.
