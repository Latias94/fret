# ImUi Collection Zoom v1 - M0 Baseline Audit

Date: 2026-04-23
Status: baseline confirmed

## Findings

1. The closed collection context-menu lane explicitly deferred collection zoom/layout depth.
2. The current proof surface already has the right ingredients for a narrow app-owned collection zoom slice:
   - a collection-scoped child region,
   - a reusable scroll handle seam,
   - wheel hooks at the pointer-region layer,
   - and one frozen constant column count that currently mixes layout policy into keyboard policy.
3. The scroll handle and wheel hooks already exist generically, so this lane is not a justification to widen shared helper ownership.
4. Dear ImGui keeps asset-browser zoom and layout recomputation at the proof surface instead of turning them into a generic runtime contract.

## Implication

The correct move is not to add a new `fret-ui-kit::imui` helper.

The correct move is to make the current proof surface more honest:

- move collection layout math into one explicit helper,
- make grid rendering and keyboard navigation consume the same derived columns,
- and let primary+wheel update app-owned zoom state plus scroll anchoring locally.
