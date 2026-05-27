# Material 3 TimePicker Dial Accessibility Packet v1

Date: 2026-05-28
Status: Closed

## Problem

The Material 3 picker packet closed the base TimePicker selector and modal surfaces, but left one
diagnostics-facing accessibility gap: clock dial labels are semantic buttons with labels and
selected state, but they do not expose stable value-derived `test_id`s. That makes automation rely
on dial geometry or list position when it needs to inspect or interact with a specific hour or
minute value.

## Target State

- TimePicker clock dial labels derive stable ids from the caller-provided base id.
- Dial item ids are value-derived, not render-position-derived.
- Hour and minute dial values are disambiguated by part name.
- The parent clock dial group and chrome ids remain unchanged.
- The shipped surface is covered by a focused automation gate and recorded in the component matrix.

## Source Truth

- Material spec defines the TimePicker dial as a selectable clock-face surface.
- Compose Material3 is the primary semantics reference for this slice:
  - `ClockFace` uses a selectable-group semantics surface.
  - clock values expose semantic labels, selection state, and traversal ordering.
- Fret keeps this as `ecosystem/fret-ui-material3` recipe work because the mechanism already exposes
  semantic button nodes and stable `test_id` plumbing through `PressableA11y`.

## Scope

In scope:

- Add value-derived dial label ids for hour and minute dials.
- Add tests that prove representative and complete 12-hour/minute label selectors are live.
- Update the picker packet and component matrix to distinguish resolved dial ids from remaining
  accessibility depth work.

Out of scope:

- Invalid time input error/supporting text.
- Live-region announcements.
- Locale-specific spoken labels.
- DatePicker disabled-date/locale semantics.
- 24-hour dual-ring parity beyond value-derived ids for the labels Fret currently renders.

## Architecture Direction

This is a Material recipe and diagnostics-surface change. No `crates/*` mechanism change is needed.
No shared `fret-ui-kit` policy is needed because the dial item id scheme is Material TimePicker
taxonomy, not a design-system-agnostic behavior.
