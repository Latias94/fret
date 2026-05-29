# Material3 List Roving Behavior Packet v2

Date: 2026-05-29
Task: M3PV2-082

## Truth

- The current Fret `List` recipe is a selectable Material list surface: focus movement selects the
  focused enabled item, matching the recipe's existing selection-follows-focus contract.
- ArrowDown / ArrowUp roving focus skips disabled list items and updates the bound selected value
  only when an enabled item becomes active.
- Home and End move to the first and last enabled list items.
- `loop_navigation(false)` is honored at both ends: ArrowUp at the first enabled item and
  ArrowDown at the last enabled item stay on the current item.
- If the selected value points to a disabled item, that item can remain selected semantically, but
  the list tab stop falls back to the first enabled item.

## Sources

- Compose Material3 `ListItem.kt`: interactive `ListItem` overloads use `combinedClickable` with
  `enabled`, publish selected semantics for single-selection list items, and expose disabled state
  to accessibility services when disabled.
- Compose Material3 `ListItem.kt`: ListItem itself is an item recipe, not a collection-level
  roving-focus primitive; Fret's list-level roving behavior remains a recipe policy for the current
  selectable Fret List API.
- Existing Fret Material navigation recipes (`NavigationDrawer`, `NavigationRail`, `NavigationBar`)
  are the in-tree roving exemplar for respecting `wrap` while skipping disabled destinations.

MUI Material UI is not available in this checkout's `repo-ref/`. Base UI has no direct Material
List equivalent for this surface; it remains a supporting headless reference for disabled action
semantics only.

## Layer Finding

This packet found a Material List recipe behavior bug, not a core or kit mechanism gap:

- `crates/fret-ui` already exposed roving-focus navigation, disabled item metadata, focus actions,
  and selected/disabled semantics.
- Existing Material navigation recipes already implemented correct no-loop roving behavior with
  disabled-item skipping.
- `List` had a local custom ArrowUp/ArrowDown handler that always wrapped at list edges, ignoring
  `loop_navigation(false)`.
- `List` also used the selected index as the tab stop even when the selected item was disabled,
  leaving no enabled focus action in that edge case.

No shared `crates/*` or `fret-ui-kit` change was justified.

## Artifacts

- `ecosystem/fret-ui-material3/src/list.rs`
- `ecosystem/fret-ui-material3/tests/list_state.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Wiring

- `List::into_element(...)` now filters disabled selected items out of tab-stop selection and
  falls back to the first enabled item.
- `List` disables its roving group when the whole list is disabled.
- List ArrowUp/ArrowDown navigation now mirrors the existing Material navigation recipe behavior:
  wrap only when `it.wrap` is true; otherwise search only before or after the current item.
- The new `list_state` gates use persistent models and live semantics ids to prove focus movement,
  model updates, disabled skipping, Home/End behavior, no-loop boundaries, and disabled-selected
  tab-stop fallback.

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test list_state
```

Failed because ArrowUp at the first enabled list item moved focus to the last enabled item despite
`loop_navigation(false)`, and a disabled selected value left the first enabled item without a focus
action.

Green gate during implementation:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test list_state
```

The focused gate now proves disabled-item skipping, selection-follows-focus, Home/End behavior,
no-loop edge behavior, and disabled-selected tab-stop fallback.

## Residual Risk

- This packet closes the current selectable List behavior axis; it does not add new public APIs for
  Compose's separate clickable, single-selection, multi-selection, or segmented list item overloads.
- Drag/reorder, reveal, avatars/images/video, segmented list items, and multiline wrapped
  supporting text remain future component-surface work.
- A future cleanup could extract the duplicated roving no-loop/disabled-skip algorithm shared by
  Material List, navigation, tabs, radio, and menu recipes into a local Material helper, but this
  packet kept the edit scoped to the component with the observed bug.
