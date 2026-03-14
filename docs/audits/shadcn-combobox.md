# shadcn/ui v4 Audit - Combobox

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- MUI Base UI: https://github.com/mui/base-ui
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Combobox` against upstream shadcn/ui v4 recipes and
Base UI combobox lifecycle semantics.

## Upstream references (source of truth)

- shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/combobox.mdx`
- shadcn demo recipe: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/combobox-demo.tsx`
- Base UI contract: `repo-ref/base-ui/packages/react/src/combobox/root/AriaCombobox.tsx`

## Fret implementation anchors

- Component code: `ecosystem/fret-ui-shadcn/src/combobox.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/combobox.rs`
- Copyable usage snippet: `apps/fret-ui-gallery/src/ui/snippets/combobox/usage.rs`
- Building blocks:
  - `ecosystem/fret-ui-shadcn/src/popover.rs`
  - `ecosystem/fret-ui-shadcn/src/command.rs`

## Audit checklist

### Authoring surface

- Pass: `Combobox::new(value, open)` already covers the common shadcn recipe path with controlled
  value/open models.
- Pass: `query_model(...)`, `items(...)`, `groups(...)`, `group_separators(...)`, `show_clear(...)`,
  and `auto_highlight(...)` cover the important recipe-level authoring outcomes.
- Pass: root-lane builder steps now exist for `trigger(...)`, `input(...)`, `clear(...)`, and
  `content(...)`, so first-party examples do not need to default to closure-based patch assembly
  just to customize the common recipe root surface.
- Pass: `ComboboxChips` now also exposes matching compact builder steps (`trigger(...)`,
  `input(...)`, `value(...)`, `content(...)`) so the multi-select chips recipe no longer needs a
  separate closure-based default story.
- Pass: `Combobox::into_element_parts(...)` plus `ComboboxTrigger` / `ComboboxInput` expose the
  upstream-shaped recipe patch surface for copyable examples without forcing raw Popover/Command assembly.
- Note: Because this component is intentionally a recipe over `Popover` + `Command`, Fret does not add
  a separate generic `compose()` builder beyond the existing parts-patch surface.

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

### Item modeling (structured metadata)

- Pass: `ComboboxItem::detail(...)` supports structured suffix metadata (e.g. `Next.js (React)`) without forcing callers
  to pre-format richer labels.
- Pass: `ComboboxItem::keywords(...)` is forwarded to cmdk-style filtering via `CommandItem::keywords(...)`.

## Known gaps

- Gap: Base UI callbacks include cancellable event details; Fret currently exposes reason metadata
  but does not provide cancelable `eventDetails` contracts.

## Conclusion

- Result: This component does not currently point to a missing mechanism-layer issue.
- Result: The previous parity gap on the first-party authoring surface was compact default
  storytelling rather than missing mechanism.
- Result: Common recipe customization now lands through the direct root builder chain, while the
  closure-based parts surface remains available for upstream-shaped patch stories.
- Result: That same compact-root posture now also covers the chips variant, so first-party
  `Combobox` examples no longer split into two default authoring stories.
- Result: Follow-up work should focus on concrete behavior regressions or richer recipes rather
  than adding another generic builder family.
- Authoring-lane classification: keep `Combobox` on the direct recipe root/bridge lane.
  `Combobox::new(value, open)` plus the compact direct chain
  `.trigger(...).input(...).clear(...).content(...)` is the default root story and
  `into_element_parts(...)` is the focused upstream-shaped patch seam for trigger/input/content
  customization; do not add a parallel `compose()` lane.

## Validation

- `cargo nextest run -p fret-ui-shadcn combobox_on_value_change_builder_sets_handler`
- `cargo nextest run -p fret-ui-shadcn combobox_open_change_builder_sets_handler`
- `cargo nextest run -p fret-ui-shadcn combobox_open_change_with_reason_builder_sets_handler`
- `cargo nextest run -p fret-ui-shadcn combobox_open_change_complete_builder_sets_handler`
- `cargo nextest run -p fret-ui-shadcn combobox_value_change_event_emits_only_on_state_change`
- `cargo nextest run -p fret-ui-shadcn combobox_open_change_events_emit_change_and_complete_after_settle`
- `cargo nextest run -p fret-ui-shadcn combobox_open_change_events_complete_without_animation`
- `cargo nextest run -p fret-ui-shadcn combobox_open_change_reason_maps_dismiss_reasons`
- `cargo test -p fret-ui-shadcn --lib combobox_builder_steps_apply_the_same_patch_surface -- --exact --nocapture`
- `cargo test -p fret-ui-shadcn --lib combobox_chips_builder_steps_apply_the_same_patch_surface -- --exact --nocapture`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-combobox-custom-items-detail-filter-react.json --launch -- cargo run -p fret-ui-gallery`
- `cargo check -p fret-ui-gallery --message-format short`
