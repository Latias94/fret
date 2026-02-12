# Component recipe: Dialog / Sheet

Goal: modal/overlay surfaces with correct focus trapping/restoration, dismiss rules, and predictable layering.

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/dialog, https://ui.shadcn.com/docs/components/sheet
- shadcn source (v4 New York registry):
  - Dialog: https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/dialog.tsx
  - Sheet: https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/sheet.tsx
- Radix docs: https://www.radix-ui.com/primitives/docs/components/dialog
- Radix source: https://github.com/radix-ui/primitives/tree/main/packages/react/dialog/src
- Local pinned snapshot (optional; not necessarily present on GitHub checkouts):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dialog.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sheet.tsx`
  - `repo-ref/primitives/packages/react/dialog/src/*`

## Fret building blocks

- Dialog: `fret-ui-shadcn::Dialog`, `DialogContent`, `DialogHeader`, ...
- Sheet: `fret-ui-shadcn::Sheet`, `SheetContent`, `SheetSide`, ...
- Model:
  - `Model<bool>` for open/closed
- Example usage: `apps/fret-ui-gallery/src/docs.rs` (Overlay docs/usage snippet)

## Checklist (what to verify)

- Modal focus:
  - initial focus goes to the right control
  - Tab/Shift+Tab are trapped within the dialog (when modal)
  - close restores focus to trigger
- Dismiss:
  - Escape closes (unless prevented)
  - overlay click closes (if overlay-closable)
  - close button closes
- Layering:
  - nested dialogs/menus behave correctly
  - underlay interaction is blocked when modal

## Regression gates (recommended)

- Prefer unit tests for focus trap/restore invariants.
- Add scripted repros for overlay click + escape + focus restore, and capture a bundle after each state.

## See also

- `fret-shadcn-source-alignment` (when matching upstream behavior)
- `fret-diag-workflow` (scripted repro + packaging)
