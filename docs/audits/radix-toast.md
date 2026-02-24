# Radix Primitives Audit — Toast


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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
- Pass: Hotkey focus-jump can be wired via the `toast.viewport.focus` command (default keymap binds
  `F8`) and `window_overlays::try_handle_window_command(...)` (used by `fret-bootstrap`'s UI driver).

## Gaps / intentional differences

- Pass: Live region semantics are published by the toast viewport overlay root (polite live region), enabling structured
  announcements without string-only conventions.
- Deferred: Rich per-toast announcement policy (e.g. assertive vs polite per variant, atomic announcements, relevance
  semantics) remains ecosystem-owned.
