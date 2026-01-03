# shadcn/ui v4 Audit — Combobox

This audit compares Fret’s shadcn-aligned `Combobox` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/combobox.mdx`
- Example implementation: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/combobox-demo.tsx`
- Key note: upstream “Combobox” is a **recipe** composed from `Popover` + `Command`, not a dedicated
  primitive.

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/combobox.rs`
- Reused building blocks:
  - `ecosystem/fret-ui-shadcn/src/popover.rs`
  - `ecosystem/fret-ui-shadcn/src/command.rs`

## Audit checklist

### Composition model (Popover + Command)

- Pass: `Combobox` now renders as a `Popover` overlay containing a `CommandPalette` (cmdk-style,
  active-descendant navigation).
- Pass: Item selection is wired through `CommandItem::on_select_action(...)`, so selection works
  both via pointer click and `Enter` on the active item.

### Placement & sizing

- Pass: Overlay placement and dismissal are delegated to `Popover` (portal-like per-window overlay
  roots + outside-press dismissal).
- Partial: Content width tracks trigger width (via `Popover::into_element_with_anchor(...)`), which
  is convenient but slightly different from the upstream demo where width is explicitly set on both
  trigger and content (`w-[200px]`).

### Keyboard & focus

- Pass: On open, focus moves into the popover (via `Popover::auto_focus(true)`), enabling keyboard
  navigation inside the command list.
- Pass: `CommandPalette` now supports input placeholder (backed by the `TextInput` placeholder
  surface in `fret-ui`), so recipes can match `CommandInput placeholder="..."` ergonomics.
- Pass: `Combobox` exposes `search_placeholder(...)` and forwards it to
  `CommandPalette::placeholder(...)`.

### Visual parity (shadcn)

- Partial: Popover content padding is aligned to the demo (`p-0`) by rendering the list inside
  `PopoverContent` with `p-0`.
- Partial: The trigger is styled via input chrome tokens; upstream uses `Button` `variant="outline"`
  with `role="combobox"`.

### Filtering semantics

- Partial: Filtering is currently label substring match implemented in `Combobox` (app-owned), not
  cmdk’s full `value`/ranking semantics.
- Pass: Highlight selection tracks `CommandItem.value` (stable across list reorder/filtering).

## Validation

- `cargo check -p fret-ui-shadcn`

## Follow-ups (recommended)

- Implement cmdk-style filtering/ranking (value/keywords + scoring) so query handling can move into
  `CommandPalette` instead of per-recipe filtering in `Combobox`.
- Consider exposing a `CommandItem` “indicator/checked” surface (and `CommandShortcut`) to better
  match upstream `cmdk` DOM patterns.
