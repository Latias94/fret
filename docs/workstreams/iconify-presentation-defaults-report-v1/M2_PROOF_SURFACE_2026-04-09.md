# M2 Proof Surface — 2026-04-09

Status: accepted proof

Related:

- `docs/workstreams/iconify-presentation-defaults-report-v1/DESIGN.md`
- `docs/workstreams/iconify-presentation-defaults-report-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-report-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-report-v1/TODO.md`
- `docs/workstreams/iconify-presentation-defaults-report-v1/MILESTONES.md`
- `docs/workstreams/iconify-presentation-defaults-report-v1/EVIDENCE_AND_GATES.md`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/suggest.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/cli/help.rs`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`

## Purpose

Record the first proof that the suggestion helper can emit a second explicit review artifact while
remaining advisory and file-based.

## What shipped in the proof

### 1. Optional report output on the existing helper

`fretboard` now accepts:

- `fretboard icons suggest presentation-defaults --provenance <file> --out <file> --report-out <file>`

The previously shipped `--provenance` + `--out` flow still works unchanged.

### 2. The report captures evidence and limitations explicitly

The new report records:

- the provenance/config paths,
- source facts such as collection, `palette`, and snapshot counts,
- the suggested pack-level default render mode,
- and explicit summary/limitation text for review.

### 3. Path conflicts fail before side effects

The helper now rejects conflicting provenance/config/report output paths before it writes files,
avoiding partial success on bad flag combinations.

### 4. Public docs now teach the report correctly

The crate usage guide and todo golden path now describe `--report-out` as an optional committed
review artifact rather than as a new import contract.

## Proof gates executed on 2026-04-09

```bash
cargo nextest run -p fretboard
cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface
```

Observed result:

- `fretboard`: `66 tests run: 66 passed`
- `fret` source-policy/doc gates: `2 tests run: 2 passed`

## M2 verdict

Treat M2 as closed on these points:

1. report output is now a real explicit optional CLI surface,
2. the report stays separate from the generator/import contract,
3. path validation avoids write-time self-overwrite,
4. and docs keep the report advisory rather than normative.
