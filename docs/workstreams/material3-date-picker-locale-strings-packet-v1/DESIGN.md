# Material 3 DatePicker Locale Strings Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The prior DatePicker packets closed row/column selectors, value-derived day-cell anchors,
selectable-date disabling, and displayed-month live-region semantics. The remaining real gap was
string ownership: month labels, weekday labels, navigation affordances, dialog labels, and day-cell
spoken descriptions were still recipe-local English text.

This is not a `fret-ui` mechanism problem. Existing semantics can express button role, labels,
selected state, disabled state, test ids, and live-region flags. Existing `fret-i18n` already
provides host-global lookup and typed arguments.

## Target State

- Route DatePicker title, scrim, actions, month navigation, month/year labels, weekday labels, and
  day-cell descriptions through Material foundation helpers.
- Keep the DatePicker public API unchanged.
- Preserve visible short navigation labels while exposing full accessibility labels.
- Seed default bootstrap Fluent resources for `en-US` and `zh-CN`.
- Prove both docked and modal surfaces with injected test lookup values and English fallback
  regressions.

## Layer Mapping

- `material_foundation`: owns `foundation::strings`, Material DatePicker key names, English
  fallbacks, month/weekday fallback tables, and typed date-description arguments.
- `material_recipe`: owns DatePicker consumption of string helpers, day-cell semantics, title/scrim
  semantics, and short-visible/full-a11y navigation labels.
- `fret-bootstrap`: owns default app Fluent resources.
- `fret-runtime` / `fret-i18n`: already own lookup service, message keys, and typed arguments.
- `fret-ui` and `fret-ui-kit`: no new mechanism or policy surface is needed.

## Non-Goals

- Do not add a new localization backend.
- Do not introduce public string-key stability before more Material components share the registry.
- Do not implement unbuilt picker modes such as text input, year selector, or range picker.
- Do not add full CLDR date formatting in this slice; the registry receives structured arguments
  so app-level localization can format locale-specific strings.

## Upstream References

- Compose Material3 `DatePicker.kt`: weekday content descriptions, month navigation descriptions,
  day-cell content description, and selected/today semantics.
- Fret i18n contract: `crates/fret-i18n/src/lib.rs`.
- Bootstrap Fluent defaults: `ecosystem/fret-bootstrap/src/lib.rs`.
