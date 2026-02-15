# `fret-ui-editor` v1 — TODO Tracker

Status: Active tracker (workstream note; not an ADR)  
Last updated: 2026-02-15

Related:

- Design / constraints: `docs/workstreams/ui-editor-v1.md`
- Alignment inventory: `docs/workstreams/ui-editor-imgui-alignment-v1.md`
- Runtime boundary: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Unified authoring: `docs/adr/0160-unified-authoring-builder-surface-v1.md`

## Milestones

### M0 — Skeleton + gates

- [x] Create `ecosystem/fret-ui-editor` crate skeleton (no shadcn dependency).
  - Evidence: `ecosystem/fret-ui-editor/Cargo.toml`, `ecosystem/fret-ui-editor/src/lib.rs`
- [x] Define feature flags:
  - [x] `state-selector`, `state-query`, `state`
  - [x] optional undo integration (feature-gated) (`state-undo`)
  - Evidence: `ecosystem/fret-ui-editor/Cargo.toml`
- [x] Document dependency boundaries in the crate README:
  - [x] allowed: `fret-ui`, `fret-ui-kit`, `fret-ui-headless`
  - [x] forbidden: `fret-ui-shadcn`, any runner/platform/render crates, and domain ecosystem crates (node/plot/chart)
  - Evidence: `ecosystem/fret-ui-editor/README.md`
- [x] Establish module layout:
  - [x] `primitives/*` (edit sessions, scrub)
  - [x] `controls/*` (drag value, numeric input, color edit, …)
  - [x] `composites/*` (property row/grid/panel recipes)
  - [x] `state/*` (feature-gated adapter helpers)
  - Evidence: `ecosystem/fret-ui-editor/src/primitives/mod.rs`, `ecosystem/fret-ui-editor/src/controls/mod.rs`, `ecosystem/fret-ui-editor/src/composites/mod.rs`, `ecosystem/fret-ui-editor/src/state/mod.rs`
- [x] Pick initial `editor.*` token keys (v1 minimal set) and add a short token table to `ui-editor-v1.md`.
  - Evidence: `ecosystem/fret-ui-editor/src/primitives/tokens.rs`, `docs/workstreams/ui-editor-v1.md`
- [x] Demo gate wiring plan:
  - [x] identify an existing harness target (preferred: `imui_editor_proof_demo`)
  - [x] define initial `test_id` anchor(s) for scripted repro
  - Evidence: `apps/fret-examples/src/imui_editor_proof_demo.rs` (`imui-editor-proof.editor.drag-value-demo`)

Feature intent matrix (recommended):

- `state-selector`: enables selector-based derived state helpers and may use `fret-selector/ui` to call `cx.use_selector(...)`.
- `state-query`: provides UI helpers for `QueryState<T>` status/error rendering (should not require `fret-query/ui`).
- `state`: umbrella for both.

Exit criteria:

- The crate compiles in default features (empty surface allowed).
- Feature flags compile when enabled (even if stubs).

### M1 — Numeric editing closed loop (small but “real”)

Deliverables:

- [~] `EditSession` primitive:
  - [x] cancel (Escape) outcome via scrub primitive
  - [x] “pre-edit value” capture and restore on cancel
  - Evidence: `ecosystem/fret-ui-editor/src/primitives/edit_session.rs`, `ecosystem/fret-ui-editor/src/primitives/drag_value_core.rs`
- [~] `DragValueCore` primitive:
  - [x] horizontal drag scrubbing
  - [x] Shift slow / Alt fast modifiers (token-tunable multipliers)
  - [x] pointer capture semantics (via `Pressable` default capture)
  - [x] best-effort cleanup when pointer-up is missed (no-buttons move fallback)
  - [x] configurable drag threshold
  - Evidence: `ecosystem/fret-ui-editor/src/primitives/drag_value_core.rs`
- [~] `NumericInput` control (typed editing path):
  - [x] parse/format hooks
  - [x] validation/error affordance slot
  - Evidence: `ecosystem/fret-ui-editor/src/controls/numeric_input.rs`
- [~] `DragValue<T>` control:
  - [x] scalar abstraction (`DragValueScalar` for `f32`/`f64`/`i32`)
  - [x] double-click to type (switch to `NumericInput`)
  - Evidence: `ecosystem/fret-ui-editor/src/controls/drag_value.rs`
- [~] `PropertyRow` composite:
  - [x] label slot + value slot + actions slot
  - [x] reset-to-default affordance (UI only; callback provided by caller)
  - Evidence: `ecosystem/fret-ui-editor/src/composites/property_row.rs`

Optional (if `fret-undo` integration is ready):

- [ ] scrub coalescing semantics using `fret-undo` (single record on pointer-up).

Exit criteria:

- A demo view exists that exercises:
  - drag + modifiers
  - double-click to type
  - Escape cancels to pre-edit value

### M2 — Property grid + density + state glue

- [~] `EditorDensity` token family (`editor.density.*`) and application in core controls.
- [ ] `FieldStatus` glue (loading/error/mixed/dirty):
  - [x] simple badge/label helpers
  - [ ] optional `state` module integration (feature-gated)
- [x] `MiniSearchBox` control (for filtering property groups and palettes).
- [x] `PropertyGroup` composite (collapsible group header + section).
- [~] `PropertyGrid` composite:
  - [x] two-column layout (via `PropertyRow` composition)
  - [x] label width policy (fixed width option, propagated to rows)
  - [ ] virtualization strategy decision:
    - [ ] composable rows path (VirtualList)
    - [ ] windowed paint path for large inspectors (if needed)
- [~] Demo: groups + filter + empty-state placeholders.
  - Evidence: `apps/fret-examples/src/imui_editor_proof_demo.rs`

Exit criteria:

- A demo grid exists with:
  - groups + filter
  - mixed value affordance
  - focus traversal sanity (Tab/Shift+Tab)

### M2.5 — Visual baseline (chrome + legibility)

Goal: make the editor-proof harness readable and stable (no overlapped text, no “unstyled” editable surfaces).

- [~] Establish a minimal chrome baseline for common editor controls:
  - [x] `DragValue` scrub mode renders an input-like frame (bg/border/radius/padding).
    - Evidence: `ecosystem/fret-ui-editor/src/controls/drag_value.rs`
  - [x] `EnumSelect` trigger includes a caret indicator and input-like frame.
    - Evidence: `ecosystem/fret-ui-editor/src/controls/enum_select.rs`
  - [x] `PropertyGroup` header has a background and divider for visual grouping.
    - Evidence: `ecosystem/fret-ui-editor/src/composites/property_group.rs`
  - [x] Proof demo uses ui-kit-styled buttons for top-level actions (avoid bare text buttons).
    - Evidence: `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - [ ] Define a shared `EditorChrome` recipe (optional): centralize token keys and defaults so controls don’t drift.
  - [ ] Ensure docking tabs remain legible under the demo theme (tab text, hover/active states).
    - Evidence anchor: `apps/fret-examples/src/imui_editor_proof_demo.rs`

### M3 — Core editor controls (Color / Vec / Transform / Asset refs)

- [x] `Checkbox` (bool + tri-state/mixed):
  - [x] bool model binding (`Model<bool>`)
  - [x] tri-state/mixed binding (`Model<Option<bool>>` where `None` = indeterminate)
  - Evidence: `ecosystem/fret-ui-editor/src/controls/checkbox.rs`, `apps/fret-examples/src/imui_editor_proof_demo.rs`
- [~] `ColorEdit` (minimal v1):
  - [x] swatch + hex input
  - [~] popup picker placeholder (can be minimal)
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs`, `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - [ ] copy/paste affordances (optional)
- [x] `Vec2Edit` / `Vec3Edit` / `Vec4Edit` (built on `DragValue<T>`):
  - [x] axis labels + axis color tokens
  - [x] per-axis reset hooks
  - Evidence: `ecosystem/fret-ui-editor/src/controls/vec_edit.rs`, `apps/fret-examples/src/imui_editor_proof_demo.rs`, `docs/workstreams/ui-editor-v1.md`
- [~] `TransformEdit` (position/rotation/scale composite):
  - [x] layout variants (row/column)
  - [x] link scale toggle (optional)
  - [x] best-effort uniform scale sync when linked
  - Evidence: `ecosystem/fret-ui-editor/src/controls/transform_edit.rs`, `apps/fret-examples/src/imui_editor_proof_demo.rs`
- [~] `EnumSelect` (filterable select surface).
  - Evidence: `ecosystem/fret-ui-editor/src/controls/enum_select.rs`, `apps/fret-examples/src/imui_editor_proof_demo.rs`
- [ ] `AssetRefField` (UI shell):
  - [ ] supports async loading states via optional query glue
  - [ ] does not define an asset system; caller supplies data and callbacks
- [ ] `InspectorPanel` recipe (search + grid + toolbar slots).

Exit criteria:

- The editor demo panel reads like a real inspector (dense, consistent spacing, predictable hand feel).

### M4 — Gradient editor spike (composition proof)

Goal: validate that editor primitives can scale to canvas-like controls without runtime changes.

- [ ] `GradientEditor` spike:
  - [ ] stop list + drag stop position
  - [ ] stop color edit reuse (`ColorEdit`)
  - [ ] angle edit reuse (`DragValue`)
- [ ] Identify what this spike forces on other ecosystems (tokens/slots/hooks).

Exit criteria:

- A runnable demo with stable `test_id` anchors for scripted repro.

### M5 — Curve editor candidate (defer unless needed)

Goal: only proceed after the spike demonstrates that the substrate is sufficient.

- [ ] `CurveEditor` candidate plan:
  - [ ] selection model
  - [ ] point drag + ordering
  - [ ] zoom/pan policy
  - [ ] snapping hooks

Exit criteria:

- A scoped plan exists; implementation only starts once substrate gaps are explicit.

## Open questions (track here)

- Should `fret-ui-editor` integrate `fret-undo` behind a feature flag or keep undo policy app-owned?
- Where should “axis color” defaults live (tokens vs theme aliases)?
  - Do we need a shared “precision format” helper module (unit formatting, angle display, percent display)?
  - What is the minimal popup picker contract for `ColorEdit` without importing a full color engine?
  - Should `VecNEdit` expose a combined model surface (e.g. `Model<[f64; 3]>`) in addition to per-axis models?

## Evidence checklist (for each landed component)

- [ ] A clear owner module and public API boundary (no duplicate implementations).
- [ ] `test_id` anchors for scripted repro (where applicable).
- [ ] A demo integration that exercises the intended hand feel.
- [ ] If new runtime API is required, open an ADR/update before expanding usage.
