# M1 Contract Freeze — 2026-04-11

Status: decision record for the active lane

## Decision

The `Dialog` / `Drawer` family does **not** justify a new shared adaptive-presentation helper yet.

Current v1 verdict:

- keep the responsive dialog example as an explicit dialog/drawer proof pairing,
- keep `device_shell_switch(...)` visible at call sites where the app/gallery layer still owns the
  semantic pairing,
- keep `Combobox` as the only current recipe-owned wrapper exemplar,
- and keep editor-rail mobile downgrade outside the dialog/drawer family entirely.

## Evidence behind the decision

### 1) The dialog/drawer proof surface is still singular

The repo has one intentional docs/golden-path pairing for this family:

- `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
- `apps/fret-ui-gallery/src/ui/pages/drawer.rs`

That surface exists to keep the desktop `Dialog` branch and mobile `Drawer` branch reviewable side
by side.

It does not yet prove repeated same-family extraction pressure.

### 2) Other device-shell pairings are not the same family

Current explicit device-shell pairings elsewhere in the repo are:

- `Date Picker`: `Popover` vs `Drawer`
- `Breadcrumb`: `DropdownMenu` vs `Drawer`
- editor notes shell: `WorkspaceFrame` rails vs `Drawer`

These surfaces share the existence of device-shell branching, but they do **not** share the same
presentation family, lifecycle policy, or owner layer.

So they are not valid evidence for a new `Dialog` / `Drawer` family helper.

### 3) Current helper thresholds are not met

The current threshold from `TARGET_INTERFACE_STATE.md` requires:

1. at least two real consumers with the same adaptive decision shape,
2. a shared part more specific than generic `device_shell_switch(...)`,
3. the same owner layer,
4. materially aligned lifecycle policy,
5. and a source gate plus behavior proof for the proposed shared shape.

The dialog/drawer family currently fails at least the first four conditions.

## Shipped contract freeze

Read the current upper-interface contract this way:

- `Dialog` / `Drawer` responsive pairing remains app/gallery composition.
- `device_shell_switch(...)` remains the shared generic strategy helper.
- recipe-owned wrappers remain family-specific and opt-in only after repeated same-family proof.
- `SidebarProvider::is_mobile(...)` remains app-shell-only vocabulary.
- editor-rail downgrade remains outer-shell-owned.

## Reopen criteria

Open a narrower follow-on only if future evidence shows all of the following:

1. a second real `Dialog` / `Drawer` family consumer with the same semantic pairing,
2. shared lifecycle policy beyond generic shell switching,
3. stable naming that is narrower than a generic adaptive manager,
4. and one focused source gate plus one behavior proof for the extracted shape.

Until then, explicit branching is the correct v1 answer.
