# ImUi Collection Delete Action v1 - Closeout Audit

Status: closed closeout record
Date: 2026-04-22

## Verdict

Treat `imui-collection-delete-action-v1` as:

- a closeout record for the landed app-owned collection delete-selected slice in
  `apps/fret-examples/src/imui_editor_proof_demo.rs`,
- an explicit reminder that the closed keyboard-owner lane and generic key-owner verdict stay
  closed,
- and a reminder that broader collection action semantics and shared helper widening still require
  different narrower follow-ons.

## What shipped

1. The collection-first asset browser proof now supports app-owned delete-selected semantics.
2. `Delete` / `Backspace` and the explicit button both route through one proof-local delete helper.
3. Remaining assets, selection, and keyboard active tile reflow explicitly after deletion.
4. Source-policy and unit-test gates now keep the slice visible in repo-first reopen flows.

## What did not ship

1. No new shared collection action helper in `fret-ui-kit::imui`.
2. No runtime contract changes.
3. No select-all, rename, or broader command palette integration.
4. No lasso / freeform drag-rectangle policy.
5. No reopening of the generic key-owner lane or the closed keyboard-owner folder.

## Reopen policy

Start a different narrower follow-on only if stronger first-party proof shows either:

- a second real surface now needs shared collection action helper growth,
- the remaining pressure is primarily select-all / rename / context menu command breadth,
- or lasso / broader collection depth dominates more than delete-selected semantics.
