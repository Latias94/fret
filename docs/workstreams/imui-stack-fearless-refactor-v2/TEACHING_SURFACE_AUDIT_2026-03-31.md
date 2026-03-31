# imui stack fearless refactor v2 - teaching surface audit

Status: completed
Last updated: 2026-03-31

Related:

- `docs/workstreams/imui-stack-fearless-refactor-v2/DESIGN.md`
- `docs/workstreams/imui-stack-fearless-refactor-v2/TODO.md`
- `docs/workstreams/imui-stack-fearless-refactor-v2/MILESTONES.md`
- `docs/workstreams/imui-stack-fearless-refactor-v2/BASELINE_AUDIT_2026-03-31.md`

## Scope

This audit checks only the current first-party `imui` teaching surfaces that downstream readers are
most likely to copy from today.

Audited sources:

- `apps/fret-examples/src/imui_hello_demo.rs`
- `apps/fret-examples/src/imui_floating_windows_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/imui_node_graph_demo.rs`
- `apps/fret-cookbook/examples/imui_action_basics.rs`

## Findings

### 1. Active first-party examples teach the current facade and adapter entrypoints

The audited examples consistently route immediate authoring through the current public teaching
surface:

- generic immediate vocabulary is taught through `fret_ui_kit::imui::UiWriterImUiFacadeExt`
- shared chrome helpers still use `UiWriterUiKitExt` when a demo needs generic UI-kit composition
- editor-owned immediate nouns are taught through `fret_ui_editor::imui`

This means the examples no longer imply that downstream users should drop below the current facade
surface just to author normal immediate controls.

### 2. Deleted historical names do not appear in active teaching surfaces

The audited sources no longer teach the deleted or superseded historical names that motivated this
lane's cleanup, including:

- `select_model_ex`
- `window_ex`
- `window_open_ex`
- `floating_area_show_ex`
- `begin_disabled`
- built-in sample wrappers such as `button_adapter(...)` and `checkbox_model_adapter(...)`

The active examples also do not teach `fret_ui_kit::imui::adapters` as a direct authoring surface.
That module now stays contract-only and is reserved for external adapter seams and tests.

### 3. Compatibility-oriented exceptions are explicitly labeled as non-default

`apps/fret-examples/src/imui_node_graph_demo.rs` still exists because the retained-bridge node
graph path is useful proof coverage, but the file now clearly states that it is compatibility
oriented and not the default downstream authoring path.

That note matters because it keeps the teaching surface honest without deleting valuable bridge
evidence.

### 4. The surviving owner split is readable in code

After the current audit, the owner map reads cleanly:

- generic immediate helpers: `ecosystem/fret-ui-kit::imui`
- editor-owned immediate adapters: `ecosystem/fret-ui-editor::imui`
- contract-only adapter seam: `ecosystem/fret-ui-kit::imui::adapters`
- retained bridge compatibility proof: `apps/fret-examples/src/imui_node_graph_demo.rs`

No active example currently blurs those owners by teaching deleted helper names or bypassing the
official editor adapter layer where the adapter now exists.

## Regression anchors

Source-policy gates:

- `apps/fret-examples/src/lib.rs`
- `apps/fret-cookbook/src/lib.rs`

Immediate proof anchor:

- `apps/fret-examples/src/imui_editor_proof_demo.rs`

## Conclusion

The active first-party `imui` teaching surface is now aligned with the intended boundary story for
this lane.

What remains for closeout is a final single audit that summarizes the full survive/promote/delete
result of the v2 lane.
