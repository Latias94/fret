# Navigation Golden Baseline v1

Status: red baseline
Date: 2026-05-27

## Command

```powershell
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_navigation_suite_goldens_v1
```

## Result

The gate fails at:

```text
goldens/material3-headless/v1/material3-navigation.scale1_0.dark.tonal_spot.json
```

The first visible mismatch includes the same four cases:

- `bar.selected`
- `drawer.selected`
- `modal_drawer.open`
- `rail.selected`

The signatures are broadly stable in the first mismatch. The visible drift is geometry-heavy:

- `bar.selected` container width differs.
- `drawer.selected` container height and selected pill width/radius differ.
- `modal_drawer.open` underlay/panel/selected pill geometry differs.
- `rail.selected` container height differs.

## Initial Classification

This is a baseline red gate for M3ND-010. It is not safe to resolve it with a blanket golden
refresh.

Classification:

- `bar.selected`: stale harness/slot expectation. The fixture wraps the NavigationBar in
  `with_padding` without an explicit full-width slot. The current output shrink-wraps the bar while
  the old golden expected implicit stretching to the padded viewport.
- `rail.selected`: stale harness/slot expectation for the same reason on the vertical axis. The
  NavigationRail recipe has a fixed width and `Length::Fill` height, but the test fixture does not
  explicitly provide a full-height slot.
- `drawer.selected`: likely real recipe/harness boundary drift. The drawer container remains fixed
  width, but the selected active pill shrinks from the intended full item row to an icon-sized
  rectangle. That points at an internal flex/fill propagation issue in the drawer item stack rather
  than only an outer fixture size expectation.
- `modal_drawer.open`: mixed. The underlay button stretch is stale fixture expectation, but the
  modal drawer selected pill shows the same icon-sized shrink as `drawer.selected`.

No renderer or `crates/*` mechanism defect is proven by this red gate. The next task should inspect
NavigationDrawer's internal roving flex/item fill constraints before refreshing navigation goldens.

## Next Evidence Needed

- Packet NavigationDrawer and ModalNavigationDrawer selectors/semantics/focus gates in M3ND-020.
- In M3ND-030, repair drawer selected-pill geometry if the recipe is missing explicit internal
  fill constraints.
- Only then refresh the stale navigation goldens and rerun
  `material3_headless_navigation_suite_goldens_v1`.
