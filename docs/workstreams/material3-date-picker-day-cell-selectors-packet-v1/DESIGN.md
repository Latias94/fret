# Material 3 DatePicker Day Cell Selectors Packet v1

Date: 2026-05-28
Status: Closed

## Problem

The picker packet made DatePicker automation stable enough for modal/docked surfaces, but day cells
still expose only row/column ids such as `date_picker.cell.0.0`. Those ids are stable for layout
inspection but weak for parity and accessibility gates that need to address a specific calendar date.

## Target State

- DatePicker keeps the existing row/column cell ids.
- Each rendered day cell also exposes a value-derived id: `date_picker.cell.<yyyy-mm-dd>`.
- The value-derived id is available for both in-month and outside-month cells because both are
  rendered and semantically selectable in the current recipe.
- Tests prove representative docked and modal date ids.
- The component matrix records this diagnostics/recipe improvement separately from larger
  selectable-date and localization follow-ons.

## Source Truth

- Compose Material3 models date selection around actual dates and `SelectableDates`, not row/column
  positions.
- Compose DatePicker day semantics attach content descriptions and enabled state to the date surface.
- This packet does not port the full `SelectableDates` policy; it improves the Fret automation
  surface so later accessibility gates can target actual dates.

## Layer Ownership

This is `ecosystem/fret-ui-material3` recipe work:

- The `fret-ui` mechanism already supports semantic test ids and hidden diagnostic anchors.
- `fret-ui-kit` calendar already owns month-grid generation.
- Material DatePicker owns how rendered calendar dates map to recipe-facing selectors.

## Non-Goals

- Add a `SelectableDates` public API.
- Disable non-selectable days.
- Add live-region month announcements.
- Change existing row/column selectors.

Status note (2026-05-28): localized date spoken labels were closed later by
`docs/workstreams/material3-date-picker-locale-strings-packet-v1/`.
