# Shadcn Component Conformance Matrix (Fret)

This document tracks **shadcn/ui (New York v4)** alignment work in Fret, and provides a single place to
record:

- What we consider “done” for a component (fonts/layout/behavior/hit-testing).
- What tests exist today (unit/integration + `fretboard diag` scripts).
- What’s still missing (gaps + next actions).

## Sources of truth (pinned references)

- shadcn/ui v4 registry: `repo-ref/ui` (see `registry/new-york-v4/ui/*`).
- Radix primitives: `repo-ref/primitives` (behavioral reference for overlays/focus/dismiss).
- Base UI: `repo-ref/base-ui` (additional reference for interaction + accessibility conventions).

## How we test (recommended layers)

1. **Deterministic Rust tests** (policy/state machine/layout helpers)
   - Prefer small unit tests close to the component/policy layer in `ecosystem/fret-ui-shadcn`.
2. **Web-vs-Fret alignment tests** (geometry/placement matrices)
   - Existing example: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`.
3. **End-to-end `fretboard diag` scripts** (routing/focus/click-through/geometry under the real runner)
   - Scripts live in `tools/diag-scripts/*.json`.
   - Prefer `test_id` selectors and geometry predicates (avoid pixel-only assertions unless needed).

## Running the shadcn conformance suite

Run the curated shadcn-focused suite (recommended):

```bash
cargo run -p fretboard -- diag suite ui-gallery-shadcn-conformance --launch -- cargo run -p fret-ui-gallery --release
```

Notes:

- The suite enables screenshots when needed and can be used as a CI-friendly gate.
- For deterministic fonts on desktop, the suite sets `FRET_UI_GALLERY_BOOTSTRAP_FONTS=1` (bundled fonts).

## Font / text stress strategy (practical)

We treat “fonts overlap” as: **no unintended bounds overlap** between sibling UI elements where
layout expects separation (icon vs label, leading addon vs control, etc.).

Recommended approach:

- Add stable `test_id` to **sub-parts** (icon/label/leading/trailing/control) on the UI Gallery page.
- Add diag predicates:
  - `bounds_non_overlapping` for “must not overlap”.
  - `bounds_within_window` for “must not clip outside the window”.
  - `bounds_min_size` for “must remain tappable”.

Optional (advanced):

- Run the same scripts under alternate font family configs by launching UI Gallery with a different
  project root (see `FRET_UI_GALLERY_PROJECT_ROOT`) that contains a `.fret/settings.json`.

## Conformance matrix

Legend:

- **Diag Scripts**: primary `fretboard diag` coverage for the component (useful repros + gates).
- **Rust Tests**: unit/integration tests in `ecosystem/fret-ui-shadcn` (or related crates).
- **Coverage (Fonts/Hit/Layout)**: what the current tests actually exercise.
- **Status**: `TODO` / `In progress` / `Aligned` / `Aligned (with gaps)`.

| Component | UI Gallery Page | Diag Scripts | Rust Tests | Coverage (Fonts/Hit/Layout) | Status | Notes |
|---|---|---|---|---|---|---|
| Accordion | `accordion` | TODO | TODO | TODO | TODO | |
| Alert | `alert` | TODO | TODO | TODO | TODO | |
| Alert Dialog | `alert_dialog` | `tools/diag-scripts/ui-gallery-alert-dialog-least-destructive-initial-focus.json` | TODO | Focus + least-destructive default | In progress | Add dismiss + click-through gates. |
| Aspect Ratio | `aspect_ratio` | TODO | TODO | TODO | TODO | |
| Avatar | `avatar` | TODO | TODO | TODO | TODO | |
| Badge | `badge` | TODO | TODO | TODO | TODO | |
| Breadcrumb | `breadcrumb` | TODO | TODO | TODO | TODO | |
| Button | `button` | `tools/diag-scripts/ui-gallery-button-with-icon-non-overlap.json` | TODO | Fonts/layout (icon vs label non-overlap) | In progress | Extend: min hit-size + disabled/pressed routing. |
| Button Group | `button_group` | TODO | TODO | TODO | TODO | |
| Calendar | `calendar` | TODO | TODO | TODO | TODO | |
| Card | `card` | `tools/diag-scripts/ui-gallery-card-description-no-early-wrap.json` | TODO | Text layout (subtitle no early wrap) | In progress | Add `CardHeader` title/action spacing invariants. |
| Carousel | `carousel` | TODO | TODO | TODO | TODO | |
| Chart | `chart` | TODO | TODO | TODO | TODO | |
| Checkbox | `checkbox` | TODO | TODO | TODO | TODO | |
| Collapsible | `collapsible` | TODO | TODO | TODO | TODO | |
| Combobox | `combobox` | `tools/diag-scripts/ui-gallery-combobox-open-select-focus-restore.json`, `tools/diag-scripts/ui-gallery-combobox-commit-pixels-changed.json` | TODO | Focus restore + selection commit | In progress | Add disabled item + outside press policy. |
| Command Palette | `command` | TODO | TODO | TODO | TODO | |
| Context Menu | `context_menu` | `tools/diag-scripts/ui-gallery-context-menu-right-click.json` | TODO | Right-click routing + overlay | In progress | Add click-through + submenu bounds gates. |
| DataGrid | `data_grid` | TODO | TODO | TODO | TODO | |
| DataTable | `data_table` | TODO | TODO | TODO | TODO | |
| Date Picker | `date_picker` | TODO | TODO | TODO | TODO | |
| Dialog | `dialog` | `tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json`, `tools/diag-scripts/ui-gallery-dialog-docs-order-smoke.json` | TODO | Escape dismiss + focus restore | In progress | Add underlay/click-through policy gates. |
| Drawer | `drawer` | TODO | TODO | TODO | TODO | |
| Dropdown Menu | `dropdown_menu` | `tools/diag-scripts/ui-gallery-dropdown-open-select.json`, `tools/diag-scripts/ui-gallery-dropdown-submenu-underlay-dismiss.json` | TODO | Open/select + submenu dismiss | In progress | Add safe-corridor sweep + bounds gate. |
| Empty | `empty` | TODO | TODO | TODO | TODO | |
| Extras | `shadcn_extras` | `tools/diag-scripts/ui-gallery-shadcn-extras-screenshots.json` | TODO | Visual smoke + layout | In progress | Convert screenshots to specific invariants over time. |
| Field | `field` | TODO | TODO | TODO | TODO | |
| Form | `form` | TODO | TODO | TODO | TODO | |
| Forms | `forms` | TODO | TODO | TODO | TODO | |
| Hover Card | `hover_card` | `tools/diag-scripts/ui-gallery-tooltip-hovercard-scroll-clamp.json` | TODO | Overlay scroll clamp | In progress | Add hover-intent + focus behavior gates. |
| Input | `input` | TODO | TODO | TODO | TODO | |
| Input Group | `input_group` | `tools/diag-scripts/ui-gallery-input-group-text-non-overlap.json` | TODO | Fonts/layout (leading vs trailing non-overlap) | In progress | Add control bounds + min-size checks. |
| Input OTP | `input_otp` | TODO | TODO | TODO | TODO | |
| Item | `item` | TODO | TODO | TODO | TODO | |
| Kbd | `kbd` | TODO | TODO | TODO | TODO | |
| Label | `label` | TODO | TODO | TODO | TODO | |
| Menubar | `menubar` | `tools/diag-scripts/ui-gallery-menubar-keyboard-nav.json` | TODO | Keyboard navigation | In progress | Add open/close + focus return gates. |
| Menus | `menus` | TODO | TODO | TODO | TODO | |
| Native Select | `native_select` | TODO | TODO | TODO | TODO | |
| Navigation Menu | `navigation_menu` | TODO | TODO | TODO | TODO | |
| Pagination | `pagination` | TODO | TODO | TODO | TODO | |
| Popover | `popover` | `tools/diag-scripts/ui-gallery-popover-click-through-outside-press-focus-underlay.json`, `tools/diag-scripts/ui-gallery-popover-dialog-escape-underlay.json` | TODO | Click-through + outside press + focus | In progress | Add placement bounds gate. |
| Progress | `progress` | TODO | TODO | TODO | TODO | |
| Radio Group | `radio_group` | TODO | TODO | TODO | TODO | |
| Resizable | `resizable` | TODO | TODO | TODO | TODO | |
| Scroll Area | `scroll_area` | TODO | TODO | TODO | TODO | |
| Select | `select` | `fretboard diag suite ui-gallery-select` (runs `tools/diag-scripts/ui-gallery-select-*.json`) | TODO | Open/close, commit, disabled, bounds | In progress | Add click-through + focus trap variants if needed. |
| Separator | `separator` | TODO | TODO | TODO | TODO | |
| Sheet | `sheet` | TODO | TODO | TODO | TODO | |
| Sidebar | `sidebar` | TODO | TODO | TODO | TODO | |
| Skeleton | `skeleton` | TODO | TODO | TODO | TODO | |
| Slider | `slider` | `tools/diag-scripts/ui-gallery-slider-set-value.json` | TODO | Interaction routing | In progress | Add keyboard + bounds invariants. |
| Sonner | `sonner` | TODO | TODO | TODO | TODO | |
| Spinner | `spinner` | TODO | TODO | TODO | TODO | |
| Switch | `switch` | TODO | TODO | TODO | TODO | |
| Table | `table` | TODO | TODO | TODO | TODO | |
| Tabs | `tabs` | TODO | TODO | TODO | TODO | |
| Textarea | `textarea` | TODO | TODO | TODO | TODO | |
| Toast | `toast` | TODO | TODO | TODO | TODO | |
| Toggle | `toggle` | TODO | TODO | TODO | TODO | |
| Toggle Group | `toggle_group` | TODO | TODO | TODO | TODO | |
| Tooltip | `tooltip` | `tools/diag-scripts/ui-gallery-tooltip-repeat-hover.json` | TODO | Hover repeat stability | In progress | Add dismiss + no-click-through assertions. |
| Typography | `typography` | TODO | TODO | TODO | TODO | |
