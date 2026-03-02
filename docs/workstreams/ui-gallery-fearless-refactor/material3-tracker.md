# Material 3 UI Gallery Tracker (Snippet-backed Migration)

This tracker scopes the **Material 3** portion of UI Gallery.

Status (2026-03-02):

- shadcn + AI Elements pages are snippet-backed (Preview ≡ Code).
- Material 3 pages are still implemented as legacy previews under `apps/fret-ui-gallery/src/ui/previews/material3/**`
  and are **not** snippet-backed yet.

Goal:

- Migrate Material 3 pages to the same snippet-backed contract:
  - Preview renders compiled Rust snippet code.
  - Code tab displays `include_str!` of that same snippet file (drift-free).

References (reading aids):

- In-tree implementation: `ecosystem/fret-ui-material3`
- Upstream reading: `repo-ref/material-ui` and `repo-ref/compose-multiplatform-core` (when present locally)

## Page tracker

Legend:

- **Snippet-backed**: `No | Partial | Yes`
- **Target**: where we want the snippet + page to live once migrated.

| Page id | Preview fn | Current impl | Snippet-backed | Target page | Target snippet | Notes |
|---|---|---|---:|---|---|---|
| `material3_gallery` | `preview_material3_gallery` | `apps/fret-ui-gallery/src/ui/previews/material3/gallery/gallery_page.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_gallery.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/gallery.rs` | Likely becomes an index page linking to the individual component pages; keep the Standard/Expressive toggle visible. |
| `material3_button` | `preview_material3_button` | `apps/fret-ui-gallery/src/ui/previews/material3/buttons.rs` | Partial | `apps/fret-ui-gallery/src/ui/pages/material3_button.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/button.rs` | First scaffolded example: code tab is file-backed, but the page still lives under `previews/material3/**` for now. |
| `material3_icon_button` | `preview_material3_icon_button` | `apps/fret-ui-gallery/src/ui/previews/material3/controls.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_icon_button.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/icon_button.rs` | Ensure touch targets and disabled states are visible. |
| `material3_checkbox` | `preview_material3_checkbox` | `apps/fret-ui-gallery/src/ui/previews/material3/controls.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_checkbox.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/checkbox.rs` | Track tri-state + focus-visible when supported. |
| `material3_switch` | `preview_material3_switch` | `apps/fret-ui-gallery/src/ui/previews/material3/controls.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_switch.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/switch.rs` | Track thumb/track motion preset integration. |
| `material3_slider` | `preview_material3_slider` | `apps/fret-ui-gallery/src/ui/previews/material3/controls.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_slider.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/slider.rs` | Ensure value label / tick marks (if present) match the intended spec. |
| `material3_radio` | `preview_material3_radio` | `apps/fret-ui-gallery/src/ui/previews/material3/controls.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_radio.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/radio.rs` | Roving focus + group semantics. |
| `material3_badge` | `preview_material3_badge` | `apps/fret-ui-gallery/src/ui/previews/material3/navigation.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_badge.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/badge.rs` | Validate placement with navigation bar/rail examples. |
| `material3_segmented_button` | `preview_material3_segmented_button` | `apps/fret-ui-gallery/src/ui/previews/material3/forms/segmented_button.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_segmented_button.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/segmented_button.rs` | Selection model + disabled items. |
| `material3_top_app_bar` | `preview_material3_top_app_bar` | `apps/fret-ui-gallery/src/ui/previews/material3/navigation.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_top_app_bar.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/top_app_bar.rs` | Track scroll/condense behavior if implemented. |
| `material3_bottom_sheet` | `preview_material3_bottom_sheet` | `apps/fret-ui-gallery/src/ui/previews/material3/forms/bottom_sheet.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_bottom_sheet.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/bottom_sheet.rs` | Overlay + drag affordances; requires explicit layout constraints. |
| `material3_date_picker` | `preview_material3_date_picker` | `apps/fret-ui-gallery/src/ui/previews/material3/forms/date_picker.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_date_picker.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/date_picker.rs` | Ensure locale/week-start semantics are explicit in snippet. |
| `material3_time_picker` | `preview_material3_time_picker` | `apps/fret-ui-gallery/src/ui/previews/material3/forms/time_picker.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_time_picker.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/time_picker.rs` | Clock dial may need explicit viewport sizing. |
| `material3_autocomplete` | `preview_material3_autocomplete` | `apps/fret-ui-gallery/src/ui/previews/material3/forms/autocomplete.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_autocomplete.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/autocomplete.rs` | Interaction policy lives in kit; keep page focused on outcomes. |
| `material3_select` | `preview_material3_select` | `apps/fret-ui-gallery/src/ui/previews/material3/forms/select.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_select.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/select.rs` | Overlay + list semantics; likely depends on focus/keyboard coverage. |
| `material3_text_field` | `preview_material3_text_field` | `apps/fret-ui-gallery/src/ui/previews/material3/forms/text_field.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_text_field.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/text_field.rs` | Error/supporting text layout needs explicit constraints. |
| `material3_tabs` | `preview_material3_tabs` | `apps/fret-ui-gallery/src/ui/previews/material3/nav.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_tabs.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/tabs.rs` | Track indicator motion + roving focus behavior. |
| `material3_navigation_bar` | `preview_material3_navigation_bar` | `apps/fret-ui-gallery/src/ui/previews/material3/nav.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_navigation_bar.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/navigation_bar.rs` | Active/inactive label + icon states. |
| `material3_navigation_rail` | `preview_material3_navigation_rail` | `apps/fret-ui-gallery/src/ui/previews/material3/nav.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_navigation_rail.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/navigation_rail.rs` | Extended rail + badges. |
| `material3_navigation_drawer` | `preview_material3_navigation_drawer` | `apps/fret-ui-gallery/src/ui/previews/material3/nav.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_navigation_drawer.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/navigation_drawer.rs` | Docked vs modal behaviors should be distinct. |
| `material3_modal_navigation_drawer` | `preview_material3_modal_navigation_drawer` | `apps/fret-ui-gallery/src/ui/previews/material3/nav.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_modal_navigation_drawer.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/modal_navigation_drawer.rs` | Overlay + scrim + focus restore. |
| `material3_dialog` | `preview_material3_dialog` | `apps/fret-ui-gallery/src/ui/previews/material3/overlays/dialog.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_dialog.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/dialog.rs` | Dismiss + focus trap + restore. |
| `material3_menu` | `preview_material3_menu` | `apps/fret-ui-gallery/src/ui/previews/material3/overlays/menu.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_menu.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/menu.rs` | Keyboard navigation + typeahead (when available). |
| `material3_list` | `preview_material3_list` | `apps/fret-ui-gallery/src/ui/previews/material3/overlays/list.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_list.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/list.rs` | Roving focus; list item density. |
| `material3_snackbar` | `preview_material3_snackbar` | `apps/fret-ui-gallery/src/ui/previews/material3/overlays/snackbar.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_snackbar.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/snackbar.rs` | Queue/stack policy is app-level; show a minimal example. |
| `material3_tooltip` | `preview_material3_tooltip` | `apps/fret-ui-gallery/src/ui/previews/material3/overlays/tooltip.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_tooltip.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/tooltip.rs` | Hover + keyboard focus + delay policy. |
| `material3_state_matrix` | `preview_material3_state_matrix` | `apps/fret-ui-gallery/src/ui/previews/material3/gallery/state_matrix.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_state_matrix.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/state_matrix.rs` | Useful for regression screenshots across tokens. |
| `material3_touch_targets` | `preview_material3_touch_targets` | `apps/fret-ui-gallery/src/ui/previews/material3/gallery/touch_targets.rs` | No | `apps/fret-ui-gallery/src/ui/pages/material3_touch_targets.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/touch_targets.rs` | Keep as a “mechanism pressure test” for min touch target contracts. |
