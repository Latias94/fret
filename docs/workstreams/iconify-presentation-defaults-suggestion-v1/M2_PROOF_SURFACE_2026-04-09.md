# M2 Proof Surface — 2026-04-09

Status: accepted proof

Related:

- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/DESIGN.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/TODO.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/MILESTONES.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/EVIDENCE_AND_GATES.md`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/suggest.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/cli/help.rs`
- `crates/fret-icons-generator/src/presentation_defaults.rs`

## Purpose

Record the first real proof that acquisition provenance can drive a thin, explicit suggestion flow
without changing the generator contract.

## What shipped in the proof

### 1) New public CLI branch

`fretboard` now exposes:

- `fretboard icons suggest presentation-defaults --provenance <file> --out <file>`

This keeps the helper explicit and file-based.

### 2) Suggestion logic is narrow and evidence-bound

The helper:

- reads acquisition provenance,
- accepts only `iconify-collection` provenance,
- maps `palette=false` to `default_render_mode = "mask"`,
- maps `palette=true` to `default_render_mode = "original-colors"`,
- and errors when `palette` is missing.

### 3) The existing import path remains unchanged

The helper does not mutate import behavior directly.

Instead it emits a normal `presentation-defaults.json` file that the existing import path already
consumes.

### 4) End-to-end repo proof exists

`fretboard` now has a repo proof test that:

- writes acquisition provenance,
- derives a suggested `presentation-defaults.json`,
- imports a local Iconify snapshot with that file,
- and proves the generated pack lands on the expected explicit render mode.

## Proof gates executed on 2026-04-09

```bash
cargo nextest run -p fretboard
cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface
```

Observed result:

- `fretboard`: `63 tests run: 63 passed`
- `fret` source-policy/doc gates: `2 tests run: 2 passed`

## M2 verdict

Treat M2 as closed on these points:

1. provenance-driven suggestion is now a real explicit CLI surface;
2. the helper reuses the existing config contract instead of inventing a new one;
3. the helper remains advisory because import still needs the emitted file explicitly;
4. missing `palette` stays an explicit failure rather than a hidden guess.
