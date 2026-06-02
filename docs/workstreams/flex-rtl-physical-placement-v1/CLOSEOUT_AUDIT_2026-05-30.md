# Flex RTL Physical Placement v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-30

## Shipped

- Captured the nearest `LayoutDirection` provider value on `AnyElement` during construction.
- Copied the captured direction into `ElementRecord` during mount.
- Added a shared `taffy_flex_direction` bridge that maps horizontal RTL Flex rows to
  `FlexDirection::RowReverse`.
- Routed the flow engine and measure-path Flex root styles through that bridge.
- Kept the layout-time horizontal auto-margin correction on the LTR path only; RTL rows rely on
  Taffy's row-reverse auto-margin behavior instead of applying the old LTR-only postprocess.
- Rejected horizontal RTL Flex rows from the clean-geometry fast path so cached LTR proofs cannot
  derive stale child bounds.

## Verification

Passed on 2026-05-30:

- `cargo fmt -p fret-ui -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui --lib flex_row_uses_provided_rtl_layout_direction_for_physical_order flex_row_uses_nested_ltr_provider_inside_rtl_scope mechanism_harness_layout_primitives_match_oracles mechanism_harness_layout_primitives_rtl_physical_edges_match_oracles`
- `cargo check -p fret-ui --tests`
- `cargo check -p fret-ui-shadcn --tests`
- `cargo clippy -p fret-ui --tests --no-deps -- -D warnings`
- `cargo clippy -p fret-ui-shadcn --tests --no-deps -- -D warnings`
- `python -m json.tool docs/workstreams/flex-rtl-physical-placement-v1/WORKSTREAM.json | Out-Null`
- `python tools/check_layering.py`
- `python tools/report_largest_files.py --top 30 --min-lines 800`
- `python tools/check_workstream_catalog.py`
- `git diff --check`

## Residuals

- This is horizontal Flex physical placement only.
- Logical margins, logical insets, and column cross-axis RTL mirroring remain follow-ons.
- Component-level keyboard and indicator policies remain in ecosystem crates.
- Provider-sensitive cached subtrees keep the direction captured when they were built, matching the
  existing view-cache provider contract.
