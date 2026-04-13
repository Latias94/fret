# ImUi Response Status Lifecycle v1 - Milestones

Status: active execution lane
Last updated: 2026-04-13

## M0 - Baseline and owner freeze

Exit criteria:

- the repo explicitly states why this is a new narrow follow-on instead of a reopened umbrella P0
  backlog,
- the shared `fret-authoring::Response` boundary is frozen as unchanged for this lane,
- and the first-open proof/gate/evidence surfaces are named.

Primary evidence:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-13.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

Current status:

- Closed on 2026-04-13 via `M0_BASELINE_AUDIT_2026-04-13.md`.

## M1 - Lifecycle vocabulary freeze

Exit criteria:

- the first lifecycle quartet is explicit,
- the semantics distinguish click-only versus value-editing controls cleanly,
- and the lane explicitly records what is still deferred.

Primary evidence:

- `DESIGN.md`
- `TODO.md`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`

Current status:

- Closed on 2026-04-13.
- The lane-opening design already narrows the candidate first slice to:
  `activated`, `deactivated`, `edited`, and `deactivated_after_edit`.
- The first shipped rule is now explicit:
  click-only controls keep `edited = false`,
  value-editing controls align `edited` with existing `core.changed` evidence where possible,
  and `deactivated_after_edit` remains tied to the same active session instead of becoming a
  second change signal.
- The explicit defer list is now also frozen:
  key-owner semantics, broader collection proof breadth, menu-bar/submenu trigger policy depth,
  and tab-trigger outward response work remain out of scope unless another narrow follow-on proves
  they should move.

## M2 - First implementation slice

Exit criteria:

- `ResponseExt` exposes the shipped first lifecycle vocabulary as facade-only status,
- the first relevant immediate controls report it consistently,
- and one demo plus focused tests keep the semantics executable.

Primary evidence:

- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_model_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-imui/src/tests/models.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`

Current status:

- Closed on 2026-04-13.
- `ResponseExt` now exposes the first lifecycle quartet as facade-only status while
  `fret-authoring::Response` remains unchanged.
- The first landed slice now covers:
  direct pressables, boolean controls, slider, and text entry.
- The current expansion slice now also covers:
  `menu_item_with_options`, `combo_with_options`, and `combo_model_with_options`.
- Current focused proof now includes:
  `shared_and_facade_response_boundary_compiles`,
  `facade_drag_and_long_press_accessors_compile`,
  `button_lifecycle_edges_follow_press_session`,
  `menu_item_lifecycle_edges_follow_press_session`,
  `checkbox_lifecycle_reports_edit_and_deactivated_after_edit`,
  `combo_lifecycle_tracks_open_session_edges`,
  `combo_model_lifecycle_reports_edit_on_option_pick`,
  and `imui_response_signals_demo_keeps_menu_and_combo_lifecycle_proof`.

## M3 - Expansion or closeout

Exit criteria:

- the lane either closes with a bounded first vocabulary and explicit defer list,
- or splits again because later pressure is really about another owner/problem.

Primary evidence:

- `WORKSTREAM.json`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- future closeout note or follow-on lane docs

Current status:

- In progress.
- The first bounded expansion already landed on public menu/combo response surfaces.
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/` now owns the helper-owned
  menu/submenu/tab trigger response-surface decision.
- Richer menu-bar/submenu policy depth and broader tab policy still remain outside this lane.
