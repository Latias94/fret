# Material 3 UI Gallery Tracker (Snippet-backed Migration + Legacy Preview Retirement)

This tracker scopes the **Material 3** portion of UI Gallery.

Status (2026-03-14):

- shadcn + AI Elements pages are snippet-backed (Preview ≡ Code).
- Material 3 pages are snippet-backed (Preview ≡ Code) and routed through
  `apps/fret-ui-gallery/src/ui/pages/material3/mod.rs`.
- `apps/fret-ui-gallery/src/ui/content.rs` now calls `pages::material3::*` only; the routed page
  surface no longer depends on `src/ui/previews/material3/**`.
- Material 3 pages share a single `render_material3_demo_page(...)` scaffold helper in
  `apps/fret-ui-gallery/src/ui/pages/material3/shared.rs` (reduces per-page boilerplate).
- Material 3 overlay snippets now keep `new(open)` as the explicit externally owned seam, while
  the default copyable path uses `*::uncontrolled(cx)` plus `open_model()` so docs/examples do not
  need snippet-local `local_model_keyed("open", ...)` state for the default case.
- `SearchView` now follows the same split for its open/query pair: keep `new(open, query)` as the
  explicit controlled seam, and use `SearchView::uncontrolled(cx)` for the copyable state-matrix
  demo path.
- `Autocomplete` now follows the same query-state split: keep `new(query)` as the explicit
  controlled seam, and use `Autocomplete::uncontrolled(cx)` plus `query_model()` for the copyable
  page/demo path. The dialog probe on that page also uses `Dialog::uncontrolled(cx)`.
- `ExposedDropdown` now follows the same committed-selection/query split: keep
  `new(selected_value).query(query)` as the explicit controlled seams, and use
  `new_controllable(...)` / `uncontrolled(cx)` plus accessors on the copyable demo path.
- `Select` now follows the same committed-value split: keep `new(selected_value)` as the explicit
  controlled seam, and use `new_controllable(...)` / `uncontrolled(cx)` plus `value_model()` on
  the copyable demo path. The first-party snippet no longer hand-rolls snippet-local selection
  models for its default demo/probes.
- `Tabs`, `NavigationBar`, `NavigationRail`, `NavigationDrawer`, and `List` now follow the same
  committed-value split: keep `new(model)` as the explicit controlled seam, and use
  `new_controllable(...)` / `uncontrolled(cx, default)` plus `value_model()` on the copyable demo
  path.
- `TextField` now follows the same value split: keep `new(model)` as the explicit controlled seam,
  and use `new_controllable(...)` / `uncontrolled(cx)` plus `value_model()` on the copyable demo
  path.
- `Checkbox`, `Switch`, `RadioGroup`, and standalone `Radio` now follow the same choice-state
  split: keep controlled `new(...)` / `new_optional(...)` as the explicit seams, and use
  `new_controllable(...)` / `uncontrolled(...)` plus `checked_model()` /
  `optional_checked_model()` / `selected_model()` / `value_model()` on the copyable demo path.
- UI Gallery no longer routes demo-only runtime/page models for Material 3 tabs, list,
  navigation-bar/rail/drawer, or text-field value state; those defaults now live beside the
  snippets that users copy.
- UI Gallery no longer routes demo-only runtime/page models for Material 3 checkbox, switch, or
  radio state. Those defaults now live beside the copyable snippets, and the composite
  `gallery`/`state_matrix`/`touch_targets` surfaces construct their own local uncontrolled roots.
- `ModalNavigationDrawer` now follows the same open-state split: keep `new(open)` as the explicit
  controlled seam, and use `new_controllable(...)` / `uncontrolled(cx)` plus `open_model()` on the
  copyable demo path.
- `DatePickerDialog` now follows the same root-state split: keep `new(open, month, selected)` as
  the explicit controlled seam, and use `new_controllable(...)` / `uncontrolled(cx)` plus
  `open_model()` / `month_model()` / `selected_model()` on the copyable demo path.
- `TimePickerDialog` now follows the same root-state split: keep `new(open, selected)` as the
  explicit controlled seam, and use `new_controllable(...)` / `uncontrolled(cx)` plus
  `open_model()` / `selected_model()` on the copyable demo path.
- That Material 3 authoring split is now guarded by source gates in
  `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` and
  `ecosystem/fret-ui-material3/src/lib.rs`.
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
| `material3_gallery` | `preview_material3_gallery` | `apps/fret-ui-gallery/src/ui/pages/material3/gallery.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_gallery.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/gallery.rs` | Likely becomes an index page linking to the individual component pages; keep the Standard/Expressive toggle visible. Composite choice controls now use snippet-local uncontrolled roots instead of page-owned checkbox/switch/radio models. |
| `material3_button` | `preview_material3_button` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_button.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/button.rs` | Live routed page already depends only on snippet + shared scaffold. |
| `material3_icon_button` | `preview_material3_icon_button` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_icon_button.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/icon_button.rs` | Keep `ui-gallery-material3-icon-button-centered` stable for existing diag scripts. |
| `material3_checkbox` | `preview_material3_checkbox` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_checkbox.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/checkbox.rs` | Keep `ui-gallery-material3-checkbox-tristate` stable for existing diag scripts. Default snippet now uses `Checkbox::uncontrolled(cx, false)` / `Checkbox::uncontrolled_optional(cx, None)` plus `checked_model()` / `optional_checked_model()`, while `new(checked)` / `new_optional(checked)` stay as the explicit controlled seams. |
| `material3_switch` | `preview_material3_switch` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_switch.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/switch.rs` | Keep `ui-gallery-material3-switch-*` test ids stable for existing diag scripts. Default snippet now uses `Switch::uncontrolled(cx, false)` plus `selected_model()`, while `new(selected)` stays as the explicit controlled seam. |
| `material3_slider` | `preview_material3_slider` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_slider.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/slider.rs` | Keep `ui-gallery-material3-slider-*` test ids stable for existing diag scripts. |
| `material3_radio` | `preview_material3_radio` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_radio.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/radio.rs` | Keep `ui-gallery-material3-radio-*` test ids stable for existing diag scripts. Default snippet now uses `RadioGroup::uncontrolled(cx, None)` + `value_model()` and `Radio::uncontrolled(cx, false)` + `selected_model()`, while `RadioGroup::new(model)` / `Radio::new(selected)` stay as the explicit controlled seams. |
| `material3_badge` | `preview_material3_badge` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_badge.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/badge.rs` | Validate placement with navigation bar/rail examples. |
| `material3_segmented_button` | `preview_material3_segmented_button` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_segmented_button.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/segmented_button.rs` | Selection model + disabled items. |
| `material3_top_app_bar` | `preview_material3_top_app_bar` | `apps/fret-ui-gallery/src/ui/pages/material3/navigation.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_top_app_bar.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/top_app_bar.rs` | Track scroll/condense behavior if implemented. |
| `material3_bottom_sheet` | `preview_material3_bottom_sheet` | `apps/fret-ui-gallery/src/ui/pages/material3/overlays.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_bottom_sheet.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/bottom_sheet.rs` | Overlay + drag affordances; default snippet now uses `ModalBottomSheet::uncontrolled(cx)` + `open_model()`, while `new(open)` remains the explicit app-owned seam. |
| `material3_date_picker` | `preview_material3_date_picker` | `apps/fret-ui-gallery/src/ui/pages/material3/inputs.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_date_picker.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/date_picker.rs` | Ensure locale/week-start semantics are explicit in snippet. Default snippet now uses `DatePickerDialog::uncontrolled(cx)` + `open_model()` / `month_model()` / `selected_model()`, while `new(open, month, selected)` stays as the explicit app-owned seam. |
| `material3_time_picker` | `preview_material3_time_picker` | `apps/fret-ui-gallery/src/ui/pages/material3/inputs.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_time_picker.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/time_picker.rs` | Clock dial may need explicit viewport sizing. Default snippet now uses `TimePickerDialog::uncontrolled(cx)` + `open_model()` / `selected_model()`, while `new(open, selected)` stays as the explicit app-owned seam. |
| `material3_autocomplete` | `preview_material3_autocomplete` | `apps/fret-ui-gallery/src/ui/pages/material3/inputs.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_autocomplete.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/autocomplete.rs` | Interaction policy lives in kit; keep page focused on outcomes. |
| `material3_select` | `preview_material3_select` | `apps/fret-ui-gallery/src/ui/pages/material3/inputs.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_select.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/select.rs` | Overlay + list semantics; default snippet now uses `Select::uncontrolled(cx)` + `value_model()`, while `new(selected_value)` remains the explicit app-owned seam. |
| `material3_text_field` | `preview_material3_text_field` | `apps/fret-ui-gallery/src/ui/pages/material3/inputs.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_text_field.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/text_field.rs` | Error/supporting text layout needs explicit constraints. Default snippet now uses `TextField::uncontrolled(cx)` + `value_model()`, while `new(model)` stays as the explicit app-owned seam. |
| `material3_tabs` | `preview_material3_tabs` | `apps/fret-ui-gallery/src/ui/pages/material3/navigation.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_tabs.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/tabs.rs` | Track indicator motion + roving focus behavior. Default snippet now uses `Tabs::uncontrolled(cx, "overview")` + `value_model()`, while `new(model)` stays as the explicit app-owned seam. |
| `material3_navigation_bar` | `preview_material3_navigation_bar` | `apps/fret-ui-gallery/src/ui/pages/material3/navigation.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_navigation_bar.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/navigation_bar.rs` | Active/inactive label + icon states. Default snippet now uses `NavigationBar::uncontrolled(cx, "search")` + `value_model()`, while `new(model)` stays as the explicit app-owned seam. |
| `material3_navigation_rail` | `preview_material3_navigation_rail` | `apps/fret-ui-gallery/src/ui/pages/material3/navigation.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_navigation_rail.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/navigation_rail.rs` | Extended rail + badges. Default snippet now uses `NavigationRail::uncontrolled(cx, "search")` + `value_model()`, while `new(model)` stays as the explicit app-owned seam. |
| `material3_navigation_drawer` | `preview_material3_navigation_drawer` | `apps/fret-ui-gallery/src/ui/pages/material3/navigation.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_navigation_drawer.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/navigation_drawer.rs` | Docked vs modal behaviors should be distinct. Default snippet now uses `NavigationDrawer::uncontrolled(cx, "search")` + `value_model()`, while `new(model)` stays as the explicit app-owned seam. |
| `material3_modal_navigation_drawer` | `preview_material3_modal_navigation_drawer` | `apps/fret-ui-gallery/src/ui/pages/material3/navigation.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_modal_navigation_drawer.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/modal_navigation_drawer.rs` | Overlay + scrim + focus restore. Default snippet now uses `ModalNavigationDrawer::uncontrolled(cx)` + `open_model()`; `new(open)` stays as the explicit externally owned seam. |
| `material3_dialog` | `preview_material3_dialog` | `apps/fret-ui-gallery/src/ui/pages/material3/overlays.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_dialog.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/dialog.rs` | Dismiss + focus trap + restore. Default snippet now uses `Dialog::uncontrolled(cx)` + `open_model()`; `new(open)` stays as the explicit externally owned seam. |
| `material3_menu` | `preview_material3_menu` | `apps/fret-ui-gallery/src/ui/pages/material3/overlays.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_menu.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/menu.rs` | Keyboard navigation + typeahead (when available). Default snippet now uses `DropdownMenu::uncontrolled(cx)` + `open_model()`; `new(open)` stays available for explicit controlled ownership. |
| `material3_list` | `preview_material3_list` | `apps/fret-ui-gallery/src/ui/pages/material3/navigation.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_list.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/list.rs` | Roving focus; list item density. Default snippet now uses `List::uncontrolled(cx, "alpha")` + `value_model()`, while `new(model)` stays as the explicit app-owned seam. |
| `material3_snackbar` | `preview_material3_snackbar` | `apps/fret-ui-gallery/src/ui/pages/material3/overlays.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_snackbar.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/snackbar.rs` | Queue/stack policy is app-level; show a minimal example. |
| `material3_tooltip` | `preview_material3_tooltip` | `apps/fret-ui-gallery/src/ui/pages/material3/overlays.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_tooltip.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/tooltip.rs` | Hover + keyboard focus + delay policy. |
| `material3_state_matrix` | `preview_material3_state_matrix` | `apps/fret-ui-gallery/src/ui/pages/material3/gallery.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_state_matrix.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/state_matrix.rs` | Useful for regression screenshots across tokens. Keep `ui-gallery-material3-search-view*` test ids stable for diag scripts. Checkbox/switch/radio probes now use snippet-local uncontrolled roots instead of page-owned runtime models. |
| `material3_touch_targets` | `preview_material3_touch_targets` | `apps/fret-ui-gallery/src/ui/pages/material3/controls.rs` | Yes | `apps/fret-ui-gallery/src/ui/pages/material3_touch_targets.rs` | `apps/fret-ui-gallery/src/ui/snippets/material3/touch_targets.rs` | Keep as a “mechanism pressure test” for min touch target contracts. Checkbox/switch/radio/tabs now use local uncontrolled roots instead of page-owned selection plumbing. |
