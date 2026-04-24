# ImUi Active Trigger Behavior Kernel v1 TODO

Status: closed execution checklist
Last updated: 2026-04-24

## P0 - Freeze Boundary

- [x] Confirm this is a follow-on to the closed full pressable item kernel lane.
- [x] Identify active-only trigger families:
  - `switch_model_with_options`;
  - menu items;
  - menu triggers;
  - submenu triggers;
  - tab triggers.
- [x] Exclude slider value editing from this kernel.
- [x] Keep menu/tab policy and popup open/close local.

## P1 - Land Private Kernel Slice

- [x] Add `active_trigger_behavior.rs`.
- [x] Migrate switch active/lifecycle/hover response wiring.
- [x] Migrate menu item active/lifecycle/hover response wiring.
- [x] Migrate menu trigger / submenu trigger active/lifecycle/hover response wiring.
- [x] Migrate tab trigger active/lifecycle/hover response wiring.

## P2 - Verify And Decide Closeout

- [x] Run focused `fret-imui` switch/menu/tab gates.
- [x] Run full `fret-ui-kit --features imui` and `fret-imui` gates.
- [x] Run proof demo build.
- [x] Close this lane if no remaining active-only duplication belongs here.
