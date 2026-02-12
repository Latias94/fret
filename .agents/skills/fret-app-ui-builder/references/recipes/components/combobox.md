# Component recipe: Combobox

Goal: a shadcn-style combobox (searchable select) with correct a11y semantics and Radix-like overlay behavior.

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/combobox
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/combobox.tsx
- shadcn “Command” docs (composition building block): https://ui.shadcn.com/docs/components/command
- shadcn “Command” source: https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/command.tsx
- Radix docs (overlay primitive used by shadcn): https://www.radix-ui.com/primitives/docs/components/popover
- Radix source: https://github.com/radix-ui/primitives/tree/main/packages/react/popover/src

## Fret building blocks

- Component surface: `fret-ui-shadcn::Combobox` + `ComboboxItem`.
- Models:
  - `Model<Option<Arc<str>>>` for the selected value
  - `Model<bool>` for open/closed
- Composition:
  - Popover-like overlay + command palette list (filtering + navigation).
  - Responsive behavior may switch to a drawer-like surface on small viewports (upstream shadcn pattern).

## Checklist (what to verify)

- Dismiss:
  - Escape closes
  - outside press closes (policy-dependent; be explicit)
  - re-clicking trigger closes
- Focus:
  - open puts focus in the search input (if `search_enabled`)
  - close restores focus to trigger
  - keyboard nav does not steal focus from the input if using active-descendant
- Listbox semantics:
  - input uses `aria-expanded`, `aria-controls`, and `aria-activedescendant` when appropriate
  - list items expose disabled/selected state
- Placement:
  - clamped in constrained viewports
  - scrollable results with max-height (tiny window variants)

## `test_id` suggestions

- `combobox-trigger`
- `combobox-content`
- `combobox-search`
- `combobox-item-<value>`

## Regression gates (recommended)

- Scripted repro:
  - open, type filter, arrow down/up, select, assert value and close/focus restore
  - tiny viewport placement clamp (screenshot optional)
- Add a small test for active-descendant semantics or focus restore if that’s the fragile area.

## See also

- `references/mind-models/mm-overlays-and-focus.md`
- `references/mind-models/mm-a11y-and-testid.md`
- `fret-shadcn-source-alignment`
- `fret-diag-workflow`
