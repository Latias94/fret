# Material 3 DatePicker Month Live Region Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The DatePicker selectable-date packet left a broad accessibility follow-on for locale labels and
live-region announcements. This packet closed the displayed-month live-region slice. The locale
string slice was closed later by
`docs/workstreams/material3-date-picker-locale-strings-packet-v1/`.

Fret's DatePicker already rendered the month/year label, but it had no stable part id and no live
region semantics.

## Target State

- Docked and modal DatePicker month labels expose stable part ids:
  - `date_picker.docked.month-label`
  - `date_picker.modal.month-label`
- The month label exposes its text as the semantics label.
- The month label is a polite atomic live region.
- Navigating to the next/previous month updates the same live region label.
- No new mechanism or kit policy is added.

## Truth Set

- Truth 1: The initial docked month label exposes `January 2026`.
- Truth 2: The docked month label exposes `SemanticsLive::Polite` and `live_atomic = true`.
- Truth 3: Activating the next-month button updates the month label to `February 2026`.
- Truth 4: The month label remains a polite atomic live region after navigation.
- Truth 5: The stable DatePicker automation-surface test covers the new docked and modal
  `month-label` part ids.

## Layer Mapping

- `ecosystem/fret-ui-material3/src/date_picker.rs`: Material recipe owns the displayed month label,
  part id, and live-region semantics.
- `crates/fret-ui` / `crates/fret-core`: existing semantics decoration and live-region flags are
  reused.
- `ecosystem/fret-ui-kit`: no shared policy is needed.
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`: focused proof covers label updates and
  selector readiness.

## Non-Goals

- Do not add localized month/day strings in this slice; they were closed later by
  `docs/workstreams/material3-date-picker-locale-strings-packet-v1/`.
- Do not add full APG grid navigation or announcement coverage.
- Do not change DatePicker selection behavior.
- Do not add platform-specific announcement APIs.

## Upstream References

- Compose Material3 `DatePicker.kt`: `YearPickerMenuButton` text semantics set
  `liveRegion = LiveRegionMode.Polite` and `contentDescription = yearPickerText`.
