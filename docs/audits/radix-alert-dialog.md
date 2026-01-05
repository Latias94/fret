# Radix Primitives Audit — Alert Dialog

This audit compares Fret's Radix-aligned alert-dialog substrate against the upstream Radix
`@radix-ui/react-alert-dialog` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/alert-dialog/src/alert-dialog.tsx`
- Public exports: `repo-ref/primitives/packages/react/alert-dialog/src/index.ts`

Key upstream concepts:

- AlertDialog is a constrained Dialog:
  - always modal,
  - prevents outside interactions from dismissing,
  - prefers focusing the `Cancel` action on open.
- It composes `@radix-ui/react-dialog` and overrides content interaction handlers to enforce the
  safety defaults.

## Fret mapping

Fret models alert-dialog outcomes as a modal overlay recipe with safety defaults:

- Modal overlay infrastructure: `ecosystem/fret-ui-kit/src/window_overlays/*` via `OverlayController`.
- Focus preference for `Cancel`: `ecosystem/fret-ui-kit/src/primitives/alert_dialog.rs`.
- shadcn skin/recipes: `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`.

## Current parity notes

- Pass: Alert dialog is modal (barrier-backed) and scopes focus traversal (ADR 0068).
- Pass: Overlay click-to-dismiss is disabled by default in the shadcn recipe (Radix safety outcome).
- Pass: Initial focus prefers the first registered `Cancel` action when present, via
  `primitives::alert_dialog` registry.
- Partial: Radix also prevents `interact outside` dismissal at the primitive layer; in Fret, modal
  outside-press dismissal is an opt-in recipe behavior (barrier click handler), so alert-dialog
  achieves the same outcome by not installing a closable barrier.

## Follow-ups (recommended)

- Consider exposing an explicit `AlertDialogOptions` builder in `primitives::alert_dialog` if
  non-shadcn consumers need the same safety defaults without adopting the shadcn recipe layer.

