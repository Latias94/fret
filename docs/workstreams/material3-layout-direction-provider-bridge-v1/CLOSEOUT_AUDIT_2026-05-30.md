# Material3 Layout Direction Provider Bridge v1 Closeout Audit

Status: Closed
Date: 2026-05-30

## Outcome

This follow-on closes the gap between Material3-owned direction policy and the core layout direction
provider consumed by Fret layout mechanisms.

## Shipped Changes

- `with_material_layout_direction` and `with_material_layout_direction_override` now install the
  resolved core `LayoutDirection` provider for their subtree.
- `with_default_material_layout_direction` now takes an explicit fallback direction so "use
  default" scopes can mask an outer Material override while still giving core layout a concrete
  direction.
- `with_material_resolved_layout_direction` resolves Material override/theme fallback direction and
  provides it to descendants as core layout direction.
- Tabs uses the resolved direction bridge around its horizontal row subtree.
- Tests prove both foundation provider behavior and Tabs RTL physical order.

## Verification

- `cargo fmt -p fret-ui-material3`
- `cargo nextest run -p fret-ui-material3 --lib layout_direction_inherits_masks_and_restores material_resolved_layout_direction_provides_core_direction_for_elements`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state rtl_tabs_theme_direction_mirrors_physical_tab_order`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `python -m json.tool docs\workstreams\material3-layout-direction-provider-bridge-v1\WORKSTREAM.json | Out-Null`
- `python tools\check_workstream_catalog.py`
- `python tools\check_layering.py`
- `git diff --check`

## Residual Risk

- Logical edge padding/margins/insets are not solved here.
- Tabs is the first consumer proof, not a complete Material3 RTL visual sweep.
- Chip, segmented button, slider, badge, and navigation RTL visuals should be audited in separate
  component follow-ons.
