# Radix Primitives Audit — Dialog

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
- Pass: Trigger can stamp Radix-like `expanded` + `controls` relationships via
  `apply_dialog_trigger_a11y(...)`.
- Pass: Modal outside-click dismissal is authored via a Radix-named primitive helper
  (`primitives::dialog::modal_barrier(...)`), keeping the policy reusable outside the shadcn layer.
  This still uses a barrier element (rather than the outside-press observer pass), which is
  intentional: the observer is click-through for non-modal overlays in Fret.
- Note: Title/description presence warnings are not currently modeled (Radix dev warnings).

## Follow-ups (recommended)

- Consider a `DialogOptions` builder in `primitives::dialog` to make focus targets and dismissal
  knobs explicit (e.g. overlay-click closable vs not).
