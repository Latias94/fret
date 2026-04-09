# Diag Skill Evidence Owner Drift v1

Status: Closed
Last updated: 2026-04-10

Related:

- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `BASELINE_AUDIT_2026-04-10.md`
- `M1_CONTRACT_FREEZE_2026-04-10.md`
- `M2_PROOF_SURFACE_2026-04-10.md`
- `CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/workstreams/fretboard-public-dev-implementation-v1/FINAL_STATUS.md`
- `docs/workstreams/fretboard-public-diag-implementation-v1/DESIGN.md`
- `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
- `.agents/skills/fret_skills.py`
- `.agents/skills/fret-diag-workflow/SKILL.md`
- `crates/fretboard/src/cli/help.rs`
- `apps/fretboard/src/cli/help.rs`

Status note (2026-04-10): this lane is now closed on one narrow governance fix:

- `fret-diag-workflow` now names the correct public and workspace-dev diagnostics help owners,
- the symbol validator checks `fretboard diag run` against the public CLI help surface and
  `fretboard-dev diag run` against the mono-repo developer wrapper,
- and the skill body explicitly warns maintainers not to mix those two evidence owners.

Read the landed proof in `M2_PROOF_SURFACE_2026-04-10.md` and the final verdict in
`CLOSEOUT_AUDIT_2026-04-10.md`.

This lane is a narrow governance follow-on to the recent public vs workspace-dev CLI split around
`fretboard` and `fretboard-dev`. It does not reopen the public diagnostics contract, the public
`dev` contract, or the broader skill-validator architecture.

It owns one narrower question:

> now that the repo has both a shipped public `fretboard` help surface and a richer mono-repo
> `fretboard-dev` help surface, how should the diagnostics skill record and validate those owners
> without teaching the wrong file path or forcing CLI output changes just to satisfy a drift check?

## Why this lane exists

The current repo state already had the correct product split:

- public project-facing help lives in `crates/fretboard/src/cli/help.rs`,
- workspace-dev maintainer help lives in `apps/fretboard/src/cli/help.rs`,
- and both surfaces legitimately teach different example commands.

But the diagnostics skill validator still bound one public evidence check to the workspace-dev
owner:

- `fret-diag-workflow` expected `fretboard diag run ...` inside
  `apps/fretboard/src/cli/help.rs`,
- while that file only teaches `fretboard-dev diag run ...`,
- and the real public `fretboard diag run ...` example lives in
  `crates/fretboard/src/cli/help.rs`.

That turned a governance guard into evidence-owner drift:

- future maintainers could "fix" the validator by editing the wrong help file,
- skill validation could fail for the wrong reason,
- and the owner skill did not explain the split clearly enough to prevent recurrence.

## Assumptions-first baseline

### 1) This is a governance follow-on, not a product-lane reopening

- Area: lane ownership
- Assumption: the right fix is a narrow skill-governance lane rather than reopening the public
  `dev` or public `diag` implementation lanes.
- Evidence:
  - `docs/workstreams/fretboard-public-dev-implementation-v1/FINAL_STATUS.md`
  - `docs/workstreams/fretboard-public-diag-implementation-v1/DESIGN.md`
  - `docs/workstreams/README.md`
- Confidence: Confident
- Consequence if wrong: we would blur an evidence-owner correction with ongoing CLI feature work.

### 2) Public and workspace-dev diagnostics help surfaces are intentionally different

- Area: contract ownership
- Assumption: `crates/fretboard/src/cli/help.rs` owns public `fretboard ...` examples and
  `apps/fretboard/src/cli/help.rs` owns workspace-dev `fretboard-dev ...` examples.
- Evidence:
  - `crates/fretboard/src/cli/help.rs`
  - `apps/fretboard/src/cli/help.rs`
  - `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
- Confidence: Confident
- Consequence if wrong: the lane would encode a split that the product surface does not actually
  guarantee.

### 3) The correct repair is to realign evidence owners, not to mutate help text

- Area: remediation choice
- Assumption: the right fix is to update the validator and owner skill so they point at the real
  owner files, not to duplicate examples across both CLIs.
- Evidence:
  - `.agents/skills/fret_skills.py`
  - `crates/fretboard/src/cli/help.rs`
  - `apps/fretboard/src/cli/help.rs`
- Confidence: Confident
- Consequence if wrong: we would teach maintainers to pad CLI help for validator convenience rather
  than accuracy.

### 4) The owner skill must teach the split explicitly

- Area: reusable process
- Assumption: refreshing `fret-diag-workflow` is part of the fix, because the validator alone does
  not tell future maintainers why the two owner paths differ.
- Evidence:
  - `.agents/skills/fret-diag-workflow/SKILL.md`
  - `.agents/skills/fret_skills.py`
- Confidence: Likely
- Consequence if wrong: the repo would keep the correct regexes but still invite future drift when
  the next maintainer updates evidence anchors.

### 5) The smallest proof is validator plus both help smokes

- Area: proof surface
- Assumption: strict skill validation plus the two root-help commands are sufficient proof for this
  narrow lane.
- Evidence:
  - `.agents/skills/fret_skills.py`
  - `crates/fretboard/src/cli/help.rs`
  - `apps/fretboard/src/cli/help.rs`
- Confidence: Likely
- Consequence if wrong: we would close the lane without demonstrating that the owner split still
  matches shipped help output.

## In scope

- Realign `fret-diag-workflow` symbol checks to the correct public and workspace-dev help owners.
- Refresh `fret-diag-workflow` evidence anchors and pitfalls so the split is explicit.
- Leave one narrow governance record with reproducible gates and closeout guidance.

## Out of scope

- Changing `fretboard` or `fretboard-dev` diagnostics CLI contracts.
- Changing the public vs workspace-dev crate/package taxonomy.
- Reworking the broader skill validator schema or adding new validator subsystems.
- Reopening unrelated workstream-index or documentation-catalog governance beyond what is necessary
  to make this lane discoverable.

## Target shipped state

When this lane is done, the following must be true:

1. `fret-diag-workflow` validates public `fretboard diag run ...` against
   `crates/fretboard/src/cli/help.rs`.
2. `fret-diag-workflow` validates workspace-dev `fretboard-dev diag run ...` against
   `apps/fretboard/src/cli/help.rs`.
3. The skill body names both help surfaces explicitly and warns against mixing them.
4. Strict skill validation passes without requiring any CLI-output distortion.
5. Future governance work can treat this lane as closed and open a narrower follow-on for any
   broader validator/catalog drift.
