# Material3 Field Family Production Alignment v1

Status: Closed
Last updated: 2026-05-31

## Problem

The Material3 field family had already aligned floating labels and supporting text to logical
inline insets, but the wider production surface still needed a source-backed audit across:

- `TextField`
- `Autocomplete`
- `ExposedDropdown`
- `Select`

The audit needed to distinguish mechanism gaps from component gaps. In particular, editable
comboboxes should keep focus on the input and use active descendant semantics, while Select should
remain a non-editable combobox trigger with roving focus inside its listbox.

## Source Stack

- Material/Compose field chrome: `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/internal/TextFieldImpl.kt`
- Material/Compose exposed dropdown anchoring: `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ExposedDropdownMenu.kt`
- Base UI combobox input semantics: `repo-ref/base-ui/packages/react/src/combobox/input/ComboboxInput.tsx`
- Base UI select trigger semantics: `repo-ref/base-ui/packages/react/src/select/trigger/SelectTrigger.tsx`
- Base UI select item semantics: `repo-ref/base-ui/packages/react/src/select/item/SelectItem.tsx`

## Findings

- `TextField` was the right shared editable field chrome for Autocomplete and ExposedDropdown.
- `Select` should not be collapsed into `TextField`; it owns a non-editable trigger, popup initial
  focus, typeahead, and roving listbox focus policy.
- The remaining shared foundation gap was field icon slot geometry: TextField leading/trailing
  icons and their input padding still used physical left/right edges, so RTL fields could mirror
  label/supporting text while leaving icon slots LTR.
- The Select trigger had a component-level automation gap: the trigger leading icon had no stable
  part `test_id`.

## Shipped Slice

- Added `foundation::field::material_field_icon_adjusted_padding` so field input padding maps
  leading/trailing icon slots through logical inline start/end.
- Migrated TextField leading/trailing icon hit targets to logical inline start/end insets.
- Added Select trigger `*.leading-icon` part test id.
- Made Select trigger row padding and row layout direction follow the resolved Material layout
  direction.

## Follow-Ons

- Decide whether multiline `TextAreaStyle` needs asymmetric logical padding support, since it
  currently reduces padding to symmetric `padding_x`.
- Extract a shared selectable menu row renderer only if Select and Autocomplete option rows start
  duplicating more behavior than token fallback.
- Consider public optional builder helpers for field-family composition if more recipes need to
  forward optional TextField props.
