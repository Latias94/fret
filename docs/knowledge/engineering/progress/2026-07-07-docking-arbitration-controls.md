---
type: "Work Progress"
title: "Docking arbitration controls binding"
description: "Work Progress for the docking arbitration raw model controls cleanup."
timestamp: 2026-07-07T06:30:00Z
tags: ["fret", "examples", "docking", "public-surface", "raw-model", "controls"]
git_branch: "refactor/docking-arbitration-controls"
verified_by: "cargo nextest run -p fret-examples --test docking_arbitration_surface docking_arbitration_demo_model_writes_stay_behind_controls_binding --no-fail-fast"
---

# Summary

`docking_arbitration_demo.rs` now keeps its overlay/control diagnostic models behind
`DockingArbitrationControls` instead of exposing a raw `DockingArbitrationPanelModels` service plus
a separate `ViewportDebugService` raw model map.

# Details

- Replaced `DockingArbitrationPanelModelsService` with `DockingArbitrationControlsService`.
- Moved the five startup model allocations behind `DockingArbitrationControls::new(...)`.
- Added `DockingArbitrationControlsSnapshot` so render observes and reads diagnostic state through
  one named binding.
- Routed drop-mask toggles, synthetic pointer debug writes, and viewport-input diagnostic writes
  through semantic controls methods.
- Deleted the old `DockingArbitrationModelOwner`; `DockingArbitrationControls` now owns the
  demo-local `ModelStore` writes directly.
- Deleted `ViewportDebugService`; viewport input now resolves the same controls binding used by the
  controls panel.
- Strengthened the behavior test so it starts from `build_ui(...)`, retrieves the registered
  controls service, drives the host-facing drop-mask wrapper, and exercises the production synth
  debug and viewport-input paths.
- Fixed a pre-existing synthetic pointer edge case in the same harness: disabling a pressed
  synthetic pointer now lets pending release events pass the dispatch guard.
- Strengthened the source gate to catch simple `models_mut()` aliases that call
  `update(...)`/`update_any(...)` outside the controls binding.

# Verification

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test docking_arbitration_surface docking_arbitration_demo_model_writes_stay_behind_controls_binding --no-fail-fast`
  failed on the old source because `DockingArbitrationControls` did not exist.
- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test docking_arbitration_surface docking_arbitration_demo_model_writes_stay_behind_controls_binding --no-fail-fast`
- `cargo nextest run -p fret-examples docking_arbitration_controls_service_preserves_diagnostic_state_transitions --no-fail-fast`

# Next Action

Apply the same bundle-first rule to remaining advanced demos where an app-global service exposes a
set of raw runtime models that represent one local control surface.

Also follow up on one pre-existing docking arbitration design risk found during review: the
per-window drop-mask readout model can diverge from the global docking policy flag when multiple
windows expose the controls panel. A future docking diagnostics cleanup should make this state
single-source before broadening multi-window arbitration tests.

# Citations

- [docking_arbitration_demo.rs](../../../../apps/fret-examples/src/docking_arbitration_demo.rs)
- [docking_arbitration_surface.rs](../../../../apps/fret-examples/tests/docking_arbitration_surface.rs)
