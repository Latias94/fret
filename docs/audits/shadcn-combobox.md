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
- Pass: Open lifecycle callback is exposed via `Combobox::on_open_change`
  (Base UI `onOpenChange`).

### Placement & sizing

- Pass: Overlay placement and dismissal are delegated to `Popover` (portal-like per-window overlay
  roots + outside-press dismissal).
- Pass: By default, content width tracks the trigger width (via `Popover::into_element_with_anchor(...)`);
  recipes can opt into a fixed width via `Combobox::width(Px(...))` to match upstream demos
  (`w-[200px]`).

### Keyboard & focus

- Pass: On open, focus moves into the popover (via `Popover::auto_focus(true)`), enabling keyboard
  navigation inside the command list.
- Pass: `CommandPalette` now supports input placeholder (backed by the `TextInput` placeholder
  surface in `fret-ui`), so recipes can match `CommandInput placeholder="..."` ergonomics.
- Pass: When used by `Combobox`, the command input is exposed as a `ComboBox` semantics role (so the
  focused editable surface reports combobox semantics instead of only `TextField`).
- Pass: `Combobox` exposes `search_placeholder(...)` and forwards it to
  `CommandPalette::placeholder(...)`.

### Visual parity (shadcn)

- Pass: Popover content padding matches the demo (`p-0`).
- Pass: Trigger styling matches the upstream intent (`Button` `variant="outline"` + `role="combobox"`)
  using outline-button-like tokens while preserving `SemanticsRole::ComboBox` + `expanded` semantics.

### Filtering semantics

- Pass: Filtering and ranking are implemented in `CommandPalette` via the shared cmdk-style scoring
  helper (`fret-ui-kit::headless::cmdk_score`). Ungrouped items are sorted by score and groups are
  sorted by their highest item score (cmdk `sort()` semantics). `CommandItem.value` participates as
  an alias.
- Pass: `CommandItem.keywords([...])` is supported, aligning with cmdk’s `keywords` taxonomy.
- Pass: Highlight selection tracks `CommandItem.value` (stable across list reorder/filtering).
- Pass: Default rows can render cmdk-style match highlighting via `cmdk_score::command_match_ranges`.

## Validation

- `cargo check -p fret-ui-shadcn`
- Contract test: `combobox_open_change_builder_sets_handler`
- Contract test: `combobox_open_change_event_emits_only_on_state_change`
- Highlighted option chrome gates (hover/highlight background + text color): `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_combobox_demo_highlighted_option_chrome_matches_web`, `web_vs_fret_combobox_demo_highlighted_option_chrome_matches_web_dark`).
- Highlight-state golden: `goldens/shadcn-web/v4/new-york-v4/combobox-demo.highlight-first.open.json`

## Follow-ups (recommended)

_None tracked at this time._
