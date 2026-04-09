# M2 Proof Surface — 2026-04-10

Status: closed proof record

Related:

- `docs/workstreams/diag-skill-evidence-owner-drift-v1/DESIGN.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/M1_CONTRACT_FREEZE_2026-04-10.md`
- `.agents/skills/fret_skills.py`
- `.agents/skills/fret-diag-workflow/SKILL.md`
- `crates/fretboard/src/cli/help.rs`
- `apps/fretboard/src/cli/help.rs`

## Proof commands

```bash
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
cargo run -p fretboard -- --help
cargo run -p fretboard-dev -- --help
git diff --check
```

## Observed proof

### 1) Strict skill validation

- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
  passed after the owner-map correction.

This proves the diagnostics skill now cites existing anchors and the symbol checks resolve against
the intended files.

### 2) Public root help smoke

- `cargo run -p fretboard -- --help`
  rendered successfully.
- The output still includes the public example:
  `fretboard diag run ./diag/dialog-escape.json --launch -- cargo run --manifest-path ./Cargo.toml`.

This proves the public evidence owner remains `crates/fretboard/src/cli/help.rs`.

### 3) Workspace-dev root help smoke

- `cargo run -p fretboard-dev -- --help`
  rendered successfully.
- The output still includes the workspace-dev example:
  `fretboard-dev diag run tools/diag-scripts/todo-baseline.json --dir target/fret-diag-todo-auto --launch -- cargo run -p fret-demo --bin todo_demo`.

This proves the workspace-dev evidence owner remains `apps/fretboard/src/cli/help.rs`.

### 4) Diff hygiene

- `git diff --check`
  passed after the docs and skill updates.

## Proof conclusion

The lane's smallest intended outcome is now demonstrated:

- the validator points at the correct owners,
- the owner skill teaches the split explicitly,
- and both shipped help surfaces remain unchanged and correct.
