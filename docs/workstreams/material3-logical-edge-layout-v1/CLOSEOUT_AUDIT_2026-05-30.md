# Material3 Logical Edge Layout v1 Closeout Audit

Status: Closed
Date: 2026-05-30

## Outcome

This follow-on closes the first logical-edge gap exposed by the Material3 direction-provider bridge:
chip content rows can mirror under RTL without keeping inline-start/inline-end spacing pinned to
physical left/right.

## Shipped Changes

- Added `foundation::logical_edges` with:
  - `horizontal_logical_edges` for logical inline-start/end padding.
  - `set_inset_inline_end` for absolute inline-end overlay placement.
- FilterChip and InputChip now build content rows under the resolved Material layout direction
  provider.
- FilterChip and InputChip use logical padding for leading/trailing content spacing.
- FilterChip and InputChip trailing action overlays now pin to inline-end, not physical right.
- `chip_state` now verifies RTL FilterChip/InputChip content geometry.

## Verification

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

## Residual Risk

- Select/TextField field-family inline text and label insets still use physical edges.
- ChipSet/SegmentedButton row-level physical order and inline edge details still need a broader RTL
  visual sweep.
- The helper is Material foundation-local for now; promotion to `fret-ui-kit` should wait for a
  second design-system consumer.
