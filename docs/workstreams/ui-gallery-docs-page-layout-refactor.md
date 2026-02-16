# UI Gallery Docs-Style Page Layout — Refactor Tracker

Status: In progress

This document tracks the ongoing refactor of `apps/fret-ui-gallery` component pages toward a **shadcn-docs-like**
single-page layout:

- A page is composed of **sections** (Preview + optional Code + short text explanation).
- Section titles default to **single-line + ellipsis** (avoid narrow-window character wrapping surprises).
- Code samples use `fret-ui-ai`'s `CodeBlock` (copyable, language-labeled).
- Track whether the **Preview examples** follow the upstream shadcn docs headings/order, and whether the page provides
  an **API reference pointer** (typically in the top-level Notes tab).

Non-goals:

- This tracker is not about component behavior parity (that lives in shadcn/Base UI alignment workstreams).
- This tracker does not attempt to unify the top-level UI Gallery “Preview/Usage/Notes” tabs in `content.rs`.

## Definition of Done (per component page)

Mark a page “Docs-style” when all are true:

1. Uses `doc_layout::render_doc_page` + `DocSection` for the page body.
2. Every section has at least **one line** of explanation text (either in the section or in a Notes section).
3. Key sections include a minimal code sample (does not need to cover every variant).
4. Keeps critical `test_id`s stable; if a change is unavoidable, update corresponding diag scripts.
5. The table row includes a non-empty **Diag Coverage** entry (either a glob/prefix or a specific script file).
6. The table row includes a non-empty **Docs Parity** entry (Preview examples order + API reference pointer status).

## Progress Table

Legend:

- **Docs-style**: migrated to `DocSection` sections.
- **Legacy tabs**: still uses `render_component_page_tabs` (Component / Code / Notes).
- **Docs Parity**: `Examples ✅/Partial/❌` indicates whether Preview sections match upstream shadcn docs headings/order;
  `API ✅/❌` indicates whether Notes/Usage provide an API reference pointer (upstream link and/or in-tree anchors).

### Shadcn/forms

| Component | Entry point | Layout | Docs Parity | Section text | Code samples | Diag Coverage | Notes |
|---|---|---|---|---|---|---|---|
| Select | `apps/fret-ui-gallery/src/ui/previews/gallery/forms/select.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Per-section | `tools/diag-scripts/ui-gallery-select-*.json` (16) | Keeps existing diag `test_id`s for trigger/items. |
| Combobox | `apps/fret-ui-gallery/src/ui/pages/combobox.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-combobox-*.json` (10) | `apps/fret-ui-gallery/src/ui/pages/combobox/sections.rs` returns “pure content” (no nested cards). |
| Date Picker | `apps/fret-ui-gallery/src/ui/pages/date_picker.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-date-picker-range-roving-skips-disabled.json` | Keeps range-roving regression script compatible (role/name driven). |
| Field | `apps/fret-ui-gallery/src/ui/pages/field.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | None yet | Keeps existing `ui-gallery-field-*` test IDs for future diag scripts. |
| Input | `apps/fret-ui-gallery/src/ui/pages/input.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-input-ime-tab-suppressed.json` | Keeps `ui-gallery-input-basic` test ID stable for IME routing gates. |
| Input Group | `apps/fret-ui-gallery/src/ui/pages/input_group.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-input-group-text-non-overlap.json` | Keeps `ui-gallery-input-group-text-*` test IDs stable for non-overlap gates. |
| Label | `apps/fret-ui-gallery/src/ui/pages/label.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | None yet | Keeps examples small; relies on single-line section titles with ellipsis. |
| Checkbox | `apps/fret-ui-gallery/src/ui/pages/checkbox.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-checkbox-rtl-and-checked-wrap.json` | Keeps `ui-gallery-checkbox-*` test IDs stable for the wrap/RTL regression script. |
| Native Select | `apps/fret-ui-gallery/src/ui/pages/native_select.rs` | Legacy tabs | Examples ✅ / API ✅ | Partial | Tab-only | None yet | Candidate: clarify platform/native intent vs shadcn Select. |
| Form | `apps/fret-ui-gallery/src/ui/pages/form.rs` | Legacy tabs | Examples Partial / API ✅ | Partial | Tab-only | None yet | Candidate: single “forms doc” page with sections per recipe. |

### Shadcn/overlays

| Component | Entry point | Layout | Docs Parity | Section text | Code samples | Diag Coverage | Notes |
|---|---|---|---|---|---|---|---|
| Alert Dialog | `apps/fret-ui-gallery/src/ui/pages/alert_dialog.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-alert-dialog-*.json` (4) | Notes are a dedicated section; headings are nowrap+ellipsis by default. |
| Dialog | `apps/fret-ui-gallery/src/ui/pages/dialog.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-dialog-*.json` (3) | `docs-order-smoke` targets this page; `escape-focus-restore*` targets the Overlay page dialog widget. |
| Drawer | `apps/fret-ui-gallery/src/ui/pages/drawer.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-drawer-*.json` (4) | Keeps existing `test_id`s for demo + snap-points scenarios. |
| Dropdown Menu | `apps/fret-ui-gallery/src/ui/pages/dropdown_menu.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-dropdown-menu-docs-smoke.json` (1) | Keeps existing `test_id`s for triggers and demo items. |
| Tooltip | `apps/fret-ui-gallery/src/ui/pages/tooltip.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-tooltip-*.json` (3) | `repeat-hover`/`scroll-clamp` target the Overlay page tooltip widget; `docs-smoke` targets this page. |
| Context Menu | `apps/fret-ui-gallery/src/ui/pages/context_menu.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-context-menu-*.json` (5) | Adds a page-level docs smoke; existing overlay scripts still gate right-click/keyboard paths. |
| Hover Card | `apps/fret-ui-gallery/src/ui/pages/hover_card.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-hover-card-docs-smoke.json` (1) | Overlay-level clamp gating lives in `tools/diag-scripts/ui-gallery-tooltip-hovercard-scroll-clamp.json`. |

### Shadcn/navigation + misc

| Component | Entry point | Layout | Docs Parity | Section text | Code samples | Diag Coverage | Notes |
|---|---|---|---|---|---|---|---|
| Breadcrumb | `apps/fret-ui-gallery/src/ui/pages/breadcrumb.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-breadcrumb-*.json` (2) | Keeps existing section-title `test_id`s for single-line heading gates. |
| Toggle | `apps/fret-ui-gallery/src/ui/pages/toggle.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | None yet | Keeps `ui-gallery-toggle-*` test IDs stable for future diag scripts. |
| Toggle Group | `apps/fret-ui-gallery/src/ui/pages/toggle_group.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | None yet | Keeps `ui-gallery-toggle-group-*` test IDs stable for future diag scripts. |
| Typography | `apps/fret-ui-gallery/src/ui/pages/typography.rs` | Docs-style | Examples Partial / API ✅ | Yes | Key sections only | None yet | Candidate: add truncation/wrap sections for narrow windows + RTL. |
| Kbd | `apps/fret-ui-gallery/src/ui/pages/kbd.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-kbd-docs-smoke.json` (1) | Keeps existing `test_id`s for demo/group/button/input-group. |
| Item | `apps/fret-ui-gallery/src/ui/pages/item.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | None yet | Candidate: add truncation/ellipsis examples for narrow sidebars. |
| Collapsible | `apps/fret-ui-gallery/src/ui/pages/collapsible.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-collapsible-*.json` (3) | Keeps `ui-gallery-collapsible-component` + demo/basic trigger/content test IDs for existing diag gates. |
| Aspect Ratio | `apps/fret-ui-gallery/src/ui/pages/aspect_ratio.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | None yet | Keeps `ui-gallery-aspect-ratio-*` test IDs stable for future diag scripts. |
| Alert | `apps/fret-ui-gallery/src/ui/pages/alert.rs` | Legacy tabs | Examples ✅ / API ✅ | Partial | Tab-only | `tools/diag-scripts/ui-gallery-alert-static-bundle.json`, `tools/diag-scripts/ui-gallery-alert-tabs-shared-indicator-pixels-changed-fixed-frame-delta.json` | Candidate: variant matrix sections. |
| Empty | `apps/fret-ui-gallery/src/ui/pages/empty.rs` | Docs-style | Examples ✅ / API ✅ | Yes | Key sections only | `tools/diag-scripts/ui-gallery-empty-docs-smoke.json` (1) | Keeps existing `test_id`s for each recipe. |

## Suggested Migration Order (next)

Prefer pages that are:

1. High-traffic in daily alignment work (overlays + forms).
2. Have active diag scripts (keep regression gates stable).
3. Have obvious section boundaries (state matrices).

Recommended next batch:

1. `Dialog`
2. `Dropdown Menu`
3. `Tooltip`
4. `Field`
5. `Checkbox`
