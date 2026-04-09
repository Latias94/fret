# Evidence And Gates

Status: Closed
Last updated: 2026-04-09

## Smallest repro

1. Start from explicit Iconify acquisition provenance:
   `fretboard icons acquire iconify-collection --collection mdi --icon home --out ./iconify/mdi-home.json`
2. Emit both the config suggestion and the optional report:
   `fretboard icons suggest presentation-defaults --provenance ./iconify/mdi-home.provenance.json --out ./iconify/presentation-defaults.json --report-out ./iconify/presentation-defaults.report.json`
3. Review both artifacts, then pass only the config file into import:
   `fretboard icons import iconify-collection --source ./iconify/mdi-home.json --crate-name mdi-icons --vendor-namespace mdi --presentation-defaults ./iconify/presentation-defaults.json`

## Gates

```bash
cargo nextest run -p fretboard
cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface
```

## Evidence anchors

- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/suggest.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/cli/help.rs`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`
- `ecosystem/fret/src/lib.rs`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/CLOSEOUT_AUDIT_2026-04-09.md`
