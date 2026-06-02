# Material3 Headless Golden Harness Split v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-31

## Gates

- Format:
  - `cargo fmt --package fret-ui-material3 --check`
- Focused Radio binary:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment`
- Broad headless golden binary:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_headless_goldens`
- Focused Radio-owned filter:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment radio`
- Package checks:
  - `cargo check -p fret-ui-material3 --features diagnostics --test material3_headless_goldens --test radio_alignment`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Workstream state:
  - `python -m json.tool docs/workstreams/material3-headless-golden-harness-split-v1/WORKSTREAM.json | Out-Null`
  - `python tools/check_workstream_catalog.py`
- Layering:
  - `python tools/check_layering.py`
- Diff hygiene:
  - `git diff --check`

## Evidence Anchors

- `ecosystem/fret-ui-material3/tests/material3_headless_goldens.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `ecosystem/fret-ui-material3/tests/support/goldens.rs`
- `docs/workstreams/material3-headless-golden-harness-split-v1/DESIGN.md`
- `docs/workstreams/material3-headless-golden-harness-split-v1/TODO.md`
- `docs/workstreams/material3-headless-golden-harness-split-v1/CLOSEOUT_AUDIT_2026-05-31.md`

## Closeout Evidence

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_alignment`:
  51 passed, 0 skipped.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test material3_headless_goldens`:
  19 passed, 2 skipped.
- Remaining gates in this file passed during closeout.
