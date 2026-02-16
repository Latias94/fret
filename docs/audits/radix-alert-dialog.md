# Radix Primitives Audit — Alert Dialog


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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

- Pass: Controlled/uncontrolled open modeling is available via
  `primitives::alert_dialog::alert_dialog_use_open_model(...)` (backed by the shared
  controllable-state substrate).
- Pass: Alert dialog is modal (barrier-backed) and scopes focus traversal (ADR 0068).
- Pass: Overlay click-to-dismiss is disabled by default in the shadcn recipe (Radix safety outcome).
- Pass: Initial focus prefers the first registered `Cancel` action when present, via
  `primitives::alert_dialog` registry.
- Pass: Radix prevents `interact outside` dismissal at the primitive layer; in Fret, modal
  outside-press dismissal is implemented by the modal barrier policy. AlertDialog exposes the same
  safety default via `primitives::alert_dialog::alert_dialog_modal_barrier(...)`, which is always
  non-dismissable by outside press.

## Follow-ups (recommended)

- Consider exposing an explicit `AlertDialogOptions` builder in `primitives::alert_dialog` if
  non-shadcn consumers need the same safety defaults without adopting the shadcn recipe layer.

## Conformance gate

- Radix Web overlay geometry parity: `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`
  (`radix_web_alert_dialog_open_geometry_matches_fret`).
