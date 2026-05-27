# Material 3 TimePicker String Registry Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The TimePicker accessibility-label packet aligned roles, selected state, period grouping, and spoken
hour/minute values, but those strings were still hard-coded in the TimePicker recipe. That left a
real Material design-system gap: app-level `I18nService` already existed, yet `fret-ui-material3`
had no Material-owned registry helper for component strings.

This is not a `fret-ui` mechanism problem. Runtime/global lookup and typed message arguments are
already available through `fret-i18n` via `fret_runtime::fret_i18n`.

## Target State

- Add a Material foundation string helper that reads `I18nService` from the host and falls back to
  English Material outcomes when no backend or key is available.
- Route TimePicker visible labels, a11y labels, spoken values, input supporting/error text, period
  labels, scrim label, and action button labels through typed helpers.
- Keep the TimePicker public API unchanged.
- Seed default bootstrap Fluent resources for `en-US` and `zh-CN`.
- Prove the wiring by injecting a test lookup and asserting semantics/action outcomes, while keeping
  the existing English fallback a11y test.

## Layer Mapping

- `material_foundation`: owns `foundation::strings`, Material string key naming, lookup fallback,
  and typed TimePicker string helpers.
- `material_recipe`: owns where TimePicker consumes the Material string helpers.
- `fret-runtime` / `fret-i18n`: already own lookup service, message keys, and typed arguments.
- `fret-bootstrap`: owns default app bootstrap Fluent resources.
- `fret-ui` and `fret-ui-kit`: no new mechanism or policy surface is needed.

## Non-Goals

- Do not add a new localization backend.
- Do not make string keys public API before more components share the registry surface.
- Do not solve DatePicker locale-aware date descriptions in this slice.
- Do not add general locale-aware number/date formatting beyond the existing `MessageArgs` path.

## Upstream References

- Compose Material3 `TimePicker.kt`: string outcomes for selectors, input labels, error labels,
  period labels, and spoken hour/minute descriptions.
- Fret i18n contract: `crates/fret-i18n/src/lib.rs`.
- Bootstrap Fluent defaults: `ecosystem/fret-bootstrap/src/lib.rs`.
