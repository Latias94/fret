# Radix Primitives Audit — Select

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
- Pass: Pointer modality details are mapped: mouse opens on `pointerdown`, touch/pen open on
  click-like pointer up (movement threshold) to avoid scroll-to-open.
- Pass: Item-aligned positioning (Radix `SelectItemAlignedPosition`) is implemented as reusable
  headless geometry math in `ecosystem/fret-ui-kit/src/headless/select_item_aligned.rs` and is
  available to recipes (shadcn select exposes it via `SelectPosition::ItemAligned`).

## Follow-ups (recommended)

- Done: Close-on-window-blur/resize is supported in the overlay controller layer and enabled by
  default for Radix Select content overlays.
