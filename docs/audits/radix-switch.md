# Radix Primitives Audit — Switch

This audit compares Fret's Radix-aligned switch substrate against the upstream Radix
`@radix-ui/react-switch` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/switch/src/switch.tsx`
- Public exports: `repo-ref/primitives/packages/react/switch/src/index.ts`

Key upstream concepts:

- `Switch` is a button-like control with `role="switch"` and `aria-checked`.
- `SwitchThumb` is a visual slot and inherits state from context.
- A hidden input is used for form integration; this is DOM-specific.

## Fret mapping

- Semantics role: `fret_core::SemanticsRole::Switch`.
- Checked flag: `checked: Some(bool)` on the semantics node.
- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/switch.rs` (`switch_a11y(...)`).

## Current parity notes

- Pass: A11y stamping helper matches Radix "role + checked" outcomes.
- N/A: HTML form integration (`BubbleInput`) is intentionally not modeled at the primitives layer.

