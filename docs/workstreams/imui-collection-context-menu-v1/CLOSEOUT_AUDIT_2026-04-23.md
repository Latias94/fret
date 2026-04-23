# ImUi Collection Context Menu v1 - Closeout Audit

Status: closed closeout record
Date: 2026-04-23

## Verdict

Treat `imui-collection-context-menu-v1` as:

- a closeout record for the landed app-owned collection context-menu slice in
  `apps/fret-examples/src/imui_editor_proof_demo.rs`,
- an explicit reminder that the closed delete-action lane and generic menu/key-owner verdicts stay
  closed,
- and a reminder that broader select-all / rename / helper-widening pressure still requires
  different narrower follow-ons.

## What shipped

1. The collection-first asset browser proof now supports app-owned context-menu quick actions.
2. Right-click on assets/background routes through one shared popup scope.
3. Item context adopts an unselected asset into selection before opening the popup.
4. Delete from the popup reuses the existing delete helper and reflow policy.
5. Source-policy and unit-test gates now keep the slice visible in repo-first reopen flows.

## What did not ship

1. No new shared collection context-menu helper in `fret-ui-kit::imui`.
2. No runtime contract changes.
3. No select-all, rename, or broader command palette integration.
4. No lasso / freeform drag-rectangle policy.
5. No reopening of the closed delete-action lane or the generic menu/key-owner lanes.

## Reopen policy

Start a different narrower follow-on only if stronger first-party proof shows either:

- a second real surface now needs shared collection context-menu/helper growth,
- the remaining pressure is primarily select-all / rename / richer command breadth,
- or lasso / broader collection depth dominates more than context-menu quick actions.
