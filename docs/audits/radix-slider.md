# Radix Primitives Audit — Slider

This audit compares Fret's Radix-aligned slider substrate against the upstream Radix
`@radix-ui/react-slider` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/slider/src/slider.tsx`
- Public exports: `repo-ref/primitives/packages/react/slider/src/index.ts`

Key upstream concepts:

- `Slider` supports one or multiple thumbs (array of values).
- `Slider` exposes accessible range/value via `role="slider"` and value attributes.
- Pointer + keyboard interactions clamp and snap values to steps.

## Fret mapping

- Headless math helpers: `ecosystem/fret-ui-kit/src/headless/slider.rs`.
- Wiring helper (single-thumb): `ecosystem/fret-ui-kit/src/declarative/slider.rs`.
- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/slider.rs`.

## Current parity notes

- Pass: Normalization/snap behavior is reusable via `headless::slider`.
- Pass: shadcn `Slider` uses `primitives::slider` for semantics value formatting and pointer update.
- Pass: Headless multi-thumb modeling is available (`closest_value_index`, sorting, minimum steps
  between thumbs).
- Partial: Higher-level recipes still use the single-thumb surface; multi-thumb pointer wiring
  requires tracking `value_index_to_change` across drag (Radix `valueIndexToChangeRef`).
