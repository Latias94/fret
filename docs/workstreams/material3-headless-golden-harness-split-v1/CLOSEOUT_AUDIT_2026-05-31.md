# Material3 Headless Golden Harness Split v1 Closeout Audit

Status: Closed
Date: 2026-05-31

## Summary

This lane split broad Material3 headless golden suites out of the historical
`radio_alignment.rs` god-test file. The default Radio binary now runs focused interaction and
Radio-owned checks, while broad headless golden maintenance lives in the new
`material3_headless_goldens` integration test binary.

## Completed Scope

- Moved all 21 `material3_headless_*_suite_goldens_v1` tests.
- Preserved ignored maintenance status for stale navigation and overlay broad golden suites.
- Removed broad golden imports from `radio_alignment.rs`.
- Kept `support::goldens` as the shared snapshot/assertion mechanism.

## Final Gates

- `cargo fmt --package fret-ui-material3 --check`: passed.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment`:
  51 passed, 0 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_headless_goldens`:
  19 passed, 2 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment radio`:
  passed.
- `cargo check -p fret-ui-material3 --features diagnostics --test material3_headless_goldens --test radio_alignment`:
  passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
  passed.
- `python -m json.tool docs/workstreams/material3-headless-golden-harness-split-v1/WORKSTREAM.json | Out-Null`:
  passed.
- `python tools/check_workstream_catalog.py`: passed.
- `python tools/check_layering.py`: passed.
- `git diff --check`: passed.

## Residuals

- Navigation and overlay broad expected payloads are still stale and remain ignored by default.
- `radio_alignment.rs` still has non-Radio interaction regressions that need future ownership
  decisions.
- The new headless golden owner file can be split further once fixture-driven family ownership is
  selected.
