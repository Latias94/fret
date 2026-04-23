# ImUi Collection Select-All v1 - M0 Baseline Audit

Date: 2026-04-23
Status: baseline confirmed

## Findings

1. The closed collection zoom lane explicitly deferred collection select-all breadth.
2. The current proof surface already has the right ingredients for a narrow app-owned collection select-all slice:
   - a collection-scoped keyboard owner,
   - visible-order collection math,
   - explicit selection and active-tile models,
   - and one real first-party asset-browser proof surface.
3. The collection-scope key-owner and visible-order helpers already exist locally, so this lane is not a justification to widen shared helper ownership.
4. Dear ImGui keeps Ctrl+A selection breadth in the multi-select proof surface instead of turning it into a generic runtime contract.

## Implication

The correct move is not to add a new `fret-ui-kit::imui` helper.

The correct move is to keep the current proof surface honest:

- make select-all explicit and app-owned,
- route it through the current collection-scope key owner,
- and leave rename / broader command breadth for different follow-ons.
