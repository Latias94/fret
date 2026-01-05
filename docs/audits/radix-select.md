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

- Pass: Select can be rendered in a modal overlay layer to block underlay interaction, matching the
  Radix "disable outside pointer events" outcome.
- Pass: Trigger can stamp Radix-like `expanded` + `controls` relationships via
  `apply_select_trigger_a11y(...)`.
- Partial: Typeahead + roving navigation is implemented in the shadcn layer today; the facade
  currently focuses on stable a11y + overlay wiring entry points.

## Follow-ups (recommended)

- Downshift trigger open-keys + closed-state typeahead policy into `primitives::select` so non-shadcn
  consumers can reuse the same behavior outcomes.

