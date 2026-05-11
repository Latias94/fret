# M4 Feature Payload Bundle Assertion

Status: Landed
Date: 2026-05-12

## Decision

The code-editor torture feature payload fixture now has a bundle-level assertion. The gate verifies
that, after warmup, UI Gallery `code_editor.torture.feature_payloads` snapshots exist and expose a
stable, non-zero payload package:

- diagnostic spans,
- diagnostic line summaries,
- range decorations,
- gutter markers,
- semantic tokens,
- schema/version fields,
- buffer revision,
- display-map epoch.

This turns the public feature payload surface into a repeatable diagnostics contract instead of a
visual-only gallery fixture.

## Public Gate

Use the public `fretboard diag stats` path against a captured bundle:

```powershell
cargo run -p fretboard -- diag stats <bundle.schema2.json> --warmup-frames 5 --check-ui-gallery-code-editor-torture-feature-payloads-stable
```

The gate writes:

```text
check.ui_gallery_code_editor_torture_feature_payloads_stable.json
```

next to the bundle artifact.

## Scope

The assertion is intentionally data-contract focused. It does not require a final visual treatment
for diagnostics, gutter icons, semantic token colors, hover cards, or code actions. Those remain
feature/policy follow-ups above the editor payload model.

## Evidence

- Stats checker:
  `crates/fret-diag/src/stats/ui_gallery_code_editor.rs`
- Public `diag stats` flag:
  `crates/fret-diag/src/cli/contracts/commands/stats.rs`
- Stats command wiring:
  `crates/fret-diag/src/diag_stats.rs`
- CLI migration coverage:
  `crates/fret-diag/src/cli/cutover.rs`
- Post-run registry entry:
  `crates/fret-diag/src/registry/checks/builtin_post_run/ui_gallery/code_editor.rs`
- Unit coverage:
  `crates/fret-diag/src/tests.rs`

## Gates

```powershell
cargo fmt -p fret-diag --check
cargo check -p fret-diag
cargo nextest run -p fret-diag ui_gallery_code_editor_feature_payloads_gate --no-fail-fast
cargo nextest run -p fret-diag migrated_stats_builds_a_real_context --no-fail-fast
cargo run -p fretboard -- diag stats <bundle.schema2.json> --warmup-frames 5 --check-ui-gallery-code-editor-torture-feature-payloads-stable
```
