# Material 3 shadcn-level completeness matrix v1

Status: Active
Last updated: 2026-05-31

This matrix tracks `ecosystem/fret-ui-material3` against the completeness bar we use for mature
`fret-ui-shadcn` recipes. It is not a visual-copying exercise: Material 3 remains spec/token-driven,
while shadcn is used as a Fret-side reference for API breadth, stable parts, semantics, behavior
gates, and gallery teaching surfaces.

## Axes

- API: public authoring surface covers the normal upstream use cases without bespoke app code.
- Parts: stable `test_id` anchors exist for root/trigger/content/important slots.
- Semantics: roles, labels, relations, and collection metadata are exposed.
- Behavior: keyboard/focus/dismiss/selection behavior is policy-complete for the component family.
- Tokens/style: Material tokens and ADR 0220-style override surfaces cover expected visual slots.
- Motion/ink: state layers, ripple, indicator, presence, or overlay motion are foundation-backed.
- Gallery/gates: first-party snippet plus targeted tests or diag scripts prove the important claims.

Legend: `Complete`, `Strong`, `Partial`, `MVP`, `Gap`.

## Component Matrix

| Component family | API | Parts | Semantics | Behavior | Tokens/style | Motion/ink | Gallery/gates | Notes |
|---|---|---:|---:|---:|---:|---:|---:|---|
| Button / FAB / IconButton | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Main residual is broader expressive variants rather than shadcn-level surface shape. |
| Checkbox / Radio / Switch | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Choice controls share `WidgetStates::SELECTED` and Material indication. |
| Slider / RangeSlider | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Keep future work focused on higher-density examples and AT validation. |
| SegmentedButton | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Single and multi-select are covered. |
| Tabs | Strong | Strong | Strong | Strong | Strong | Strong | Strong | 2026-05-31: gained active `TabPanel` content API, panel semantics, label relation, and content `test_id`. Residuals: optional force-mount/presence panels and richer overflow polish. |
| Menu / DropdownMenu | Strong | Strong | Strong | Strong | Strong | Strong | Strong | 2026-05-31: gained labels, rich slots, checkbox/radio items, two-line rows, and close-on-select coverage. Residuals: grouped API, submenus, long-menu scroll affordances. |
| Select / ExposedDropdown / Autocomplete | Strong | Strong | Strong | Strong | Strong | Partial | Strong | Field-family overlay behavior is mature; keep improving active-descendant/AT and motion evidence. |
| TextField | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Floating-label and token work are mature; keep extending field-family compositions. |
| Dialog / BottomSheet | Strong | Strong | Strong | Strong | Strong | Partial | Strong | Overlay policy is in ecosystem layers; motion/presence parity remains the main polish area. |
| Tooltip | Partial | Strong | Strong | Partial | Strong | Partial | Strong | Rich tooltip actions are constrained by non-hit-testable tooltip overlay policy. |
| Snackbar | Partial | Partial | Strong | Strong | Partial | Partial | Strong | Uses toast-layer infrastructure; public style surface and richer action layout remain v2 work. |
| NavigationBar / Rail / Drawer | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Continue polishing adaptive examples and route/content integration. |
| TopAppBar | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Scroll behavior policy is in Material ecosystem code; nested-scroll consumption remains a future mechanism trigger. |
| DatePicker / TimePicker | Strong | Strong | Strong | Strong | Strong | Partial | Strong | Main residual is broader composition/demo coverage, not basic API completeness. |
| SearchBar / SearchView | Strong | Strong | Strong | Strong | Strong | Partial | Strong | Keep validating overlay/content relation and mobile/fullscreen behavior. |

## Current Batches

- 2026-05-31 Menu breadth:
  - `MenuLabel`, rich menu item slots, checkbox/radio item models, two-line row metrics, stable part
    ids, and gallery examples.
  - Gates: `menu_state`, `menu` / `dropdown_menu` lib tests, automation-surface test, clippy, layering.
- 2026-05-31 Tabs panel ownership:
  - `TabPanel`, `.panel/.panels`, active tabpanel semantics, `labelled_by` relation to the selected
    tab, derived tab `controls`, and gallery snippet content panels.
  - Gates: `tabs_state`, tabs automation-surface test, clippy, layering.

## Next Recommended Focus

1. Finish Tabs residuals only if panel presence becomes a real app need:
   `force_mount` content, presence motion, and overflow/scroll affordance polish.
2. Move to Snackbar style/API breadth:
   public `SnackbarStyle`/host override story, action/dismiss slot anchors, and multi-line layout
   gate.
3. Revisit Tooltip policy:
   decide whether rich tooltip actions require a popover-like hit-testable surface or should remain
   non-interactive per tooltip semantics.
