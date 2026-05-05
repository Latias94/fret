# ImUi Text Input Filter Policy v1 TODO

Status: Closed
Last updated: 2026-05-04

## M0 - Source Mapping

- [x] Confirm Dear ImGui named filter flags and order from `repo-ref/imgui/imgui.h`.
- [x] Confirm runtime should expose only a generic insert-filter mechanism.
- [x] Keep callback-heavy `CallbackCharFilter` out of this slice.

## M1 - Runtime Mechanism

- [x] Add a cloneable `TextInputInsertFilter` mechanism to runtime text input props.
- [x] Apply the filter to text input insertion.
- [x] Apply the filter after single-line clipboard normalization.
- [x] Keep rejected non-empty insertions from deleting existing selections/ranges.

## M2 - IMUI Policy

- [x] Add `InputTextFilters` to `fret-ui-kit::imui`.
- [x] Support decimal, hexadecimal, scientific, uppercase, and no-blank named filters.
- [x] Wire `InputTextOptions::filters` to `TextInputProps::insert_filter`.

## M3 - Gates And Closeout

- [x] Add runtime and IMUI model tests.
- [x] Update IMUI gap audit and workstream indexes.
- [x] Leave follow-on policy for custom callback filters and deeper text editing parity.
