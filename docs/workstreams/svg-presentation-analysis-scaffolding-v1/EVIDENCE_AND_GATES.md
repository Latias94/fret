# Evidence And Gates

Status: Closed
Last updated: 2026-04-09

## Smallest repro

1. Start from a local SVG directory:
   `fretboard icons suggest svg-dir-presentation-overrides --source ./icons --out ./presentation-defaults.json --report-out ./presentation-defaults.report.json`
2. Review the emitted config/report and keep only the explicit config on the import path:
   `fretboard icons import svg-dir --source ./icons --crate-name my-icons --vendor-namespace app --presentation-defaults ./presentation-defaults.json`

## Gates

```bash
cargo nextest run -p fretboard
cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface
git diff --check
```

## Evidence anchors

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
- `docs/workstreams/iconify-presentation-defaults-report-v1/CLOSEOUT_AUDIT_2026-04-09.md`
