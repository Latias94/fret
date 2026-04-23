# ImUi Collection Rename v1 - Closeout Audit

Status: closed closeout record
Date: 2026-04-23

## Verdict

Treat `imui-collection-rename-v1` as:

- a closeout record for the landed app-owned collection rename slice in
  `apps/fret-examples/src/imui_editor_proof_demo.rs`,
- an explicit reminder that the closed select-all lane stays closed,
- and a reminder that second-proof-surface / broader collection helper pressure still requires
  different narrower follow-ons.

## What shipped

1. The collection-first asset browser proof now supports app-owned rename breadth.
2. F2 and the existing context-menu action open a rename modal for the active collection asset.
3. Rename commit updates the proof-local visible label while preserving stable ids and collection order.
4. The popup/input seams stay product-owned.
5. Source-policy and unit-test gates now keep the slice visible in repo-first reopen flows.

## What did not ship

1. No new shared collection rename helper in `fret-ui-kit::imui`.
2. No runtime contract changes.
3. No generic inline-edit surface.
4. No second proof surface.
5. No reopening of the closed select-all lane or wider generic key-owner/helper questions.

## Reopen policy

Start a different narrower follow-on only if stronger first-party proof shows either:

- a second real surface now needs shared collection action/helper growth,
- the remaining pressure is primarily a broader collection command package,
- or product depth now clearly exceeds this bounded rename slice.

No reopening of the closed select-all lane or wider generic key-owner/helper questions.
