# Flex RTL Physical Placement v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-30

## Evidence Anchors

- `crates/fret-core/src/layout_direction.rs`
- `crates/fret-ui/src/element.rs`
- `crates/fret-ui/src/elements/cx.rs`
- `crates/fret-ui/src/declarative/frame.rs`
- `crates/fret-ui/src/declarative/taffy_layout.rs`
- `crates/fret-ui/src/layout/engine/flow.rs`
- `crates/fret-ui/src/declarative/host_widget/measure.rs`
- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
- `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`

## Gates

- `cargo fmt -p fret-ui -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui --lib flex_row_uses_provided_rtl_layout_direction_for_physical_order flex_row_uses_nested_ltr_provider_inside_rtl_scope mechanism_harness_layout_primitives_match_oracles mechanism_harness_layout_primitives_rtl_physical_edges_match_oracles`
- `cargo check -p fret-ui --tests`
- `cargo check -p fret-ui-shadcn --tests`
- `cargo clippy -p fret-ui --tests --no-deps -- -D warnings`
- `cargo clippy -p fret-ui-shadcn --tests --no-deps -- -D warnings`
- `python -m json.tool docs/workstreams/flex-rtl-physical-placement-v1/WORKSTREAM.json | Out-Null`
- `python tools/check_layering.py`
- `python tools/check_workstream_catalog.py`
- `python tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`

All gates passed on 2026-05-30.
