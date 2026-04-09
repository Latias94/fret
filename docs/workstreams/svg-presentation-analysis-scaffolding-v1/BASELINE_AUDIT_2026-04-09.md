# Baseline Audit — 2026-04-09

Status: accepted baseline

Related:

- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/DESIGN.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-report-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `crates/fret-icons-generator/src/naming.rs`
- `crates/fretboard/src/icons/mod.rs`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`

## Findings

### 1. The shipped config contract already solves the hard part

Fret already has a versioned `presentation-defaults.json` contract and import/generator support
for explicit per-icon or pack-level presentation policy. This follow-on does not need a new file
format.

### 2. The missing surface is local-SVG scaffolding, not another import rule

The existing helper lanes cover Iconify provenance well, but users starting from a repository-owned
SVG directory still have to author per-icon override scaffolding by hand.

### 3. Generic SVG analysis is too weak for hidden pack-level defaults

Without explicit provenance like Iconify `palette`, SVG analysis can at best provide conservative
signals for per-icon authored-color exceptions. It should not guess a pack-wide
`default_render_mode`.

### 4. Suggested icon ids must match generator/import naming exactly

Any helper that emits `icon_name` overrides must share the same normalization rules as
`icons import svg-dir ...`, or the output becomes invalid at the first rename edge case.

### 5. Analysis failures should remain review artifacts, not global failure

Large third-party SVG directories often contain malformed or unsupported files. The helper should
keep those failures visible in a report while still producing useful output for the analyzable
subset.

## Baseline verdict

Treat this as a narrow helper-owned follow-on:

- keep the existing config contract,
- expose one explicit local-SVG suggestion helper,
- classify only strong per-icon authored-color evidence,
- and leave pack-level defaults unchanged.
