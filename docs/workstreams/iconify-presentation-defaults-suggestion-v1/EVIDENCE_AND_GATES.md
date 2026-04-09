# Iconify Presentation Defaults Suggestion v1 — Evidence and Gates

Status: Closed
Last updated: 2026-04-09

## Smallest current repro

Use this sequence before changing the shipped suggestion helper:

```bash
cargo nextest run -p fretboard
cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface
```

What this proves now:

- the helper CLI and end-to-end import proof still work,
- the existing import path still consumes the emitted config file,
- and public docs still teach the helper as an explicit suggestion rather than a hidden default.

## Current evidence set

- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/BASELINE_AUDIT_2026-04-09.md`
  freezes the baseline gap between acquisition provenance and generated-pack config.
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
  freezes the helper shape:
  - separate `icons suggest ...` command,
  - Iconify acquisition provenance input only,
  - existing config schema output,
  - and strict missing-evidence failure.
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/M2_PROOF_SURFACE_2026-04-09.md`
  closes the proof on:
  - helper CLI implementation,
  - schema-aligned output,
  - and end-to-end import flow.
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/CLOSEOUT_AUDIT_2026-04-09.md`
  closes the lane on the shipped helper and docs.

## Gate set

### Helper + import proof

```bash
cargo nextest run -p fretboard
```

### Public docs / source-policy gate

```bash
cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface
```

## Evidence anchors

- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/DESIGN.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/TODO.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/MILESTONES.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/M2_PROOF_SURFACE_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/suggest.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/icons/acquire.rs`
- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/cli/help.rs`
- `ecosystem/fret/src/lib.rs`

## Reference posture

- `palette` is suggestion evidence only, not the hidden import default.
- Missing `palette` remains an explicit failure in v1.
- Keep this lane closed on the thin helper surface rather than widening it into generic SVG
  analysis or pack-policy inference.
