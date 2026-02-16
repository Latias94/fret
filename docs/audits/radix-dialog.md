# Radix Primitives Audit — Dialog


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's Radix-aligned dialog substrate against the upstream Radix
`@radix-ui/react-dialog` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/dialog/src/dialog.tsx`
- Public exports: `repo-ref/primitives/packages/react/dialog/src/index.ts`

Key upstream concepts:

- `Dialog` root owns shared state: `open`, `onOpenChange`, and generated ids for `content`, `title`,
  and `description`.
- `DialogTrigger` toggles open and stamps `aria-expanded` + `aria-controls`.
- `DialogPortal` + `Presence` implement conditional mounting / `forceMount`.
- `DialogContent` composes:
  - `FocusScope` (trap focus when modal),
  - `DismissableLayer` (escape/outside/focus-outside dismissal),
  - aria hiding + scroll lock when modal.

## Fret mapping

Fret does not use React context. Instead, dialog behavior is composed via:

- Runtime mechanisms: `crates/fret-ui` (focus traversal, hit-testing, semantics snapshot).
- Overlay portal + dismissal + focus restore/initial focus: `ecosystem/fret-ui-kit/src/window_overlays/*`
  via `OverlayController` (ADR 0067, ADR 0068, ADR 0069).
- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/dialog.rs`.
- shadcn skin/recipes: `ecosystem/fret-ui-shadcn/src/dialog.rs`.

## Current parity notes

- Pass: Controlled/uncontrolled open modeling is available via `DialogRoot` (recommended) or
  `dialog_use_open_model(...)` (thin helper), backed by the shared controllable-state substrate.
- Pass: Modal focus traversal is scoped to the modal barrier layer (ADR 0068).
- Pass: Escape dismiss is handled by the shared dismissible root used by modal overlays.
- Pass: Dismissals can be intercepted (Radix `DismissableLayer` "preventDefault" outcome) via
  `modal_dialog_request_with_options_and_dismiss_handler(...)` and
  `modal_barrier_with_dismiss_handler(...)`. Callers choose whether to close the `open` model.
- Pass: Trigger can stamp Radix-like `expanded` + `controls` relationships via
  `apply_dialog_trigger_a11y(...)`.
- Pass: Modal outside-click dismissal is authored via a Radix-named primitive helper
  (`primitives::dialog::modal_barrier(...)`), keeping the policy reusable outside the shadcn layer.
  This still uses a barrier element (rather than the outside-press observer pass), which is
  intentional: the observer is click-through for non-modal overlays in Fret.
- Note: Title/description presence warnings are not currently modeled (Radix dev warnings).

## Follow-ups (recommended)

- None currently tracked.

## Conformance gate

- Radix Web overlay geometry parity: `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`
  (`radix_web_dialog_open_geometry_matches_fret`).
