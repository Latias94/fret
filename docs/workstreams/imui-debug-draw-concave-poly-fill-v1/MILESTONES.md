# ImUi Debug Draw Concave Poly Fill v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M0 - Reference Decision

- Dear ImGui reference: `AddConcavePolyFilled` and `PathFillConcave`.
- Fret decision: expose distinct facade semantics but lower through the existing Canvas fill path.

## M1 - Implementation Slice

- Added direct concave polygon fill recording.
- Added scoped path `fill_concave`.
- Added a dedicated command variant to avoid overloading convex semantics.

## M2 - Verification Slice

- Added unit coverage for direct command recording, path finisher behavior, invalid point clearing,
  and closed path generation.
- Added public smoke compile coverage.
- Ran focused and full `fret-ui-kit --features imui` gates.

## M3 - Closeout

- Updated repo workstream indexes and the IMUI gap audit.
- Closed this lane as a narrow follow-on.
