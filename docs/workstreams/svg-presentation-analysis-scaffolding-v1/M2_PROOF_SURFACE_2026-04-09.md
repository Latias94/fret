# M2 Proof Surface — 2026-04-09

Status: accepted proof

Related:

- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/DESIGN.md`
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/TODO.md`
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/MILESTONES.md`
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/EVIDENCE_AND_GATES.md`
- `crates/fret-icons-generator/src/lib.rs`
- `crates/fret-icons-generator/src/naming.rs`
- `crates/fretboard/Cargo.toml`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/icons/suggest_svg.rs`
- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/cli/help.rs`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`

## Purpose

Record the first proof that Fret can scaffold conservative local-SVG presentation overrides while
keeping the stable generator/import contract unchanged.

## What shipped in the proof

### 1. Local SVG analysis now has an explicit public helper

`fretboard` now accepts:

- `fretboard icons suggest svg-dir-presentation-overrides --source <dir> --out <file> [--report-out <file>]`

This sits alongside the existing explicit `acquire`, `suggest presentation-defaults`, and `import`
stages rather than replacing them.

### 2. Helper output reuses generator naming and config contracts

`fret-icons-generator` now exports shared SVG-dir icon-name normalization so the helper emits
`icon_name` values that match `icons import svg-dir ...`.

The emitted config stays on the existing versioned `presentation-defaults.json` contract and only
contains per-icon overrides.

### 3. Strong evidence produces `original-colors` overrides; weak evidence stays unclassified

The helper now suggests per-icon `original-colors` overrides when SVG analysis finds:

- multiple distinct solid colors,
- gradients,
- patterns,
- embedded raster images,
- or embedded SVG images.

Single-color assets remain unclassified by design, and parse failures stay in the optional report
without aborting the whole run.

### 4. Public docs keep the helper advisory

The crate usage guide and todo golden path now teach the local-SVG helper as:

- explicit,
- conservative,
- review-first,
- and separate from import defaults.

## Proof gates executed on 2026-04-09

```bash
cargo nextest run -p fretboard
cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface
git diff --check
```

Observed result:

- `fretboard`: `72 tests run: 72 passed`
- `fret` source-policy/doc gates: expected green once the public doc updates land for this slice

## M2 verdict

Treat M2 as closed on these points:

1. local SVG analysis is now a real explicit helper surface,
2. the helper stays aligned with generator naming without widening the generator contract,
3. only strong evidence produces per-icon overrides,
4. pack-level defaults and import behavior remain unchanged,
5. and docs keep the helper advisory rather than normative.
