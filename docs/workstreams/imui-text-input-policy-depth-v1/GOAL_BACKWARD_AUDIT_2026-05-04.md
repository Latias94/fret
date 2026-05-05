# ImUi Goal-Backward Audit - 2026-05-04

Status: current slice audit.

This note checks the broader "make IMUI usable and move toward Dear ImGui-class capability"
objective against the concrete work landed in `imui-text-input-policy-depth-v1`.

## Verdict

This slice makes the app-facing IMUI lane more usable, but it does not close Dear ImGui parity as a
whole.

The correct architecture is still:

- `fret-imui`: thin immediate authoring frontend and ID/layout mount primitives.
- `fret-ui-kit::imui`: generic imgui-like widget policy and response helpers.
- `fret-ui-editor::imui`: editor-grade widgets such as numeric, color, vector, transform, and
  inspector controls.
- `fret-docking::imui`: docking/workbench helpers.
- `fret::imui::{prelude::*, kit, editor, docking}`: app-facing entry surface.

Do not refactor this into a monolithic `fret-imui` widget crate. That would copy Dear ImGui's shape
instead of preserving Fret's mechanism/policy split.

## What This Slice Now Proves

1. Runtime text inputs and text areas have mechanism-level read-only behavior.
   - Read-only keeps focus, selection, copy, and select-all available.
   - Read-only blocks text input, paste, cut, clear, delete, primary-selection paste, and platform
     text replacement.
   - Semantics expose read-only and non-editable state.

2. IMUI has policy-level text flags for the common Dear ImGui first slice.
   - `InputTextOptions::read_only`
   - `InputTextOptions::select_all_on_focus`
   - `TextAreaOptions::read_only`
   - `TextAreaOptions::select_all_on_focus`
   - `TextAreaOptions::allow_tab_input`

3. Focus-time select-all is correctly policy-owned.
   - It uses element-scoped transient state and a zero-delay timer.
   - The next render only dispatches `edit.select_all` if the same element is still focused.
   - A stale timer cannot select text in another focused input.

4. App authors can reach the feature through the product surface.
   - `apps/fret-cookbook/examples/imui_action_basics.rs` now uses
     `fret::imui::{prelude::*, kit::{ButtonOptions, InputTextOptions}}`.
   - The cookbook source-policy test locks the example away from direct `fret-ui-kit` /
     `fret-imui` imports.

5. `ImUi::push_id` is now explicit-key-driven.
   - This aligns better with Dear ImGui's `PushID` mental model.
   - It removes the previous callsite-location identity leak for repeated render closures and
     reordered model-backed controls.

6. IMUI multiline Tab input is now opt-in.
   - Runtime text areas expose the mechanism flag.
   - IMUI textareas default to no Tab mutation.
   - `TextAreaOptions::allow_tab_input=true` inserts `\t` and reports `changed()`.

## Current Dear ImGui-Class Coverage

Fret IMUI now has meaningful breadth:

- explicit identity scopes and keyed iteration,
- rows/columns/grid/scroll/child regions,
- buttons, invisible buttons, arrow/small variants,
- menus, submenus, menu bars, menu items, command-backed menu/button helpers,
- popups, modal popups, tooltips,
- combos, selectables, multi-selectable helpers,
- tabs and tables,
- virtual lists,
- text input and textarea model helpers,
- disabled scopes,
- typed drag source and drop target helpers,
- floating areas/windows,
- editor adapters for numeric, color, vector, transform, property grid, and inspector controls,
- docking helper entry points through `fret::imui::docking`.

The biggest gap is no longer "basic widgets are missing." The biggest gap is depth, shell feel,
and proof coverage for editor-grade workflows.

## Remaining Gaps That Should Become Follow-On Lanes

### P0 - Text Editing Callback And Completion Policy

Dear ImGui's `InputText` family includes callback/history/completion/filter/editing hooks. Fret
should not clone callback-heavy C++ semantics directly into `crates/fret-ui`.

Recommended owner:

- `fret-ui-kit::imui` for generic policy flags and simple filters.
- `fret-ui-editor::imui` for completion/history/editor assist behavior.

Next proof:

- a command palette/search/filter surface that needs completion or history,
- plus a focused model-backed test proving blocked/accepted edits and response `changed()` timing.

### P1 - Numeric Scalar Text-Edit Fallback

Dear ImGui sliders and drags can switch into text editing. Fret already has editor numeric controls,
but the generic IMUI slider lane still needs a stronger "drag/slider plus text edit" story.

Recommended owner:

- `fret-ui-editor::imui` first, then promote only the generic part into `fret-ui-kit::imui`.

Next proof:

- one property inspector row with drag value, manual text entry, clamp policy, and stable
  `changed()` semantics.

### P1 - Draw/Debug Overlay Lane

Dear ImGui has draw lists. Fret should not put a generic draw list into `fret-imui` by default, but
editor/debug tooling still needs an immediate custom drawing story.

Recommended owner:

- a dedicated debug-draw/canvas adapter in ecosystem,
- or editor/gizmo-specific helpers where the product proof is concrete.

Next proof:

- a viewport overlay or gizmo debug surface with hit regions, draw order, and diagnostics evidence.

### P1 - Docking And Multi-Window Hand Feel

Dear ImGui's multi-viewport feel depends heavily on backend cooperation. Fret's owner is docking,
runner, and platform code, not `fret-imui`.

Recommended owner:

- `ecosystem/fret-docking`,
- `crates/fret-launch`,
- platform backends.

Next proof:

- cross-window drag/release/cancel paths, mixed-DPI follow behavior, and overlap routing diagnostics.

### P2 - Test Harness Decomposition

The behavior coverage is useful, but IMUI tests are still large and hard to navigate. This is a
refactor hazard.

Recommended owner:

- `ecosystem/fret-imui/src/tests/*`,
- fixture-driven harnesses for repetitive state/response matrices.

Next proof:

- split text, popup, floating, menu/tab, drag/drop, and model-state tests by capability without
  weakening existing assertions.

## What Not To Do

- Do not fatten `fret-imui` into a widget crate.
- Do not move read-only or select-all policy into `crates/fret-ui` beyond mechanism hooks.
- Do not copy Dear ImGui's global style stack unless a concrete editor workflow proves it is needed.
- Do not add a second ID hashing runtime.
- Do not treat generic IMUI as the owner of docking, OS-window, or shell workspace policy.

## Gates Run For This Slice

- `cargo fmt --package fret-ui --package fret-ui-kit --package fret-imui --package fret-cookbook`
- `cargo nextest run -p fret-ui text_area_tab_key_respects_allow_tab_input_policy --no-fail-fast`
- `cargo nextest run -p fret-imui textarea_tab_key_does_not_insert_by_default textarea_allow_tab_input_inserts_tab_and_reports_changed --no-fail-fast`
- `cargo nextest run -p fret-ui text_input text_area --no-fail-fast`
- `cargo nextest run -p fret-imui models_text --no-fail-fast`
- `cargo nextest run -p fret-cookbook cookbook_imui_example_keeps_current_facade_teaching_surface --no-fail-fast`
- `cargo check -p fret-cookbook --features cookbook-imui --example imui_action_basics`
- `python tools/check_layering.py`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-text-input-policy-depth-v1/WORKSTREAM.json`
- `git diff --check`
