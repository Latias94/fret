# Material3 Logical Edge Layout v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-30

## Repro

- `cargo nextest run -p fret-ui-material3 --lib foundation::logical_edges`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state rtl_filter_and_input_chips_mirror_inline_content_edges`

## Gates

- `cargo fmt -p fret-ui-material3`
- `cargo nextest run -p fret-ui-material3 --lib foundation::logical_edges`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state rtl_filter_and_input_chips_mirror_inline_content_edges`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `python -m json.tool docs/workstreams/material3-logical-edge-layout-v1/WORKSTREAM.json | Out-Null`
- `python tools/check_workstream_catalog.py`
- `python tools/check_layering.py`
- `git diff --check`

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/foundation/logical_edges.rs`
- `ecosystem/fret-ui-material3/src/filter_chip.rs`
- `ecosystem/fret-ui-material3/src/input_chip.rs`
- `ecosystem/fret-ui-material3/tests/chip_state.rs`

## Verified On 2026-05-30

- `cargo fmt -p fret-ui-material3`
- `cargo nextest run -p fret-ui-material3 --lib foundation::logical_edges`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state rtl_filter_and_input_chips_mirror_inline_content_edges`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `python -m json.tool docs\workstreams\material3-logical-edge-layout-v1\WORKSTREAM.json | Out-Null`
- `python tools\check_workstream_catalog.py`
- `python tools\check_layering.py`
- `git diff --check`
