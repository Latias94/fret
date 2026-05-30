# Material 3 TimePicker String Registry Packet v1 - Milestones

Status: Closed
Last updated: 2026-05-28

## M0 - Layer Classification

Closed. The missing behavior was not a mechanism gap. It belonged in Material foundation because the
runtime i18n service already exists and the design-system crate needed a Material string-key bridge.

## M1 - Foundation And Recipe Wiring

Closed. `foundation::strings` now provides TimePicker-specific helpers over `I18nService`, and
TimePicker consumes them for labels, values, input supporting/error text, period controls, scrim,
and action buttons.

## M2 - Evidence And Closeout

Closed. The automation-surface test injects a lookup and verifies registry strings across docked
dial, docked input, and modal dialog surfaces. Bootstrap default Fluent resources now format both
plain and argument-backed Material3 TimePicker strings.
