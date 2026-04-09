# Diag Skill Evidence Owner Drift v1 — Milestones

Status: Closed
Last updated: 2026-04-10

## M0 — Baseline audit

Exit criteria:

- The drift is framed as evidence-owner mismatch, not missing CLI behavior.
- The public and workspace-dev help owners are both audited from source.
- The lane's non-goals are explicit.

Primary evidence:

- `docs/workstreams/diag-skill-evidence-owner-drift-v1/DESIGN.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/TODO.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/BASELINE_AUDIT_2026-04-10.md`
- `docs/workstreams/fretboard-public-dev-implementation-v1/FINAL_STATUS.md`
- `docs/workstreams/fretboard-public-diag-implementation-v1/DESIGN.md`
- `crates/fretboard/src/cli/help.rs`
- `apps/fretboard/src/cli/help.rs`

Current status:

- M0 baseline audit closed on 2026-04-10.

## M1 — Owner contract freeze

Exit criteria:

- The public help owner is explicit.
- The workspace-dev help owner is explicit.
- The owner skill wording required to prevent recurrence is explicit.

Primary evidence:

- `docs/workstreams/diag-skill-evidence-owner-drift-v1/DESIGN.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/BASELINE_AUDIT_2026-04-10.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/M1_CONTRACT_FREEZE_2026-04-10.md`
- `.agents/skills/fret_skills.py`
- `.agents/skills/fret-diag-workflow/SKILL.md`

Current status:

- M1 owner contract freeze closed on 2026-04-10.

## M2 — Proof surface

Exit criteria:

- The validator points to the correct owner files.
- The skill body teaches the split explicitly.
- Strict validation and both help smokes pass.

Primary gates:

- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `cargo run -p fretboard -- --help`
- `cargo run -p fretboard-dev -- --help`
- `git diff --check`

Current status:

- M2 proof surface closed on 2026-04-10.
- See `docs/workstreams/diag-skill-evidence-owner-drift-v1/M2_PROOF_SURFACE_2026-04-10.md`.

## M3 — Closeout

Exit criteria:

- The narrow governance record is discoverable.
- Follow-on policy is explicit.
- The lane closes without reopening public/dev CLI feature scope.

Primary evidence:

- `docs/workstreams/diag-skill-evidence-owner-drift-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/workstreams/README.md`

Current status:

- M3 closeout closed on 2026-04-10.
