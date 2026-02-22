---
name: fret-crate-audits
description: "This skill should be used when the user asks to \"audit a crate\", \"review contract surfaces\", \"assess refactor hazards\", or \"produce a crate audit note\". Provides a crate-by-crate audit workflow (purpose/exports/deps/hazards) plus a small gate set to turn findings into landable steps."
---

# Fret crate audits (code-quality pass)

This skill is for the **second pass** of the Bottom-Up Fearless Refactor program: reading crates and
reviewing them for Rust best practices, boundary hygiene, and refactor hazards.

## When to use

- You want a structured “audit note” for a crate before a refactor.
- You need to check public API surface hygiene (exports/re-exports) and dependency posture.
- You are about to touch hot paths (dispatch/layout/paint) and want a minimal gate set.

## Inputs to collect (ask the user)

- Target crate name (package name), e.g. `fret-runtime`
- Optional focus (pick 1–2):
  - public surface (exports / accidental re-exports)
  - dependency posture (backend coupling risks)
  - hot paths (dispatch/layout/paint)
  - serialization formats / stability
  - error handling and diagnostics
- Risk posture:
  - is this a “touch contracts” change (needs ADR alignment + extra gates) or an internal refactor?
- Expected deliverable:
  - audit note only, or audit note + landable refactor steps + gates?

Defaults if unclear:

- Do an L0 snapshot with `tools/audit_crate.py`, list hazards, and propose 3–8 landable steps with one gate.

## Smallest starting point (one command)

- `python3 tools/audit_crate.py --crate fret-ui`

## Quick start (L0 audit)

1. Generate a quick audit report (facts + evidence anchors):
   - `python3 tools/audit_crate.py --crate fret-runtime`
2. Create (or update) a crate audit note using the template:
   - `docs/workstreams/bottom-up-fearless-refactor-v1-crate-audit-template.md`
3. Update the audits index:
   - `docs/workstreams/bottom-up-fearless-refactor-v1-crate-audits.md`

## Workflow

1. Snapshot facts (exports, deps, size, hotspots):
   - `python3 tools/audit_crate.py --crate <crate>`
2. Read the crate with one focus at a time:
   - boundary posture (can this crate stay portable?)
   - public surface hygiene (what is accidentally public?)
   - hot paths (what is per-frame / per-event?)
3. Turn findings into landable steps:
   - 3–8 small refactors with clear “done” criteria
4. Add at least one regression artifact when behavior changes:
   - unit/integration test and/or `fretboard diag` script for state machines
5. Run minimum gates and re-check layering:
   - `cargo fmt`
   - `cargo nextest run -p <crate>`
   - `python3 tools/check_layering.py`

## Definition of done (what to leave behind)

- Minimum deliverables (3-pack): Repro (audit snapshot), Gate (crate tests + layering), Evidence (audit note). See `fret-skills-playbook`.
- A short audit note exists (purpose/exports/deps/hazards) with 1–3 evidence anchors per major claim.
- 3–8 landable refactor steps are listed with “done” criteria (not just narrative).
- At least one regression artifact exists if behavior or contracts change (test and/or diag script).
- Minimum gates are green for the touched crate and layering stays green:
  - `cargo fmt`
  - `cargo nextest run -p <crate>`
  - `python3 tools/check_layering.py`

## Minimum gates (recommended)

- Format (touched crates only): `cargo fmt`
- Tests (touched crate): `cargo nextest run -p <crate>`
- Layering: `python3 tools/check_layering.py`

If you touch hot paths, add at least one additional regression gate (unit/integration/diag).

## Output expectations

Prefer producing:

- 3–10 hazards + at least one missing gate to add
- 3–8 concrete, landable refactor steps
- 1–3 evidence anchors per major claim (paths + key functions/tests)

## Evidence anchors

- Crate audit snapshot: `tools/audit_crate.py`
- Layering policy: `docs/dependency-policy.md`
- Audit note template: `docs/workstreams/bottom-up-fearless-refactor-v1-crate-audit-template.md`

## Examples

- Example: pre-refactor hazard scan
  - User says: "Can we safely refactor this crate?"
  - Actions: map purpose/exports/deps, identify hidden coupling, propose a minimal gate set.
  - Result: a short audit note with next steps and risks.

## Common pitfalls

- Writing a long narrative without extracting landable refactor steps.
- Touching public API surfaces without an explicit “surface diff” and a gate.
- Fixing a layering violation by adding allowlists instead of moving code.

## Troubleshooting

- Symptom: the audit balloons into a rewrite.
  - Fix: timebox, focus on hazards + boundaries first; defer style cleanup.
- Symptom: unclear public surface.
  - Fix: start from the crate root exports and look for cross-crate callers before changing structure.

## Related skills

- `fret-boundary-checks` (fast guardrails)
- `fret-diag-workflow` (turn behavior bugs into repro gates)
- `fret-diag-workflow` (perf gates + worst-bundle evidence for hot path changes)
