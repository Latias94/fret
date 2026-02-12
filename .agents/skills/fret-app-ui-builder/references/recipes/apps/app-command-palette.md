# Recipe: Command palette (cmdk-style)

Goal: build a command palette surface that feels like shadcn/Radix (keyboard-first, a11y-friendly),
and is regression-gated.

## Building blocks

- Surface: shadcn `CommandDialog` / `CommandPalette` family (Fret: `fret-ui-shadcn`).
- Semantics: listbox + options, active descendant (when focus stays in the input).
- Overlay rules: see `../../mind-models/mm-overlays-and-focus.md`.

## Checklist

- Input keeps focus while arrow keys move the active item (active descendant semantics).
- Typeahead/filtering is stable and doesn’t allocate excessively.
- Escape closes; focus restores to trigger.
- Provide `test_id` for: trigger, input, list root, first item, disabled item.

## Regression gates (recommended)

1. Unit test: asserts active-descendant semantics updates when navigating.
2. `fretboard diag` script:
   - open palette
   - type text
   - ArrowDown/ArrowUp
   - capture bundle (+ screenshot if needed)
