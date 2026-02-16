# Radix Primitives Audit — Select


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's Radix-aligned select substrate against the upstream Radix
`@radix-ui/react-select` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/select/src/select.tsx`
- Public exports: `repo-ref/primitives/packages/react/select/src/index.ts`

Key upstream concepts:

- `Select` root owns shared state: `open`, `onOpenChange`, `value`, `onValueChange`, ids, and refs.
- Trigger open keys (`OPEN_KEYS`): `Space`, `Enter`, `ArrowUp`, `ArrowDown`.
- Trigger typeahead while closed updates selection without opening.
- Content composes:
  - `Popper.Content` for placement,
  - `FocusScope` + `DismissableLayer` for focus + dismissal,
  - outside interaction blocking + aria hiding while open (`disableOutsidePointerEvents`,
    `hideOthers`, `RemoveScroll`).

## Fret mapping

Fret models Radix Select outcomes by composing:

- Placement: `ecosystem/fret-ui-kit/src/primitives/popper.rs` (+ `popper_content.rs`).
- Dismissal + focus restore/initial focus: modal overlays via `OverlayController`
  (`ecosystem/fret-ui-kit/src/window_overlays/*`).
- Trigger semantics: `SemanticsRole::ComboBox` and `expanded/controls` relationships.
- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/select.rs`.

## Current parity notes

- Pass: Controlled/uncontrolled open modeling is available via `SelectRoot` (recommended) or
  `select_use_open_model(...)` (thin helper), backed by the shared controllable-state substrate.
- Pass: Select can be rendered in a modal overlay layer to block underlay interaction, matching the
  Radix "disable outside pointer events" outcome.
- Pass: Trigger can stamp Radix-like `expanded` + `controls` relationships via
  `apply_select_trigger_a11y(...)`.
- Pass: Trigger open keys + closed-state typeahead policy is exposed via the Radix-named facade.
- Pass: Content open-state key policy (Escape/Home/End/Arrow keys/Enter/Space + typeahead) is
  implemented in `ecosystem/fret-ui-kit/src/primitives/select.rs` and consumed by the shadcn select.
- Pass: Controlled/uncontrolled selection (`value` / `defaultValue`) can be modeled via
  `select_use_value_model(...)`.
- Pass: Pointer modality details are mapped: mouse opens on `pointerdown`, touch/pen open on
  click-like pointer up (movement threshold) to avoid scroll-to-open.
- Pass: Item-aligned positioning (Radix `SelectItemAlignedPosition`) is implemented as reusable
  headless geometry math in `ecosystem/fret-ui-kit/src/headless/select_item_aligned.rs` and is
  available to recipes (shadcn select exposes it via `SelectPosition::ItemAligned`).
- Pass: When the select content is scrollable, opening the listbox performs a one-shot “align active
  option to the top edge” scroll so the first option sits directly under the scroll-up button,
  matching the upstream scrollable select outcome (group labels are scrolled behind the button).
- Pass: Dismissals can be intercepted (Radix `DismissableLayer` "preventDefault" outcome) via
  `modal_select_request_with_dismiss_handler(...)` and `select_modal_barrier_with_dismiss_handler(...)`.
  When provided, the handler receives the `DismissReason` and can decide whether to close the `open`
  model.

## Follow-ups (recommended)

- Done: Close-on-window-blur/resize is supported in the overlay controller layer and enabled by
  default for Radix Select content overlays.

## Conformance gates

- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs` validates item-aligned select
  placement against the Radix Vega web golden (`goldens/radix-web/v4/radix-vega/select-example.select.open-navigate-select.light.json`).
- Upstream style reference: `repo-ref/ui/apps/v4/public/r/styles/radix-vega/select.json` (`min-w-36`,
  `position=\"item-aligned\"`).
- Run: `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
- `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` validates select scrollable
  placement + panel sizing against shadcn web goldens (including a constrained viewport variant:
  `goldens/shadcn-web/v4/new-york-v4/select-scrollable.vp1440x450.open.json`).
- `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` also gates the scrollable
  select's scroll buttons and the “top inset to first option” outcome via
  `web_vs_fret_select_scrollable_listbox_option_insets_match` (and the small-viewport variant).
