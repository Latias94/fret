# Material field-family checklist

Use this checklist for Material field-family work:

- `TextField`
- `Select`
- `Autocomplete`
- `ExposedDropdown`
- `DatePicker`
- `TimePicker`

These components often look “almost right” while still drifting in the most expensive places:
state ownership, popup choreography, floating labels, and accessibility wiring.

Goal: catch high-ROI parity issues before you spend time on token tweaks.

## 1) State ownership and committed value

Decide ownership first:

- What is the **committed value**?
- What is the **editable query** (if any)?
- When does the query synchronize from the committed value?
- When does user input update the committed value?
- Does blur restore display text, preserve partial input, or keep the selected label?

Start points:

- `ecosystem/fret-ui-material3/src/select.rs`
- `ecosystem/fret-ui-material3/src/exposed_dropdown.rs`
- `ecosystem/fret-ui-material3/src/autocomplete.rs`
- `ecosystem/fret-ui-material3/src/text_field.rs`

## 2) Trigger / input semantics

Verify the input surface contract:

- Is the trigger/input exposed as the right semantic role?
- Does it report `expanded` correctly?
- Does it expose the popup relationship via `controls` / `labelled_by` / `described_by` when applicable?
- Is keyboard focus on the correct element when opening/closing?
- If this is a listbox-style popup, does the popup expose `active_descendant` correctly?

For field-family overlays, always verify trigger/input ↔ popup semantics together.

## 3) Floating label choreography

Check the complete lifecycle, not just the final positions:

- resting empty state,
- focused state,
- open state,
- populated state,
- disabled/error states,
- transition timing and interruption behavior.

Common drift:

- label floats only on focus but not on open,
- label snaps instead of animating consistently,
- error/disabled colors do not override correctly.

Start points:

- `ecosystem/fret-ui-material3/src/foundation/floating_label.rs`
- `ecosystem/fret-ui-material3/src/tokens/text_field.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-text-field-hover-label-color-expressive-screenshots.json`

## 4) Container chrome and active indicator

Check these outcomes together:

- container height,
- outline/filled container background,
- active indicator presence and thickness,
- error overrides,
- focused/hovered/pressed/disabled chroming,
- icon color and opacity overrides.

Material drift often comes from one part reading the wrong token fallback chain.

Start points:

- `ecosystem/fret-ui-material3/src/tokens/select.rs`
- `ecosystem/fret-ui-material3/src/tokens/text_field.rs`
- `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`

## 5) Supporting text, error text, and icons

Verify content ownership:

- supporting text stays attached to the right field,
- error text overrides supporting text styling correctly,
- leading/trailing icons preserve spacing and a11y labels,
- trailing dropdown icons and clear/search affordances do not steal focus unexpectedly.

Check whether icon events are routed through the correct surface and whether they need independent `test_id`s.

## 6) Popup policy and width behavior

For `Select`, `Autocomplete`, and `ExposedDropdown`, verify:

- popup opens from the correct interaction,
- popup stays within the viewport/window,
- width floor is correct,
- content may expand when the recipe expects it,
- collision and transform behavior do not break hit-testing,
- dialog-vs-anchored presentation rules are explicit when both modes exist.

Start points:

- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-menu-width-floor-screenshots.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-menu-positioning-transform-screenshots.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-exposed-dropdown-filtering.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-autocomplete-filtering.json`

## 7) Filtering, typeahead, and open choreography

Check the exact interaction model:

- `open_on_focus` behavior,
- query filtering behavior,
- selection-on-click / selection-on-enter,
- typeahead delay,
- blur synchronization,
- close-on-select behavior,
- focus restore after close.

Start points:

- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-typeahead-delay.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-autocomplete-filtering.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-exposed-dropdown-filtering.json`

## 8) Stable automation surfaces

Before adding diag coverage, make sure the recipe exposes stable, intent-level selectors:

- field root / trigger / input,
- popup / listbox / dialog surface,
- representative option/item ids,
- trailing icon or secondary affordance when it drives behavior.

Avoid selectors based only on position or list index.

## 9) Recommended gates

Pick the smallest gate that proves the specific parity outcome:

- logic/state ownership → focused unit test near the component,
- field chrome / geometry → deterministic scene or geometry assertions,
- popup choreography / filtering / typeahead / focus flow → `tools/diag-scripts/*.json`,
- a11y contract → semantics assertions or a11y bundle capture.

Useful existing gate shapes:

- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-a11y-parity-bundle.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-item-chrome-fill.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-rich-options-screenshots.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-autocomplete-option-chrome-fill.json`

## 10) Common failure signatures

- Popup looks correct, but focus/selection semantics are wrong.
- Query and committed selection diverge after blur.
- Width floor regresses when long options appear.
- Floating label is correct in one state but not across the full lifecycle.
- Supporting text/error text attaches to the wrong visual owner.
- A trailing icon closes/opens the popup but has no stable automation target.
- The visual transform looks right, but hit-testing or outside-press behavior drifts.
