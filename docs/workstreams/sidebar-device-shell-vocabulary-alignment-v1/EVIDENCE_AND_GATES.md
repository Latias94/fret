# Sidebar Device-Shell Vocabulary Alignment v1 — Evidence and Gates

Status: Closed

## Smallest current repro

Use this sequence to reopen the shipped rename evidence:

```bash
rg -n "pub fn device_shell_mode|pub fn device_shell_switch_policy|pub device_shell_mode: DeviceShellMode" ecosystem/fret-ui-shadcn/src/sidebar.rs
rg -n "SidebarProvider::device_shell_mode|device_shell_switch_policy|DeviceShellMode::Mobile" apps/fret-ui-gallery/src/ui/pages/sidebar.rs apps/fret-ui-gallery/src/ui/snippets/sidebar
cargo nextest run -p fret-ui-gallery --test device_shell_recipe_wrapper_surface --test device_shell_strategy_surface --test sidebar_docs_surface --no-fail-fast
```

What this proves:

- the sidebar provider/context no longer teach the old `is_mobile*` vocabulary,
- the gallery now teaches the shared device-shell nouns directly,
- and the existing adaptive source-policy gates still keep sidebar on the app-shell boundary.

## Gate set

### Source-policy + behavior gates

```bash
cargo nextest run -p fret-ui-gallery --test device_shell_recipe_wrapper_surface --test device_shell_strategy_surface --test sidebar_docs_surface --no-fail-fast
cargo nextest run -p fret-ui-shadcn --lib sidebar_provider_custom_device_shell_policy_can_force_mobile_sheet_branch --no-fail-fast
```

### Lane hygiene

```bash
python3 tools/check_workstream_catalog.py
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
python3 -m json.tool docs/workstreams/sidebar-device-shell-vocabulary-alignment-v1/WORKSTREAM.json > /dev/null
git diff --check
```

## Current evidence after landing

- `ecosystem/fret-ui-shadcn/src/sidebar.rs` now exposes `device_shell_mode(...)` and
  `device_shell_switch_policy(...)` instead of `is_mobile(...)` / `is_mobile_breakpoint(...)`.
- `SidebarContext` now carries `device_shell_mode: DeviceShellMode`.
- `ecosystem/fret-ui-shadcn/src/adaptive_shell.rs` stays crate-private while resolving a full
  `DeviceShellMode` for sidebar.
- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs` and sidebar snippets teach the new vocabulary and
  keep the app-shell-vs-panel boundary explicit.
- `apps/fret-ui-gallery/tests/device_shell_recipe_wrapper_surface.rs`,
  `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`, and
  `apps/fret-ui-gallery/tests/sidebar_docs_surface.rs` pin the updated surface.

## Evidence anchors

- `docs/workstreams/sidebar-device-shell-vocabulary-alignment-v1/DESIGN.md`
- `docs/workstreams/sidebar-device-shell-vocabulary-alignment-v1/TODO.md`
- `docs/workstreams/sidebar-device-shell-vocabulary-alignment-v1/MILESTONES.md`
- `docs/workstreams/sidebar-device-shell-vocabulary-alignment-v1/CLOSEOUT_AUDIT_2026-04-20.md`
- `docs/workstreams/sidebar-device-shell-vocabulary-alignment-v1/WORKSTREAM.json`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `ecosystem/fret-ui-kit/src/adaptive.rs`
- `ecosystem/fret-ui-shadcn/src/adaptive_shell.rs`
- `ecosystem/fret-ui-shadcn/src/sidebar.rs`
- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
- `apps/fret-ui-gallery/src/ui/snippets/sidebar/mobile.rs`
- `apps/fret-ui-gallery/src/ui/snippets/sidebar/use_sidebar.rs`
- `apps/fret-ui-gallery/src/ui/snippets/sidebar/{demo.rs,controlled.rs,structure.rs,app_sidebar.rs,rtl.rs}`
- `apps/fret-ui-gallery/tests/device_shell_recipe_wrapper_surface.rs`
- `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`
- `apps/fret-ui-gallery/tests/sidebar_docs_surface.rs`
