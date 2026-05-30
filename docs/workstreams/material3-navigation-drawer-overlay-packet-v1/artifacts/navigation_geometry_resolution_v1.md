# Navigation Geometry Resolution v1

Status: done
Date: 2026-05-27

## Problem

M3ND-010 showed two different drift classes in the navigation golden suite:

- stale outer slot expectations for NavigationBar, NavigationRail, and the modal underlay probe,
- a real Drawer/ModalDrawer selected-pill fill-boundary drift where the selected pill shrank to an
  icon-sized rectangle.

## Fix

`NavigationDrawer` now gives its internal `RovingFlexProps` an explicit full-size layout:

```rust
props.flex.layout.size.width = Length::Fill;
props.flex.layout.size.height = Length::Fill;
```

That keeps drawer items stretched inside the fixed-width drawer container and restores the selected
pill to the intended full row width in both standard and modal drawer compositions.

## Golden Decision

After the recipe fix, the remaining navigation suite mismatch was the stale fixture expectation
identified by M3ND-010:

- NavigationBar was previously expected to stretch to the padded viewport even though the fixture
  did not provide an explicit full-width slot.
- NavigationRail and NavigationDrawer containers were previously expected to stretch to the padded
  viewport height even though the fixture used `with_padding` without a full-height slot.
- The modal underlay probe was previously expected to stretch to the padded viewport, which is not
  intrinsic to the button or modal drawer recipe.

The navigation goldens were refreshed after this classification.

## Proof

```powershell
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_navigation_suite_goldens_v1; Remove-Item Env:FRET_UPDATE_GOLDENS
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_navigation_suite_goldens_v1
```

Both commands passed on 2026-05-27. The second command passed without `FRET_UPDATE_GOLDENS`.

## Layer Classification

- `material_recipe`: owns the drawer internal fill constraint fix.
- `test_harness`: owned stale outer slot expectations and refreshed navigation snapshots.
- `material_foundation`: no shared navigation foundation change was needed.
- `kit_policy`: overlay/focus behavior was unchanged.
- `mechanism`: no `crates/*` gap was found.
