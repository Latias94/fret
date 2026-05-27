# Navigation Drawer And Modal Drawer Packet v1

Status: packet done with geometry follow-on
Date: 2026-05-27

## Truth

- NavigationDrawer exposes stable root/item/item.chrome selectors for automation.
- ModalNavigationDrawer exposes stable root/scrim/scrim.chrome/panel selectors for automation.
- Modal drawer focus is contained while open and restored to the trigger after close.
- Overlay dismissal, focus trap, and focus restore stay in `fret-ui-kit`.
- Drawer selected-pill geometry remains unresolved until M3ND-030 repairs or disproves the
  fill-boundary drift found by M3ND-010.

## Artifacts

- `ecosystem/fret-ui-material3/src/navigation_drawer.rs`
- `ecosystem/fret-ui-material3/src/modal_navigation_drawer.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `artifacts/navigation_golden_baseline_v1.md`

## Wiring

- `NavigationDrawerItem::test_id` stamps item roots and derives `.chrome` through
  `foundation::test_id`.
- `ModalNavigationDrawer::test_id` now derives dotted overlay parts:
  - `<base>`
  - `<base>.scrim`
  - `<base>.scrim.chrome`
  - `<base>.panel`
- `automation_surface.rs` renders both NavigationDrawer and an open ModalNavigationDrawer and
  asserts live visual bounds for those selectors.
- `modal_navigation_drawer_focus_is_contained_and_restored_across_schemes` verifies focus
  containment and focus restore through the existing overlay controller/focus-scope policy.

## Layer Classification

- `material_recipe`: owns drawer item chrome, selected pill composition, modal drawer root/scrim
  panel composition, and stable part IDs.
- `kit_policy`: owns modal overlay request, scrim dismissal, focus containment, and focus restore.
- `diagnostics/test_harness`: owns live selector assertions and the remaining navigation golden
  drift classification.
- `mechanism`: no `crates/*` gap is proven by this packet.

## Proof

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_navigation_drawer_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_modal_navigation_drawer_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment modal_navigation_drawer_focus_is_contained_and_restored_across_schemes
```

All three M3ND-020 gates passed on 2026-05-27.

## Residual Risk

M3ND-010 found that Drawer and ModalDrawer selected pills shrink to icon-sized rectangles in the
current generated navigation golden output. This packet proves selector and overlay policy
boundaries, not final visual geometry. M3ND-030 owns the selected-pill geometry repair or disproval
before navigation goldens can be refreshed.
