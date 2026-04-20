# Resizable Adaptive Panel Proof v1

Status: Closed historical design note
Last updated: 2026-04-20

Status note (2026-04-20): this document remains useful for the lane-opening rationale, but the
shipped verdict now lives in `CLOSEOUT_AUDIT_2026-04-20.md` and `WORKSTREAM.json`. Read the
execution framing below as the historical setup that led to the landed proof promotion.

Related:

- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `CLOSEOUT_AUDIT_2026-04-20.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `apps/fret-ui-gallery/src/ui/pages/resizable.rs`
- `apps/fret-ui-gallery/src/ui/snippets/resizable/adaptive_panel.rs`
- `apps/fret-ui-gallery/tests/resizable_docs_surface.rs`
- `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-adaptive-panel-proof.json`

This workstream is a narrow follow-on to the already-closed adaptive taxonomy lane.

It does not reopen:

- the query-axis taxonomy,
- the adaptive helper owner split,
- or the docking demo proof that already existed as a lower-level regression lane.

The narrow problem is first-party teaching-surface drift.

ADR 0325 already requires one fixed-window panel-resize/container-query proof surface.
The repo technically had that proof in the docking demo and its diagnostics, but the default
first-party `Resizable` docs path still stopped at API parity and left the stronger panel-width
teaching burden on a different demo family.

That left the repo with an avoidable mismatch:

- the mechanism and diagnostics story was already good enough,
- the adaptive taxonomy closeout already said the proof obligation was important,
- but the first place a framework consumer reads `Resizable` still did not teach the proof
  directly.

## Must-be-true outcomes

1. The `Resizable` UI Gallery page gains an explicit fixed-window panel-resize section in its
   first-party docs path.
2. That section proves container-driven adaptation without changing the viewport width.
3. The proof uses stable `test_id` surfaces and a promoted diag script that capture layout-sidecar,
   screenshot, and bundle evidence.
4. ADR 0325 alignment now points at the first-party `Resizable` page instead of relying on the
   docking demo alone for the gallery/docs teaching obligation.
5. No runtime or policy widening is introduced; the change stays on the docs/gallery/recipe
   teaching surface.

## In scope

- Add an `Adaptive Panel Proof` section to the `Resizable` gallery page.
- Add a real fixed-window request-panel snippet with explicit wide/compact state ids.
- Add a dedicated diag script for the promoted gallery proof.
- Update source-policy tests, roadmap/workstream notes, and ADR alignment.

## Out of scope

- Changing `crates/fret-ui` resize semantics.
- Reopening the closed adaptive taxonomy lane.
- Replacing the docking demo proof surface.
- Growing a broader high-level adaptive authoring helper because of one gallery docs gap.

## Owner split

### `crates/fret-ui`

Owns splitter semantics, keyboard nudges, hit-testing, and container-query mechanism plumbing.

### `ecosystem/fret-ui-shadcn`

Owns the recipe surfaces reused by the proof, including `ResizablePanelGroup` chrome and
`FieldOrientation::ContainerAdaptive`.

### `apps/fret-ui-gallery`

Owns the first-party teaching surface, the copy that explains the axis, and the stable diagnostic
selectors for the promoted proof.

### `tools/diag-scripts`

Own the deterministic before/after proof artifact for this gallery surface.

## Target shipped state

The `Resizable` family should now teach three things in one place:

- the shadcn-aligned authoring surface,
- the caller-owned fixed shell around that surface,
- and one explicit right-panel width transition that proves the compact branch is container-driven.

That shipped state is recorded in `CLOSEOUT_AUDIT_2026-04-20.md`.
