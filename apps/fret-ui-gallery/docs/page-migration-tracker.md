# UI Gallery page migration tracker

This document tracks the ongoing refactor that moves UI Gallery pages away from the legacy
right-pane `Preview/Usage/Notes` layout and towards self-contained pages built with
`doc_layout::render_doc_page` + `DocSection` (preview + optional code tabs + notes).

## Goals

- Each page owns its own layout and documentation (examples, code, notes).
- Remove duplicated outer chrome (tabs, copy actions, and page-level Usage/Notes panels).
- Keep the right pane structurally simple: `header + page content`.

## Status

- Outer `Preview/Usage/Notes` tabs: removed.
- Outer right-pane Usage/Notes panel: removed (pages must embed their own docs).

## Shadcn component pages (`apps/fret-ui-gallery/src/ui/pages/`)

All pages in this folder should render through `doc_layout::render_doc_page`.

As part of this migration, each example section should expose **both** `Preview` and `Code` tabs.
During the transition we may temporarily use `doc_layout::TODO_RUST_CODE` as a placeholder snippet;
this is intentionally searchable so we can drive the remaining cleanup.

| Page id | Module | Uses DocSection | Code tabs status |
| --- | --- | --- | --- |
| `alert` | `ui/pages/alert.rs` | Yes | Complete |
| `alert_dialog` | `ui/pages/alert_dialog.rs` | Yes | Complete |
| `aspect_ratio` | `ui/pages/aspect_ratio.rs` | Yes | Complete |
| `breadcrumb` | `ui/pages/breadcrumb.rs` | Yes | Complete |
| `carousel` | `ui/pages/carousel.rs` | Yes | Complete |
| `chart` | `ui/pages/chart.rs` | Yes | Complete |
| `checkbox` | `ui/pages/checkbox.rs` | Yes | Complete |
| `collapsible` | `ui/pages/collapsible.rs` | Yes | Complete |
| `combobox` | `ui/pages/combobox.rs` | Yes | Complete |
| `command` | `ui/pages/command.rs` | Yes | Complete |
| `context_menu` | `ui/pages/context_menu.rs` | Yes | Complete |
| `data_table` | `ui/pages/data_table.rs` | Yes | Complete |
| `date_picker` | `ui/pages/date_picker.rs` | Yes | Complete |
| `dialog` | `ui/pages/dialog.rs` | Yes | Complete |
| `drawer` | `ui/pages/drawer.rs` | Yes | Complete |
| `dropdown_menu` | `ui/pages/dropdown_menu.rs` | Yes | Complete |
| `empty` | `ui/pages/empty.rs` | Yes | Complete |
| `field` | `ui/pages/field.rs` | Yes | Complete |
| `form` | `ui/pages/form.rs` | Yes | Complete |
| `hover_card` | `ui/pages/hover_card.rs` | Yes | Complete |
| `input` | `ui/pages/input.rs` | Yes | Complete |
| `input_group` | `ui/pages/input_group.rs` | Yes | Complete |
| `input_otp` | `ui/pages/input_otp.rs` | Yes | Complete |
| `item` | `ui/pages/item.rs` | Yes | Complete |
| `kbd` | `ui/pages/kbd.rs` | Yes | Complete |
| `label` | `ui/pages/label.rs` | Yes | Complete |
| `menubar` | `ui/pages/menubar.rs` | Yes | Complete |
| `motion_presets` | `ui/pages/motion_presets.rs` | Yes | Complete |
| `native_select` | `ui/pages/native_select.rs` | Yes | Complete |
| `navigation_menu` | `ui/pages/navigation_menu.rs` | Yes | Complete |
| `shadcn_extras` | `ui/pages/shadcn_extras.rs` | Yes | Complete |
| `toggle` | `ui/pages/toggle.rs` | Yes | Complete |
| `toggle_group` | `ui/pages/toggle_group.rs` | Yes | Complete |
| `tooltip` | `ui/pages/tooltip.rs` | Yes | Complete |
| `typography` | `ui/pages/typography.rs` | Yes | Complete |

## Legacy docs system (to delete)

Once every remaining non-component page embeds its own docs (or no longer needs them), we can
remove the old docs plumbing entirely:

- `apps/fret-ui-gallery/src/docs/` (hardcoded markdown strings)
- `PageSpec.docs_md` / `PageSpec.usage_md`
- Any remaining commands/UI related to page-level Usage/Notes panels

## Core harness pages (`apps/fret-ui-gallery/src/ui/previews/pages/harness/`)

These pages are not shadcn components, but should still be self-contained and use `DocSection`
instead of relying on an outer docs panel.

| Page id | Module | Uses DocSection |
| --- | --- | --- |
| `intro` | `ui/previews/pages/harness/intro.rs` | Yes |
| `layout` | `ui/previews/pages/harness/layout.rs` | Yes |
| `view_cache` | `ui/previews/pages/harness/view_cache.rs` | Yes |
| `hit_test_only_paint_cache_probe` | `ui/previews/pages/harness/hit_test_only_paint_cache_probe.rs` | Yes |
| `ui_kit_list_torture` | `ui/previews/pages/harness/ui_kit_list_torture.rs` | Yes |
| `virtual_list_torture` | `ui/previews/pages/harness/virtual_list_torture.rs` | Yes |

## Torture pages (`apps/fret-ui-gallery/src/ui/previews/pages/torture/`)

| Page id | Module | Uses DocSection |
| --- | --- | --- |
| `chart_torture` | `ui/previews/pages/torture/chart_torture.rs` | Yes |
| `canvas_cull_torture` | `ui/previews/pages/torture/canvas_cull_torture.rs` | Yes |
| `node_graph_cull_torture` | `ui/previews/pages/torture/node_graph_cull_torture.rs` | Yes |
| `chrome_torture` | `ui/previews/pages/torture/chrome_torture.rs` | Yes |
| `windowed_rows_surface_torture` | `ui/previews/pages/torture/windowed_rows_surface_torture.rs` | Yes |
| `windowed_rows_surface_interactive_torture` | `ui/previews/pages/torture/windowed_rows_surface_interactive_torture.rs` | Yes |

## Editor/harness pages (`apps/fret-ui-gallery/src/ui/previews/pages/editors/`)

These pages are still "preview functions", but they should render a self-contained doc-style page.

| Page id | Module | Uses DocSection |
| --- | --- | --- |
| `code_view_torture` | `ui/previews/pages/editors/code_view.rs` | Yes |
| `code_editor_mvp` | `ui/previews/pages/editors/code_editor/mvp.rs` | Yes |
| `code_editor_torture` | `ui/previews/pages/editors/code_editor/torture.rs` | Yes |
| `markdown_editor_source` | `ui/previews/pages/editors/markdown.rs` | Yes |
| `web_ime_harness` | `ui/previews/pages/editors/web_ime.rs` | Yes |
| `text_selection_perf` | `ui/previews/pages/editors/text/selection_perf.rs` | Yes |
| `text_bidi_rtl_conformance` | `ui/previews/pages/editors/text/bidi_rtl_conformance.rs` | Yes |
| `text_mixed_script_fallback` | `ui/previews/pages/editors/text/mixed_script_fallback.rs` | Yes |
| `text_measure_overlay` | `ui/previews/pages/editors/text/measure_overlay.rs` | Yes |
| `text_feature_toggles` | `ui/previews/pages/editors/text/feature_toggles.rs` | Yes |
