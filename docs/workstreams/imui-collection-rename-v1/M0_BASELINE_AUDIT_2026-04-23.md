# ImUi Collection Rename v1 - M0 Baseline Audit

Date: 2026-04-23
Status: baseline confirmed

## Findings

1. The closed collection select-all lane explicitly deferred rename breadth.
2. The current proof surface already has the right ingredients for a narrow app-owned collection rename slice:
   - a collection-scoped keyboard owner,
   - an existing context menu,
   - existing text-input and popup seams,
   - and one real first-party asset-browser proof surface.
3. The current proof already has popup and text-input seams, so this lane is not a justification to widen shared helper ownership.
4. Dear ImGui keeps rename breadth close to the current proof surface instead of turning it into a generic runtime contract.

## Implication

The correct move is not to add a new `fret-ui-kit::imui` helper.

The correct move is to keep the current proof surface honest:

- make rename explicit and app-owned,
- route it through the current collection-scope key owner plus popup/input seams,
- and leave second-proof-surface / broader helper pressure for different follow-ons.
