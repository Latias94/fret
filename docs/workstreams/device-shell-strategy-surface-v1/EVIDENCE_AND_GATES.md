# Device-Shell Strategy Surface v1 — Evidence and Gates

Status: Active
Last updated: 2026-04-11

## Smallest current repro

Use this focused source/gate set before proposing any shared device-shell strategy helper:

```bash
cargo nextest run -p fret-ui-gallery --test device_shell_strategy_surface --test sidebar_docs_surface --no-fail-fast
cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app drawer_responsive_dialog_keeps_desktop_dialog_on_composable_content_lane drawer_page_marks_usage_as_default_and_snap_points_as_follow_up remaining_app_facing_tail_snippets_prefer_ui_cx_on_the_default_app_surface date_picker_usage_snippet_stays_on_the_composed_popover_calendar_lane --no-fail-fast
git diff --check
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
```

What this proves now:

- the repo still contains at least one app-level raw viewport branch for `Popover` vs `Drawer`,
- the repo also still contains an app-level raw desktop/mobile branch for `DropdownMenu` vs
  `Drawer`,
- the docs-path `Dialog` vs `Drawer` pairing remains explicit and reviewable,
- combobox already demonstrates one recipe-owned explicit device-shell API shape,
- and sidebar device-shell ownership is already pinned away from the editor-rail story.

## Current evidence set

- `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
  - closed-lane taxonomy baseline; raw reads vs shared adaptive policy lanes are already split.
- `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`
  - follow-on policy baseline that justifies opening this narrower lane instead of reopening the
    broad adaptive closure tracker.
- `docs/workstreams/device-shell-strategy-surface-v1/M0_BRANCH_SITE_AUDIT_2026-04-11.md`
  - records the first owner-split inventory and extraction ranking for this lane.
- `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`
  - focused source gate for current raw branch sites, recipe-owned explicit naming, and app-shell
    boundary evidence.
- `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
  - current raw `viewport_width_at_least(...)` branch for `Popover` vs `Drawer`.
- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
  - current raw `viewport_width_at_least(...)` branch for `DropdownMenu` vs `Drawer`.
- `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
  - current explicit `Dialog` vs `Drawer` pairing proof surface.
- `apps/fret-ui-gallery/src/ui/pages/combobox.rs`
  - current docs-path note that treats `Combobox::device_shell_responsive(true)` as an explicit
    follow-up lane.
- `ecosystem/fret-ui-shadcn/src/combobox.rs`
  - current recipe-owned explicit device-shell API and viewport-driven branch implementation.
- `apps/fret-ui-gallery/src/ui/pages/drawer.rs`
  - current docs-path teaching copy that keeps responsive dialog as explicit desktop/mobile
    pairing rather than a shared helper.
- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
  - current docs-path boundary that keeps sidebar app-shell/device-shell only.
- `apps/fret-ui-gallery/tests/sidebar_docs_surface.rs`
  - focused source test that keeps the sidebar owner split explicit.
- `docs/audits/shadcn-sidebar.md`
  - source-aligned audit showing sidebar's mobile branch is app-shell/device-shell policy rather
    than generic panel adaptation.
- `ecosystem/fret-ui-shadcn/src/sidebar.rs`
  - current provider-owned `is_mobile` / `is_mobile_breakpoint` path.
- `ecosystem/fret-ui-shadcn/src/dialog.rs`
  - current viewport-query-based docs-aligned dialog layout decisions.
- `ecosystem/fret-ui-shadcn/src/sheet.rs`
  - current viewport-query-based sheet sizing policy.

## Active gate set

### Source evidence gate

```bash
cargo nextest run -p fret-ui-gallery --test device_shell_strategy_surface --test sidebar_docs_surface --no-fail-fast
```

### Default-app authoring gate

```bash
cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app drawer_responsive_dialog_keeps_desktop_dialog_on_composable_content_lane drawer_page_marks_usage_as_default_and_snap_points_as_follow_up remaining_app_facing_tail_snippets_prefer_ui_cx_on_the_default_app_surface date_picker_usage_snippet_stays_on_the_composed_popover_calendar_lane --no-fail-fast
```

### Diff hygiene

```bash
git diff --check
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
```

## Next active gap

The next gap is not another broad adaptive audit.

It is one contract decision:

- should Fret expose a shared higher-level device-shell strategy helper/wrapper above raw viewport
  queries for the repeated `Popover` / `DropdownMenu` / `Drawer` family, and if yes, which layer
  should own it?
