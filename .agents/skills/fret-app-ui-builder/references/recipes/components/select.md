# Component recipe: Select

Goal: a shadcn/Radix-like Select (listbox in an overlay) that is keyboard-friendly, clamped in small viewports, and regression-gated.

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/select
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/select.tsx
- Radix docs: https://www.radix-ui.com/primitives/docs/components/select
- Radix source: https://github.com/radix-ui/primitives/tree/main/packages/react/select/src

## Fret building blocks

- Component surface: `fret-ui-shadcn::Select`, `SelectItem`, `SelectGroup`, `SelectSeparator`, ...
- Models:
  - `Model<Option<Arc<str>>>` for the selected value
  - `Model<bool>` for the open/closed state
- Example usage: `apps/fret-ui-gallery/src/docs.rs` (Select docs/usage snippet)

## Checklist (what to verify)

- Open/close:
  - trigger click opens
  - outside press / Escape closes
  - selecting an item closes (unless explicitly configured otherwise)
- Keyboard:
  - Arrow keys move focus/active option
  - Typeahead works (when supported)
  - Home/End jumps (when supported)
- Placement and clamping:
  - content stays within viewport on tiny height (scrollable listbox + max-height)
  - wheel scrolling works inside the viewport
- Semantics:
  - listbox role + option roles
  - disabled options are not selectable

## `test_id` suggestions (automation-first)

Add stable IDs so scripts don’t depend on coordinates:

- `select-trigger`
- `select-content`
- `select-viewport`
- `select-item-<value>`

## Regression gates (recommended)

- Scripted repro (start from existing examples):
  - `tools/diag-scripts/ui-gallery-select-wheel-scroll.json`
  - `tools/diag-scripts/ui-gallery-select-open-jitter-screenshots.json` (visual gate when needed)
- Add one invariant test for the most fragile rule (e.g., “content is clamped within viewport”).

## See also

- `fret-shadcn-source-alignment` (when matching upstream behavior)
- `fret-diag-workflow` (scripted repro + packaging)
