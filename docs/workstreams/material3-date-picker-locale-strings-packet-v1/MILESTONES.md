# Material 3 DatePicker Locale Strings Packet v1 - Milestones

Status: Closed
Last updated: 2026-05-28

## M0 - Boundary Classification

- DatePicker locale/date-description drift is classified as Material-owned recipe/foundation work.
- Existing semantics and i18n mechanisms are sufficient.

## M1 - Implementation

- Material DatePicker string helpers are available in `foundation::strings`.
- Docked and modal DatePicker surfaces consume the helpers.
- Day cells expose role, selected state, and localized date descriptions.
- Button supports a visible label plus an explicit accessibility label for compact affordances.
- Bootstrap defaults include `en-US` and `zh-CN` DatePicker Fluent keys.

## M2 - Verification And Closeout

- Focused DatePicker automation tests pass.
- Bootstrap DatePicker i18n formatting test passes.
- Component matrix and picker packet residual risks no longer list DatePicker locale strings as
  open.
