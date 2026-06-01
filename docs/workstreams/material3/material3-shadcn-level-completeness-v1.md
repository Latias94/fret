# Material 3 shadcn-level completeness matrix v1

Status: Active
Last updated: 2026-06-01

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
| Select / ExposedDropdown / Autocomplete | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Motion gates cover Select, Autocomplete, and ExposedDropdown chevron + overlay open/close fade/scale. |
| TextField | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Floating-label and token work are mature; keep extending field-family compositions. |
| Dialog / BottomSheet | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Dialog gates cover open/close fade + rise/scale; BottomSheet gates cover open/close sheet-height slide without panel fade. |
| Tooltip | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Rich tooltips now expose an action slot that opts into hit-testable tooltip content while plain/descriptive tooltips remain click-through by default. |
| Snackbar | Strong | Strong | Strong | Strong | Strong | Strong | Strong | 2026-06-01: gained public `SnackbarStyle`, host `.style(...)`, direct Material style adoption in the shared toast renderer, and style override paint/layout gates. Residuals: richer app-level queue policy examples. |
| NavigationBar / Rail / Drawer | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Continue polishing adaptive examples and route/content integration. |
| TopAppBar | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Scroll behavior policy is in Material ecosystem code; nested-scroll consumption remains a future mechanism trigger. |
| DatePicker / TimePicker | Strong | Strong | Strong | Strong | Strong | Strong | Strong | Motion gates cover modal fade/rise/scale and TimePicker docked clock-face transitions; residuals are broader composition/demo coverage. |
| SearchBar / SearchView | Strong | Strong | Strong | Strong | Strong | Strong | Strong | `SearchBarStyle`/`SearchViewStyle` now cover visual slots; motion gates cover hover/ripple, docked fade/height, and full-screen geometry expand/collapse. |

## Current Batches

- 2026-05-31 Menu breadth:
  - `MenuLabel`, rich menu item slots, checkbox/radio item models, two-line row metrics, stable part
    ids, and gallery examples.
  - Gates: `menu_state`, `menu` / `dropdown_menu` lib tests, automation-surface test, clippy, layering.
- 2026-05-31 Tabs panel ownership:
  - `TabPanel`, `.panel/.panels`, active tabpanel semantics, `labelled_by` relation to the selected
    tab, derived tab `controls`, and gallery snippet content panels.
  - Gates: `tabs_state`, tabs automation-surface test, clippy, layering.
- 2026-06-01 Snackbar style/API hardening:
  - `SnackbarStyle` now follows ADR 0220 for container/supporting/action/close colors plus
    container shape, padding, and single/two-line heights.
  - `fret-ui-kit` toast rendering now consumes direct `ToastLayerStyle` background/foreground,
    text, border, action, cancel, and close style fields instead of hardcoding the Sonner skin.
  - Gates: `window_overlays::tests::toast::toast_layer_style_direct_colors_are_painted`,
    `snackbar_state` including `snackbar_style_overrides_paint_and_layout_contract`.
- 2026-06-01 Tooltip visual style/API and rich-action hardening:
  - `TooltipStyle` follows ADR 0220 for plain/rich surface colors, text colors/text styles, shape,
    padding, max width, min size, rich elevation/shadow, rich text gap, and rich action label/row
    slots.
  - `RichTooltip::action_element(...)` now exposes the Material rich tooltip action slot and uses a
    `fret-ui-kit` tooltip hit-test opt-in so the action can receive pointer input without changing
    the plain tooltip click-through default.
  - Gallery examples now include styled plain/rich variants and an action-bearing rich tooltip.
  - Gates: `tooltip_state` including
    `plain_tooltip_style_overrides_paint_and_layout_contract` and
    `rich_tooltip_style_overrides_paint_parts_and_layout_contract`, plus
    `rich_tooltip_action_slot_is_hit_testable_and_stable`.
- 2026-06-01 Search style/API and motion evidence calibration:
  - `SearchBarStyle` now covers search field surface, shape, sizing, row padding/gap, input text,
    icon colors, and state-layer color.
  - `SearchViewStyle` now covers overlay container/divider/body/header slots and forwards
    `SearchBarStyle` to docked/full-screen search headers.
  - Gates: `search_bar_motion` and `search_view_behavior`, including style paint/layout tests plus
    existing hover/ripple and docked/full-screen motion tests.
- 2026-06-01 Date/Time motion evidence calibration:
  - DatePicker modal motion is already covered by fixed-frame fade + rise/scale tests.
  - TimePicker modal motion is covered for dial and input modes; docked clock-face transitions are
    covered by crossfade and selector-translation tests.
  - Gates: `date_picker_motion`, `time_picker_motion`.
- 2026-06-01 Dialog/BottomSheet motion hardening:
  - Dialog motion evidence was already present in `dialog_state` for open/close fade + rise/scale.
  - BottomSheet motion now asserts both open and close sheet-height slide and guards against
    accidental panel fade; scrim alpha remains token-driven paint, not panel opacity.
  - Gates: `dialog_state::dialog_scrim_and_panel_animate_on_open_close_frames`,
    `bottom_sheet_motion::modal_bottom_sheet_slides_from_own_height_without_panel_fade`.
- 2026-06-01 Select/Autocomplete motion evidence calibration:
  - Select motion gates cover chevron rotation and overlay fade/scale on open and close.
  - Autocomplete and ExposedDropdown motion gates cover chevron rotation and popup fade/scale on
    open and close.
  - Gates: `select_behavior::select_chevron_rotates_on_first_open_frame`,
    `autocomplete_motion::{autocomplete_popup_and_chevron_animate_on_open_close_frames,
    exposed_dropdown_popup_and_chevron_animate_on_open_close_frames}`.
- 2026-06-01 Field-overlay composition inside Dialog:
  - `Select` and `Autocomplete` now have dialog-nested overlay gates that prove popover-above-modal
    stacking, first-Escape nested dismissal, focus restoration/retention inside the dialog, and
    second-Escape modal dismissal.
  - Gates: `material3_overlay_interactions::{select_inside_dialog_closes_inner_popover_before_modal_dialog,
    autocomplete_inside_dialog_escape_closes_inner_popover_before_modal_dialog}` plus diag scripts
    `ui-gallery-material3-dialog-select-nested-overlay.json` and
    `ui-gallery-material3-autocomplete-dialog-nested-overlay.json`.

## Next Recommended Focus

1. Finish Tabs residuals only if panel presence becomes a real app need:
   `force_mount` content, presence motion, and overflow/scroll affordance polish.
2. Broaden gallery/diag coverage for field overlays inside `ModalBottomSheet`, then Search + Menu
   and navigation routed-content compositions through `material3-composition-hardening-v1.md`.
3. Start a second-pass polish lane for cross-component compositions: Search + Menu, field overlays
   inside Dialog/BottomSheet, and navigation surfaces with routed panel content.
