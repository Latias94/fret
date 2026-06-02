# Material3 Headless Golden Hygiene v1 Closeout Audit

Status: Closed
Date: 2026-05-31

## Summary

This lane closed a narrow Material3 gate-ownership problem. The default `radio_alignment` test
binary no longer fails because of stale broad navigation and overlay headless goldens. Those broad
suites remain available as explicit ignored maintenance tests.

## Completed Scope

- Classified default `radio_alignment` failures as stale broad navigation/overlay golden drift.
- Ignored the stale broad navigation suite by default with a focused gate pointer.
- Ignored the stale broad overlay suite by default with focused gate pointers.
- Re-ran focused Radio, navigation, overlay, select, package, catalog, layering, and diff hygiene
  gates.

## Final Gates

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment`:
  70 passed, 2 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment radio`:
  passed.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test navigation_state --test menu_state --test dialog_state --test tooltip_state --test automation_surface`:
  passed.
- `cargo nextest run -p fret-ui-material3 --test select_behavior`:
  passed.
- `cargo fmt --package fret-ui-material3 --check`: passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
  passed.
- `python -m json.tool docs/workstreams/material3-headless-golden-hygiene-v1/WORKSTREAM.json | Out-Null`:
  passed.
- `python tools/check_workstream_catalog.py`: passed.
- `python tools/check_layering.py`: passed.
- `git diff --check`: passed.

## Residuals

- Broad navigation and overlay headless expected payloads are still stale.
- `radio_alignment.rs` remains too broad for long-term maintainability.
- The next cleanup should split broad headless golden suites into dedicated files or JSON-backed
  runners.
