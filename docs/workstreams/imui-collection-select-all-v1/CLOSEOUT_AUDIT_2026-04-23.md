# ImUi Collection Select-All v1 - Closeout Audit

Status: closed closeout record
Date: 2026-04-23

## Verdict

Treat `imui-collection-select-all-v1` as:

- a closeout record for the landed app-owned collection select-all slice in
  `apps/fret-examples/src/imui_editor_proof_demo.rs`,
- an explicit reminder that the closed zoom lane stays closed,
- and a reminder that broader rename / second-proof-surface pressure still requires different
  narrower follow-ons.

## What shipped

1. The collection-first asset browser proof now supports app-owned select-all breadth.
2. Primary+A selects all visible assets within the focused collection scope.
3. The select-all helper uses current visible order and preserves the current active tile when possible.
4. The popup/menu surface stays unchanged.
5. Source-policy and unit-test gates now keep the slice visible in repo-first reopen flows.

## What did not ship

1. No new shared collection select-all helper in `fret-ui-kit::imui`.
2. No runtime contract changes.
3. No rename or broader command palette integration.
4. No second proof surface.
5. No reopening of the closed zoom lane or wider generic key-owner/helper questions.

## Reopen policy

Start a different narrower follow-on only if stronger first-party proof shows either:

- a second real surface now needs shared collection action/helper growth,
- the remaining pressure is primarily rename or richer command breadth,
- or broader collection product depth dominates more than select-all closure.

No reopening of the closed zoom lane or wider generic key-owner/helper questions.
