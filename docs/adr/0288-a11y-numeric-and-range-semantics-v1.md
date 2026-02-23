# ADR 0288: A11y Numeric and Range Semantics (v1)

Status: Accepted

## Context

Fret’s portable semantics tree (`crates/fret-core/src/semantics.rs`) supports roles, boolean flags, relations
(`labelled_by`, `described_by`, `controls`, `active_descendant`), and string `label`/`value`. This is sufficient for
basic widgets, but it is not enough for **range / numeric controls** (sliders, progress indicators, scroll-like
containers) where assistive technologies expect **structured numeric properties** (min/max/now/step) instead of
scraping or announcing a single formatted string (e.g. `"50%"`).

The lack of a structured numeric surface also makes UI automation brittle: diagnostics scripts that want to
`set_slider_value` must parse floats out of strings, which is locale- and formatting-sensitive.

We need a portable contract that:

- stays mechanism-only (ADR 0066),
- maps cleanly to platform accessibility APIs (via AccessKit),
- and can be used by diagnostics automation without per-widget string conventions.

## Goals

1. Add a portable structured semantics surface for numeric/range widgets.
2. Define clear invariants and validation behavior (reject invalid snapshots; do not silently clamp).
3. Map structured numeric fields into AccessKit numeric properties on supported platforms.
4. Keep `SemanticsNode.value` as a human-readable string (display/announcement/debug), without making it the contract
   source of truth for numeric automation.
5. Keep policy/formatting decisions (percent formatting, localization, “nice” rounding) out of mechanism crates.

## Non-goals (v1)

- Full ARIA parity for every attribute (e.g. live regions, full math of `aria-valuetext` formatting).
- A full “value formatting/localization” layer in `crates/`.
- Implementing every numeric-like widget (spinbuttons, scrollbars, splitters) as part of this ADR.

## Decision

### D1 — Add an additive “extras bucket” to `SemanticsNode`

Instead of growing many top-level optional fields on `SemanticsNode`, we introduce an additive extension bucket:

- `SemanticsNode { .., extra: SemanticsNodeExtra, .. }`
- `SemanticsNodeExtra` contains:
  - `numeric: SemanticsNumeric { value/min/max/step/jump }`
  - `scroll: SemanticsScroll { x/x_min/x_max/y/y_min/y_max }`
  - `orientation: Option<SemanticsOrientation>` (horizontal/vertical when applicable)
  - plus a small set of other mechanismizable fields (`placeholder`, `url`, `level`) that map directly into AccessKit.

Rationale:

- keeps the core struct stable as we add more optional metadata,
- encourages a “best-effort when present” mapping strategy,
- avoids repeated one-off top-level additions that cause churn.

### D2 — Numeric semantics contract (`SemanticsNumeric`)

`SemanticsNumeric` represents numeric/range-like widgets:

- `value`: current numeric value (if determinate)
- `min` / `max`: inclusive bounds (when known)
- `step`: single-step increment (ArrowUp/ArrowDown-class)
- `jump`: larger “page” increment (PageUp/PageDown-class)

Conventions:

- **Indeterminate progress** is represented by omitting `value` (keep role `ProgressBar`).
- `value` MAY exist without `min/max` if a widget has a meaningful value but no defined range (rare).

### D3 — Validation invariants (reject invalid snapshots)

`SemanticsNode::validate()` enforces:

- numeric/scroll fields must be finite if present,
- `min <= max` when both are present,
- `value` must be within `[min,max]` when all are present,
- `step > 0` and `jump > 0` when present,
- scroll positions must be within `[min,max]` when all are present (per axis),
- `level` is 1-based when present.

Rationale:

- the semantics snapshot is a cross-layer contract; invalid values should surface as errors early,
  not be silently sanitized into platform APIs.

### D4 — Portable action surfaces for numeric/range widgets

To avoid forcing platforms to use `SetValue` as the only control surface:

- Sliders expose portable **stepper actions**: `SemanticsActions.increment` / `SemanticsActions.decrement`.
- Scrollable widgets expose a portable **scroll by delta** action: `SemanticsActions.scroll_by` (payload carried by the
  accessibility action request).

Additionally, sliders MAY expose `SetValue(NumericValue)` as a best-effort surface, but only when the runtime can act on
it deterministically (D5).

### D5 — Conservative slider `SetValue` exposure (runtime-gated)

Some assistive technology stacks issue `SetValue(NumericValue)` for sliders. Fret exposes `SemanticsActions.set_value`
for sliders only when:

- the widget exposes stepper actions (increment/decrement), and
- the semantics snapshot includes structured numeric metadata sufficient to converge on a target value:
  `numeric.value`, `numeric.min`, `numeric.max`, and a positive finite `numeric.step`.

Rationale:

- prevents “fake capability”: advertising `SetValue` without the ability to honor it deterministically.

The default app driver MAY implement slider `SetValue` by translating target values into an equivalent key sequence
(`Home/End/PageUp/PageDown/ArrowUp/ArrowDown`) so it reuses the widget’s existing keyboard semantics and keeps policy in
ecosystem layers.

### D6 — AccessKit mapping is best-effort and additive

When structured fields are present, the AccessKit adapter emits:

- numeric properties (`numeric_value`, `min_numeric_value`, `max_numeric_value`, `numeric_value_step`, `numeric_value_jump`)
- scroll properties (`scroll_x`, `scroll_x_min`, …; per-axis)
- orientation (`orientation`) when provided
- other extras (`placeholder`, `url`, `level`)
- action availability (increment/decrement/scroll_by/set_value) mapped to AccessKit actions when supported.

## Evidence (implementation)

- Contract + validation: `crates/fret-core/src/semantics.rs`
- Snapshot production + slider `SetValue` gating: `crates/fret-ui/src/tree/ui_tree_semantics.rs`
- Slider stepper actions (mechanism surface): `crates/fret-ui/src/widget.rs`
- AccessKit mapping + action decode: `crates/fret-a11y-accesskit/src/{mapping.rs,actions.rs}`
- Default driver hooks:
  - dispatch: `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
  - default app driver wiring: `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- Best-effort slider `SetValue(NumericValue)` handling: `crates/fret-ui-app/src/accessibility_actions.rs`
- Ecosystem adoption:
  - slider: `ecosystem/fret-ui-shadcn/src/slider.rs`
  - progress: `ecosystem/fret-ui-shadcn/src/progress.rs`
- Gates:
  - mechanism gating: `crates/fret-ui/src/tree/tests/semantics_slider_set_value_gate.rs`
  - convergence unit tests: `crates/fret-ui-app/src/accessibility_actions.rs`
  - shadcn E2E: `ecosystem/fret-ui-shadcn/src/slider.rs`

## Alternatives considered

1. **Keep range values as formatted strings only.**
   - Pros: no new contract fields.
   - Cons: brittle for automation; hard to map to platform-native slider/progress semantics; locale/formatting issues.
2. **Add top-level fields to `SemanticsNode` instead of an `extra` bucket.**
   - Pros: direct access; slightly simpler struct.
   - Cons: repeated churn; more disruptive as we expand semantics metadata over time.

## Follow-ups (not required by this ADR)

- Audit `SemanticsRole::Viewport` usage across scroll-like containers vs embedded viewport surfaces; if needed, introduce a
  dedicated scroll-container role in a future ADR.
- Expand numeric/range semantics to additional widgets (spinbutton, scrollbar, splitter) once ownership boundaries are
  clear.
