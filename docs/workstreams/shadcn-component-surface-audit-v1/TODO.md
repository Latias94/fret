## Shadcn Component Surface Audit v1 (TODO + Tracker)

Last updated: 2026-03-02.

### Upstream index

- Docs: `repo-ref/ui/apps/v4/content/docs/components/radix/`
- Base sources: `repo-ref/ui/apps/v4/registry/bases/radix/ui/`

### Status legend

- `Not audited`
- `In progress`
- `Done (with known gaps)`
- `Done`

### Tracker (initial)

| Component | Upstream doc | Upstream base | Fret module | Gates | Status |
|---|---|---|---|---|---|
| `accordion` | `repo-ref/ui/apps/v4/content/docs/components/radix/accordion.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/accordion.tsx` | `ecosystem/fret-ui-shadcn/src/accordion.rs` | (none yet) | Not audited |
| `alert` | `repo-ref/ui/apps/v4/content/docs/components/radix/alert.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/alert.tsx` | `ecosystem/fret-ui-shadcn/src/alert.rs` | (none yet) | Not audited |
| `alert-dialog` | `repo-ref/ui/apps/v4/content/docs/components/radix/alert-dialog.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/alert-dialog.tsx` | `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` | (none yet) | Not audited |
| `aspect-ratio` | `repo-ref/ui/apps/v4/content/docs/components/radix/aspect-ratio.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/aspect-ratio.tsx` | `ecosystem/fret-ui-shadcn/src/aspect_ratio.rs` | (none yet) | Not audited |
| `avatar` | `repo-ref/ui/apps/v4/content/docs/components/radix/avatar.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/avatar.tsx` | `ecosystem/fret-ui-shadcn/src/avatar.rs` | (none yet) | Not audited |
| `badge` | `repo-ref/ui/apps/v4/content/docs/components/radix/badge.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/badge.tsx` | `ecosystem/fret-ui-shadcn/src/badge.rs` | (none yet) | Not audited |
| `breadcrumb` | `repo-ref/ui/apps/v4/content/docs/components/radix/breadcrumb.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/breadcrumb.tsx` | `ecosystem/fret-ui-shadcn/src/breadcrumb.rs` | (none yet) | Not audited |
| `button` | `repo-ref/ui/apps/v4/content/docs/components/radix/button.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/button.tsx` | `ecosystem/fret-ui-shadcn/src/button.rs` | (none yet) | Not audited |
| `button-group` | `repo-ref/ui/apps/v4/content/docs/components/radix/button-group.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/button-group.tsx` | `ecosystem/fret-ui-shadcn/src/button_group.rs` | (none yet) | Not audited |
| `calendar` | `repo-ref/ui/apps/v4/content/docs/components/radix/calendar.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/calendar.tsx` | `ecosystem/fret-ui-shadcn/src/calendar.rs` | (none yet) | Not audited |
| `card` | `repo-ref/ui/apps/v4/content/docs/components/radix/card.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/card.tsx` | `ecosystem/fret-ui-shadcn/src/card.rs` | (none yet) | Not audited |
| `carousel` | `repo-ref/ui/apps/v4/content/docs/components/radix/carousel.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/carousel.tsx` | `ecosystem/fret-ui-shadcn/src/carousel.rs` | (none yet) | Not audited |
| `chart` | `repo-ref/ui/apps/v4/content/docs/components/radix/chart.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/chart.tsx` | `ecosystem/fret-ui-shadcn/src/chart.rs` | (none yet) | Not audited |
| `checkbox` | `repo-ref/ui/apps/v4/content/docs/components/radix/checkbox.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/checkbox.tsx` | `ecosystem/fret-ui-shadcn/src/checkbox.rs` | (none yet) | Not audited |
| `collapsible` | `repo-ref/ui/apps/v4/content/docs/components/radix/collapsible.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/collapsible.tsx` | `ecosystem/fret-ui-shadcn/src/collapsible.rs` | (none yet) | Not audited |
| `combobox` | `repo-ref/ui/apps/v4/content/docs/components/radix/combobox.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/combobox.tsx` | `ecosystem/fret-ui-shadcn/src/combobox.rs` | `ecosystem/fret-ui-shadcn/tests/combobox_test_id_prefix_semantics.rs`, `ecosystem/fret-ui-shadcn/tests/combobox_keyboard_navigation.rs`, `ecosystem/fret-ui-shadcn/tests/combobox_escape_dismiss_focus_restore.rs`, `ecosystem/fret-ui-shadcn/tests/combobox_filtering.rs` | In progress |
| `command` | `repo-ref/ui/apps/v4/content/docs/components/radix/command.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/command.tsx` | `ecosystem/fret-ui-shadcn/src/command.rs` | (none yet) | Not audited |
| `context-menu` | `repo-ref/ui/apps/v4/content/docs/components/radix/context-menu.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/context-menu.tsx` | `ecosystem/fret-ui-shadcn/src/context_menu.rs` | `ecosystem/fret-ui-shadcn/tests/context_menu_escape_dismiss_focus_clears.rs`, `ecosystem/fret-ui-shadcn/tests/context_menu_keyboard_navigation.rs` | In progress |
| `data-table` | `repo-ref/ui/apps/v4/content/docs/components/radix/data-table.mdx` | (docs-only) | `ecosystem/fret-ui-shadcn/src/data_table.rs` | (none yet) | Not audited |
| `date-picker` | `repo-ref/ui/apps/v4/content/docs/components/radix/date-picker.mdx` | (docs-only) | `ecosystem/fret-ui-shadcn/src/date_picker.rs` | (none yet) | Not audited |
| `dialog` | `repo-ref/ui/apps/v4/content/docs/components/radix/dialog.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/dialog.tsx` | `ecosystem/fret-ui-shadcn/src/dialog.rs` | `ecosystem/fret-ui-shadcn/tests/dialog_escape_dismiss_focus_restore.rs`, `ecosystem/fret-ui-shadcn/tests/dialog_overlay_click_dismiss_focus_restore.rs` | In progress |
| `direction` | `repo-ref/ui/apps/v4/content/docs/components/radix/direction.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/direction.tsx` | `ecosystem/fret-ui-shadcn/src/direction.rs` | (none yet) | Not audited |
| `drawer` | `repo-ref/ui/apps/v4/content/docs/components/radix/drawer.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/drawer.tsx` | `ecosystem/fret-ui-shadcn/src/drawer.rs` | (none yet) | Not audited |
| `dropdown-menu` | `repo-ref/ui/apps/v4/content/docs/components/radix/dropdown-menu.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/dropdown-menu.tsx` | `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` | `ecosystem/fret-ui-shadcn/tests/dropdown_menu_escape_dismiss_focus_restore.rs`, `ecosystem/fret-ui-shadcn/tests/dropdown_menu_keyboard_navigation.rs` | In progress |
| `empty` | `repo-ref/ui/apps/v4/content/docs/components/radix/empty.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/empty.tsx` | `ecosystem/fret-ui-shadcn/src/empty.rs` | (none yet) | Not audited |
| `field` | `repo-ref/ui/apps/v4/content/docs/components/radix/field.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/field.tsx` | `ecosystem/fret-ui-shadcn/src/field.rs` | (none yet) | Not audited |
| `hover-card` | `repo-ref/ui/apps/v4/content/docs/components/radix/hover-card.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/hover-card.tsx` | `ecosystem/fret-ui-shadcn/src/hover_card.rs` | (none yet) | Not audited |
| `input` | `repo-ref/ui/apps/v4/content/docs/components/radix/input.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/input.tsx` | `ecosystem/fret-ui-shadcn/src/input.rs` | (none yet) | Not audited |
| `input-group` | `repo-ref/ui/apps/v4/content/docs/components/radix/input-group.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/input-group.tsx` | `ecosystem/fret-ui-shadcn/src/input_group.rs` | (none yet) | Not audited |
| `input-otp` | `repo-ref/ui/apps/v4/content/docs/components/radix/input-otp.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/input-otp.tsx` | `ecosystem/fret-ui-shadcn/src/input_otp.rs` | (none yet) | Not audited |
| `item` | `repo-ref/ui/apps/v4/content/docs/components/radix/item.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/item.tsx` | `ecosystem/fret-ui-shadcn/src/item.rs` | (none yet) | Not audited |
| `kbd` | `repo-ref/ui/apps/v4/content/docs/components/radix/kbd.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/kbd.tsx` | `ecosystem/fret-ui-shadcn/src/kbd.rs` | (none yet) | Not audited |
| `label` | `repo-ref/ui/apps/v4/content/docs/components/radix/label.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/label.tsx` | `ecosystem/fret-ui-shadcn/src/label.rs` | (none yet) | Not audited |
| `menubar` | `repo-ref/ui/apps/v4/content/docs/components/radix/menubar.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/menubar.tsx` | `ecosystem/fret-ui-shadcn/src/menubar.rs` | `ecosystem/fret-ui-shadcn/tests/menubar_escape_dismiss_focus_restore.rs`, `ecosystem/fret-ui-shadcn/tests/menubar_keyboard_navigation.rs` | In progress |
| `native-select` | `repo-ref/ui/apps/v4/content/docs/components/radix/native-select.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/native-select.tsx` | `ecosystem/fret-ui-shadcn/src/native_select.rs` | (none yet) | Not audited |
| `navigation-menu` | `repo-ref/ui/apps/v4/content/docs/components/radix/navigation-menu.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/navigation-menu.tsx` | `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` | `ecosystem/fret-ui-shadcn/tests/navigation_menu_escape_dismiss_focus_restore.rs`, `ecosystem/fret-ui-shadcn/tests/navigation_menu_keyboard_navigation.rs` | In progress |
| `pagination` | `repo-ref/ui/apps/v4/content/docs/components/radix/pagination.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/pagination.tsx` | `ecosystem/fret-ui-shadcn/src/pagination.rs` | (none yet) | Not audited |
| `popover` | `repo-ref/ui/apps/v4/content/docs/components/radix/popover.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/popover.tsx` | `ecosystem/fret-ui-shadcn/src/popover.rs` | `ecosystem/fret-ui-shadcn/tests/popover_escape_dismiss_focus_restore.rs`, `ecosystem/fret-ui-shadcn/tests/popover_outside_click_dismiss_focus_restore.rs` | In progress |
| `progress` | `repo-ref/ui/apps/v4/content/docs/components/radix/progress.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/progress.tsx` | `ecosystem/fret-ui-shadcn/src/progress.rs` | (none yet) | Not audited |
| `radio-group` | `repo-ref/ui/apps/v4/content/docs/components/radix/radio-group.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/radio-group.tsx` | `ecosystem/fret-ui-shadcn/src/radio_group.rs` | (none yet) | Not audited |
| `resizable` | `repo-ref/ui/apps/v4/content/docs/components/radix/resizable.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/resizable.tsx` | `ecosystem/fret-ui-shadcn/src/resizable.rs` | (none yet) | Not audited |
| `scroll-area` | `repo-ref/ui/apps/v4/content/docs/components/radix/scroll-area.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/scroll-area.tsx` | `ecosystem/fret-ui-shadcn/src/scroll_area.rs` | (none yet) | Not audited |
| `select` | `repo-ref/ui/apps/v4/content/docs/components/radix/select.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/select.tsx` | `ecosystem/fret-ui-shadcn/src/select.rs` | `ecosystem/fret-ui-shadcn/tests/select_test_id_stability.rs`, `ecosystem/fret-ui-shadcn/tests/select_keyboard_navigation.rs`, `ecosystem/fret-ui-shadcn/tests/select_escape_dismiss_focus_restore.rs`, `ecosystem/fret-ui-shadcn/tests/select_typeahead.rs` | In progress |
| `separator` | `repo-ref/ui/apps/v4/content/docs/components/radix/separator.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/separator.tsx` | `ecosystem/fret-ui-shadcn/src/separator.rs` | (none yet) | Not audited |
| `sheet` | `repo-ref/ui/apps/v4/content/docs/components/radix/sheet.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/sheet.tsx` | `ecosystem/fret-ui-shadcn/src/sheet.rs` | (none yet) | Not audited |
| `sidebar` | `repo-ref/ui/apps/v4/content/docs/components/radix/sidebar.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/sidebar.tsx` | `ecosystem/fret-ui-shadcn/src/sidebar.rs` | (none yet) | Not audited |
| `skeleton` | `repo-ref/ui/apps/v4/content/docs/components/radix/skeleton.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/skeleton.tsx` | `ecosystem/fret-ui-shadcn/src/skeleton.rs` | (none yet) | Not audited |
| `slider` | `repo-ref/ui/apps/v4/content/docs/components/radix/slider.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/slider.tsx` | `ecosystem/fret-ui-shadcn/src/slider.rs` | (none yet) | Not audited |
| `sonner` | `repo-ref/ui/apps/v4/content/docs/components/radix/sonner.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/sonner.tsx` | `ecosystem/fret-ui-shadcn/src/sonner.rs` | (none yet) | Not audited |
| `spinner` | `repo-ref/ui/apps/v4/content/docs/components/radix/spinner.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/spinner.tsx` | `ecosystem/fret-ui-shadcn/src/spinner.rs` | (none yet) | Not audited |
| `switch` | `repo-ref/ui/apps/v4/content/docs/components/radix/switch.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/switch.tsx` | `ecosystem/fret-ui-shadcn/src/switch.rs` | (none yet) | Not audited |
| `table` | `repo-ref/ui/apps/v4/content/docs/components/radix/table.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/table.tsx` | `ecosystem/fret-ui-shadcn/src/table.rs` | (none yet) | Not audited |
| `tabs` | `repo-ref/ui/apps/v4/content/docs/components/radix/tabs.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/tabs.tsx` | `ecosystem/fret-ui-shadcn/src/tabs.rs` | (none yet) | Not audited |
| `textarea` | `repo-ref/ui/apps/v4/content/docs/components/radix/textarea.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/textarea.tsx` | `ecosystem/fret-ui-shadcn/src/textarea.rs` | (none yet) | Not audited |
| `toast` | `repo-ref/ui/apps/v4/content/docs/components/radix/toast.mdx` | (docs-only) | `ecosystem/fret-ui-shadcn/src/toast.rs` | (none yet) | Not audited |
| `toggle` | `repo-ref/ui/apps/v4/content/docs/components/radix/toggle.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/toggle.tsx` | `ecosystem/fret-ui-shadcn/src/toggle.rs` | (none yet) | Not audited |
| `toggle-group` | `repo-ref/ui/apps/v4/content/docs/components/radix/toggle-group.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/toggle-group.tsx` | `ecosystem/fret-ui-shadcn/src/toggle_group.rs` | (none yet) | Not audited |
| `tooltip` | `repo-ref/ui/apps/v4/content/docs/components/radix/tooltip.mdx` | `repo-ref/ui/apps/v4/registry/bases/radix/ui/tooltip.tsx` | `ecosystem/fret-ui-shadcn/src/tooltip.rs` | `ecosystem/fret-ui-shadcn/tests/tooltip_hover_and_escape.rs` | In progress |
| `typography` | `repo-ref/ui/apps/v4/content/docs/components/radix/typography.mdx` | (docs-only) | `ecosystem/fret-ui-shadcn/src/typography.rs` | (none yet) | Not audited |

### Proposed audit order (v1)

1. `select` / `combobox` (deep redesign workstream already created)
2. Overlay family: `dropdown-menu` → `context-menu` → `dialog` → `popover` → `tooltip`
3. Composite nav: `menubar` → `navigation-menu` → `tabs`
4. Tables: `table` + `data-table` overflow and responsive behavior
