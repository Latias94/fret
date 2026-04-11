# Closeout Audit — 2026-04-11

Status: closed closeout record

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `M0_BRANCH_SITE_AUDIT_2026-04-11.md`
- `M1_CONTRACT_FREEZE_2026-04-11.md`
- `M2_FIRST_EXTRACTION_2026-04-11.md`
- `M3_SECOND_CONSUMER_PROOF_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- `ecosystem/fret-ui-kit/src/adaptive.rs`
- `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
- `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
- `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`

## Verdict

This lane is now closed.

It answered the narrow follow-on question left by the adaptive-layout closeout:

- the shared higher-level desktop/mobile shell strategy surface should live in
  `fret-ui-kit::adaptive`,
- it should stay explicitly named `device_shell_*`,
- it should remain crate-local first instead of promoting immediately into `fret::adaptive`,
- and it should only choose the desktop/mobile branch while leaving recipe policy visible at the
  call site.

## What shipped

### 1) `fret-ui-kit` now owns the shared binary helper surface

`ecosystem/fret-ui-kit/src/adaptive.rs` now ships:

- `DeviceShellMode`
- `DeviceShellSwitchPolicy`
- `device_shell_mode(...)`
- `device_shell_switch(...)`

This keeps shared device-shell branching above raw viewport queries without pushing policy into
`crates/fret-ui` or prematurely widening the `fret` facade.

### 2) The helper is proven on two real app-facing consumers

The helper now has two materially different gallery consumers:

- `Date Picker`: desktop `Popover` vs mobile `Drawer`
- `Breadcrumb`: desktop `DropdownMenu` vs mobile `Drawer`

That is enough proof for this lane's narrow owner/naming question.

### 3) Explicit non-helper boundaries stayed explicit

The lane intentionally keeps three important boundaries visible:

- `Dialog` vs `Drawer` stays an explicit docs/proof pairing
- `Combobox::device_shell_responsive(...)` stays the recipe-owned wrapper exemplar
- `SidebarProvider::is_mobile(...)` stays the app-shell/device-shell owner boundary

So the lane closes without collapsing every desktop/mobile branch into one generic helper or one
generic wrapper family.

### 4) Facade promotion remains deferred on purpose

The lane proved the helper surface is real, but it did not prove that the `fret` facade should own
it yet.

That promotion remains a separate decision because:

- the helper is still ecosystem-level strategy rather than app-default vocabulary,
- the repo already has one recipe-owned explicit wrapper example (`Combobox`),
- and no evidence in this lane required broadening the default app-facing import surface.

## Gates that define the shipped surface

- `cargo nextest run -p fret-ui-gallery --test device_shell_strategy_surface --test sidebar_docs_surface --no-fail-fast`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app drawer_responsive_dialog_keeps_desktop_dialog_on_composable_content_lane drawer_page_marks_usage_as_default_and_snap_points_as_follow_up remaining_app_facing_tail_snippets_prefer_ui_cx_on_the_default_app_surface date_picker_usage_snippet_stays_on_the_composed_popover_calendar_lane --no-fail-fast`
- `git diff --check`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/device-shell-strategy-surface-v1/WORKSTREAM.json > /dev/null`

## Follow-on policy

Do not reopen this lane for:

- another app-level proof duplicate of the same helper shape,
- immediate facade promotion into `fret::adaptive`,
- recipe-owned wrapper growth above the helper,
- or broader panel/container adaptive work.

If future work is needed, open a narrower follow-on such as:

1. `fret` facade promotion for the shipped helper,
2. a recipe-owned wrapper lane above `device_shell_switch(...)`,
3. or a more capable device-shell policy surface that goes beyond width-only switching.
