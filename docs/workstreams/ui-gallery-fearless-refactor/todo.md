# UI Gallery Fearless Refactor (TODO + Tracker)

This file is the active checklist and component tracker for the UI Gallery refactor.

Legend:

- **Snippet-backed**: preview renders compiled snippet code; code tab displays `include_str!` from the same file.
- **Drift-free**: code cannot diverge from preview by construction.

## TODO (phased)

## Status

As of 2026-03-02, UI Gallery component pages under `apps/fret-ui-gallery/src/ui/pages/**` are
**snippet-backed** (Preview ≡ Code) and the core enforcement tests are in place.

Any remaining legacy surfaces that embed raw Rust code strings are tracked via the drift audit
(`apps/fret-ui-gallery/build.rs`) and should be migrated as follow-up work.

### Foundations

- [x] Add `apps/fret-ui-gallery/src/ui/snippets/` with a minimal example.
- [x] Add a helper for `DocSection` to load code from a snippet file (and optionally a named region).
- [x] Document snippet conventions (file naming, user-facing `use` stanza, required function signatures).

### Migration batches

- [x] Migrate Button Group page(s) to snippet-backed sections.
- [x] Migrate Select page(s) to snippet-backed sections.
- [x] Migrate Input Group page(s) to snippet-backed sections.
- [x] Migrate overlay family pages (Alert Dialog, Dropdown Menu, Menubar, Context Menu, Popover, Tooltip, Dialog, Sheet, Drawer).
- [x] Migrate form family pages (Input, Input Group, Textarea, Checkbox, Radio Group, Switch, Slider, Toggle, Toggle Group, Select, Combobox).
- [x] Migrate layout + data-display pages (Tabs, Table, Data Table, Chart, Calendar, Carousel, Scroll Area, Navigation Menu, Pagination, etc).

### Enforcement

- [x] Add a lint/test that forbids new `DocSection::code("rust", r#"...")` on migrated pages (`apps/fret-ui-gallery/tests/ui_pages_deny_rust_code_literals.rs`).
- [x] Forbid new `.code("rust", ...)` literals under `src/ui/previews/pages/**` (`apps/fret-ui-gallery/tests/ui_previews_pages_deny_rust_code_literals.rs`).
- [x] Forbid `include_str!("../snippets/...")` usage in pages; require `snippets::*::SOURCE` to avoid refactor path drift (`apps/fret-ui-gallery/tests/ui_pages_deny_relative_snippet_includes.rs`).
- [x] Forbid snippet files from importing UI Gallery internals (`crate::ui`, `crate::spec`) (`apps/fret-ui-gallery/tests/ui_snippets_deny_gallery_internal_imports.rs`).
- [x] Require every snippet file to export a `SOURCE` const (`apps/fret-ui-gallery/tests/ui_snippets_require_source_const.rs`).
- [x] Add a small “drift audit” doc section in UI Gallery (optional) to list remaining legacy sections (`apps/fret-ui-gallery/build.rs`, `apps/fret-ui-gallery/src/ui/previews/pages/harness/intro.rs`).

### Next (post-migration)

- [x] Expand drift audit coverage to include non-`src/ui/pages/**` preview surfaces (`src/ui/previews/**`).
- [x] Migrate Calendar page(s) out of `src/ui/previews/**` so copyable code stays drift-free.
- [x] Migrate AI Elements gallery demos to snippet-backed pages (see `docs/workstreams/ui-gallery-fearless-refactor/ai-elements-tracker.md`).
- [x] Ensure Code tabs are vertically scrollable (wheel scrolling over CodeBlock gutters for windowed snippets).
- [ ] Normalize DocSection chrome/layout (max widths, padding, “Notes” shell usage) across pages.
  - [x] Remove redundant centering wrappers so Preview/Code tabs share consistent left padding.
  - [ ] Audit remaining max-width and padding inconsistencies across pages.
- [ ] Optional: align page taxonomy + section ordering to upstream shadcn MDX navigation.

Notes:

- Use `tools/check_ui_gallery_code_literals.py --deny --only <page.rs>` to enforce “no multi-line Rust literals”
  on snippet-backed/migrated pages.
- Material 3 pages are tracked separately because they still live under `src/ui/previews/material3/**` (even though they are now snippet-backed):
  `docs/workstreams/ui-gallery-fearless-refactor/material3-tracker.md`.

## Shadcn component tracker (gallery refactor status)

Columns:

- **Component**: upstream shadcn component name (kebab-case).
- **Base MDX / Radix MDX**: upstream doc paths (explicitly tracked to avoid “which variant?” ambiguity).
- **Fret module**: `kebab-case` → `snake_case` module name in `ecosystem/fret-ui-shadcn`.
- **Gallery Page**: where the component is showcased (initially “TBD” until we normalize page taxonomy).
- **Snippet-backed**: `No | Partial | Yes` (preview + code tab share a single compiled snippet file).
- **Status**: `Not started | In progress | Done`
- **Gates**: optional regression hooks (`test_id`, `fretboard diag` scripts, conformance fixtures).
- **Notes**: drift risks, known gaps, migration hints, or doc-variant notes.

Source list: upstream shadcn v4 Base/Radix doc trees:

- Base: `repo-ref/ui/apps/v4/content/docs/components/base/*.mdx`
- Radix: `repo-ref/ui/apps/v4/content/docs/components/radix/*.mdx`

| Component | Base MDX | Radix MDX | Fret module | Gallery Page | Snippet-backed | Status | Gates | Notes |
|---|---|---|---|---:|---:|---:|---|---|
| accordion | `repo-ref/ui/apps/v4/content/docs/components/base/accordion.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/accordion.mdx` | `accordion` | `apps/fret-ui-gallery/src/ui/pages/accordion.rs` | Yes | Done |  | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-accordion-demo-*` test_ids stable for future diag scripts. |
| alert | `repo-ref/ui/apps/v4/content/docs/components/base/alert.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/alert.mdx` | `alert` | `apps/fret-ui-gallery/src/ui/pages/alert.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/alert/ui-gallery-alert-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-alert-tabs-trigger-*` stable for diag scripts. |
| alert-dialog | `repo-ref/ui/apps/v4/content/docs/components/base/alert-dialog.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/alert-dialog.mdx` | `alert_dialog` | `apps/fret-ui-gallery/src/ui/pages/alert_dialog.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-*`, `tools/diag-scripts/ui-gallery/alert-dialog/ui-gallery-alert-dialog-part-surface-smoke.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep Basic/RTL doc section IDs stable for diag scripts. |
| aspect-ratio | `repo-ref/ui/apps/v4/content/docs/components/base/aspect-ratio.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/aspect-ratio.mdx` | `aspect_ratio` | `apps/fret-ui-gallery/src/ui/pages/aspect_ratio.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/aspect-ratio/ui-gallery-aspect-ratio-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep Demo section `docsec-demo-*` IDs stable for diag scripts. |
| avatar | `repo-ref/ui/apps/v4/content/docs/components/base/avatar.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/avatar.mdx` | `avatar` | `apps/fret-ui-gallery/src/ui/pages/avatar.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep dropdown and badge/group-count `test_id`s stable for screenshot + focus-restore gates. |
| badge | `repo-ref/ui/apps/v4/content/docs/components/base/badge.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/badge.mdx` | `badge` | `apps/fret-ui-gallery/src/ui/pages/badge.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/badge/ui-gallery-badge-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-badge-*` `test_id`s stable for diag scripts. |
| breadcrumb | `repo-ref/ui/apps/v4/content/docs/components/base/breadcrumb.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/breadcrumb.mdx` | `breadcrumb` | `apps/fret-ui-gallery/src/ui/pages/breadcrumb.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-breadcrumb-*` section title test IDs stable for diag scripts. |
| button | `repo-ref/ui/apps/v4/content/docs/components/base/button.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/button.mdx` | `button` | `apps/fret-ui-gallery/src/ui/pages/button.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/button/ui-gallery-button-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-button-variant-*` and `ui-gallery-button-render-link` stable for layout/semantics gates. |
| button-group | `repo-ref/ui/apps/v4/content/docs/components/base/button-group.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/button-group.mdx` | `button_group` | `apps/fret-ui-gallery/src/ui/pages/button_group.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/button/ui-gallery-button-group-demo-screenshots.json`, `tools/diag-scripts/ui-gallery/button/ui-gallery-button-group-select-screenshots.json` | Snippet-backed previews + region-sliced code tabs for all Button Group sections (preview ≡ code). |
| calendar | `repo-ref/ui/apps/v4/content/docs/components/base/calendar.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/calendar.mdx` | `calendar` | `apps/fret-ui-gallery/src/ui/pages/calendar.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/calendar/ui-gallery-calendar-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery.calendar.*` test_id prefixes stable for diag scripts. |
| card | `repo-ref/ui/apps/v4/content/docs/components/base/card.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/card.mdx` | `card` | `apps/fret-ui-gallery/src/ui/pages/card.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/card/ui-gallery-card-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). |
| carousel | `repo-ref/ui/apps/v4/content/docs/components/base/carousel.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/carousel.mdx` | `carousel` | `apps/fret-ui-gallery/src/ui/pages/carousel.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-carousel-demo-*` + duration + API `test_id`s stable for diag scripts. |
| chart | `repo-ref/ui/apps/v4/content/docs/components/base/chart.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/chart.mdx` | `chart` | `apps/fret-ui-gallery/src/ui/pages/chart.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/chart/ui-gallery-chart-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-chart-*` `test_id`s stable for diag scripts. |
| checkbox | `repo-ref/ui/apps/v4/content/docs/components/base/checkbox.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/checkbox.mdx` | `checkbox` | `apps/fret-ui-gallery/src/ui/pages/checkbox.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `docsec-invalid-state-tabs-trigger-code` stable for code-tab scroll-range gate. |
| collapsible | `repo-ref/ui/apps/v4/content/docs/components/base/collapsible.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/collapsible.mdx` | `collapsible` | `apps/fret-ui-gallery/src/ui/pages/collapsible.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/collapsible/ui-gallery-collapsible-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-collapsible-*` `test_id`s stable for diag scripts. |
| combobox | `repo-ref/ui/apps/v4/content/docs/components/base/combobox.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/combobox.mdx` | `combobox` | `apps/fret-ui-gallery/src/ui/pages/combobox.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-combobox-*` `test_id`s stable for diag scripts. |
| command | `repo-ref/ui/apps/v4/content/docs/components/base/command.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/command.mdx` | `command` | `apps/fret-ui-gallery/src/ui/pages/command.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/command/ui-gallery-command-*.json`, `tools/diag-scripts/ui-gallery/command/a11y-ui-gallery-command-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-command-*` `test_id`s stable for diag scripts. |
| context-menu | `repo-ref/ui/apps/v4/content/docs/components/base/context-menu.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/context-menu.mdx` | `context_menu` | `apps/fret-ui-gallery/src/ui/pages/context_menu.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/context-menu/*` | Snippet-backed previews + region-sliced code tabs for all sections (preview ≡ code). Keep trigger/item `test_id`s stable for existing diag scripts. |
| data-table | `repo-ref/ui/apps/v4/content/docs/components/base/data-table.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/data-table.mdx` | `data_table` | `apps/fret-ui-gallery/src/ui/pages/data_table.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-data-table-*` `test_id`s stable for diag scripts. |
| date-picker | `repo-ref/ui/apps/v4/content/docs/components/base/date-picker.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/date-picker.mdx` | `date_picker` | `apps/fret-ui-gallery/src/ui/pages/date_picker.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/date-picker/ui-gallery-date-picker-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-date-picker-*` `test_id`s stable for existing diag scripts. |
| dialog | `repo-ref/ui/apps/v4/content/docs/components/base/dialog.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/dialog.mdx` | `dialog` | `apps/fret-ui-gallery/src/ui/pages/dialog.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/dialog/*`, `tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-*` | Snippet-backed previews + region-sliced code tabs for all sections (preview ≡ code). Keep parts/custom-close/no-close/sticky/scrollable/rtl `test_id`s stable for existing diag scripts. |
| direction | `repo-ref/ui/apps/v4/content/docs/components/base/direction.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/direction.mdx` | — | TBD | No | Not started |  | Doc-only (directionality guidance), not a component. |
| drawer | `repo-ref/ui/apps/v4/content/docs/components/base/drawer.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/drawer.mdx` | `drawer` | `apps/fret-ui-gallery/src/ui/pages/drawer.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/drawer/ui-gallery-drawer-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-drawer-*` `test_id`s stable for diag scripts. |
| dropdown-menu | `repo-ref/ui/apps/v4/content/docs/components/base/dropdown-menu.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/dropdown-menu.mdx` | `dropdown_menu` | `apps/fret-ui-gallery/src/ui/pages/dropdown_menu.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/dropdown-menu/*` | Snippet-backed previews + region-sliced code tabs for all sections (preview ≡ code). Keep trigger/item `test_id`s stable for existing diag scripts. |
| empty | `repo-ref/ui/apps/v4/content/docs/components/base/empty.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/empty.mdx` | `empty` | `apps/fret-ui-gallery/src/ui/pages/empty.rs` | Yes | Done |  | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-empty-*` `test_id`s stable. |
| field | `repo-ref/ui/apps/v4/content/docs/components/base/field.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/field.mdx` | `field` | `apps/fret-ui-gallery/src/ui/pages/field.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/field/ui-gallery-field-docs-smoke.json`, `tools/diag-scripts/ui-gallery/field/ui-gallery-field-radio-screenshot-zinc-dark.json`, `tools/diag-scripts/ui-gallery/field/ui-gallery-field-responsive-orientation-container-md.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep responsive width switch + name content/input `test_id`s stable for container-query orientation gates. |
| hover-card | `repo-ref/ui/apps/v4/content/docs/components/base/hover-card.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/hover-card.mdx` | `hover_card` | `apps/fret-ui-gallery/src/ui/pages/hover_card.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/hover-card/*` | Snippet-backed previews + region-sliced code tabs for all sections (preview ≡ code). Keep `ui-gallery-hover-card-*` `test_id`s stable for existing diag scripts. |
| input | `repo-ref/ui/apps/v4/content/docs/components/base/input.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/input.mdx` | `input` | `apps/fret-ui-gallery/src/ui/pages/input.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/text-ime/ui-gallery-input-ime-tab-suppressed.json`, `tools/diag-scripts/ui-gallery/input/ui-gallery-input-file-browse-mocked.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-input-basic` and file browse test IDs stable for IME + mocked file dialog gates. |
| input-group | `repo-ref/ui/apps/v4/content/docs/components/base/input-group.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/input-group.mdx` | `input_group` | `apps/fret-ui-gallery/src/ui/pages/input_group.rs` | Yes | Done |  | Snippet-backed previews + region-sliced code tabs (preview ≡ code). |
| input-otp | `repo-ref/ui/apps/v4/content/docs/components/base/input-otp.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/input-otp.mdx` | `input_otp` | `apps/fret-ui-gallery/src/ui/pages/input_otp.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/input/ui-gallery-input-otp-docs-smoke.json` | Snippet-backed demo + region-sliced code tab (preview ≡ code). Keep `ui-gallery-input-otp-*.slot.*` and `.input` test IDs stable for focus/selection gates. |
| item | `repo-ref/ui/apps/v4/content/docs/components/base/item.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/item.mdx` | `item` | `apps/fret-ui-gallery/src/ui/pages/item.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/item/ui-gallery-item-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-item-demo` + `ui-gallery-item-link-render` stable for diag scripts. |
| kbd | `repo-ref/ui/apps/v4/content/docs/components/base/kbd.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/kbd.mdx` | `kbd` | `apps/fret-ui-gallery/src/ui/pages/kbd.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/kbd/ui-gallery-kbd-docs-smoke.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-kbd-*` `test_id`s stable for diag scripts. |
| label | `repo-ref/ui/apps/v4/content/docs/components/base/label.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/label.mdx` | `label` | `apps/fret-ui-gallery/src/ui/pages/label.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/label/ui-gallery-label-docs-smoke.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-label-*` `test_id`s stable for diag scripts. |
| menubar | `repo-ref/ui/apps/v4/content/docs/components/base/menubar.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/menubar.mdx` | `menubar` | `apps/fret-ui-gallery/src/ui/pages/menubar.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/menubar/*` | Snippet-backed previews + region-sliced code tabs for all sections (preview ≡ code). Keep `ui-gallery-menubar-with-icons-*` and parts `test_id`s stable for existing diag scripts. |
| native-select | `repo-ref/ui/apps/v4/content/docs/components/base/native-select.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/native-select.mdx` | `native_select` | `apps/fret-ui-gallery/src/ui/pages/native_select.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/native-select/ui-gallery-native-select-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-native-select-*-native-trigger` test IDs stable for screenshot/hover gates. |
| navigation-menu | `repo-ref/ui/apps/v4/content/docs/components/base/navigation-menu.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/navigation-menu.mdx` | `navigation_menu` | `apps/fret-ui-gallery/src/ui/pages/navigation_menu.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/navigation/ui-gallery-navigation-menu-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-navigation-menu-*` `test_id`s stable for diag scripts. |
| pagination | `repo-ref/ui/apps/v4/content/docs/components/base/pagination.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/pagination.mdx` | `pagination` | `apps/fret-ui-gallery/src/ui/pages/pagination.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/pagination/ui-gallery-pagination-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-pagination-*` `test_id`s stable for diag scripts. |
| popover | `repo-ref/ui/apps/v4/content/docs/components/base/popover.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/popover.mdx` | `popover` | `apps/fret-ui-gallery/src/ui/pages/popover.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/overlay/ui-gallery-popover-doc-page-opens.json`, `tools/diag-scripts/ui-gallery/overlay/ui-gallery-popover-basic-open-screenshot-zinc-dark.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). |
| progress | `repo-ref/ui/apps/v4/content/docs/components/base/progress.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/progress.mdx` | `progress` | `apps/fret-ui-gallery/src/ui/pages/progress.rs` | Yes | Done |  | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-progress-*` test IDs stable for future diag gates. |
| radio-group | `repo-ref/ui/apps/v4/content/docs/components/base/radio-group.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/radio-group.mdx` | `radio_group` | `apps/fret-ui-gallery/src/ui/pages/radio_group.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/radio-group/ui-gallery-radio-group-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-radio-group-description` stable for layout gates. |
| resizable | `repo-ref/ui/apps/v4/content/docs/components/base/resizable.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/resizable.mdx` | `resizable` | `apps/fret-ui-gallery/src/ui/pages/resizable.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-resizable-*` `test_id`s stable for existing diag scripts. |
| scroll-area | `repo-ref/ui/apps/v4/content/docs/components/base/scroll-area.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/scroll-area.mdx` | `scroll_area` | `apps/fret-ui-gallery/src/ui/pages/scroll_area.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-wheel-scroll.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-scroll-area-*-viewport` `test_id`s stable for diag scripts. |
| select | `repo-ref/ui/apps/v4/content/docs/components/base/select.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/select.mdx` | `select` | `apps/fret-ui-gallery/src/ui/pages/select.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/select/ui-gallery-select-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-select-*` `test_id`s stable for diag scripts. |
| separator | `repo-ref/ui/apps/v4/content/docs/components/base/separator.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/separator.mdx` | `separator` | `apps/fret-ui-gallery/src/ui/pages/separator.rs` | Yes | Done |  | Snippet-backed previews + region-sliced code tabs (preview ≡ code). |
| sheet | `repo-ref/ui/apps/v4/content/docs/components/base/sheet.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/sheet.mdx` | `sheet` | `apps/fret-ui-gallery/src/ui/pages/sheet.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/overlay/ui-gallery-sheet-part-surface-smoke.json`, `tools/diag-scripts/ui-gallery/overlay/ui-gallery-sheet-side-top-bottom-screenshots.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). |
| sidebar | `repo-ref/ui/apps/v4/content/docs/components/base/sidebar.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/sidebar.mdx` | `sidebar` | `apps/fret-ui-gallery/src/ui/pages/sidebar.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/sidebar/*` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep DocSection `test_id_prefix` stable (tab trigger IDs) and preserve `ui-gallery-sidebar-*` `test_id`s for diag scripts. |
| skeleton | `repo-ref/ui/apps/v4/content/docs/components/base/skeleton.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/skeleton.mdx` | `skeleton` | `apps/fret-ui-gallery/src/ui/pages/skeleton.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/skeleton/ui-gallery-skeleton-demo-screenshot-zinc-dark.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-skeleton-demo` semantics test id stable for screenshot gates. |
| slider | `repo-ref/ui/apps/v4/content/docs/components/base/slider.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/slider.mdx` | `slider` | `apps/fret-ui-gallery/src/ui/pages/slider.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-slider-*` test IDs stable for drag/set-value gates. |
| sonner | `repo-ref/ui/apps/v4/content/docs/components/base/sonner.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/sonner.mdx` | `sonner` | `apps/fret-ui-gallery/src/ui/pages/sonner.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/sonner/ui-gallery-sonner-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-sonner-*` `test_id`s stable for existing diag scripts. |
| spinner | `repo-ref/ui/apps/v4/content/docs/components/base/spinner.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/spinner.mdx` | `spinner` | `apps/fret-ui-gallery/src/ui/pages/spinner.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/spinner/ui-gallery-spinner-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-spinner-extras-textarea-actions` stable for screenshot gates. |
| switch | `repo-ref/ui/apps/v4/content/docs/components/base/switch.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/switch.mdx` | `switch` | `apps/fret-ui-gallery/src/ui/pages/switch.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-switch-airplane` stable for screenshot gates. |
| table | `repo-ref/ui/apps/v4/content/docs/components/base/table.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/table.mdx` | `table` | `apps/fret-ui-gallery/src/ui/pages/table.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/table/*` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-table-*` test IDs stable for smoke + hover-row screenshot gates. |
| tabs | `repo-ref/ui/apps/v4/content/docs/components/base/tabs.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/tabs.mdx` | `tabs` | `apps/fret-ui-gallery/src/ui/pages/tabs.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/tabs/*`, `tools/diag-scripts/ui-gallery/shadcn-conformance/ui-gallery-shadcn-tabs-demo-screenshot.json`, `tools/diag-scripts/ui-gallery/control-chrome/ui-gallery-control-chrome-tabs-flex1-trigger-fill.json`, `tools/diag-scripts/ui-gallery/text-wrap/ui-gallery-tabs-wrap-and-baseline-screenshots.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-tabs-*` `test_id`s stable for diag + screenshot gates. |
| textarea | `repo-ref/ui/apps/v4/content/docs/components/base/textarea.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/textarea.mdx` | `textarea` | `apps/fret-ui-gallery/src/ui/pages/textarea.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/textarea/ui-gallery-textarea-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-textarea-demo` stable for caret gates. |
| toast | `repo-ref/ui/apps/v4/content/docs/components/base/toast.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/toast.mdx` | `toast` | `apps/fret-ui-gallery/src/ui/pages/toast.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/toast/*` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Upstream docs mark Toast deprecated; this page keeps only deprecation guidance and a shortcut to Sonner. |
| toggle | `repo-ref/ui/apps/v4/content/docs/components/base/toggle.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/toggle.mdx` | `toggle` | `apps/fret-ui-gallery/src/ui/pages/toggle.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/toggle/ui-gallery-toggle-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-toggle-demo` stable for docs-smoke and screenshots. |
| toggle-group | `repo-ref/ui/apps/v4/content/docs/components/base/toggle-group.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/toggle-group.mdx` | `toggle_group` | `apps/fret-ui-gallery/src/ui/pages/toggle_group.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/toggle/ui-gallery-toggle-group-*.json`, `tools/diag-scripts/ui-gallery/control-chrome/ui-gallery-control-chrome-toggle-group-flex1-item-fill.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-toggle-group-stretch*` stable for flex-1 chrome fill gate. |
| tooltip | `repo-ref/ui/apps/v4/content/docs/components/base/tooltip.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/tooltip.mdx` | `tooltip` | `apps/fret-ui-gallery/src/ui/pages/tooltip.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/overlay/ui-gallery-tooltip-*` | Snippet-backed previews + region-sliced code tabs for all sections (preview ≡ code). Keep demo/focus/keyboard/rtl `test_id`s stable for diag scripts. |
| typography | `repo-ref/ui/apps/v4/content/docs/components/base/typography.mdx` | `repo-ref/ui/apps/v4/content/docs/components/radix/typography.mdx` | `typography` | `apps/fret-ui-gallery/src/ui/pages/typography.rs` | Yes | Done | `tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-*.json` | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `docsec-inline-code-tabs-trigger-code` stable for code-tab scroll-range gates. |

## Fret-only modules (extensions)

These are present in `ecosystem/fret-ui-shadcn` but do not have 1:1 upstream MDX pages. We track
their closest upstream anchor (if any) to keep parity discussions explicit.

| Fret module | Upstream anchor | Gallery Page | Notes |
|---|---|---:|---|
| `app_integration` | — | TBD | App wiring helpers; not a shadcn component. |
| `calendar_hijri` | `calendar.mdx` | TBD | Variant recipe; anchor to Calendar docs. |
| `calendar_multiple` | `calendar.mdx` | TBD | Variant recipe; anchor to Calendar docs. |
| `calendar_range` | `calendar.mdx` | TBD | Variant recipe; anchor to Calendar docs. |
| `combobox_chips` | `combobox.mdx` | TBD | Variant recipe; anchor to Combobox docs. |
| `data_grid_canvas` | — | TBD | Fret-specific; no upstream shadcn page. |
| `date_picker_with_presets` | `date-picker.mdx` | TBD | Variant recipe; anchor to Date Picker docs. |
| `date_range_picker` | `date-picker.mdx` | TBD | Variant recipe; anchor to Date Picker docs. |
| `experimental` | — | TBD | Non-normative incubations; migrate last. |
| `extras` | — | TBD | Non-normative shadcn-styled blocks; tracked separately. |
| `form` | — | TBD | Large composite demo; treat as “page-level example”, not a component. |
| `media_image` | — | TBD | Fret-specific media utilities; not in upstream shadcn MDX. |
| `recharts_geometry` | `chart.mdx` | TBD | Implementation detail/bridge; anchor to Chart docs. |
| `shadcn_themes` | — | TBD | Theme/catalog wiring; not a component. |
| `shortcut_hint` | — | TBD | Fret-specific; not in upstream shadcn MDX. |
| `state` | — | TBD | Internal/shared state glue; not a component. |

## UI Gallery legacy pages tracker (non-shadcn)

These pages use the same `DocSection` infrastructure but are not directly tracked in the shadcn v4
component table above. We still migrate them to snippet-backed sections so the gallery stays
drift-free (preview ≡ copyable code).

| Page | Gallery Page | Snippet-backed | Status | Notes |
|---|---:|---:|---:|---|
| icons | `apps/fret-ui-gallery/src/ui/pages/icons.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). |
| empty | `apps/fret-ui-gallery/src/ui/pages/empty.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). |
| card | `apps/fret-ui-gallery/src/ui/pages/card.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). |
| item | `apps/fret-ui-gallery/src/ui/pages/item.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Gates: `tools/diag-scripts/ui-gallery/item/ui-gallery-item-*.json`. |
| image-object-fit | `apps/fret-ui-gallery/src/ui/pages/image_object_fit.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). |
| motion-presets | `apps/fret-ui-gallery/src/ui/pages/motion_presets.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Gates: `tools/diag-scripts/ui-gallery/motion-presets/ui-gallery-motion-presets-*.json`. Keep `ui-gallery-motion-presets-*` `test_id`s stable. |
| collapsible | `apps/fret-ui-gallery/src/ui/pages/collapsible.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-collapsible-*` `test_id`s stable for diag scripts. |
| carousel | `apps/fret-ui-gallery/src/ui/pages/carousel.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Gates: `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-*.json`. |
| chart | `apps/fret-ui-gallery/src/ui/pages/chart.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-chart-*` `test_id`s stable for diag scripts. |
| data-table | `apps/fret-ui-gallery/src/ui/pages/data_table.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-data-table-*` `test_id`s stable for diag scripts. |
| date-picker | `apps/fret-ui-gallery/src/ui/pages/date_picker.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). |
| navigation-menu | `apps/fret-ui-gallery/src/ui/pages/navigation_menu.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Keep `ui-gallery-navigation-menu-*` `test_id`s stable for diag scripts. |
| form | `apps/fret-ui-gallery/src/ui/pages/form.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). |
| shadcn-extras | `apps/fret-ui-gallery/src/ui/pages/shadcn_extras.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). |
| combobox | `apps/fret-ui-gallery/src/ui/pages/combobox.rs` | Yes | Done | Snippet-backed previews + region-sliced code tabs (preview ≡ code). Gates: `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-*.json`. |
