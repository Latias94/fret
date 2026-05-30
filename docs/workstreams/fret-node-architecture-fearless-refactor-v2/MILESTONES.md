# `fret-node` Architecture Fearless Refactor v2 - Milestones

Status: Complete
Last updated: 2026-05-27

## M0 - Scope And Baseline

Exit criteria:

- Workstream docs exist and agree.
- Baseline `fret-node` gate is recorded.
- First executable task is identified.

## M1 - Canonical Graph Mutation Module

Exit criteria:

- Structural graph mutation has one canonical seam.
- Node/port ownership invariants are enforced through that seam.
- Diff, inverse, and change projection no longer require callers to know raw op ordering.

Status: Complete as of `FNAR-021`.

## M2 - Store Authority And Document Replacement Events

Exit criteria:

- Store is the authoritative source for graph/view/editor config.
- Document replacement emits an explicit event.
- UI binding/controller code stops relying on stale mirror synchronization as the core truth path.

Status: Complete as of `FNAR-030`.

## M3 - Document / Editor Policy State Split

Exit criteria:

- Headless graph document state is semantic.
- Editor policy and derived UI state are explicitly separated.
- Persistence behavior is documented and tested.

Status: Complete as of `FNAR-040`.

## M4 - Full-Fidelity Patch Stream

Exit criteria:

- Commit callbacks can observe full graph-resource changes.
- XYFlow-style node/edge changes remain available only as a projection adapter.
- Controlled-mode docs explain the fidelity tradeoff.

Status: Complete as of `FNAR-050`.

## M5 - Canvas Mechanism Extraction

Exit criteria:

- Generic canvas mechanisms extracted from `fret-node` are reusable by another Domain UI Package.
- Node graph UI uses adapters for node/edge policy.
- Layering checks still pass.

Status: Complete as of `FNAR-060`.

## M6 - Seam Test Replacement

Exit criteria:

- Source-text policy tests are narrow and intentional.
- Core contracts are protected by compile, behavior, transaction, event, or diagnostics tests.
- Refactors no longer require broad implementation-name updates to unrelated tests.

Status: Complete as of `FNAR-070`.

## M7 - Closeout

Exit criteria:

- Fresh gates pass.
- Workstream docs and roadmap/parity docs match shipped behavior.
- Remaining work is either closed or split into named follow-ons.

Status: Complete as of `FNAR-080`.
