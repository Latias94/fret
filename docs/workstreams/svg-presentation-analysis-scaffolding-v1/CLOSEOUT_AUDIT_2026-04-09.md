# Closeout Audit — 2026-04-09

Status: closed closeout record

Related:

- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/DESIGN.md`
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/M2_PROOF_SURFACE_2026-04-09.md`
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/TODO.md`
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/MILESTONES.md`
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/iconify-presentation-defaults-report-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
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
- `ecosystem/fret/src/lib.rs`

## Verdict

This lane is now closed.

It successfully landed the narrow local-SVG follow-on that the earlier icon helper lanes left
explicitly open:

- one explicit `icons suggest svg-dir-presentation-overrides` helper,
- shared generator-aligned naming for emitted `icon_name` values,
- conservative per-icon `original-colors` suggestions only when evidence is strong,
- optional versioned report output for review,
- and public docs/source-policy coverage that keeps the helper advisory.

## What shipped

### 1) Local SVG analysis is now an explicit stage

The public surface now distinguishes:

- `icons acquire ...`
- `icons suggest presentation-defaults ...`
- `icons suggest svg-dir-presentation-overrides ...`
- `icons import ...`

This keeps source-specific convenience logic explicit instead of turning it into hidden import
policy.

### 2) The helper stayed out of generator/import defaults

Import behavior is unchanged:

- the helper emits only `presentation-defaults.json` plus an optional report,
- `default_render_mode` remains unset by the helper,
- and import still consumes only the explicit config file passed by the user.

### 3) Correctness stayed conservative

The helper suggests `original-colors` overrides only when SVG analysis finds strong authored-color
signals. It intentionally accepts false negatives for single-color icons rather than over-classify
and lock authored-color policy incorrectly.

### 4) Reviewability improved without blocking useful output

The optional report preserves:

- observed colors and SVG feature evidence,
- parse failures and warnings,
- and the helper's explicit limitations.

Broken SVGs remain visible but do not stop the helper from producing overrides for the rest of the
directory.

## Gates that now define the shipped surface

- `cargo nextest run -p fretboard`
- `cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface`
- `git diff --check`

## Follow-on policy

Do not reopen this lane for:

- pack-level `default_render_mode` inference,
- automatic `mask` guessing,
- changing import defaults,
- or widening the generator contract with helper heuristics.

If future work is needed, open a narrower follow-on such as:

1. richer report consumers or formatting variants,
2. explicit source-specific heuristics with new evidence and their own proof gates,
3. or first-party curated pack policy outside the generic import helper path.
