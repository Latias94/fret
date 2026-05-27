# `fret-node` Retained Exit And Parity Seams (v1) - Milestones

Status: Closed
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

Exit when:

- Workstream docs exist and agree on scope, non-goals, and gates.
- `WORKSTREAM.json` is valid JSON.
- Baseline no-default and layering gates are recorded.

## M1 - Retained Compatibility Island Exit

Exit when:

- `compat-retained-canvas` is removed or the lane records why it cannot be removed.
- Retained-only widget context adapters are deleted or no longer reachable from supported builds.
- Default and no-default `fret-node` gates pass without retained feature gates.
- Source-policy tests no longer encode the obsolete retained compatibility shape.

## M2 - Public Docs And API Narrative Cleanup

Exit when:

- Public node graph docs teach controller/store/declarative composition.
- Old retained constructor/callback examples are gone from teaching docs.
- XyFlow parity notes no longer point retained widget constructors at downstream app authors.

## M3 - Additional Canvas Mechanism Extraction

Exit when:

- One bounded generic canvas helper lives in `fret-canvas`.
- `fret-node` contains only the graph-specific adapter or policy around that helper.
- `fret-canvas` and `fret-node` targeted gates pass.

## M4 - XyFlow Hook/Focus Parity Seam

Exit when:

- One hook/focus parity seam is explicit in code and tests.
- The seam is not coupled to retained widget contexts.
- `docs/node-graph-xyflow-parity.md` records current behavior and remaining gaps precisely.

## M5 - Closeout

Exit when:

- Final validation gates pass with fresh evidence.
- `TODO.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and `HANDOFF.md` reflect shipped state.
- The Codex goal is marked complete only after the above are true.

Closed on 2026-05-28 after all scoped milestones exited and the closeout gate set passed.
