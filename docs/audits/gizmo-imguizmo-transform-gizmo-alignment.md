# Audit: 3D Transform Gizmo Alignment (ImGuizmo + transform-gizmo)

This audit compares Fret's `ecosystem/fret-gizmo` against two widely used reference implementations:

- **ImGuizmo** (Dear ImGui overlay gizmo): `repo-ref/ImGuizmo`
- **transform-gizmo** (Rust gizmo with sub-gizmos): optional `repo-ref/transform-gizmo`

Goal: enumerate **feature-level parity** (what users can do + how it feels), mark the **current alignment
status** per item, and provide an ordered list of **next alignment targets**.

Note: Fret intentionally renders gizmos as **engine-pass 3D geometry** (depth-tested) rather than a pure
UI overlay, per ADR 0130. That means some "rendering topology" items will be *intentionally different*
while still aiming for equivalent UX outcomes.

## References (source of truth)

### ImGuizmo

- Primary header/API: `repo-ref/ImGuizmo/ImGuizmo.h`
- README and feature list: `repo-ref/ImGuizmo/README.md`

Key surfaces:

- `Manipulate(view, projection, OPERATION, MODE, matrix, deltaMatrix, snap, localBounds, boundsSnap)`
- `OPERATION` bitmask includes:
  - translate axes: `TRANSLATE_X/Y/Z`
  - rotate axes + screen: `ROTATE_X/Y/Z`, `ROTATE_SCREEN`
  - scale axes: `SCALE_X/Y/Z`
  - bounds: `BOUNDS` (+ `localBounds`, `boundsSnap`)
  - universal: `UNIVERSAL = TRANSLATE | ROTATE | SCALEU` (scale-universal flavor)
- Interaction queries: `IsOver(...)`, `IsUsing(...)`
- View gizmo: `ViewManipulate(...)`
- Behavior knobs: axis flip, axis/plane fade limits, axis masking
- Style knobs: per-part colors and line/arrow thickness (`Style`)

### transform-gizmo

Note: `repo-ref/transform-gizmo` is not guaranteed to be present in all workspaces (see `docs/repo-ref.md`).
If it is missing locally, clone the upstream into `repo-ref/transform-gizmo` before following the file path anchors below.

- Crate overview: `repo-ref/transform-gizmo/crates/transform-gizmo/src/lib.rs`
- Configuration and mode taxonomy: `repo-ref/transform-gizmo/crates/transform-gizmo/src/config.rs`
- Core update/draw: `repo-ref/transform-gizmo/crates/transform-gizmo/src/gizmo.rs`

Key surfaces:

- `Gizmo::update(interaction, targets) -> Option<(GizmoResult, Vec<Transform>)>`
- `Gizmo::draw() -> GizmoDrawData` (vertices in viewport coordinates)
- Mode taxonomy is **fine-grained** (subset selection via `EnumSet<GizmoMode>`), including:
  - translation: `TranslateX/Y/Z`, `TranslateXY/XZ/YZ`, `TranslateView`
  - rotation: `RotateX/Y/Z`, `RotateView`, plus `Arcball`
  - scale: `ScaleX/Y/Z`, `ScaleUniform`, and plane scale `ScaleXY/XZ/YZ`
- Config includes visuals and pixel ratio: `GizmoVisuals`, `pixels_per_point`

## Fret implementation surfaces (what we have today)

- Gizmo crate: `ecosystem/fret-gizmo`
  - API entry points: `ecosystem/fret-gizmo/src/lib.rs`
  - Core behavior: `ecosystem/fret-gizmo/src/gizmo.rs`
  - Projection/picking math: `ecosystem/fret-gizmo/src/math.rs`
- Demo (integration + rendering): `apps/fret-examples/src/gizmo3d_demo.rs`
- Engine-pass overlay hook substrate (runner boundary): ADR 0038 direction
  - `crates/fret-launch/src/runner/common.rs` (`ViewportOverlay3dHooksService`, `record_viewport_overlay_3d`)

Fret's current contract:

- **Update/draw split**:
  - `Gizmo::update(view_proj, viewport, input, active_target, targets) -> Option<GizmoUpdate>`
  - `Gizmo::draw(view_proj, viewport, active_target, targets) -> GizmoDrawList3d`
- Output is **3D world-space line/triangle lists** with a depth policy:
  - `Line3d` / `Triangle3d` with `DepthMode::{Test,Ghost,Always}`
- Input is a small host-provided state machine:
  - `GizmoInput { cursor_px, hovered, drag_started, dragging, snap, cancel, precision }`
- Multi-target support is built-in:
  - `GizmoUpdate` returns `updated_targets: Vec<GizmoTarget3d>` and includes begin/update/commit/cancel phases.
- DPI / scale-factor handling is host-driven:
  - `GizmoConfig::scale_for_pixels_per_point(scale_factor)`
  - `ViewGizmoConfig::scale_for_pixels_per_point(scale_factor)`

## Alignment legend

- **Aligned**: equivalent user-visible feature exists and the semantics match closely.
- **Partially aligned**: feature exists but differs in a meaningful way (missing sub-modes, weaker heuristics, no UX affordance).
- **Not implemented**: missing.
- **Not audited**: unknown / not validated against the reference yet (often camera/projection edge cases).
- **Intentional divergence**: different architecture, but the UX target remains; document why.

## Vocabulary mapping (reference → Fret)

### Operations / modes

| Concept | ImGuizmo | transform-gizmo | Fret |
| --- | --- | --- | --- |
| Translate | `TRANSLATE_*` (bitmask) | `Translate*` modes | `GizmoMode::Translate` |
| Rotate | `ROTATE_*`, `ROTATE_SCREEN` | `Rotate*`, `RotateView`, `Arcball` | `GizmoMode::Rotate` |
| Scale | `SCALE_*`, `SCALEU`, `BOUNDS` | `Scale*` + plane scale | `GizmoMode::Scale` |
| "Universal" (combo) | `UNIVERSAL` | `EnumSet<GizmoMode>` (multi-mode) | `GizmoMode::Universal` (translate+rotate+axis scale by default; see `universal_includes_scale`) |
| World vs local | `MODE::{WORLD,LOCAL}` | `GizmoOrientation` | `GizmoOrientation::{World,Local}` |
| Pivot | (external; matrix origin) | `TransformPivotPoint` | `GizmoPivotMode::{Active,Center}` |

### Rendering topology

| Concept | ImGuizmo | transform-gizmo | Fret |
| --- | --- | --- | --- |
| Draw space | ImGui draw list (2D overlay) | viewport-space vertices (2D overlay) | **world-space 3D geometry** (engine pass) |
| Depth / occlusion | overlay (no depth test) | overlay (no depth test) | depth-tested optional (`DepthMode`) |
| Constant pixel size | yes (clip-space sizing knobs) | yes (`scale_factor`, `pixels_per_point`) | yes (`axis_length_world(...)` + host `scale_for_pixels_per_point`) |

## Feature alignment matrix (detailed)

### 0) API and data model

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| Target representation | 4x4 matrix | TRS (`Transform`) | **Aligned (by intent)** | Fret uses TRS (`Transform3d`) to avoid decomposing matrices. `ecosystem/fret-gizmo/src/gizmo.rs` (`Transform3d`). |
| Decompose/recompose helpers | Yes (`DecomposeMatrixToComponents`, `Recompose...`) | N/A | **Aligned (basic)** | `Transform3d::try_from_mat4_trs` and `Transform3d::to_mat4` provide TRS round-tripping (`ecosystem/fret-gizmo/src/gizmo/types.rs`). |
| "Delta matrix" output | Yes (`deltaMatrix`) | No (returns result + updated transforms) | **Aligned (TRS-only)** | `GizmoUpdate::delta_matrix_for(start)` computes an ImGuizmo-style `deltaMatrix` for TRS transforms (`ecosystem/fret-gizmo/src/gizmo/runtime.rs`, `ecosystem/fret-gizmo/src/gizmo/types.rs`). |
| Semantic delta output | Partial | Yes (`GizmoResult`) | **Aligned** | `GizmoResult::{Translation,Rotation,Scale}` includes `delta` + `total`. |
| Multi-target update in one call | External | Yes | **Aligned** | `GizmoUpdate.updated_targets: Vec<GizmoTarget3d>`. |
| Begin/update/commit/cancel phases | No | Partial (implicit) | **Aligned (Fret-specific)** | `GizmoPhase` enables clean undo grouping; host decides persistence. |
| "IsOver / IsUsing" queries | Yes (`IsOver`, `IsUsing`) | Yes (`is_focused`, active subgizmo) | **Aligned** | Fret exposes `GizmoState { hovered, active }` plus helper methods `GizmoState::{is_over,is_using,is_over_handle,is_using_handle}` for the common queries. |
| Fine-grained operation selection | Yes (bitmask) | Yes (`EnumSet`) | **Aligned (basic)** | `GizmoConfig::operation_mask: Option<GizmoOps>` enables fine-grained sub-operation selection (translate axis/plane/view, rotate axis/view/arcball, scale axis/plane/uniform/bounds) without changing the coarse `GizmoMode` API. |

### A) Core transform handles

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| Translate axis X/Y/Z | Yes | Yes | **Aligned** | Axis handles exist + picking + axis constraint. `ecosystem/fret-gizmo/src/gizmo.rs` (`pick_translate_handle`, `begin_translate_drag`). |
| Translate plane XY/XZ/YZ | Yes | Yes | **Aligned** | Plane quads + picking. `pick_translate_handle` + `translate_plane_quad_world`. |
| Translate "screen-plane" (center handle) | Yes (center screen-plane) | No (not in core modes) | **Aligned** | Fret's center handle (`TranslateHandle::Screen`) constrains motion to the camera-facing plane at the gizmo origin. `translate_constraint_for_handle` handle id `10`. |
| Translate "depth" (move toward/away camera) | No (not explicit) | Yes (`TranslateView`, along view forward axis) | **Aligned (transform-gizmo) / Fret extension (ImGuizmo)** | View-direction "dolly" handle (`TranslateHandle::Depth`, id `11`) with screen-delta mapping for stability: `translate_constraint_for_handle`, `begin_translate_drag`, and the Translate update path in `ecosystem/fret-gizmo/src/gizmo.rs`. |
| Rotate axis X/Y/Z rings | Yes | Yes | **Aligned** | Rendered as a thick ring band (triangles) + edge stroke, with per-part thickness via `GizmoPartVisuals::rotate_ring_thickness_scale`; picking uses screen-space primitives (`PickSegmentCapsule2d`) over the projected ring polyline: `draw_rotate_rings`, `pick_rotate_axis`, `ecosystem/fret-gizmo/src/picking.rs`. |
| Rotate around view axis (screen ring) | Yes (`ROTATE_SCREEN`) | Yes (`RotateView`) | **Aligned** | `show_view_axis_ring` + handle id 8, rendered as an outer ring (`GizmoPartVisuals::rotate_view_ring_radius_scale`). `pick_rotate_axis` view ring path, `begin_rotate_drag` view-axis mode. |
| Arcball rotation | No | Yes (`Arcball`) | **Aligned (basic)** | Fret supports arcball free-rotation in `GizmoMode::Rotate` via `GizmoConfig::show_arcball` and emits `GizmoResult::Arcball { delta, total }` (quat-based), matching transform-gizmo’s contract shape. The arcball ring is rendered as a thin band (triangles) in `draw_rotate_rings`. |
| Scale axis X/Y/Z | Yes | Yes | **Aligned** | Axis scaling is supported and axis picking matches the rendered end boxes (not the whole shaft). `pick_scale_handle`, `begin_scale_drag`. |
| Scale uniform (center handle) | Partial (via `SCALEU`/center) | Yes (`ScaleUniform`) | **Aligned** | Center uniform handle id 7, with pivot-respecting translation compensation. |
| Scale plane XY/XZ/YZ | No | Yes (`ScaleXY/XZ/YZ`) | **Aligned** | Plane scale handles (XY/XZ/YZ) are supported in `Scale` mode. `pick_scale_handle` + `begin_scale_drag` + scale update path. |
| Bounds / box scaling | Yes (`BOUNDS`, `localBounds`, `boundsSnap`) | No | **Aligned (basic)** | Fret supports a bounds-style box in `GizmoMode::Scale` gated by `GizmoConfig::show_bounds`, with corner + face handles. Selection bounds can be derived from `GizmoTarget3d::local_bounds` (local AABB) when provided, otherwise it falls back to translations (`ecosystem/fret-gizmo/src/gizmo.rs`). Gaps: multi-selection scaling semantics are TRS-only (no shear). |

### B) "Universal" / multi-mode behavior

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| Combine translate + rotate | Yes | Yes | **Aligned** | Fret `GizmoMode::Universal` overlays translate + rotate and resolves pick conflicts. `pick_universal_handle`. |
| Combine translate + rotate + scale | Yes (`UNIVERSAL`) | Yes (via mode set) | **Aligned (with known gaps)** | Fret Universal supports axis scaling alongside translate+rotate when `universal_includes_scale` is enabled. Note: uniform scale (center handle) remains exclusive to `Scale` mode to avoid center-handle conflicts with view-plane translation. |
| Fine-grained mode toggles | Yes (bitmask `OPERATION`) | Yes (`EnumSet<GizmoMode>`) | **Aligned (basic)** | Fret supports both: coarse `GizmoMode` and `GizmoConfig::operation_mask: Option<GizmoOps>` for sub-operation selection. Note: `GizmoMode::Universal` also includes a dedicated toggle for depth translate (`universal_includes_translate_depth`). |

### C) Orientation, pivot, multi-selection

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| World vs local orientation | Yes | Yes | **Aligned** | `GizmoOrientation::{World,Local}` and axis generation. `axis_dirs(...)`. |
| Pivot at active selection | Implicit (matrix) | Yes | **Aligned** | `GizmoPivotMode::Active`. |
| Pivot at selection center | External | Yes | **Aligned** | `GizmoPivotMode::Center` uses the **selection world AABB center** when bounds are available via `GizmoTarget3d::local_bounds` (editor convention), otherwise it falls back to the average of target translations. See `pivot_origin(...)` and `selection_world_aabb(...)` in `ecosystem/fret-gizmo/src/gizmo.rs`. |
| Multiple targets updated per drag | External | Yes | **Aligned** | `GizmoUpdate.updated_targets` returns all updated targets each frame. |
| Rotation of multiple targets around pivot | External | Yes | **Aligned** | Rotation updates translate+rotate around pivot. `GizmoMode::Rotate` update path. |
| Scale of multiple targets around pivot | External | Yes | **Aligned (basic)** | Multi-target scale updates both translation (about pivot) and scale factors for axis/plane/uniform: `scale_axis_scales_multiple_targets_about_pivot`, `scale_plane_scales_multiple_targets_about_pivot`, `scale_uniform_scales_multiple_targets_about_pivot` in `ecosystem/fret-gizmo/src/gizmo.rs`. Remaining known gap: TRS-only (no shear) limits "true world-axis scale" for rotated targets. |

### D) Snapping and precision controls

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| Toggle snapping during drag | Yes | Yes | **Aligned** | `GizmoInput.snap` gates step snapping. |
| Precision modifier (fine control) | External (editor convention) | External (editor convention) | **Aligned (Fret enhancement)** | `GizmoInput.precision` scales drag deltas without hard-coded keybindings (host-defined). Demo maps Shift to `0.2` and Ctrl/Meta to snapping (`apps/fret-examples/src/gizmo3d_demo.rs`). |
| Translation snap step | Yes | Yes | **Aligned** | `translate_snap_step: Option<f32>`. |
| Rotation snap step | Yes (degrees) | Yes (radians) | **Aligned** | `rotate_snap_step_radians: Option<f32>`; note unit differences vs ImGuizmo. |
| Scale snap step | Yes | Yes | **Aligned** | `scale_snap_step: Option<f32>`. |
| Snap visualization (rotation ticks) | No (varies) | Yes (visuals) | **Aligned (basic)** | Fret renders rotate tick marks plus translate/scale snap guides when snapping is active: `draw_rotate_feedback`, `draw_translate_feedback`, `draw_scale_feedback` in `ecosystem/fret-gizmo/src/gizmo.rs`. |
| Numeric HUD (snap + delta readout) | Yes (typical editor UX) | Partial | **Aligned (basic)** | `apps/fret-examples/src/gizmo3d_demo.rs` renders a small viewport HUD showing active/hover handle, snapping state + step, and `GizmoResult` delta/total during drags. |
| Bounds snap | Yes (`boundsSnap`) | No | **Aligned (basic)** | `GizmoConfig::bounds_snap_step` snaps bounds scaling to per-axis extent steps (ImGuizmo-style), gated by `GizmoInput.snap` (`ecosystem/fret-gizmo/src/gizmo.rs`). |

### E) Interaction lifecycle and "editor feel"

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| Hover highlight | Yes | Yes | **Aligned** | `GizmoState.hovered` + per-handle color selection in draw helpers. |
| Active handle lock (no retarget mid-drag) | Yes | Yes | **Aligned** | Hover picking is disabled while `state.active.is_some()`. |
| Drag start threshold | Partial | Yes (subgizmo pick gating) | **Aligned** | `drag_start_threshold_px` arms drag and emits `Begin` after movement. |
| Begin/Update/Commit phases | No (implicit) | Partial | **Aligned (Fret-specific)** | `GizmoPhase::{Begin,Update,Commit,Cancel}` enables undo boundaries cleanly. |
| Cancel interaction (Escape) | External | External | **Aligned (Fret-specific)** | `GizmoInput.cancel` → `GizmoPhase::Cancel`, restore is host-owned. |
| Picking priority heuristics (avoid mis-click) | Yes (tuned) | Yes (subgizmo ordering) | **Aligned (basic)** | Fret uses explicit per-mode ladders plus a unified mixed-mode policy (`GizmoPickPolicy` + `pick_best_mixed_handle`) to resolve overlaps deterministically. |

#### Picking priority ladders (explicit per-mode audit)

| Mode | Reference behavior (high level) | Fret status | Evidence / notes |
| --- | --- | --- | --- |
| Translate | Center/view-plane > plane interior > axis (avoid axis stealing near origin). | **Aligned (basic)** | Explicit early-outs exist for center handle and plane interior: `translate_center_handle_wins_near_origin`, `translate_plane_inside_wins_over_axis_when_both_hit` in `ecosystem/fret-gizmo/src/gizmo.rs`. |
| Rotate | Prefer the ring the user aims at; disambiguate view ring vs axis rings; avoid "wrong ring" when rings overlap in screen space. | **Aligned (basic)** | `pick_rotate_axis` has explicit view-ring vs axis-ring disambiguation (axis "strong hit" wins), backed by `rotate_view_ring_does_not_steal_axis_ring_when_both_hit`. Axis rings also fade out (and become unpickable) when edge-on: `rotate_ring_fade_hides_edge_on_axis_ring`. |
| Scale | Prefer axis end boxes when cursor is on the shaft; prefer center uniform only when close to origin; avoid fighting Universal overlays. | **Aligned (basic)** | Plane scale (XY/XZ/YZ) is implemented in `Scale` mode, and bounds handles win when overlapping scale axis end boxes: `scale_prefers_bounds_face_handle_over_axis_end_box_when_overlapping`. |
| Universal | Protect translate center/planes and scale end boxes; otherwise disambiguate rotate vs scale vs translate deterministically. | **Aligned (with known gaps)** | `pick_universal_handle` enforces a priority ladder (translate center/plane interior wins; scale end boxes win; otherwise tie-break rotate > scale > translate). Regression coverage includes orthographic, wide-FOV, and close near-plane overlap cases (`ecosystem/fret-gizmo/src/gizmo/tests.rs`, `universal_translate_tip_intent_*`). |

#### Drag stability invariants (what we must lock down)

These are the editor-feel invariants that the audit treats as P0 correctness requirements:

- Returning the cursor to the start position returns the total delta close to zero (no drift / no integration error).
- Moving the cursor forward then backward produces symmetric deltas (no "runaway" when reversing direction).
- Active handle never changes mid-drag (no retargeting).

### F) Camera/projection and robustness

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| Perspective camera | Yes | Yes | **Aligned** | Primary path; demo uses perspective. |
| Orthographic camera | Yes (`SetOrthographic`) | Yes (projection inference) | **Aligned (basic)** | Ortho projection is covered by invariants tests (translate axis drag stability) in `ecosystem/fret-gizmo/src/gizmo.rs`. |
| Left-handed vs right-handed | N/A (depends) | Yes (detects) | **Aligned (host opt-in)** | Fret models handedness via `GizmoConfig::handedness` to control the user-facing rotation sign (evidence: `GizmoHandedness`, `handedness_rotation_sign`, tests in `ecosystem/fret-gizmo/src/gizmo.rs`). For hosts that want auto-detection, `GizmoHandedness::detect_from_projection(projection)` is available as a convenience helper (`ecosystem/fret-gizmo/src/gizmo/types.rs`). |
| Behind-camera culling / stability | Yes | Yes | **Aligned (basic)** | `project_point` rejects behind-camera points (`clip.w <= 0`), and regression tests cover translate/rotate/scale (including Universal) behind-camera and near-plane scenarios in `ecosystem/fret-gizmo/src/gizmo.rs`. |
| Numeric stability at large scales | Mixed | Better (f64) | **Partially aligned** | Fret is f32-first, but supports an opt-in `fret-gizmo/f64-math` feature that uses f64 for projection/unprojection (picking-critical) math. Remaining gap: most gizmo update math still runs in f32; full large-world support may require broader internal f64 or an explicit scene-units/rebasing policy. |

### G) Rendering, styling, and customization

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| Constant pixel size | Yes | Yes | **Aligned** | `axis_length_world(...)` maps desired pixel size to world length. |
| Configurable line thickness | Yes (`Style`) | Yes (`stroke_width`) | **Aligned** | `line_thickness_px` used by demo shader. |
| Per-part visuals (sizes/thickness/alphas) | Yes (`Style`) | Partial | **Aligned (basic)** | `GizmoPartVisuals` + `GizmoVisualPreset::apply_to_gizmo` (`ecosystem/fret-gizmo/src/style.rs`), consumed by draw/picking via `state.part_visuals` (e.g. rotate ring thickness scaling in `draw_rotate_rings`). |
| Per-axis colors + hover color | Yes | Yes | **Aligned** | `GizmoConfig::{x_color,y_color,z_color,hover_color}`. |
| Occluded feedback | N/A (overlay) | N/A (overlay) | **Aligned (Fret enhancement)** | `DepthMode::Ghost` + `show_occluded` and `occluded_alpha`. |
| Per-part occlusion policy | N/A | N/A | **Aligned (Fret enhancement)** | `GizmoPartVisuals::occlusion: GizmoOcclusionPolicy` allows enabling/disabling the occluded ghost pass per feature group (rings, plane fills, bounds, handles, feedback). Feedback defaults to non-ghost overlay semantics (Always + no ghost triangles) to match mature editor UX. |
| Depth-tested gizmo geometry | No | No | **Intentional divergence / enhancement** | Fret expects engine-pass depth testing (ADR 0130). |
| Axis flip, axis masking, axis/plane fade limits | Yes | Partial | **Aligned (with known gaps)** | Fret supports `allow_axis_flip`, `axis_mask`, `axis_fade_px`, `plane_fade_px2` in `GizmoConfig`. Rotate rings use a separate view-angle fade window (`rotate_ring_fade_dot`) to avoid edge-on rings stealing interaction. |
| View gizmo (camera cube) | Yes (`ViewManipulate`) | No | **Partially aligned (basic)** | Fret provides `ViewGizmo` (cube) with hover + face/edge/corner click -> view intent, drag orbit output, center-button projection toggle, and basic labels rendered by the host (`ecosystem/fret-gizmo/src/view_gizmo.rs`, demo integration in `apps/fret-examples/src/gizmo3d_demo.rs`). Missing: richer host camera policies. |
| Grid draw helper | Yes (`DrawGrid`) | No | **Aligned (basic)** | `Grid3d` helper outputs depth-tested line geometry (`ecosystem/fret-gizmo/src/grid.rs`) and is rendered in `apps/fret-examples/src/gizmo3d_demo.rs` via the viewport overlay pass. |

## What to align next (recommended order)

This is a suggested sequence for reaching "mature editor" parity without over-design.

### What is already strong (baseline coverage)

- Core transform set: translate (axis/plane/screen/dolly), rotate (axis/view/arcball), scale (axis/plane/uniform), bounds/box scaling.
- Fine-grained operation selection: `GizmoConfig::operation_mask: Option<GizmoOps>` (ImGuizmo-like `OPERATION` / transform-gizmo `EnumSet`).
- Editor-feel invariants are guarded by tests (return-to-zero stability, behind-camera culling, near-plane edge cases).
- Universal overlap is guarded by tests across size policies (PixelsClampedBySelectionBounds/SelectionBounds), including view ring + arcball collisions (`ecosystem/fret-gizmo/src/gizmo/tests.rs`).

### Biggest remaining gaps vs. "mature editor feel" (P0)

1. **Viewport redraw policy (first frame + interaction-driven redraw)**
   - Goal: demo/editor should not require any input to present the first frame; interaction should always feel responsive.
   - Status: **Aligned (demo/runner)** - the winit runner requests an initial redraw on window creation, and the gizmo demo
     requests redraw on viewport input and while camera frame animations are active.
   - Remaining: ensure other viewport demos follow the same "interaction-driven redraw" rule (especially web/wasm paths).
2. **Universal mode surface + policy**
   - Status: **Aligned (basic)** - Universal has explicit inclusion toggles:
     `universal_includes_translate_depth`, `universal_includes_scale`,
     `universal_includes_rotate_view_ring`, `universal_includes_arcball`.
   - Remaining: add more overlap regression tests (near-plane + extremely close camera) and keep the demo's default policies conservative.
3. **Precision controls**
   - Implemented: `GizmoInput.precision` scales drag deltas without hard-binding to a specific input system.
   - Remaining: align demo/editor modifier mapping with the host's input conventions and ensure it composes well with snapping + camera navigation.
4. **Styling API parity**
   - ImGuizmo offers per-part thickness/sizes (translation arrow size, center circle size, ring thickness, etc).
   - transform-gizmo offers a compact visuals struct (stroke width + gizmo size + highlight alpha).
   - Implemented: reusable visuals structs (`GizmoVisuals`, `ViewGizmoVisuals`) and per-part metrics (`GizmoPartVisuals`) in
     `ecosystem/fret-gizmo/src/style.rs`, applied via `GizmoVisualPreset::apply_to_gizmo`.
   - Remaining: keep expanding per-part visuals (view ring + arcball styling, per-part alpha policies, label/overlay styling) while
     keeping `GizmoConfig` backwards compatible and documenting which fields are "semantic" vs purely visual.

### Feature breadth beyond the core (P1)

1. **Matrix tooling convenience**
   - Optional helpers for `TRS <-> Mat4` (decompose/recompose, delta matrix output) if we want easier ImGuizmo-style integrations.
2. **Custom gizmo extensibility**
   - Godot-style plugin surface: allow tools to contribute custom handles with explicit picking shapes (segments/capsules/triangles)
      in addition to draw geometry, so editor tools can build domain gizmos (lights, cameras, physics, nav, etc).
   - Status: **Aligned (with known gaps)** - Fret now has:
     - reusable pick-primitive layer (`PickCircle2d`, `PickSegmentCapsule2d`, `PickConvexQuad2d`) in `ecosystem/fret-gizmo/src/picking.rs`
     - an explicit plugin/handle namespace contract + manager (`GizmoPlugin`, `GizmoPluginManager`) in `ecosystem/fret-gizmo/src/plugin.rs` (see ADR 0140)
     - built-in transform gizmo routed through the manager (`TransformGizmoPlugin` in `ecosystem/fret-gizmo/src/transform_plugin.rs`)
     - custom property edit payloads for non-transform gizmos (`GizmoUpdate.custom_edits`, `GizmoCustomEdit`, `GizmoPropertyKey`) in `ecosystem/fret-gizmo/src/gizmo/runtime.rs`
     - real custom plugins shipped:
       - `RingScaleGizmoPlugin` (example plugin, transform-affecting) in `ecosystem/fret-gizmo/src/ring_scale_plugin.rs`
       - `LightRadiusGizmoPlugin` (non-transform scalar edits) in `ecosystem/fret-gizmo/src/light_radius_plugin.rs`

     Known gaps (future-facing):
     - Host-side property source contract is read-only today (ADR 0152). Writes remain host-owned via `GizmoCustomEdit` (no direct write API).
     - 3D picking primitives / acceleration (Godot-style collision + BVH) for complex gizmos.
     - Engine/editor undo/redo coalescing integration for `custom_edits` (framework support is still evolving).

### Roadmap (suggested, editor-first)

This is a pragmatic path that keeps core stable while opening extensibility points:

1. **MVP: reliable transform manipulator**
   - Translate/Rotate/Scale/Universal + snapping + robust redraw (no first-frame "click to show").
2. **MVP+: production-feel polish**
   - Precision modifier inputs + richer style controls + more overlap tests for Universal.
3. **Extensibility milestone**
   - Introduce a gizmo plugin contract with explicit picking primitives + draw lists, inspired by Godot's `EditorNode3DGizmoPlugin`
     (contract + manager skeleton implemented; integration + 3D picking pending).

## Notes / open design questions

- **Mode taxonomy**: ImGuizmo and transform-gizmo both expose a *fine-grained* operation set. Fret currently exposes
  coarse `GizmoMode`. If we want long-term parity, consider evolving `GizmoConfig` from "one mode enum" into a
  bitmask/flags set (while keeping a simple default constructor), and keep policy in ecosystem/app code.
- **Float precision**: transform-gizmo uses f64 for math; Fret uses f32. For editor workflows with huge worlds,
  we may need an internal f64 path (even if the public API stays f32) or a "scene units" policy.

## Godot lens (editor extensibility + picking topology)

This audit is scoped to ImGuizmo + transform-gizmo for parity, but Godot is a useful third reference for
how "real" game editors scale gizmos beyond the core transform manipulator.

Key Godot implementation traits (source anchors):

- **Plugin-based extensibility**: `EditorNode3DGizmoPlugin` provides callbacks for draw + interaction
  (`repo-ref/godot/editor/scene/3d/node_3d_editor_gizmos.h`).
- **Explicit begin/set/commit lifecycle**: `begin_handle_action`, `set_handle`, `commit_handle` mirror
  the "Begin/Update/Commit/Cancel" phases we already model in Fret (`GizmoPhase`).
- **Picking via collision geometry + BVH**: gizmos register collision segments/meshes and maintain an
  internal BVH (`collision_segments`, `collision_meshes`, `_update_bvh`) rather than relying on draw
  topology alone.
- **Rendering as 3D instances/layers**: gizmo meshes are real render instances on a dedicated layer,
  with optional "on top" state (`VISIBLE/HIDDEN/ON_TOP`) rather than a pure UI overlay.

Implication for Fret:

- Fret's engine-pass 3D draw lists + explicit interaction phases are compatible with the Godot-style
  approach, but a future "custom gizmo/plugin" surface likely needs an explicit **picking primitive**
  contract (segments/triangles or analytic shapes) in addition to "draw triangles/lines".
