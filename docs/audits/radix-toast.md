# Radix Primitives Audit — Toast

This audit compares Fret's toast substrate against the upstream Radix
`@radix-ui/react-toast` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/toast/src/toast.tsx`
- Public exports: `repo-ref/primitives/packages/react/toast/src/index.ts`

Key upstream concepts:

- `ToastProvider` defines global defaults (duration, swipe direction/threshold) and tracks the
  viewport element.
- `ToastViewport` is the focus target for the "jump to notifications" hotkey and is responsible
  for pausing/resuming the close timers based on focus/hover.
- `Toast` roots are controlled/uncontrolled `open` states with close timers (pause/resume).
- Swipe gesture closes the toast in the configured direction.

## Fret mapping

Fret does not use React context nor DOM events. Toast outcomes are composed via:

- Toast store + timers: `ecosystem/fret-ui-kit/src/window_overlays/toast.rs`
- Overlay install/render substrate: `ecosystem/fret-ui-kit/src/window_overlays/*`
- Radix-named primitives facade: `ecosystem/fret-ui-kit/src/primitives/toast.rs`
- shadcn recipe surface (`Sonner`): `ecosystem/fret-ui-shadcn/src/sonner.rs`

## Current parity notes

- Pass: Upsert-by-id behavior (Radix `open` roots can be updated while present).
- Pass: Auto-close timers, hover pause, and close transition removal are implemented in the store.
- Pass: Swipe-to-dismiss supports Radix-aligned `swipeDirection` + `swipeThreshold` defaults.
- Pass: Per-window max-toasts limiting is supported.
- Pass: A reusable, Radix-named viewport root exists via `ToastViewport` (overlay-backed).

## Gaps / intentional differences

- Not implemented: Radix `ToastViewport` hotkey focus-jump behavior.
- Deferred: Radix `ToastViewport` hotkey binding surface (default `F8`) and focus-jump behavior.
- Deferred: A11y announcement semantics (Radix uses ARIA live region patterns).
