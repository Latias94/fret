# ImUi Collection Zoom v1 - Closeout Audit

Status: closed closeout record
Date: 2026-04-23

## Verdict

Treat `imui-collection-zoom-v1` as:

- a closeout record for the landed app-owned collection zoom/layout slice in
  `apps/fret-examples/src/imui_editor_proof_demo.rs`,
- an explicit reminder that the closed context-menu lane stays closed,
- and a reminder that broader select-all / rename / second-proof-surface pressure still requires
  different narrower follow-ons.

## What shipped

1. The collection-first asset browser proof now supports app-owned zoom/layout depth.
2. Grid layout metrics now derive from viewport width plus app-owned zoom state.
3. Primary+Wheel reflows tile extent locally while reusing the existing scroll handle for row
   anchoring.
4. Keyboard navigation now tracks the derived grid columns instead of a frozen constant.
5. Source-policy and unit-test gates now keep the slice visible in repo-first reopen flows.

## What did not ship

1. No new shared collection zoom helper in `fret-ui-kit::imui`.
2. No runtime contract changes.
3. No select-all, rename, or broader command palette integration.
4. No second proof surface.
5. No reopening of the closed context-menu lane or wider generic layout/helper questions.

## Reopen policy

Start a different narrower follow-on only if stronger first-party proof shows either:

- a second real surface now needs shared collection zoom/helper growth,
- the remaining pressure is primarily select-all / rename / richer command breadth,
- or broader collection product depth dominates more than zoom/layout closure.

No reopening of the closed context-menu lane or wider generic layout/helper questions.
