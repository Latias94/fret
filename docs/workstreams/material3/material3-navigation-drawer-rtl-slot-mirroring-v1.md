Status: Complete
Last updated: 2026-06-01

# Material 3 NavigationDrawer RTL Slot Mirroring

## Truth

- `NavigationDrawerItem` treats its icon/label group as the logical inline-start content and its
  badge as logical inline-end content, matching Compose Material3's `icon -> label(weight) -> badge`
  row.
- In LTR, the icon appears before the label on the physical left, and the badge appears on the
  physical right.
- In RTL, the same logical order mirrors physically: the badge moves to the physical left, and the
  icon appears at the physical right edge of the item.
- Drawer item inline padding uses Material logical insets (`start = 16dp`, `end = 24dp`) instead of
  fixed physical left/right padding.
- NavigationDrawer roving remains vertical; RTL should not change Up/Down, Home/End, selection, or
  semantics metadata.

## Artifacts

- Component recipe:
  `ecosystem/fret-ui-material3/src/navigation_drawer.rs`
- Regression test:
  `ecosystem/fret-ui-material3/tests/navigation_state.rs`
- Completeness tracker:
  `docs/workstreams/material3/material3-shadcn-level-completeness-v1.md`

## Wiring

- The fix belongs in the Material3 component recipe plus existing Material layout-direction
  foundation. No `crates/fret-ui` mechanism change is expected.
- `NavigationDrawer` should resolve the Material theme default layout direction and provide it to
  its subtree, matching the pattern already used by `NavigationBar` and `Tabs`.
- The item row should map Compose's logical `Row` behavior onto Fret's current physical flex
  primitives by explicitly mirroring child order and logical edge padding under RTL.

## Proof

- Add a focused geometry test with stable part ids:
  `navigation_drawer_rtl_theme_direction_mirrors_item_slots_and_padding`.
- The test must compare LTR and RTL item slot order and assert logical start/end padding against
  the item chrome bounds.
- Validation:
  - `cargo fmt -p fret-ui-material3`
  - `cargo test -p fret-ui-material3 --features diagnostics --test navigation_state navigation_drawer_rtl_theme_direction_mirrors_item_slots_and_padding -- --exact`
  - `cargo test -p fret-ui-material3 --features diagnostics --test navigation_state`
  - `cargo test -p fret-ui-material3 --features diagnostics --test navigation_drawer_state`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --test navigation_state --no-deps -- -D warnings`
  - `python tools/check_layering.py`
  - `python tools/check_workstream_catalog.py`
  - `git diff --check`

## Residual Risk

- This batch covers slot mirroring and logical padding, not future predictive-back drawer motion.
- Gallery/diag expansion remains optional because this batch is covered by deterministic geometry
  assertions with stable drawer part ids.
