# Direction Infrastructure v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-30

## Evidence Anchors

- `crates/fret-core/src/layout_direction.rs`
- `crates/fret-ui/src/overlay_placement/types.rs`
- `ecosystem/fret-ui-kit/src/primitives/direction.rs`
- `ecosystem/fret-ui-kit/src/primitives/roving_focus_group.rs`
- `ecosystem/fret-ui-shadcn/src/direction.rs`
- `ecosystem/fret-ui-shadcn/src/rtl.rs`
- `ecosystem/fret-ui-material3/src/{chip_set.rs,segmented_button.rs,tabs.rs}`
- `repo-ref/base-ui/packages/react/src/direction-provider/DirectionProvider.tsx`
- `repo-ref/base-ui/packages/react/src/internals/direction-context/DirectionContext.tsx`

## Gates

- `cargo fmt -p fret-ui-kit -p fret-ui-shadcn -p fret-ui-material3`
- `cargo nextest run -p fret-ui-kit --lib primitives::direction primitives::roving_focus_group`
- `cargo nextest run -p fret-ui-shadcn --lib rtl`
- `cargo nextest run -p fret-ui-material3 --lib tabs::tests::tab_indicator_fallback_position_mirrors_logical_index_in_rtl`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state rtl_tabs_arrow_left_moves_to_next_logical_tab_without_wrapping`
- `cargo check -p fret-ui-kit --tests`
- `cargo check -p fret-ui-shadcn --tests`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-kit --tests --no-deps -- -D warnings`
- `cargo clippy -p fret-ui-shadcn --tests --no-deps -- -D warnings`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `python -m json.tool docs/workstreams/direction-infrastructure-v1/WORKSTREAM.json | Out-Null`
- `python tools/check_layering.py`
- `python tools/report_largest_files.py --top 30 --min-lines 800`
- `python tools/check_workstream_catalog.py`

All gates passed on 2026-05-30.

## Residuals

- This lane does not add global RTL physical placement to Fret Flex.
- Material3 still has additional component-specific RTL callsites, such as sliders and deletable
  chips, that should migrate opportunistically when their component lanes are touched.
