# ImUi Text Input Filter Policy v1 Design

Status: Closed
Last updated: 2026-05-04

## Problem

Dear ImGui's `InputText` has lightweight named character filters:

- `CharsDecimal`
- `CharsHexadecimal`
- `CharsScientific`
- `CharsUppercase`
- `CharsNoBlank`

Fret already has the broader text-input policy slice for read-only, password display,
select-all-on-focus, AllowTabInput, and completion/history command routing. The remaining narrow
gap here is the named filter set, not the full mutable-buffer callback model.

## Layer Decision

- `crates/fret-ui` owns only the generic mechanism: a `TextInputProps::insert_filter` hook that
  transforms text about to be inserted.
- `ecosystem/fret-ui-kit::imui` owns the Dear ImGui-shaped policy: `InputTextFilters`.
- `ecosystem/fret-imui` remains a thin mounting/facade crate and gains only regression coverage.

This keeps the runtime mechanism reusable for other component crates while keeping named-filter
policy out of `crates/fret-ui`.

## Must-Be-True Outcomes

- A focused single-line IMUI input can reject decimal/scientific disallowed characters before they
  reach the model.
- Uppercase and no-blank policy transforms/rejects inserted text through the public
  `InputTextOptions` path.
- Runtime text insertion applies the same generic filter to normal text input and paste-like paths.
- Dear ImGui's callback-heavy `CallbackCharFilter` remains explicitly out of scope for this slice.

## Non-Goals

- No mutable text buffer callback API in `crates/fret-ui`.
- No `fret-imui` widget implementation growth.
- No locale decimal-point override or full-width numeric conversion in this first filter slice.
- No multiline-specific filter policy beyond the generic runtime insertion mechanism.
