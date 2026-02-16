# UI Gallery Docs-Style Page Layout — Refactor Tracker

Status: In progress

This document tracks the ongoing refactor of `apps/fret-ui-gallery` component pages toward a **shadcn-docs-like**
single-page layout:

- A page is composed of **sections** (Preview + optional Code + short text explanation).
- Section titles default to **single-line + ellipsis** (avoid narrow-window character wrapping surprises).
- Code samples use `fret-ui-ai`'s `CodeBlock` (copyable, language-labeled).

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

## Progress Table

Legend:

- **Docs-style**: migrated to `DocSection` sections.
- **Legacy tabs**: still uses `render_component_page_tabs` (Component / Code / Notes).

### Shadcn/forms

| Component | Entry point | Layout | Section text | Code samples | Diag Coverage | Notes |
|---|---|---|---|---|---|---|
| Select | `apps/fret-ui-gallery/src/ui/previews/gallery/forms/select.rs` | Docs-style | Yes | Per-section | `tools/diag-scripts/ui-gallery-select-*.json` (16) | Keeps existing diag `test_id`s for trigger/items. |
| Combobox | `apps/fret-ui-gallery/src/ui/pages/combobox.rs` | Docs-style | Yes | Key sections only | `tools/diag-scripts/ui-gallery-combobox-*.json` (10) | `apps/fret-ui-gallery/src/ui/pages/combobox/sections.rs` returns “pure content” (no nested cards). |
| Date Picker | `apps/fret-ui-gallery/src/ui/pages/date_picker.rs` | Docs-style | Yes | Key sections only | `tools/diag-scripts/ui-gallery-date-picker-range-roving-skips-disabled.json` | Keeps range-roving regression script compatible (role/name driven). |
| Field | `apps/fret-ui-gallery/src/ui/pages/field.rs` | Legacy tabs | Partial | Tab-only | None yet | Candidate: convert to sections; keep `Field` examples grouped by recipe. |
| Input | `apps/fret-ui-gallery/src/ui/pages/input.rs` | Legacy tabs | Partial | Tab-only | `tools/diag-scripts/ui-gallery-input-*.json` (2) | Candidate: split by size/disabled/invalid/password. |
| Input Group | `apps/fret-ui-gallery/src/ui/pages/input_group.rs` | Legacy tabs | Partial | Tab-only | `tools/diag-scripts/ui-gallery-input-group-*.json` (1) | Candidate: sections per composition recipe. |
| Label | `apps/fret-ui-gallery/src/ui/pages/label.rs` | Legacy tabs | Partial | Tab-only | None yet | Candidate: keep examples small; rely on ellipsis title defaults. |
| Checkbox | `apps/fret-ui-gallery/src/ui/pages/checkbox.rs` | Legacy tabs | Partial | Tab-only | `tools/diag-scripts/ui-gallery-checkbox-rtl-and-checked-wrap.json` | Candidate: align with shadcn docs matrix (checked/indeterminate/disabled). |
| Native Select | `apps/fret-ui-gallery/src/ui/pages/native_select.rs` | Legacy tabs | Partial | Tab-only | None yet | Candidate: clarify platform/native intent vs shadcn Select. |
| Form | `apps/fret-ui-gallery/src/ui/pages/form.rs` | Legacy tabs | Partial | Tab-only | None yet | Candidate: single “forms doc” page with sections per recipe. |

### Shadcn/overlays

| Component | Entry point | Layout | Section text | Code samples | Diag Coverage | Notes |
|---|---|---|---|---|---|---|
| Alert Dialog | `apps/fret-ui-gallery/src/ui/pages/alert_dialog.rs` | Docs-style | Yes | Key sections only | `tools/diag-scripts/ui-gallery-alert-dialog-*.json` (4) | Notes are a dedicated section; headings are nowrap+ellipsis by default. |
| Dialog | `apps/fret-ui-gallery/src/ui/pages/dialog.rs` | Docs-style | Yes | Key sections only | `tools/diag-scripts/ui-gallery-dialog-*.json` (3) | `docs-order-smoke` targets this page; `escape-focus-restore*` targets the Overlay page dialog widget. |
| Drawer | `apps/fret-ui-gallery/src/ui/pages/drawer.rs` | Docs-style | Yes | Key sections only | `tools/diag-scripts/ui-gallery-drawer-*.json` (4) | Keeps existing `test_id`s for demo + snap-points scenarios. |
| Dropdown Menu | `apps/fret-ui-gallery/src/ui/pages/dropdown_menu.rs` | Docs-style | Yes | Key sections only | `tools/diag-scripts/ui-gallery-dropdown-menu-docs-smoke.json` (1) | Keeps existing `test_id`s for triggers and demo items. |
| Tooltip | `apps/fret-ui-gallery/src/ui/pages/tooltip.rs` | Docs-style | Yes | Key sections only | `tools/diag-scripts/ui-gallery-tooltip-*.json` (3) | `repeat-hover`/`scroll-clamp` target the Overlay page tooltip widget; `docs-smoke` targets this page. |
| Context Menu | `apps/fret-ui-gallery/src/ui/pages/context_menu.rs` | Docs-style | Yes | Key sections only | `tools/diag-scripts/ui-gallery-context-menu-*.json` (5) | Adds a page-level docs smoke; existing overlay scripts still gate right-click/keyboard paths. |
| Hover Card | `apps/fret-ui-gallery/src/ui/pages/hover_card.rs` | Docs-style | Yes | Key sections only | `tools/diag-scripts/ui-gallery-hover-card-docs-smoke.json` (1) | Overlay-level clamp gating lives in `tools/diag-scripts/ui-gallery-tooltip-hovercard-scroll-clamp.json`. |

### Shadcn/navigation + misc

| Component | Entry point | Layout | Section text | Code samples | Diag Coverage | Notes |
|---|---|---|---|---|---|---|
| Breadcrumb | `apps/fret-ui-gallery/src/ui/pages/breadcrumb.rs` | Legacy tabs | Partial | Tab-only | `tools/diag-scripts/ui-gallery-breadcrumb-*.json` (2) | Candidate: small surface; easy migration. |
| Toggle | `apps/fret-ui-gallery/src/ui/pages/toggle.rs` | Legacy tabs | Partial | Tab-only | None yet | Candidate: state matrix sections. |
| Toggle Group | `apps/fret-ui-gallery/src/ui/pages/toggle_group.rs` | Legacy tabs | Partial | Tab-only | None yet | Candidate: single/multi + orientation. |
| Typography | `apps/fret-ui-gallery/src/ui/pages/typography.rs` | Legacy tabs | Partial | Tab-only | None yet | Candidate: sections per token preset + truncation/wrap behavior. |
| Kbd | `apps/fret-ui-gallery/src/ui/pages/kbd.rs` | Legacy tabs | Partial | Tab-only | None yet | Candidate: short page; easy migration. |
| Item | `apps/fret-ui-gallery/src/ui/pages/item.rs` | Legacy tabs | Partial | Tab-only | None yet | Candidate: list item patterns + icons + truncation. |
| Collapsible | `apps/fret-ui-gallery/src/ui/pages/collapsible.rs` | Legacy tabs | Partial | Tab-only | `tools/diag-scripts/ui-gallery-collapsible-*.json` (3) | Candidate: accordion/collapsible behavior notes. |
| Aspect Ratio | `apps/fret-ui-gallery/src/ui/pages/aspect_ratio.rs` | Legacy tabs | Partial | Tab-only | None yet | Candidate: minimal surface. |
| Alert | `apps/fret-ui-gallery/src/ui/pages/alert.rs` | Legacy tabs | Partial | Tab-only | `tools/diag-scripts/ui-gallery-alert-static-bundle.json`, `tools/diag-scripts/ui-gallery-alert-tabs-shared-indicator-pixels-changed-fixed-frame-delta.json` | Candidate: variant matrix sections. |
| Empty | `apps/fret-ui-gallery/src/ui/pages/empty.rs` | Legacy tabs | Partial | Tab-only | None yet | Candidate: quick conversion. |

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
