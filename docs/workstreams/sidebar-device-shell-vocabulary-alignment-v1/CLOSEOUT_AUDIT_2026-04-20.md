# Closeout Audit — 2026-04-20

Status: closed closeout record

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `docs/workstreams/adaptive-presentation-surface-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/device-shell-recipe-wrapper-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `ecosystem/fret-ui-kit/src/adaptive.rs`
- `ecosystem/fret-ui-shadcn/src/{adaptive_shell.rs,sidebar.rs}`
- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
- `apps/fret-ui-gallery/tests/{device_shell_recipe_wrapper_surface.rs,device_shell_strategy_surface.rs,sidebar_docs_surface.rs}`

## Verdict

This lane is now closed.

The correct sidebar app-shell surface is not the old boolean vocabulary.

The repo should keep sidebar on the app-shell lane, but it should name that lane with the same
device-shell nouns already used by the shared adaptive owner.

## What shipped

### 1) `SidebarProvider` now speaks shared device-shell vocabulary

`ecosystem/fret-ui-shadcn/src/sidebar.rs` now exposes:

- `SidebarProvider::device_shell_mode(DeviceShellMode)`
- `SidebarProvider::device_shell_switch_policy(DeviceShellSwitchPolicy)`

This keeps the sidebar family app-shell-owned while removing the ad-hoc `is_mobile(...)` /
`is_mobile_breakpoint(...)` vocabulary.

### 2) `SidebarContext` now exposes `device_shell_mode`

`use_sidebar(cx)` consumers no longer read `ctx.is_mobile`.

They now read:

- `ctx.device_shell_mode.is_mobile()`
- `ctx.device_shell_mode.is_desktop()`

That keeps provider and consumer code on one explicit device-shell vocabulary.

### 3) The private shadcn adaptive seam stayed narrow

`ecosystem/fret-ui-shadcn/src/adaptive_shell.rs` remains crate-private.

It now exposes a crate-private `resolve_device_shell_mode(...)` helper for sidebar while retaining
the existing boolean `is_desktop_shell(...)` helper for drawer's internal parity logic.

This avoids duplicating shared helper logic locally without promoting a second public strategy
owner.

### 4) Gallery/docs and source-policy tests were updated together

The sidebar docs page, snippets, and source-policy gates now teach:

- `SidebarProvider::device_shell_mode(...)`
- `SidebarProvider::device_shell_switch_policy(...)`
- `DeviceShellMode::Mobile`

while still stating that sidebar is an app-shell/device-shell surface, not a generic
panel/container adaptive helper.

## Gates that define the shipped surface

- `cargo nextest run -p fret-ui-gallery --test device_shell_recipe_wrapper_surface --test device_shell_strategy_surface --test sidebar_docs_surface --no-fail-fast`
- `cargo nextest run -p fret-ui-shadcn --lib sidebar_provider_custom_device_shell_policy_can_force_mobile_sheet_branch --no-fail-fast`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/sidebar-device-shell-vocabulary-alignment-v1/WORKSTREAM.json > /dev/null`
- `git diff --check`

## Remaining gaps after closeout

This lane intentionally does not solve:

- stronger fixed-window panel-resize proof for container-query teaching,
- a fuller public adaptive authoring story above the current strategy vocabulary,
- or any new editor-rail / inspector-sidebar extraction.

Those remain separate follow-ons if future evidence justifies them.
