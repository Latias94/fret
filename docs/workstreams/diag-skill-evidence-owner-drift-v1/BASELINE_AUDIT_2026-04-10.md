# Baseline Audit — 2026-04-10

Status: closed audit record

Related:

- `docs/workstreams/diag-skill-evidence-owner-drift-v1/DESIGN.md`
- `.agents/skills/fret_skills.py`
- `.agents/skills/fret-diag-workflow/SKILL.md`
- `crates/fretboard/src/cli/help.rs`
- `apps/fretboard/src/cli/help.rs`
- `docs/workstreams/fretboard-public-dev-implementation-v1/FINAL_STATUS.md`
- `docs/workstreams/fretboard-public-diag-implementation-v1/DESIGN.md`

## Audit question

Does the current diagnostics skill validator point at the right evidence owners for the shipped
public and workspace-dev `diag run` help surfaces?

## Baseline findings

### 1) The validator mis-owned the public diagnostics example

`fret-diag-workflow` previously bound `\bfretboard\s+diag\s+run\b` to
`apps/fretboard/src/cli/help.rs`.

That file is the workspace-dev wrapper, not the public `fretboard` owner.

### 2) The actual public owner is `crates/fretboard/src/cli/help.rs`

The public help surface contains:

- `fretboard diag latest`
- `fretboard diag run ./diag/dialog-escape.json --launch -- cargo run --manifest-path ./Cargo.toml`

So the public diagnostics example already existed, but in the crate that ships the public CLI.

### 3) The workspace-dev owner is `apps/fretboard/src/cli/help.rs`

The workspace-dev help surface contains:

- `fretboard-dev diag --help`
- `fretboard-dev diag latest`
- `fretboard-dev diag run tools/diag-scripts/todo-baseline.json --dir ...`

This file legitimately teaches repo-only wrappers and example paths.

### 4) The owner skill did not explain the split clearly enough

Before this lane, `fret-diag-workflow` cited diagnostics evidence broadly but did not spell out the
public-vs-workspace-dev help boundary.

That left future maintainers with too much room to cargo-cult the wrong owner file.

## Audit conclusion

This is evidence-owner drift, not missing CLI functionality.

The correct repair is:

1. move the public symbol check to `crates/fretboard/src/cli/help.rs`,
2. add the workspace-dev check explicitly for `apps/fretboard/src/cli/help.rs`,
3. and teach that split in the owner skill itself.

The correct repair is not:

- adding public `fretboard ...` strings to the workspace-dev help file,
- removing the workspace-dev examples,
- or widening this lane into a broader CLI-contract rewrite.
