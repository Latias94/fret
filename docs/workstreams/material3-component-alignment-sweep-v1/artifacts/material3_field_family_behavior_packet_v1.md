# Material 3 Field Family Behavior Packet v1

Date: 2026-05-27
Task: M3CAS-050
Status: packet complete with known follow-ons

## Scope

Components:

- `TextField`
- `Autocomplete`
- `ExposedDropdown`
- `SearchBar`
- `SearchView`
- Seed anchor: `Select`

This packet covers field-family behavior that can be verified without first building picker dialogs
or full-screen search state machines. `DatePicker` and `TimePicker` remain queued for M3CAS-060.

## Source Truth

Local reference anchors:

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TextField.kt`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/OutlinedTextField.kt`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ExposedDropdownMenu.kt`
- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt`
- `repo-ref/material-web/field/internal/field.ts`
- `repo-ref/material-web/textfield/internal/text-field.ts`
- `repo-ref/material-web/select/internal/select.ts`
- `repo-ref/material-ui/packages/mui-material/src/Autocomplete/Autocomplete.js`
- `repo-ref/material-ui/packages/mui-material/src/FormControl/FormControl.js`
- `repo-ref/material-ui/packages/mui-material/src/TextField/TextField.js`
- `repo-ref/material-ui/packages/mui-material/src/Select/SelectInput.js`

Material outcomes used by this packet:

- Field chrome is state-driven: focus/open/populated affect floating label, placeholder visibility,
  outline or active-indicator thickness/color, and icon colors.
- Filled fields expose an active indicator; outlined fields expose outline chrome and notch/cutout
  behavior around floating labels.
- Combobox popups expose `aria-expanded`, `aria-controls`, active-descendant linkage, and listbox
  options without moving focus away from the editable input.
- Autocomplete separates editable query from committed value; blur/selection policy is recipe-level.
- Exposed dropdown treats the menu anchor as an editable/selectable field and reverts an uncommitted
  query to the committed selection on blur.
- SearchView has a larger Material state machine in Compose; the original M3CAS-050 Fret surface was
  a docked MVP using the shared search bar plus overlay policy. The follow-on
  `material3-search-view-state-packet-v1` added an explicit full-screen presentation slice.

## Findings

### TextField

Layer classification:

- `material_foundation`: added `foundation::field::material_field_active_indicator_layer`.
- `material_recipe`: `TextField` owns floating label, placeholder opacity, icon slots, supporting
  text, and variant-specific chrome assembly.
- `diagnostics`: `automation_surface` now verifies live `.active-indicator` selectors for filled
  TextField.
- `mechanism`: no mechanism gap found.

The implementation already had token-driven floating label progress, placeholder opacity, and
animated filled active-indicator thickness/color, but the active indicator was folded into the
container border. That made it unaddressable for diagnostics and duplicated Select's field-line
canvas logic. The refactor splits the filled active indicator into a shared Material field helper and
keeps outlined fields on the existing border/outline path.

### Select

Layer classification:

- `material_foundation`: now shares the same field active-indicator layer as TextField.
- `material_recipe`: keeps combobox trigger, listbox, typeahead, selected-value display, floating
  label, and menu width behavior.
- `kit_policy`: overlay dismissal/focus policy remains in `fret-ui-kit`.
- `mechanism`: no mechanism gap found.

Select was the seed field anchor. This packet only removed local active-indicator paint duplication;
existing Select behavior gates remain the source of truth for committed value/listbox behavior.

### Autocomplete

Layer classification:

- `material_recipe`: owns query filtering, active option, Enter commit, click commit, suppress
  reopen, item ids, and option chrome.
- `material_foundation`: inherits TextField's shared field active-indicator layer when using the
  filled variant.
- `diagnostics/test_harness`: stale hyphen selectors in tests and diag scripts were the concrete
  drift found by this packet.
- `kit_policy`: popup placement/dismiss/focus routing uses existing popper/overlay primitives; no
  new kit gap found.

The red repro was `material3_autocomplete_semantics_v1`: it searched for old
`material3-autocomplete-listbox` and `material3-autocomplete-option-*` ids after the selector
contract moved to dotted part ids. Tests and diag scripts now target
`<base>.listbox`, `<base>.option.<value>`, and `<base>.option.<value>.chrome`.

### ExposedDropdown

Layer classification:

- `material_recipe`: owns committed selection, editable query, trailing icon toggle, and blur revert.
- `material_foundation`: inherits TextField/Autocomplete field chrome.
- `diagnostics/test_harness`: stale trailing-icon/listbox/option selectors were updated.
- `mechanism`: no mechanism gap found.

The focused gates prove the user-visible behavior that matters for this packet: trailing icon toggles
the overlay, and blur restores the committed selection when the query is not committed.

### SearchBar And SearchView

Layer classification:

- `material_recipe`: `SearchBar` owns its pill field chrome and icon slots; `SearchView` now owns
  docked vs full-screen presentation selection and overlay-local header composition.
- `kit_policy`: `SearchView` uses existing overlay policy for docked popovers and modal full-screen
  presentation.
- `diagnostics`: stable `.chrome`, `.leading-icon`, `.trailing-icon`, `.overlay`, and
  `.overlay.header` ids are live.
- `follow_on`: predictive back gesture progress and a generic platform-back event remain separate
  mechanism work only if a product surface needs them.

No shared Material foundation issue was proven for search beyond stable part-id coverage.

## Artifacts

- `ecosystem/fret-ui-material3/src/foundation/field.rs`
- `ecosystem/fret-ui-material3/src/text_field.rs`
- `ecosystem/fret-ui-material3/src/select.rs`
- `ecosystem/fret-ui-material3/src/autocomplete.rs`
- `ecosystem/fret-ui-material3/src/exposed_dropdown.rs`
- `ecosystem/fret-ui-material3/src/search_bar.rs`
- `ecosystem/fret-ui-material3/src/search_view.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-autocomplete-filtering.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-exposed-dropdown-filtering.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-autocomplete-option-chrome-fill.json`
- `tools/diag-scripts/ui-gallery/overlay/ui-gallery-material3-autocomplete-dialog-screenshots.json`

## Proof

Red/green evidence:

```powershell
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete_semantics_v1
```

This failed before the selector repair because the test searched for the old hyphen listbox id.

Passing gates after the repair:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test select_behavior
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_exposed_dropdown
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

Results:

- `automation_surface`: 10 tests passed, including TextField and filled Autocomplete
  `.active-indicator` coverage.
- `select_behavior`: 8 tests passed.
- `material3_autocomplete*`: 3 tests passed.
- `material3_exposed_dropdown*`: 2 tests passed.

## Residual Risk

- Full TextField error supporting text/live-region semantics are not yet separately modeled; current
  Fret API still has one `supporting_text` slot plus `error` state.
- Outlined TextField outline/cutout remains border-based; no separate `.outline` selector was added
  because this packet only proved a filled active-indicator diagnostic need.
- SearchView full-screen presentation is now covered by
  `material3-search-view-state-packet-v1`; predictive back progress and platform back events remain
  out of scope.
- Picker fields are intentionally deferred to M3CAS-060.
