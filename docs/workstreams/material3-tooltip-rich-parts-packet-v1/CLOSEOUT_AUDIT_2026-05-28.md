# Material 3 Tooltip Rich Parts Packet v1 - Closeout Audit

Date: 2026-05-28
Status: Closed

## Result

The lane is closed.

Rich tooltips now expose stable text-part automation surfaces:

- `tooltip.title`
- `tooltip.supporting-text`

Plain and rich tooltips share root/chrome semantics wiring, and `PlainTooltip` now uses the shared
tooltip policy root. Tooltip overlay input behavior remains pointer transparent.

## Gate Evidence

- `cargo fmt --package fret-ui-material3 -- --check`: passed.
- `python -m json.tool docs/workstreams/material3-tooltip-rich-parts-packet-v1/WORKSTREAM.json | Out-Null`:
  passed.
- `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json | Out-Null`:
  passed.
- `python tools/check_workstream_catalog.py`: passed, 480 dedicated directories and 47 standalone
  markdown files.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tooltip_and_snackbar_expose_stable_part_test_ids`:
  passed.
- `cargo nextest run -p fret-ui-material3 --test radio_alignment rich_tooltip_opens_and_closes_on_hover_smoke tooltip_is_click_through_and_does_not_block_underlay_activation_across_schemes`:
  passed.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
  passed.

## Boundary Notes

- No `crates/*` contract changed.
- No `fret-ui-kit` tooltip policy changed.
- Rich tooltip action interactivity remains a separate mechanism follow-on because current tooltip
  overlays are deliberately pointer transparent.

