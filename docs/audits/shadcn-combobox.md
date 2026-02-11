# shadcn/ui v4 Audit - Combobox

This audit compares Fret's shadcn-aligned `Combobox` against upstream shadcn/ui v4 recipes and
Base UI combobox lifecycle semantics.

## Upstream references (source of truth)

- shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/combobox.mdx`
- shadcn demo recipe: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/combobox-demo.tsx`
- Base UI contract: `repo-ref/base-ui/packages/react/src/combobox/root/AriaCombobox.tsx`

## Fret implementation anchors

- Component code: `ecosystem/fret-ui-shadcn/src/combobox.rs`
- Building blocks:
  - `ecosystem/fret-ui-shadcn/src/popover.rs`
  - `ecosystem/fret-ui-shadcn/src/command.rs`

## Audit checklist

### Composition model (Popover + Command)

- Pass: `Combobox` is implemented as a Popover + Command recipe (cmdk-style active descendant behavior).
- Pass: Item selection is wired through `CommandItem::on_select_action(...)` (pointer + keyboard enter).
- Pass: Supports controllable/uncontrollable construction for both selected value and open state.

### Base UI lifecycle parity

- Pass: `Combobox::on_value_change(...)` is exposed (Base UI `onValueChange`).
- Pass: `Combobox::on_open_change(...)` is exposed (Base UI `onOpenChange`).
- Pass: `Combobox::on_open_change_with_reason(...)` is exposed for reason-aware callbacks
  (`TriggerPress` / `OutsidePress` / `ItemPress` / `EscapeKey` / `FocusOut` / `None`).
- Pass: `Combobox::on_open_change_complete(...)` is exposed (Base UI `onOpenChangeComplete`).
- Pass: Value/open callbacks are edge-triggered (no duplicate emission on unchanged state).

### Placement, keyboard, and visual defaults

- Pass: Overlay placement and dismissal are delegated to `Popover` (portal-like behavior).
- Pass: Trigger/content width behavior matches recipe intent (default track-trigger width; optional fixed width).
- Pass: Opening auto-focuses the command input, and combobox semantics are reported on the editable surface.
- Pass: Trigger chrome aligns with shadcn outline-button intent while preserving combobox semantics.

## Known gaps

- Gap: Base UI callbacks include cancellable event details; Fret currently exposes reason metadata
  but does not provide cancelable `eventDetails` contracts.

## Validation

- `cargo nextest run -p fret-ui-shadcn combobox_on_value_change_builder_sets_handler`
- `cargo nextest run -p fret-ui-shadcn combobox_open_change_builder_sets_handler`
- `cargo nextest run -p fret-ui-shadcn combobox_open_change_with_reason_builder_sets_handler`
- `cargo nextest run -p fret-ui-shadcn combobox_open_change_complete_builder_sets_handler`
- `cargo nextest run -p fret-ui-shadcn combobox_value_change_event_emits_only_on_state_change`
- `cargo nextest run -p fret-ui-shadcn combobox_open_change_events_emit_change_and_complete_after_settle`
- `cargo nextest run -p fret-ui-shadcn combobox_open_change_events_complete_without_animation`
- `cargo nextest run -p fret-ui-shadcn combobox_open_change_reason_maps_dismiss_reasons`
