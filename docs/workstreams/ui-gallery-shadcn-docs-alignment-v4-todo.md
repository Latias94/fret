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
- [ ] alert-dialog
- [ ] aspect-ratio
- [ ] avatar
- [~] badge — examples mirrored; missing upstream variants (`apps/fret-ui-gallery/src/ui.rs::preview_badge`)
- [ ] breadcrumb
- [~] button (page examples re-ordered + expanded; still validating interactions)
- [x] button-group — examples mirrored (`apps/fret-ui-gallery/src/ui.rs::preview_button_group`)
- [~] calendar — chrome/layout audited; example order TBD (`apps/fret-ui-gallery/src/ui.rs::preview_calendar`)
- [x] card — examples mirrored + size/action slots (`apps/fret-ui-gallery/src/ui.rs::preview_card`, `ecosystem/fret-ui-shadcn/src/card.rs`)
- [ ] carousel
- [ ] chart
- [ ] checkbox
- [ ] collapsible
- [ ] combobox
- [ ] command
- [ ] context-menu
- [ ] data-table
- [ ] date-picker
- [ ] dialog
- [ ] direction
- [ ] drawer
- [ ] dropdown-menu
- [ ] empty
- [ ] field
- [ ] form
- [ ] hover-card
- [ ] input
- [ ] input-group
- [ ] input-otp
- [ ] item
- [ ] kbd
- [ ] label
- [ ] menubar
- [ ] native-select
- [ ] navigation-menu
- [ ] pagination
- [ ] popover
- [ ] progress
- [ ] radio-group
- [ ] resizable
- [ ] scroll-area
- [~] select (group semantics + default alignment audited; needs full docs example order)
- [ ] separator
- [ ] sheet
- [ ] sidebar
- [ ] skeleton
- [ ] slider
- [ ] sonner
- [ ] spinner
- [ ] switch
- [ ] table
- [ ] tabs
- [ ] textarea
- [ ] toast
- [ ] toggle
- [ ] toggle-group
- [ ] tooltip
- [ ] typography

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
