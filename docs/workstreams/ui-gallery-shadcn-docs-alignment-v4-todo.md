# UI Gallery — Shadcn Docs Alignment (v4) — TODO Tracker

Status: Active (workstream tracker; keep updated as pages land)

Goal: Align the `Shadcn` section of `apps/fret-ui-gallery` with upstream shadcn/ui v4 docs:

- page ordering (left nav matches docs),
- per-page example ordering (examples match docs),
- interaction/state outcomes (hover/active/focus/disabled/selected),
- minimal, repeatable repros (scripts/tests) for any parity regressions.

References:

- Canonical order: `repo-ref/ui/apps/v4/content/docs/components/radix/meta.json`
- Upstream page content: `repo-ref/ui/apps/v4/content/docs/components/radix/*.mdx`
- Gallery page list: `apps/fret-ui-gallery/src/spec.rs`
- Gallery previews: `apps/fret-ui-gallery/src/ui.rs`
- Visual parity tracker: `docs/workstreams/ui-gallery-visual-parity-todo.md`
- Web golden parity: `docs/workstreams/shadcn-web-goldens-v4-todo.md`
- Semantics decorators adoption: `docs/workstreams/semantics-decorators-adoption-v1-todo.md`

Tracking format:

- ID: `SGD-{area}-{nnn}`
- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

---

## P0 — Information Architecture (make parity debuggable)

- [x] SGD-ia-010 Reorder the `Shadcn` page group to match `radix/meta.json` order.
- [x] SGD-ia-020 Resolve non-upstream pages in the Shadcn group (e.g. `Icons`, `DataGrid`, hub pages like `Forms/Menus/Overlay`).
  - Option A (preferred): move to a separate “Shadcn (Extras)” group.
  - Option B: keep them, but place strictly after `typography` and label them “extras”.
- [x] SGD-ia-030 Ensure the gallery left-nav has a persistent selected state (matches shadcn docs UX).
- [~] SGD-ia-040 Remove the “pressed text sinks / baseline shifts” effect from nav rows (should not change layout metrics).

---

## P1 — Page Content (mirror upstream example order)

For each page:

- Mirror upstream section headings + example ordering (from `repo-ref/ui/.../<page>.mdx`).
- Prefer targeted repros over manual eyeballing:
  - add a minimal UI gallery page-local demo (or state matrix),
  - add a deterministic diag script when possible,
  - add a regression test when the contract is stable.
- When a mismatch is found, capture:
  - repro steps (ideally a diag script),
  - expected vs observed outcome,
  - the layer that owns the fix (`fret-ui` runtime vs `fret-ui-kit` vs `fret-ui-shadcn` vs gallery glue).

Component checklist (canonical order from `radix/meta.json`):

- [~] accordion — examples mirrored; parity still under audit (`apps/fret-ui-gallery/src/ui.rs::preview_accordion`)
- [ ] alert
- [~] alert-dialog - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/alert_dialog.rs`)
- [~] aspect-ratio - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/aspect_ratio.rs`)
- [~] avatar — base demo present; still missing upstream examples (badge/group/sizes/dropdown/RTL) (`apps/fret-ui-gallery/src/ui.rs::preview_avatar`, `ecosystem/fret-ui-shadcn/src/avatar.rs`)
- [~] badge — examples mirrored; missing upstream variants (`apps/fret-ui-gallery/src/ui.rs::preview_badge`)
- [~] breadcrumb - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/breadcrumb.rs`)
- [x] button - examples aligned; interaction parity validated by button hover/active reset + web-golden smoke (`ecosystem/fret-ui-shadcn/src/button.rs::tests::button_clears_hover_and_active_visuals_after_click_and_pointer_leave`, `ecosystem/fret-ui-shadcn/tests/web_goldens_smoke.rs::web_golden_button_default_smoke`)
- [x] button-group — examples mirrored (`apps/fret-ui-gallery/src/ui.rs::preview_button_group`)
- [x] calendar — examples mirrored (Basic/Range/Month+Year selector/Presets/Date+Time/Booked/Custom Cell Size/Week Numbers/RTL) (`apps/fret-ui-gallery/src/ui.rs::preview_calendar`, `ecosystem/fret-ui-shadcn/src/calendar.rs`)
- [x] card — examples mirrored + size/action slots (`apps/fret-ui-gallery/src/ui.rs::preview_card`, `ecosystem/fret-ui-shadcn/src/card.rs`)
- [~] carousel - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/carousel.rs`)
- [~] chart - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/chart.rs`)
- [~] checkbox - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/checkbox.rs`)
- [~] collapsible - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/collapsible.rs`)
- [~] combobox - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/combobox.rs`)
- [~] command - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/command.rs`)
- [~] context-menu - docs-order examples + Component/Code/Notes scaffold landed (apps/fret-ui-gallery/src/ui/pages/context_menu.rs)
- [~] data-table - docs-order sections + Component/Code/Notes scaffold landed (apps/fret-ui-gallery/src/ui/pages/data_table.rs)
- [~] date-picker - docs-order examples + Component/Code/Notes scaffold landed (apps/fret-ui-gallery/src/ui/pages/date_picker.rs)
- [~] dialog - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/dialog.rs`)
- [ ] direction
- [~] drawer - docs-order examples + Component/Code/Notes scaffold landed (apps/fret-ui-gallery/src/ui/pages/drawer.rs)
- [~] dropdown-menu - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/dropdown_menu.rs`)
- [~] empty - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/empty.rs`)
- [~] field - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/field.rs`)
- [~] form - gallery-hub examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/form.rs`)
- [~] hover-card - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/hover_card.rs`)
- [~] input - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/input.rs`)
- [~] input-group - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/input_group.rs`)
- [~] input-otp - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/input_otp.rs`)
- [ ] item
- [ ] kbd
- [ ] label
- [ ] menubar
- [ ] native-select
- [ ] navigation-menu
- [x] pagination — examples mirrored (Demo/Simple/Icons Only/RTL) (`apps/fret-ui-gallery/src/ui.rs::preview_pagination`, `ecosystem/fret-ui-shadcn/src/pagination.rs`)
- [x] popover — examples mirrored (Demo/Basic/Align/With Form/RTL) (`apps/fret-ui-gallery/src/ui.rs::preview_popover`, `ecosystem/fret-ui-shadcn/src/popover.rs`)
- [x] progress — examples mirrored (Demo/Label/Controlled/RTL) (`apps/fret-ui-gallery/src/ui.rs::preview_progress`, `ecosystem/fret-ui-shadcn/src/progress.rs`)
- [x] radio-group — examples mirrored (Demo/Description/Choice Card/Fieldset/Disabled/Invalid/RTL) (`apps/fret-ui-gallery/src/ui.rs::preview_radio_group`, `ecosystem/fret-ui-shadcn/src/radio_group.rs`, `ecosystem/fret-ui-shadcn/src/radio_group.rs::tests::*radio_group_*`)
- [x] resizable — examples mirrored (Demo/Vertical/Handle/RTL; `with_handle` approximates grip) (`apps/fret-ui-gallery/src/ui.rs::preview_resizable`, `ecosystem/fret-ui-shadcn/src/resizable.rs`)
- [x] scroll-area — examples mirrored (Demo/Horizontal/RTL) (`apps/fret-ui-gallery/src/ui.rs::preview_scroll_area`)
- [~] select (group semantics + default alignment audited; needs full docs example order)
- [x] separator ? examples mirrored (Demo/Vertical/Menu/List/RTL) (`apps/fret-ui-gallery/src/ui.rs::preview_separator`)
- [x] sheet ? examples mirrored (Demo/Side/No Close Button/RTL; `showCloseButton` behavior approximated by omitting explicit close actions) (`apps/fret-ui-gallery/src/ui.rs::preview_sheet`)
- [x] sidebar ? examples mirrored (Demo/Controlled/RTL; aligned to docs `sidebar-demo` + controlled state pattern) (`apps/fret-ui-gallery/src/ui.rs::preview_sidebar`)
- [x] skeleton ? examples mirrored (Demo/Avatar/Card/Text/Form/Table/RTL) (`apps/fret-ui-gallery/src/ui.rs::preview_skeleton`)
- [x] slider — examples mirrored (Demo/Range/Multiple Thumbs/Vertical/Controlled/Disabled/RTL; extras: inverted) (`apps/fret-ui-gallery/src/ui.rs::preview_slider`, `ecosystem/fret-ui-shadcn/src/slider.rs`, `tools/diag-scripts/ui-gallery-slider-and-avatar-screenshots.json`)
- [x] sonner — examples mirrored (Demo/Types/Description/Position, with runtime toaster position binding) (`apps/fret-ui-gallery/src/ui.rs::preview_sonner`, `apps/fret-ui-gallery/src/driver.rs`)
- [x] spinner — examples mirrored (Demo/Customization/Size/Button/Badge/Input Group/Empty/RTL) (`apps/fret-ui-gallery/src/ui.rs::preview_spinner`)
- [x] switch ? examples mirrored (Demo/Description/Choice Card/Disabled/Invalid/Size/RTL; size + invalid are style approximations in current API) (`apps/fret-ui-gallery/src/ui.rs::preview_switch`, `ecosystem/fret-ui-shadcn/src/switch.rs`)
- [x] table ? examples mirrored (Demo/Footer/Actions/RTL; action trigger uses text icon approximation) (`apps/fret-ui-gallery/src/ui.rs::preview_table`, `ecosystem/fret-ui-shadcn/src/table.rs`)
- [x] tabs ? examples mirrored (Demo/Line/Vertical/Disabled/Icons/RTL; line variant uses style approximation) (`apps/fret-ui-gallery/src/ui.rs::preview_tabs`, `ecosystem/fret-ui-shadcn/src/tabs.rs`)
- [x] textarea ? examples mirrored (Demo/Field/Disabled/Invalid/Button/RTL) (`apps/fret-ui-gallery/src/ui.rs::preview_textarea`, `ecosystem/fret-ui-shadcn/src/textarea.rs`)
- [x] toast ? docs-aligned deprecation notice only (upstream points to Sonner) (`apps/fret-ui-gallery/src/ui.rs::preview_toast`, `repo-ref/ui/apps/v4/content/docs/components/radix/toast.mdx`)
- [~] toggle - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/toggle.rs`)
- [~] toggle-group - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/toggle_group.rs`)
- [~] tooltip - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/tooltip.rs`)
- [~] typography - docs-order examples + Component/Code/Notes scaffold landed (`apps/fret-ui-gallery/src/ui/pages/typography.rs`)

---

## P2 — Known Issues to Triage (UI Gallery reports)

- [!] SGD-bug-010 Left-nav hover/selected backgrounds inconsistent (some rows highlight, others do not).
  - Could not reproduce for `SidebarMenuButton` rows via scripted diagnostics.
  - Evidence:
    - `tools/diag-scripts/ui-gallery-nav-disabled-scan.json`: 0 disabled `ui-gallery-nav-*` rows in semantics.
    - `tools/diag-scripts/ui-gallery-nav-hover-inconsistency-screenshots.json` + `FRET_DIAG_SCREENSHOT=1`: hover vs baseline crops differ for `intro/layout/card/accordion`.
  - Likely explanations:
    - hovering the group headings (plain text, not pressable),
    - a different sidebar surface (e.g. the `Sidebar` component demo) rather than the gallery nav.
  - Next: capture the exact label + page + (ideally) a screenshot or bundle to confirm the target row.
- [ ] SGD-bug-020 Left-nav “text sinks” on click (pressed state changes layout metrics).
- [ ] SGD-bug-030 Accordion parity vs upstream (trigger height/padding, chevron behavior, content animation).
- [ ] SGD-bug-040 Card preview text vertical alignment jitter when switching pages.
- [x] SGD-bug-050 Calendar “blank space to the right”: confirmed calendar examples are narrow in web goldens (expected whitespace in wide viewports).
- [x] SGD-bug-060 Switch thumb vertical centering: covered by `Switch` unit + web golden geometry tests (no repro in automated checks).
- [x] SGD-bug-080 Pressable can get stuck in Active after `PointerCancel` (release outside window), leaving shadcn buttons “greyed”.
  - Fix: clear `pressed_pressable` on `Event::PointerCancel` in `crates/fret-ui/src/tree/dispatch.rs`.
  - Regression: `crates/fret-ui/src/declarative/tests/interactions.rs::pressable_clears_pressed_state_on_pointer_cancel`.
- [!] SGD-bug-070 Menubar hover highlight background mismatches web goldens (fails in nextest).
  - Failing tests: `web_vs_fret_menubar_demo_highlighted_item_chrome_matches_web_dark`, `web_vs_fret_menubar_demo_highlighted_item_chrome_matches_web_dark_mobile_tiny_viewport`.
  - Hypothesis: hover visuals not applied (panel background quad selected instead of row highlight quad).
  - Next: verify capture/backdrop layering and confirm `PressableState.hovered` transitions for menu rows.

---

## P3 — Tooling / Verification (keep drift small)

- [ ] SGD-tool-010 Add a lightweight “walk pages in docs order” smoke runner (jump + layout + paint once).
- [ ] SGD-tool-020 Add per-page state matrix helpers for rich components (Button, Toggle, Switch, Select, Tabs).
- [ ] SGD-tool-030 Add a golden-friendly “one-shot snapshot” mode for UI gallery pages (deterministic, no input).
