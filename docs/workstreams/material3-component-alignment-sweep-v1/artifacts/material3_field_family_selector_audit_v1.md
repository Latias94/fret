# Material 3 Field Family Selector Audit v1

Date: 2026-05-27
Status: M3CAS-050 progress, not full field-family closure

## Scope

Components:

- `TextField`
- `Autocomplete`
- `ExposedDropdown`
- `SearchBar`
- `SearchView`
- Existing seed: `Select`

This audit only covers stable automation surfaces needed before field-family diagnostics and packet
fixtures can safely depend on selectors.

## Findings

- `TextField` exposed only the root input id and ad hoc hyphen-derived icon ids. It now exposes:
  - `<base>`
  - `<base>.chrome`
  - `<base>.active-indicator` for filled fields
  - `<base>.label`
  - `<base>.supporting-text`
  - `<base>.leading-icon`
  - `<base>.trailing-icon`
- `Autocomplete` inherited `TextField` root/chrome/label/supporting/icon ids, but popup ids used
  `base-listbox` and `base-option-*`. They now use:
  - `<base>.listbox`
  - `<base>.option.<sanitized-value>`
  - `<base>.option.<sanitized-value>.chrome`
  Explicit item ids are still respected, with `<item>.chrome` derived from the explicit item id.
- `ExposedDropdown` composes `Autocomplete`, so it inherits the same field and popup selector
  surface.
- `SearchBar` now derives:
  - `<base>.chrome`
  - `<base>.leading-icon`
  - `<base>.trailing-icon`
- `SearchView` composes `SearchBar` and now derives:
  - `<base>.overlay` when no explicit overlay test id is provided.
- `Select` already had stable dotted root/chrome/active-indicator/trailing-icon/listbox/item ids
  from the seed packet and M3CAS-030 automation surface gate.

## Layer Classification

- `material_recipe`: owns which field parts are externally meaningful for automation and diagnostics.
- `material_foundation`: `foundation::test_id::part_test_id` owns the dotted part-id convention.
- `diagnostics`: `automation_surface.rs` proves the ids are live in rendered trees.
- `kit_policy`: no issue found in this selector pass.
- `mechanism`: no issue found in this selector pass.

## Gates

```powershell
cargo fmt --package fret-ui-material3
cargo test -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

Result:

- `automation_surface` now has 10 passing tests.
- New field-family tests cover TextField, filled Autocomplete active-indicator inheritance,
  Autocomplete popup selectors, SearchBar, and SearchView. ExposedDropdown inherits Autocomplete's
  selector surface and is covered by behavior-specific gates in the field packet.

## Remaining M3CAS-050 Work

- Build the full field-family parity packet around committed value vs editable query.
- Classify floating label and active-indicator behavior as recipe-local vs shared field foundation.
- Add or harden diagnostics for Autocomplete/ExposedDropdown popup semantics and SearchView
  dismiss/focus behavior.
- Decide whether any overlay/focus behavior belongs in `fret-ui-kit`; no such defect is proven by
  this selector audit alone.
