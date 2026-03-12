# Material 3 UI Gallery Tracker (Snippet-backed Migration + Legacy Preview Retirement)

This tracker scopes the **Material 3** portion of UI Gallery.

Status (2026-03-11):

- shadcn + AI Elements pages are snippet-backed (Preview ≡ Code).
- Material 3 pages are snippet-backed (Preview ≡ Code) and routed through
  `apps/fret-ui-gallery/src/ui/pages/material3/mod.rs`.
- `apps/fret-ui-gallery/src/ui/content.rs` now calls `pages::material3::*` only; the routed page
  surface no longer depends on `src/ui/previews/material3/**`.
- Material 3 pages share a single `render_material3_demo_page(...)` scaffold helper in
  `apps/fret-ui-gallery/src/ui/pages/material3/shared.rs` (reduces per-page boilerplate).
- `apps/fret-ui-gallery/src/ui/pages/material3/mod.rs` is split into smaller per-area modules
  (`controls.rs`, `inputs.rs`, `navigation.rs`, `overlays.rs`, `gallery.rs`, `shared.rs`) so future
  migrations stay low-conflict.
- The legacy implementation under `apps/fret-ui-gallery/src/ui/previews/material3.rs` and
  `apps/fret-ui-gallery/src/ui/previews/material3/**` has been deleted after the retirement audit.
- The retirement is locked by the default-app authoring gate in
  `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`.
- Legacy preview-only helpers `preview_material3_fab`, `preview_material3_card`,
  `preview_material3_chip`, and `preview_material3_search_view` are no longer routed as standalone
  pages. Their useful coverage now lives inside snippet-backed composite surfaces, primarily
  `apps/fret-ui-gallery/src/ui/snippets/material3/state_matrix.rs`.

Goal:

- Keep Material 3 pages on the same snippet-backed contract:
  - Preview renders compiled Rust snippet code.
  - Code tab displays `include_str!` of that same snippet file (drift-free).
- Next cleanup:
  - Keep new Material 3 authoring on `src/ui/pages/material3/**` + `src/ui/snippets/material3/**`
    only; do not reintroduce a parallel preview layer.

References (reading aids):

- In-tree implementation: `ecosystem/fret-ui-material3`
- Upstream reading: `repo-ref/material-ui` and `repo-ref/compose-multiplatform-core` (when present locally)

## Retirement audit (2026-03-11)

### Routed surface

- `apps/fret-ui-gallery/src/ui/content.rs` references `pages::material3::*` exclusively.
- `apps/fret-ui-gallery/src/ui/pages/material3/{controls,gallery,inputs,navigation,overlays}.rs`
  render snippet-backed demos from `apps/fret-ui-gallery/src/ui/snippets/material3/*`.
- `apps/fret-ui-gallery/src/ui/pages/material3/shared.rs` owns the common page scaffold and the
  Standard/Expressive variant toggle.

### Legacy preview surface

- Before deletion, `apps/fret-ui-gallery/src/ui/previews/material3.rs` was not declared from
  `apps/fret-ui-gallery/src/ui/previews/mod.rs`, so the entire legacy tree was unreachable.
- The retired footprint was `26` orphan files (`1` root file + `25` nested files).
- That orphan tree has now been removed from the repository.

### Retirement matrix

| Bucket | Status | Evidence | Action |
|---|---|---|---|
| `src/ui/previews/material3.rs` | Deleted | Previously orphaned / not compiled | Keep deleted. |
| `src/ui/previews/material3/**` (25 files) | Deleted | Previously orphaned / not compiled | Keep deleted. |
| `preview_material3_fab/card/chip/search_view` | Folded into snippet-backed composite demos | See `src/ui/snippets/material3/state_matrix.rs` | Do not recreate as page-level APIs unless they become routed pages again. |
| `src/ui/pages/material3/**` | Live routed surface | `src/ui/content.rs` routes every Material 3 page here | Keep; continue authoring on this surface only. |
| `src/ui/snippets/material3/**` | Live source-of-truth examples | Pages render snippets + show the same source | Keep; this is the canonical teaching surface. |

## Page tracker

Legend:

- **Snippet-backed**: `No | Partial | Yes`
- **Target**: where we want the snippet + page to live once migrated.

| Page id | Preview fn | Current impl | Snippet-backed | Target page | Target snippet | Notes |
|---|---|---|---:|---|---|---|
| `material3_gallery` | `preview_material3_gallery` | `apps/fret-ui-gallery/src/ui/pages/material3/gallery.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_gallery.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/gallery.rs` | Likely becomes an index page linking to the individual component pages; keep the Standard/Expressive toggle visible. |
| `material3_button` | `preview_material3_button` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_button.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/button.rs` | Live routed page already depends only on snippet + shared scaffold. |
| `material3_icon_button` | `preview_material3_icon_button` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_icon_button.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/icon_button.rs` | Keep `ui-gallery-material3-icon-button-centered` stable for existing diag scripts. |
| `material3_checkbox` | `preview_material3_checkbox` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_checkbox.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/checkbox.rs` | Keep `ui-gallery-material3-checkbox-tristate` stable for existing diag scripts. |
| `material3_switch` | `preview_material3_switch` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_switch.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/switch.rs` | Keep `ui-gallery-material3-switch-*` test ids stable for existing diag scripts. |
| `material3_slider` | `preview_material3_slider` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_slider.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/slider.rs` | Keep `ui-gallery-material3-slider-*` test ids stable for existing diag scripts. |
| `material3_radio` | `preview_material3_radio` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_radio.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/radio.rs` | Keep `ui-gallery-material3-radio-*` test ids stable for existing diag scripts. |
| `material3_badge` | `preview_material3_badge` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_badge.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/badge.rs` | Validate placement with navigation bar/rail examples. |
| `material3_segmented_button` | `preview_material3_segmented_button` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_segmented_button.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/segmented_button.rs` | Selection model + disabled items. |
| `material3_top_app_bar` | `preview_material3_top_app_bar` | `apps/fret-ui-gallery/src/ui/pages/material3/navigation.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_top_app_bar.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/top_app_bar.rs` | Track scroll/condense behavior if implemented. |
| `material3_bottom_sheet` | `preview_material3_bottom_sheet` | `apps/fret-ui-gallery/src/ui/pages/material3/overlays.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_bottom_sheet.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/bottom_sheet.rs` | Overlay + drag affordances; requires explicit layout constraints. |
| `material3_date_picker` | `preview_material3_date_picker` | `apps/fret-ui-gallery/src/ui/pages/material3/inputs.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_date_picker.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/date_picker.rs` | Ensure locale/week-start semantics are explicit in snippet. |
| `material3_time_picker` | `preview_material3_time_picker` | `apps/fret-ui-gallery/src/ui/pages/material3/inputs.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_time_picker.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/time_picker.rs` | Clock dial may need explicit viewport sizing. |
| `material3_autocomplete` | `preview_material3_autocomplete` | `apps/fret-ui-gallery/src/ui/pages/material3/inputs.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_autocomplete.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/autocomplete.rs` | Interaction policy lives in kit; keep page focused on outcomes. |
| `material3_select` | `preview_material3_select` | `apps/fret-ui-gallery/src/ui/pages/material3/inputs.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_select.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/select.rs` | Overlay + list semantics; likely depends on focus/keyboard coverage. |
| `material3_text_field` | `preview_material3_text_field` | `apps/fret-ui-gallery/src/ui/pages/material3/inputs.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_text_field.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/text_field.rs` | Error/supporting text layout needs explicit constraints. |
| `material3_tabs` | `preview_material3_tabs` | `apps/fret-ui-gallery/src/ui/pages/material3/navigation.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_tabs.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/tabs.rs` | Track indicator motion + roving focus behavior. |
| `material3_navigation_bar` | `preview_material3_navigation_bar` | `apps/fret-ui-gallery/src/ui/pages/material3/navigation.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_navigation_bar.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/navigation_bar.rs` | Active/inactive label + icon states. |
| `material3_navigation_rail` | `preview_material3_navigation_rail` | `apps/fret-ui-gallery/src/ui/pages/material3/navigation.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_navigation_rail.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/navigation_rail.rs` | Extended rail + badges. |
| `material3_navigation_drawer` | `preview_material3_navigation_drawer` | `apps/fret-ui-gallery/src/ui/pages/material3/navigation.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_navigation_drawer.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/navigation_drawer.rs` | Docked vs modal behaviors should be distinct. |
| `material3_modal_navigation_drawer` | `preview_material3_modal_navigation_drawer` | `apps/fret-ui-gallery/src/ui/pages/material3/navigation.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_modal_navigation_drawer.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/modal_navigation_drawer.rs` | Overlay + scrim + focus restore. |
| `material3_dialog` | `preview_material3_dialog` | `apps/fret-ui-gallery/src/ui/pages/material3/overlays.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_dialog.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/dialog.rs` | Dismiss + focus trap + restore. |
| `material3_menu` | `preview_material3_menu` | `apps/fret-ui-gallery/src/ui/pages/material3/overlays.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_menu.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/menu.rs` | Keyboard navigation + typeahead (when available). |
| `material3_list` | `preview_material3_list` | `apps/fret-ui-gallery/src/ui/pages/material3/navigation.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_list.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/list.rs` | Roving focus; list item density. |
| `material3_snackbar` | `preview_material3_snackbar` | `apps/fret-ui-gallery/src/ui/pages/material3/overlays.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_snackbar.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/snackbar.rs` | Queue/stack policy is app-level; show a minimal example. |
| `material3_tooltip` | `preview_material3_tooltip` | `apps/fret-ui-gallery/src/ui/pages/material3/overlays.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_tooltip.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/tooltip.rs` | Hover + keyboard focus + delay policy. |
| `material3_state_matrix` | `preview_material3_state_matrix` | `apps/fret-ui-gallery/src/ui/pages/material3/gallery.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_state_matrix.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/state_matrix.rs` | Useful for regression screenshots across tokens. Keep `ui-gallery-material3-search-view*` test ids stable for diag scripts. |
| `material3_touch_targets` | `preview_material3_touch_targets` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_touch_targets.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/touch_targets.rs` | Keep as a “mechanism pressure test” for min touch target contracts. |
