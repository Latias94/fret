# Closeout Audit — 2026-04-10

Status: closed closeout record

Related:

- `docs/workstreams/diag-skill-evidence-owner-drift-v1/DESIGN.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/BASELINE_AUDIT_2026-04-10.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/M1_CONTRACT_FREEZE_2026-04-10.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/M2_PROOF_SURFACE_2026-04-10.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/TODO.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/MILESTONES.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/fretboard-public-dev-implementation-v1/FINAL_STATUS.md`
- `docs/workstreams/fretboard-public-diag-implementation-v1/DESIGN.md`
- `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
- `.agents/skills/fret_skills.py`
- `.agents/skills/fret-diag-workflow/SKILL.md`
- `crates/fretboard/src/cli/help.rs`
- `apps/fretboard/src/cli/help.rs`

## Verdict

This lane is now closed.

It successfully landed the narrow governance follow-on needed after the public/workspace-dev CLI
split:

- the diagnostics skill validator now checks the correct owner files,
- the diagnostics skill body names the correct help owners and warns against mixing them,
- strict skill validation and both help smokes pass,
- and no CLI behavior had to change to satisfy the governance check.

## What shipped

### 1) Owner-accurate validator mapping

`fret-diag-workflow` now validates:

- `fretboard diag run ...` against `crates/fretboard/src/cli/help.rs`,
- `fretboard-dev diag run ...` against `apps/fretboard/src/cli/help.rs`.

That restores the intended meaning of `--check-symbols`.

### 2) Reusable skill wording

The owner skill now leaves a direct trail for future maintainers:

- both help surfaces are listed under evidence anchors,
- and the common-pitfalls section explains which owner goes with which CLI.

That turns this fix from a one-off regex correction into reusable repo memory.

### 3) Product surface preserved

The lane closed without:

- changing help text,
- widening the public diagnostics contract,
- or altering the workspace-dev maintainer wrapper.

The repair stayed in governance, where it belonged.

## Gates that now define the closed surface

- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `cargo run -p fretboard -- --help`
- `cargo run -p fretboard-dev -- --help`
- `git diff --check`

## Follow-on policy

Do not reopen this lane for:

- broader `fret_skills.py` schema changes,
- general workstream-catalog governance,
- or public/dev CLI product evolution.

If future work is needed, open a narrower follow-on such as:

1. a broader skill-catalog drift lane,
2. a validator-schema hardening lane,
3. or a separate docs/workstreams catalog alignment lane if index drift becomes worth closing on
   its own.
