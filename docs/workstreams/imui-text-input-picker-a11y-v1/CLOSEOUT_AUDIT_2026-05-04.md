# ImUi Text Input Picker Accessibility v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane closes the generic IMUI picker active-descendant semantics gap at the `fret-ui-kit::imui`
policy layer.

## What Shipped

- `input_text_model_element_with_options` now has an internal sibling that accepts assistive
  semantics and forwards them to `TextInputProps`.
- Completion/history picker inputs default from text-field role to combobox role.
- Picker input semantics now expose:
  - `expanded`,
  - `controls_element` for the popup panel when mounted,
  - and `active_descendant_element` for the active keyboard candidate once its option element is
    known.
- The picker keeps candidate storage app-owned and does not move completion/history policy into
  `crates/fret-ui`.

## Proof

- `input_text_completion_picker_keyboard_navigation_exposes_active_descendant_semantics` verifies
  ComboBox role, expanded state, controls relation, and active-descendant relation from a real
  focused picker.
- Existing picker keyboard tests continue to prove ArrowUp/ArrowDown and Enter/NumpadEnter
  behavior.

## Remaining Work

Start narrower follow-ons for:

- editor-owned ranking/storage and accepted-item history,
- platform accessibility bridge announcement checks,
- generic popup role/listbox refinement,
- and deeper multiline behavior.
