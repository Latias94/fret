# Fret Node Paint Root Frame Setup Adapter v1 - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This lane is a narrow follow-on from `fret-node-paint-root-cache-plan-adapter-v1`. The parent lane
proved cache-plan host/bounds/scale-factor route inputs behind an adapter seam.

The first slice should audit frame setup operation families before implementation.

## Active Task

- Task ID: FSA-010
- Owner: unassigned
- Files: `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1`
- Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/WORKSTREAM.json`
- Status: NEEDS_CONTEXT
- Review: not started
- Evidence: `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/EVIDENCE_AND_GATES.md`

## Decisions Since Open

- Frame setup should be audited before adding an adapter.
- Bounds/viewport route inputs are the likely smallest implementation candidate.
- Clip/background/grid scene emission should not be folded into the first frame seam by default.

## Blockers

- None known.

## Next Recommended Action

- Execute FSA-010 to freeze scope, then FSA-020 to audit frame setup operation families.
