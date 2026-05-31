# Material3 Field Interaction Family Split v1 Design

Status: Closed
Last updated: 2026-05-31

## Problem

After the TimePicker split, `material3_interaction_regressions.rs` still mixed plain TextInput
mechanism coverage with Material3 field-family interaction tests for Autocomplete and
ExposedDropdown. The field tests share TextField chrome, popup/overlay behavior, combobox/listbox
semantics, query filtering, commit, blur, and trailing-icon interaction boundaries, so they need a
purpose-owned binary before the final TextInput ownership audit.

## Decision

Move the five field-family interaction tests into
`ecosystem/fret-ui-material3/tests/material3_field_interactions.rs`.

Keep `material3_interaction_regressions.rs` only for the single plain TextInput event regression
until a mechanism-layer ownership audit decides whether it belongs under `fret-ui`.

## Boundaries

- This lane is a test-ownership refactor, not a field component behavior change.
- Autocomplete and ExposedDropdown stay together because they share input + popup semantics.
- No public Material3 API, core UI mechanism, or shared test support API changes are introduced.

## Non-Goals

- Do not move the plain TextInput test in this lane.
- Do not rename or rewrite Autocomplete/ExposedDropdown APIs.
- Do not consolidate field-family component internals while splitting tests.

## Follow-On Shape

The residual file now has one remaining decision: audit
`text_input_text_input_event_updates_model` for mechanism-layer ownership. If it moves to
`fret-ui`, the residual Material3 interaction-regression binary can be deleted.
