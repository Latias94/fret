# ImUi Collection Inline Rename v1 - Closeout Audit

Status: closed closeout record
Date: 2026-04-23

Status note (2026-04-23): this closeout remains the record for the original inline-rename slice.
The current shipped trigger breadth is broader and now also includes an explicit rename button in
`docs/workstreams/imui-collection-command-package-v1/M2_APP_OWNED_RENAME_TRIGGER_SLICE_2026-04-23.md`.
References below to `F2` / context-menu trigger breadth should therefore be read as the bounded
landed slice for this lane, not the full current trigger roster.

## Verdict

Treat `imui-collection-inline-rename-v1` as:

- a closeout record for the landed app-owned collection inline rename slice in
  `apps/fret-examples/src/imui_editor_proof_demo.rs`,
- an explicit reminder that the closed modal rename lane stays closed,
- and a reminder that second-proof-surface / broader collection helper pressure still requires
  different narrower follow-ons.

## What shipped

1. The collection-first asset browser proof now supports app-owned inline rename depth.
2. F2 and the existing context-menu action start an inline editor inside the active asset tile.
3. The inline editor restores focus back to the existing collection proof after commit/cancel.
4. Rename commit still updates only the visible label while preserving stable ids and collection order.
5. Source-policy, unit-test, and surface gates now keep the slice visible in repo-first reopen flows.

## What did not ship

1. No new shared collection inline-edit helper in `fret-ui-kit::imui`.
2. No runtime contract changes.
3. No second proof surface.
4. No broader collection command package.
5. No reopening of the closed modal rename lane or wider generic key-owner/helper questions.

## Reopen policy

Start a different narrower follow-on only if stronger first-party proof shows either:

- a second real surface now needs shared collection inline-edit/helper growth,
- the remaining pressure is primarily a broader collection command package,
- or product depth now clearly exceeds this bounded inline rename slice.

No reopening of the closed modal rename lane or wider generic key-owner/helper questions.
