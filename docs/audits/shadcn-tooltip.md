# shadcn/ui v4 Audit - Tooltip

This audit compares Fret's shadcn-aligned `Tooltip` against upstream shadcn/ui v4 recipes,
Radix semantics, and Base UI `Tooltip.Root` lifecycle behavior.

## Upstream references (source of truth)

- shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/tooltip.mdx`
- shadcn registry (new-york-v4): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tooltip.tsx`
- Base UI root contract: `repo-ref/base-ui/packages/react/src/tooltip/root/TooltipRoot.tsx`

## Fret implementation anchors

- Component code: `ecosystem/fret-ui-shadcn/src/tooltip.rs`
- Hover intent and delay group: `ecosystem/fret-ui-kit/src/headless/hover_intent.rs`
- Overlay policy and placement helpers: `ecosystem/fret-ui-kit/src/overlay.rs`

## Audit checklist

### Composition & behavior surface

- Pass: Declarative `Tooltip`, `TooltipTrigger`, `TooltipContent`, and `TooltipProvider` are present.
- Pass: Supports open on hover and keyboard focus.
- Pass: Supports provider close delay aliases (`close_delay_duration_ms`, `close_delay_duration_frames`).
- Pass: Supports Base UI cursor tracking via `Tooltip::track_cursor_axis(...)`.

### Open lifecycle semantics (Base UI parity)

- Pass: `Tooltip::on_open_change(...)` emits when open state changes.
- Pass: `Tooltip::on_open_change_with_reason(...)` emits open state changes with reason metadata
  (`TriggerHover` / `TriggerFocus` / `TriggerPress` / `OutsidePress` / `FocusOutside` /
  `EscapeKey` / `Scroll` / `None`).
- Pass: `Tooltip::on_open_change_complete(...)` emits when transition settles
  (`present == open && !animating`).
- Pass: Callbacks are edge-triggered (no duplicate events when state is unchanged).

### Placement & visual defaults

- Pass: Content is rendered through overlay root (portal-like behavior).
- Pass: Supports side/align/offset and arrow defaults aligned with shadcn usage.
- Pass: Motion taxonomy aligns with shadcn expectations (fade/zoom/slide with transform origin).

## Known gaps

- Gap: Base UI `onOpenChange` includes cancellable event details (`isCanceled`, `preventUnmountOnClose`);
  Fret currently exposes non-cancelable callback metadata.

## Validation

- `cargo nextest run -p fret-ui-shadcn tooltip_open_change_handlers_can_be_set`
- `cargo nextest run -p fret-ui-shadcn tooltip_open_change_events_emit_change_and_complete_after_settle`
- `cargo nextest run -p fret-ui-shadcn tooltip_open_change_reason_mapping_covers_dismiss_reasons`
- `cargo nextest run -p fret-ui-shadcn tooltip_open_change_reason_for_transition_uses_trigger_and_close_reason`
- Overlay layout and chrome parity continue to be validated in
  `web_vs_fret_overlay_placement` / `web_vs_fret_overlay_chrome` tooltip gates.
