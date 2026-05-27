# Material 3 TimePicker String Registry Packet v1 - Handoff

Status: Closed
Last updated: 2026-05-28

## Current State

TimePicker registry wiring is closed. The component no longer owns hard-coded labels directly in the
recipe; it asks Material foundation helpers, which in turn use `I18nService` with English fallback.

## Continue Policy

Return to `material3-component-alignment-sweep-v1`.

Do not reopen this lane for DatePicker locale-aware date descriptions. Start a separate DatePicker
locale packet if the next picker slice needs localized month/day/date value descriptions.

## Watch Points

- If additional Material components need localized strings, extend `foundation::strings` carefully
  and keep key naming under the `material3-<component>-<purpose>` namespace.
- Keep the string-key enum crate-private until more components prove the public API shape.
- Bootstrap default FTL resources are convenience defaults, not the only localization backend.
