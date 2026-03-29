# imui stack fearless refactor v1 - milestones

Tracking doc: `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`

TODO board: `docs/workstreams/imui-stack-fearless-refactor-v1/TODO.md`

Historical references:

- `docs/workstreams/imui-authoring-facade-v2/imui-authoring-facade-v2.md`
- `docs/workstreams/imui-ecosystem-facade-v3/imui-ecosystem-facade-v3.md`

This file is forward-looking only.
Earlier incremental facade work remains in git history and older workstream notes; the milestones
below describe the recommended execution order for the stack reset.

## Phase A - Scope lock and deletion baseline

Status: Completed

Goal:

- freeze the ownership story,
- replace incremental-facade thinking with one stack-level plan,
- and decide what gets deleted before implementation starts.

Deliverables:

- one workstream directory with `DESIGN.md`, `TODO.md`, and `MILESTONES.md`,
- one explicit boundary split across `fret-imui`, `fret-ui-kit::imui`, and `fret-ui-editor::imui`,
- one deletion map for compatibility aliases, redundant entry points, and legacy paths,
- one canonical naming direction for layout, window, floating, popup, and disabled-scope helpers.

Exit gates:

- the new workstream documents are the primary execution surface for the refactor,
- compatibility preservation is explicitly ruled out,
- the deletion candidates are named concretely rather than described vaguely,
- and the team can tell which layer owns each surviving API family.

## Phase B - Stack simplification and public surface reset

Status: In progress

Goal:

- remove compatibility debt,
- shrink the public `imui` surface to one API per concept,
- and make the remaining surface easier to teach and maintain.

Current landed slice:

- `fret-imui` feature aliases now expose only `state-query`, `state-selector`, and `state`,
- alias-only layout/window/floating names were removed from `fret-ui-kit::imui`,
- the remaining generic explicit-options helpers were canonicalized onto `*_with_options(...)`
  instead of keeping non-legacy `*_ex(...)` names,
- in-tree `fret-imui` callers were migrated to the canonical names in the same refactor batch,
- and the dead `floating_window_impl_legacy` path was deleted outright.

Deliverables:

- `fret-imui` without backward-compatible Cargo feature aliases,
- `fret-ui-kit::imui` without alias-only or legacy public entry points,
- one canonical window API family,
- one canonical floating-area API family,
- one canonical disabled-scope API,
- one canonical layout vocabulary for immediate helpers.

Exit gates:

- old compatibility names no longer exist in public APIs,
- in-tree callers build against the new canonical surface,
- no `_legacy` path remains for floating/window helpers,
- and the surviving API reads like an intentional surface rather than a pile of historical layers.

## Phase C - Module split and editor adapter closure

Status: Completed

Goal:

- break the current monolith files into reviewable modules,
- and make `fret-ui-editor::imui` a complete thin adapter layer for the promoted editor starter set.

Current landed slice:

- `fret-imui` authoring frontend moved out of `src/lib.rs` into a focused `frontend.rs` module,
  leaving the crate root as a thinner declaration and re-export surface,
- `fret-imui` no longer keeps its full test harness inline in `src/lib.rs`; the first extraction cut
  now lives under `src/tests/`,
- `fret-imui` test coverage is now split by behavior family (`interaction`, `popup_hover`,
  `floating`, `models`, `composition`) instead of one giant test module,
- `fret-ui-kit::imui` module split has started by extracting `response.rs` and `options.rs`
  out of the monolithic root file,
- container layout builders now live in `fret-ui-kit::imui::containers`,
- floating layer / floating area / drag-surface internals now live in
  `fret-ui-kit::imui::floating_surface`,
- floating window wrapper / dispatch chains now live in `fret-ui-kit::imui::floating_window`,
- popup-scope runtime storage has moved into `fret-ui-kit::imui::popup_store`,
- popup / menu / modal lifecycle and overlay wiring now live in
  `fret-ui-kit::imui::popup_overlay`,
- disabled/hover/drag/long-press runtime helpers now live in `fret-ui-kit::imui::interaction_runtime`,
- button / action-button pressable helpers now live in `fret-ui-kit::imui::button_controls`,
- menu item rendering and menu-navigation helpers now live in `fret-ui-kit::imui::menu_controls`,
- checkbox / switch / toggle helpers now live in `fret-ui-kit::imui::boolean_controls`,
- slider helpers now live in `fret-ui-kit::imui::slider_controls`,
- select / popup-trigger helpers now live in `fret-ui-kit::imui::select_controls`,
- text input / textarea widget helpers now live in `fret-ui-kit::imui::text_controls`,
- `fret-ui-editor::imui` now exposes thin adapters for `ColorEdit`, `NumericInput`,
  `MiniSearchBox`, `TextAssistField`, `Vec2Edit`, `Vec3Edit`, `Vec4Edit`, `TransformEdit`,
  `AxisDragValue`, and `IconButton`,
- the remaining editor-inventory question is now closed by audit instead of assumption:
  `FieldStatusBadge` stays declarative-only, while the promoted input/editing starter set remains
  adapter-covered,
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs` now locks the adapter file to a thin
  one-hop `into_element` forwarding layer with no adapter-local models, action policy, or second
  implementation path.

Deliverables:

- a smaller `fret-imui` crate root with focused implementation modules,
- a split `fret-ui-kit::imui` implementation grouped by concern,
- systematic editor `imui` adapters for the existing declarative control inventory,
- no second implementation path for any promoted editor control.

Exit gates:

- module boundaries match ownership boundaries,
- `fret-ui-editor::imui` covers the promoted editor controls needed for inspector/tool UIs,
- adapter code stays thin and declarative-control ownership remains obvious,
- and the refactor diff is reviewable by subsystem instead of by one giant file.

## Phase D - Gates, proof surfaces, and documentation closure

Status: Planned

Goal:

- prove that the reset did not damage the retained substrate guarantees,
- lock the new canonical surface with tests,
- and remove stale teaching surfaces.

Current landed slice:

- focused `fret-imui` regression coverage now locks popup modal dismiss semantics
  (`close_on_outside_press` and `Escape`),
- focused `fret-ui-editor` smoke coverage now locks the promoted immediate adapters
  (`ColorEdit`, `NumericInput`, `MiniSearchBox`, `TextAssistField`, `IconButton`, `Vec2Edit`,
  `Vec3Edit`, `Vec4Edit`, `TransformEdit`) in one compile-time authoring surface gate,
- the existing editor proof surface remains runnable through
  `apps/fret-examples/src/imui_editor_proof_demo.rs` and
  `apps/fret-demo/src/bin/imui_editor_proof_demo.rs`,
- older `imui` facade/parity workstreams are now explicitly labeled as pre-reset historical notes
  so deleted compatibility names stop reading like current guidance,
- the active workstream now records an explicit closeout snapshot (`what survived / what was deleted /
  what became canonical / what is now only non-blocking future cleanup`),
- public floating/select naming debt was closed by collapsing the surviving author-facing surface
  onto `window*`, `floating_area_with_options(...)`, `floating_area_drag_surface(...)`, and
  `select_model_with_options(...)`,
- the remaining generic explicit-options helpers now also converge on `*_with_options(...)`
  naming across layout, controls, models, and popup entry points,
- the last non-legacy overlay-root `_ex` contract in `fret-ui` was replaced by
  `OverlayRootOptions` + `push_overlay_root_with_options(...)`,
- the remaining live-code `_ex` helpers were eliminated from `crates/`, `ecosystem/`, and `apps/`,
- and the existing floating/window/select regression matrix remains green against the refactored
  canonical surface.

Deliverables:

- focused regression tests for renamed or consolidated API families,
- focused regression tests for editor `imui` adapters,
- at least one runnable editor-oriented proof/demo surface after the reset,
- updated documentation that no longer points to deleted compatibility names.

Exit gates:

- the new canonical `imui` surface is covered by tests rather than by historical memory,
- the editor authoring story is reviewable on a proof/demo surface,
- no public workstream or usage note still teaches deleted names as current practice,
- and the stack can be described in one short ownership story without caveats.

## Recommended execution order

1. Finish Phase A and do not reopen scope casually.
2. Use Phase B to delete compatibility debt in one deliberate pass.
3. Use Phase C to make the internal shape and editor adapter story match the new public surface.
4. Use Phase D to lock the result with gates and documentation.
