# Diag Skill Evidence Owner Drift v1 — Evidence and Gates

Status: Closed
Last updated: 2026-04-10

## Smallest current repro

Use this sequence before changing the shipped diagnostics skill owner map:

```bash
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
cargo run -p fretboard -- --help
cargo run -p fretboard-dev -- --help
git diff --check
```

What this proves now:

- the diagnostics skill validates against the correct owner files,
- the public `fretboard` help surface still teaches `diag run ...`,
- the workspace-dev `fretboard-dev` help surface still teaches its own `diag run ...`,
- and the current docs patch is syntactically clean.

## Current evidence set

- `docs/workstreams/diag-skill-evidence-owner-drift-v1/BASELINE_AUDIT_2026-04-10.md`
  freezes the original mismatch between validator owner and shipped help surfaces.
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/M1_CONTRACT_FREEZE_2026-04-10.md`
  freezes the owner map and the non-goal against mutating CLI help output for validator
  convenience.
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/M2_PROOF_SURFACE_2026-04-10.md`
  closes the proof on:
  - validator realignment,
  - owner skill wording refresh,
  - public help smoke,
  - workspace-dev help smoke,
  - and diff hygiene.
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/CLOSEOUT_AUDIT_2026-04-10.md`
  closes the lane on the shipped owner-map correction and follow-on policy.

## Gate set

### Skill validator

```bash
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
```

### Public help smoke

```bash
cargo run -p fretboard -- --help
```

### Workspace-dev help smoke

```bash
cargo run -p fretboard-dev -- --help
```

### Diff hygiene

```bash
git diff --check
```

## Evidence anchors

- `docs/workstreams/diag-skill-evidence-owner-drift-v1/DESIGN.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/TODO.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/MILESTONES.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/BASELINE_AUDIT_2026-04-10.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/M1_CONTRACT_FREEZE_2026-04-10.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/M2_PROOF_SURFACE_2026-04-10.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/workstreams/fretboard-public-dev-implementation-v1/FINAL_STATUS.md`
- `docs/workstreams/fretboard-public-diag-implementation-v1/DESIGN.md`
- `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
- `.agents/skills/fret_skills.py`
- `.agents/skills/fret-diag-workflow/SKILL.md`
- `crates/fretboard/src/cli/help.rs`
- `apps/fretboard/src/cli/help.rs`

## Reference posture

- Treat public `fretboard diag ...` and workspace-dev `fretboard-dev diag ...` as distinct evidence
  owners.
- Keep the skill validator small and owner-accurate rather than forcing duplicated CLI examples.
- Keep this lane closed on one narrow owner-map correction. Broader skill catalog or validator
  redesign should start a separate follow-on.
