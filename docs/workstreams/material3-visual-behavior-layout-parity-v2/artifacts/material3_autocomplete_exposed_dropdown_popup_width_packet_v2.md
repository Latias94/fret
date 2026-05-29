# Material3 Autocomplete / ExposedDropdown Popup Width Packet v2

Task: M3PV2-025
Date: 2026-05-28
Status: Complete

## Truth

Autocomplete and ExposedDropdown are field-family recipes: the popup/menu belongs visually to the
field chrome, while text editing, focus, keyboard navigation, and combobox relations belong to the
input. Icon-bearing fields must therefore size and align the listbox against the full TextField
chrome, not the inner input's editable content area.

## Sources

- Compose Material3 `ExposedDropdownMenuBoxScope.exposedDropdownSize(matchAnchorWidth = true)`:
  `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ExposedDropdownMenu.kt`.
- Base UI Autocomplete part structure keeps input/trigger/popup responsibilities separate:
  `repo-ref/base-ui/test/public-types/autocomplete.tsx`.
- Fret field-family precedent: Select already uses the trigger/chrome bounds for listbox width and
  has behavior gates for width and clamping in `ecosystem/fret-ui-material3/tests/select_behavior.rs`.

## Findings

- This was a Material recipe layout issue, not a `fret-ui` or `fret-ui-kit` mechanism issue.
- `Autocomplete` already requested `field_id_out` from `TextField`, but popup placement still used
  `input_id` as the anchor.
- With leading/trailing icon fields, the listbox measured `494px` while the field chrome measured
  `496px`. The issue also affected `ExposedDropdown` because it composes `Autocomplete`.

## Changes

- `ecosystem/fret-ui-material3/src/autocomplete.rs`
  - uses `field_element_id` as the popup placement/width anchor when available;
  - falls back to `input_id` when the field id is not yet available;
  - keeps `input_id` as the dismissible popover identity/trigger, keyboard handler target,
    combobox relation source, and focus owner;
  - uses the anchor element's root bounds for popup collision/layout space.
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
  - strengthened `material3_autocomplete_exposes_stable_part_test_ids` to assert listbox x/width
    against the field chrome;
  - added `material3_exposed_dropdown_popup_matches_field_chrome_bounds` with leading icon,
    trailing icon, label, supporting text, popup part ids, and chrome/listbox width assertions.

## Proof

Red gate before the implementation change:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids material3_exposed_dropdown_popup_matches_field_chrome_bounds
```

Result: failed, run id `4cc039dc-055e-45be-9336-08d87e7cb08f`; both popups reported `494px`
against a `496px` field chrome.

Passing gates after the implementation change:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids material3_exposed_dropdown_popup_matches_field_chrome_bounds
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete_semantics_v1 material3_exposed_dropdown_trailing_icon_toggles_overlay_v1 material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1
cargo nextest run -p fret-ui-material3 --lib autocomplete_default_listbox_test_id_uses_dotted_part_contract
cargo check -p fret-ui-material3 --features diagnostics --tests
```

Results:

- Automation popup width gate: 2 passed, run id `f3546f59-293b-4602-b07d-e8798170d769`.
- Behavior/semantics regression gate: 3 passed, run id `c0d9388d-7497-4de6-874c-ea7b74177989`.
- Autocomplete listbox id contract gate: 1 passed, run id `203d19bd-2fc2-4add-a9bb-b848c279d064`.
- Diagnostics tests check: passed.

## Matrix Updates

- `autocomplete.layout`: `covered_v2`.
- `exposed_dropdown.layout`: `covered_v2`.
- `autocomplete.style` and `exposed_dropdown.style` were still open after this packet; later
  selectable-item and motion packets, plus the final closeout matrix, supersede that interim state.
- Motion is covered later by `material3_autocomplete_exposed_dropdown_motion_packet_v2.md`; this
  packet only verifies settled geometry, not transition timing.

## Residual Risk

- Popup surface style/elevation still needs a dedicated token/chrome packet.
- Popup open/close motion is covered later by
  `material3_autocomplete_exposed_dropdown_motion_packet_v2.md`.
- Multiline/TextArea-like field anchoring remains a separate TextField scenario rather than an
  Autocomplete/ExposedDropdown contract.
