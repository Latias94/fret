# Sidebar Device-Shell Vocabulary Alignment v1

Status: Closed historical design note
Last updated: 2026-04-20

Status note (2026-04-20): this document remains useful for the lane-opening rationale, but the
shipped verdict now lives in `CLOSEOUT_AUDIT_2026-04-20.md` and `WORKSTREAM.json`. Read the
execution framing below as the historical setup that led to the landed rename.

Related:

- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `CLOSEOUT_AUDIT_2026-04-20.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `docs/workstreams/adaptive-presentation-surface-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/device-shell-recipe-wrapper-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `ecosystem/fret-ui-kit/src/adaptive.rs`
- `ecosystem/fret-ui-shadcn/src/{adaptive_shell.rs,sidebar.rs}`
- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`

This workstream is a narrow follow-on to the already-closed adaptive presentation lanes.

It does not reopen the owner split:

- `Sidebar` remains an app-shell surface,
- editor rails and inspector sidebars remain separate container-aware work,
- and the shared helper owner remains `fret-ui-kit` / `fret::adaptive`.

The narrow problem is vocabulary drift.

Earlier closeouts intentionally kept `SidebarProvider` on the app-shell lane, but the live shipped
surface still exposed ad-hoc `is_mobile(...)` / `is_mobile_breakpoint(...)` naming even after the
repo had already frozen shared `DeviceShellMode` / `DeviceShellSwitchPolicy` vocabulary.

That left the repo in an awkward state:

- the mechanism and strategy owner were explicit,
- the app-shell boundary was explicit,
- but the sidebar provider/context still taught a private boolean vocabulary that no longer matched
  the rest of the adaptive surface.

## Must-be-true outcomes

1. `Sidebar` stays app-shell-owned rather than becoming a generic editor/panel adaptive answer.
2. The public sidebar provider surface uses the same device-shell nouns as the shared helper owner.
3. `use_sidebar(cx)` consumers read the resolved shell branch through the same shared vocabulary.
4. Gallery/docs and source-policy tests stop teaching the old boolean names.
5. The rename does not widen generic wrapper growth or move policy into `crates/fret-ui`.

## In scope

- Rename `SidebarProvider::is_mobile(...)` to `SidebarProvider::device_shell_mode(...)`.
- Rename `SidebarProvider::is_mobile_breakpoint(...)` to
  `SidebarProvider::device_shell_switch_policy(...)`.
- Replace `SidebarContext::is_mobile` with `SidebarContext::device_shell_mode`.
- Keep the internal shadcn adaptive seam crate-private while allowing sidebar to resolve a full
  `DeviceShellMode`.
- Update UI Gallery snippets/pages/tests and ADR alignment notes.

## Out of scope

- New editor-rail or inspector-sidebar public surfaces.
- A new generic adaptive presentation manager.
- Renaming `open_mobile` / `width_mobile`, which are still concrete sidebar app-shell state lanes.
- Reopening the closed `Combobox` wrapper-vs-helper decision.

## Owner split

### `ecosystem/fret-ui-kit`

Owns the shared adaptive nouns and switch policy:

- `DeviceShellMode`
- `DeviceShellSwitchPolicy`
- `device_shell_mode(...)`

### `ecosystem/fret-ui-shadcn`

Owns the sidebar-family provider/context authoring surface and the crate-private adaptive seam.

### UI Gallery + docs/tests

Own the first-party teaching surface and the source-policy proof that sidebar remains app-shell-only
even after the vocabulary alignment.

## Target shipped state

The repo should read coherently at every layer:

- shared adaptive lane: `DeviceShellMode` / `DeviceShellSwitchPolicy`
- sidebar provider lane: `device_shell_mode(...)` / `device_shell_switch_policy(...)`
- sidebar context lane: `device_shell_mode`
- docs language: still app-shell/device-shell only, explicitly not panel/container adaptive

That shipped state is recorded in `CLOSEOUT_AUDIT_2026-04-20.md`.
