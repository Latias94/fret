# ImUi Control Chrome Fearless Refactor v1 - Final Status

Status: closed closeout note
Last updated: 2026-04-20

Related:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-14.md`
- `M1_IMUI_VS_IMGUI_COMPONENT_AUDIT_2026-04-14.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `ecosystem/fret-ui-kit/src/imui/control_chrome.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_model_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/bullet_text_controls.rs`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`

## Decision

This lane closes as the closeout record for the shared IMUI control-chrome rewrite:

- `ecosystem/fret-ui-kit::imui` now has one shared control-chrome owner for the migrated immediate
  control families.
- Button-like, switch/toggle-like, slider, combo, combo-model, and text-entry helpers now ship on
  top of that shared owner instead of teaching text-like default interactive surfaces.
- The first Dear ImGui catch-up slice that still fits the shared owner is landed:
  `small_button`, `arrow_button`, `invisible_button`, `radio`, and `bullet_text`.
- The compact showcase proof no longer depends on the old demo-level fixed-width rail workaround;
  the shared surface now proves itself through elastic compact rail constraints plus focused source
  and diag gates.

The shipped default surface remains inside `ecosystem/fret-ui-kit::imui` and does not widen
`fret-authoring::Response`, `crates/fret-ui`, or the mechanism-layer runtime contracts.

## Proof left behind

- Focused runtime / interaction floor:
  - `cargo nextest run -p fret-imui`
  - `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_adapter_seam_smoke --test imui_combo_smoke`
  - `cargo nextest run -p fret-ui-kit --features imui --test imui_bullet_text_smoke --test imui_separator_text_smoke --test imui_button_smoke`
- Showcase / source proof:
  - `python tools/gate_imui_facade_teaching_source.py`
  - `python tools/gate_imui_shadcn_adapter_control_discoverability_source.py`
- Launched diag proof:
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-interaction-showcase-layout-compact-screenshot.json --launch -- cargo run -p fret-demo --bin imui_interaction_showcase_demo --release`
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-interaction-showcase-compact-shell-smoke.json --launch -- cargo run -p fret-demo --bin imui_interaction_showcase_demo --release`
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-control-discoverability.json --launch -- cargo run -p fret-demo --bin imui_shadcn_adapter_demo --release`

The final compact-rail re-audit evidence is the important last-closeout proof:

- the old workaround-era compact bundle kept the shell/lab cards near the historical `320px`
  rail,
- the new compact layout/bundle evidence shows the compact rail hitting the shared elastic cap
  instead,
- and the compact-shell smoke remained green after the rail stopped being fixed-width.

## Residual gap routing

Do not reopen this lane for generic IMUI parity drift.

If new pressure appears, route it by owner:

- shared field-width / compact-rail policy hardening beyond the current proof should move to a new
  narrow field-policy follow-on,
- checkbox/selectable/disclosure family parity should move to a family-specific follow-on instead
  of widening this shared chrome closeout,
- menu/tab trigger policy, key ownership, and shell-level composition concerns remain owned by
  their separate closed or active lanes.
