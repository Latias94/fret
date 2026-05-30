# Material 3 Tooltip Rich Parts Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Current Evidence

- Compose Material3 rich tooltip has title, supporting text, and optional action content.
- Fret `RichTooltip` currently exposes root/chrome selectors but not text-part selectors.
- Fret tooltip overlays are pointer transparent by current `fret-ui-kit` window overlay tests.

## Gate Set

```powershell
python -m json.tool docs/workstreams/material3-tooltip-rich-parts-packet-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
cargo fmt --package fret-ui-material3 -- --check
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tooltip_and_snackbar_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment rich_tooltip_opens_and_closes_on_hover_smoke
cargo nextest run -p fret-ui-material3 --test radio_alignment tooltip_is_click_through_and_does_not_block_underlay_activation_across_schemes
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Evidence Log

- 2026-05-28: Opened the lane from `M3CAS-070-F2` and `M3CAS-070-F4`.
- 2026-05-28: M3TT-020 selector and refactor proof.
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tooltip_and_snackbar_expose_stable_part_test_ids`:
    passed.
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment rich_tooltip_opens_and_closes_on_hover_smoke tooltip_is_click_through_and_does_not_block_underlay_activation_across_schemes`:
    passed.
- 2026-05-28: M3TT-030 closeout verification.
  - `cargo fmt --package fret-ui-material3 -- --check`: passed.
  - `python -m json.tool docs/workstreams/material3-tooltip-rich-parts-packet-v1/WORKSTREAM.json | Out-Null`:
    passed.
  - `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json | Out-Null`:
    passed.
  - `python tools/check_workstream_catalog.py`: passed, 480 dedicated directories and 47
    standalone markdown files.
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tooltip_and_snackbar_expose_stable_part_test_ids`:
    passed.
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment rich_tooltip_opens_and_closes_on_hover_smoke tooltip_is_click_through_and_does_not_block_underlay_activation_across_schemes`:
    passed.
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
    passed.
