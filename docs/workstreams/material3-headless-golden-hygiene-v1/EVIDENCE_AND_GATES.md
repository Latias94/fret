# Material3 Headless Golden Hygiene v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-31

## Repro Surface

- Mixed default gate before the hygiene change:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment --no-fail-fast`
  - Result: 70 passed, 2 failed.
  - Failures:
    - `material3_headless_navigation_suite_goldens_v1`
    - `material3_headless_overlays_suite_goldens_v1`

## Gates

- Default Radio-alignment binary:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment`
- Focused Radio behavior:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment radio`
- Focused navigation and overlay state gates:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test navigation_state --test menu_state --test dialog_state --test tooltip_state --test automation_surface`
- Select behavior:
  - `cargo nextest run -p fret-ui-material3 --test select_behavior`
- Package checks:
  - `cargo fmt --package fret-ui-material3 --check`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Workstream state:
  - `python -m json.tool docs/workstreams/material3-headless-golden-hygiene-v1/WORKSTREAM.json | Out-Null`
  - `python tools/check_workstream_catalog.py`
- Layering:
  - `python tools/check_layering.py`
- Diff hygiene:
  - `git diff --check`

## Evidence Anchors

- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `ecosystem/fret-ui-material3/tests/navigation_state.rs`
- `ecosystem/fret-ui-material3/tests/menu_state.rs`
- `ecosystem/fret-ui-material3/tests/dialog_state.rs`
- `ecosystem/fret-ui-material3/tests/tooltip_state.rs`
- `ecosystem/fret-ui-material3/tests/select_behavior.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-headless-golden-hygiene-v1/DESIGN.md`
- `docs/workstreams/material3-headless-golden-hygiene-v1/TODO.md`
- `docs/workstreams/material3-headless-golden-hygiene-v1/CLOSEOUT_AUDIT_2026-05-31.md`

## Closeout Evidence

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment`:
  70 passed, 2 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment radio`:
  passed.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test navigation_state --test menu_state --test dialog_state --test tooltip_state --test automation_surface`:
  passed.
- `cargo nextest run -p fret-ui-material3 --test select_behavior`:
  passed.
- Package, catalog, layering, and diff hygiene gates passed during closeout.
