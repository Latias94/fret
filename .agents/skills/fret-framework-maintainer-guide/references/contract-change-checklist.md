# Contract change checklist

Use this note when the framework change touches a hard contract or crosses crate boundaries.

Goal: leave behind a change that is reviewable, reproducible, and aligned with ADR-driven evolution.

## 1) ADR and alignment checklist

If the change affects input, focus, overlays, text, diagnostics, or other hard-to-change behavior:

- Update or add an ADR under `docs/adr/`.
- If the ADR is already tracked, update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with:
  - `Aligned`, `Partially aligned`, or `Not implemented`
  - 1–3 evidence anchors (paths + tests/scripts)
- Keep the implementation note narrow: document the contract change, not every incidental refactor.

## 2) Diagnostics and perf gates

Treat diagnostics artifacts as first-class regression protection:

- Correctness: `tools/diag-scripts/*.json` (schema v2 preferred) + `capture_bundle`
- Perf: `fretboard diag perf` suites or `tools/perf/*` gate scripts + worst bundle paths
- Attribution: `fretboard diag stats <bundle.json> --sort time --top 30` plus the exact failing metric/threshold key

Use `.agents/skills/fret-diag-workflow/SKILL.md` as the canonical runbook for running and packaging artifacts.

## 3) Refactor guardrails

Before/after a refactor that may cross boundaries:

- Run layering checks: `python3 tools/check_layering.py`
- Add at least one gate for any behavior change (unit test or diag script)
- If perf is in scope, record a perf gate run + worst bundles
- Keep evidence commit-addressable: reviewers should be able to rerun the exact command later

## 4) Deliverables 3-pack

Every non-trivial maintainer change should leave:

- **Repro**: smallest demo/gallery/script that shows the behavior
- **Gate**: a test, diag script, or perf gate that would catch regression
- **Evidence**: 1–3 anchors plus exact commands

See `fret-skills-playbook` for the shared wording and expectations.

## 5) Release-facing follow-up

If the change affects publishable crates or release automation, do not improvise release policy inside this skill.

Instead:

- switch to `fret-release-check-and-publish` for release scope, version-group, and CI publish checks
- leave a clear note in the maintainer change summary if release follow-up is required

## 6) High-signal anchors to keep handy

- `docs/architecture.md`
- `docs/dependency-policy.md`
- `docs/adr/`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `tools/diag-scripts/`
- `tools/perf/`
- `python3 tools/check_layering.py`
