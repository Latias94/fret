# M2 First Extraction — 2026-04-11

Status: historical landing note

Status note (2026-04-11): this file remains the first extraction record. The current shipped lane
state lives in `M3_SECOND_CONSUMER_PROOF_2026-04-11.md` and `CLOSEOUT_AUDIT_2026-04-11.md`.
References below to `Breadcrumb` staying raw should be read as historical landing-state evidence.

Related:

- `TARGET_INTERFACE_STATE.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

## Verdict

M2 now has a landed first extraction.

This slice adds a crate-local device-shell switch helper in `fret-ui-kit` and proves it on one
gallery consumer without reopening broader adaptive scope.

The landed result is:

1. `ecosystem/fret-ui-kit/src/adaptive.rs` now exposes:
   - `DeviceShellMode`
   - `DeviceShellSwitchPolicy`
   - `device_shell_mode(...)`
   - `device_shell_switch(...)`
2. `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs` now uses
   `device_shell_switch(...)` instead of a raw `viewport_width_at_least(...)` branch.
3. The `Date Picker` snippet still keeps both explicit branch bodies visible:
   - desktop `Popover`
   - mobile `Drawer`
4. `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs` remains the raw
   `DropdownMenu` vs `Drawer` proof surface for this lane.
5. `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs` remains the explicit
   `Dialog` vs `Drawer` proof surface.

## What this proves

- the first shared binary device-shell helper can live in `fret-ui-kit` without facade promotion,
- app-facing code can use the helper while keeping recipe policy and branch content explicit,
- and the lane can standardize one repeated branch shape without widening generic `children(...)`
  or touching `Sidebar` ownership.

## What this does not reopen

- `fret::adaptive` facade promotion
- recipe-owned wrappers beyond the already explicit `Combobox` surface
- panel/container adaptive helpers
- editor rail / sidebar ownership
- runtime mechanism changes in `crates/fret-ui`
