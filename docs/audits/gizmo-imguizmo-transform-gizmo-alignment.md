# Audit: 3D Transform Gizmo Alignment (ImGuizmo + transform-gizmo)

This audit compares Fret's `ecosystem/fret-gizmo` against two widely used reference implementations:

- **ImGuizmo** (Dear ImGui overlay gizmo): `repo-ref/ImGuizmo`
- **transform-gizmo** (Rust gizmo with sub-gizmos): `repo-ref/transform-gizmo`

Goal: enumerate **feature-level parity** (what users can do + how it feels), mark the **current alignment
status** per item, and provide an ordered list of **next alignment targets**.

Note: Fret intentionally renders gizmos as **engine-pass 3D geometry** (depth-tested) rather than a pure
UI overlay, per ADR 0139. That means some "rendering topology" items will be *intentionally different*
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
  - `GizmoInput { cursor_px, hovered, drag_started, dragging, snap, cancel }`
- Multi-target support is built-in:
  - `GizmoUpdate` returns `updated_targets: Vec<GizmoTarget3d>` and includes begin/update/commit/cancel phases.

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
| Constant pixel size | yes (clip-space sizing knobs) | yes (`scale_factor`, `pixels_per_point`) | yes (`axis_length_world(...)`) |

## Feature alignment matrix (detailed)

### 0) API and data model

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| Target representation | 4x4 matrix | TRS (`Transform`) | **Aligned (by intent)** | Fret uses TRS (`Transform3d`) to avoid decomposing matrices. `ecosystem/fret-gizmo/src/gizmo.rs` (`Transform3d`). |
| Decompose/recompose helpers | Yes (`DecomposeMatrixToComponents`, `Recompose...`) | N/A | **Not implemented** | Fret does not provide matrix decomposition utilities; editor apps can add them if needed. |
| "Delta matrix" output | Yes (`deltaMatrix`) | No (returns result + updated transforms) | **Not implemented** | Fret returns semantic deltas (`GizmoResult`) and updated targets, not a 4x4 delta matrix. |
| Semantic delta output | Partial | Yes (`GizmoResult`) | **Aligned** | `GizmoResult::{Translation,Rotation,Scale}` includes `delta` + `total`. |
| Multi-target update in one call | External | Yes | **Aligned** | `GizmoUpdate.updated_targets: Vec<GizmoTarget3d>`. |
| Begin/update/commit/cancel phases | No | Partial (implicit) | **Aligned (Fret-specific)** | `GizmoPhase` enables clean undo grouping; host decides persistence. |
| "IsOver / IsUsing" queries | Yes (`IsOver`, `IsUsing`) | Yes (`is_focused`, active subgizmo) | **Partially aligned** | Fret exposes `GizmoState { hovered, active }` but no dedicated helper methods yet. |
| Fine-grained operation selection | Yes (bitmask) | Yes (`EnumSet`) | **Not implemented** | Fret currently selects a single coarse mode. |

### A) Core transform handles

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| Translate axis X/Y/Z | Yes | Yes | **Aligned** | Axis handles exist + picking + axis constraint. `ecosystem/fret-gizmo/src/gizmo.rs` (`pick_translate_handle`, `begin_translate_drag`). |
| Translate plane XY/XZ/YZ | Yes | Yes | **Aligned** | Plane quads + picking. `pick_translate_handle` + `translate_plane_quad_world`. |
| Translate "screen-plane" (center handle) | Yes (screen component) | Yes (`TranslateView`, but implemented as a view-plane in code) | **Aligned** | Fret's center handle (`TranslateHandle::Screen`) constrains motion to the camera-facing plane at the gizmo origin. `translate_constraint_for_handle` handle id `10`. |
| Translate "depth" (move toward/away camera) | No (not explicit) | No (not in core modes) | **Not implemented** | Optional future feature (sometimes called "dolly" translation). Not required for parity with these two references. |
| Rotate axis X/Y/Z rings | Yes | Yes | **Aligned** | Ring drawing + pick based on distance-to-segment. `draw_rotate_rings`, `pick_rotate_axis`. |
| Rotate around view axis (screen ring) | Yes (`ROTATE_SCREEN`) | Yes (`RotateView`) | **Aligned** | `show_view_axis_ring` + handle id 8, rendered as an outer ring (`view_axis_ring_radius_scale`). `pick_rotate_axis` view ring path, `begin_rotate_drag` view-axis mode. |
| Arcball rotation | No | Yes (`Arcball`) | **Aligned (basic)** | Fret supports arcball free-rotation in `GizmoMode::Rotate` via `GizmoConfig::show_arcball` and emits `GizmoResult::Arcball { delta, total }` (quat-based), matching transform-gizmo’s contract shape. |
| Scale axis X/Y/Z | Yes | Yes | **Aligned** | Axis scaling is supported and axis picking matches the rendered end boxes (not the whole shaft). `pick_scale_handle`, `begin_scale_drag`. |
| Scale uniform (center handle) | Partial (via `SCALEU`/center) | Yes (`ScaleUniform`) | **Aligned** | Center uniform handle id 7, with pivot-respecting translation compensation. |
| Scale plane XY/XZ/YZ | No | Yes (`ScaleXY/XZ/YZ`) | **Aligned** | Plane scale handles (XY/XZ/YZ) are supported in `Scale` mode. `pick_scale_handle` + `begin_scale_drag` + scale update path. |
| Bounds / box scaling | Yes (`BOUNDS`, `localBounds`, `boundsSnap`) | No | **Aligned (basic)** | Fret supports a bounds-style box in `GizmoMode::Scale` gated by `GizmoConfig::show_bounds`, with corner + face handles. Selection bounds can be derived from `GizmoTarget3d::local_bounds` (local AABB) when provided, otherwise it falls back to translations (`ecosystem/fret-gizmo/src/gizmo.rs`). Gaps: multi-selection scaling semantics are TRS-only (no shear). |

### B) "Universal" / multi-mode behavior

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| Combine translate + rotate | Yes | Yes | **Aligned** | Fret `GizmoMode::Universal` overlays translate + rotate and resolves pick conflicts. `pick_universal_handle`. |
| Combine translate + rotate + scale | Yes (`UNIVERSAL`) | Yes (via mode set) | **Aligned (with known gaps)** | Fret Universal supports axis scaling alongside translate+rotate when `universal_includes_scale` is enabled. Note: uniform scale (center handle) remains exclusive to `Scale` mode to avoid center-handle conflicts with view-plane translation. |
| Fine-grained mode toggles | Yes (bitmask `OPERATION`) | Yes (`EnumSet<GizmoMode>`) | **Not implemented** | Fret uses coarse `GizmoMode` and hard-coded handle sets. Consider evolving to a bitmask/flags surface (without committing policy to `fret-ui`). |

### C) Orientation, pivot, multi-selection

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| World vs local orientation | Yes | Yes | **Aligned** | `GizmoOrientation::{World,Local}` and axis generation. `axis_dirs(...)`. |
| Pivot at active selection | Implicit (matrix) | Yes | **Aligned** | `GizmoPivotMode::Active`. |
| Pivot at selection center | External | Yes | **Aligned** | `GizmoPivotMode::Center`. |
| Multiple targets updated per drag | External | Yes | **Aligned** | `GizmoUpdate.updated_targets` returns all updated targets each frame. |
| Rotation of multiple targets around pivot | External | Yes | **Aligned** | Rotation updates translate+rotate around pivot. `GizmoMode::Rotate` update path. |
| Scale of multiple targets around pivot | External | Yes | **Partially aligned** | Supported, but the exact policy differs from transform-gizmo's per-mode semantics (no plane scale, no view-axis translate). Needs behavior audit tests. |

### D) Snapping and precision controls

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| Toggle snapping during drag | Yes | Yes | **Aligned** | `GizmoInput.snap` gates step snapping. |
| Translation snap step | Yes | Yes | **Aligned** | `translate_snap_step: Option<f32>`. |
| Rotation snap step | Yes (degrees) | Yes (radians) | **Aligned** | `rotate_snap_step_radians: Option<f32>`; note unit differences vs ImGuizmo. |
| Scale snap step | Yes | Yes | **Aligned** | `scale_snap_step: Option<f32>`. |
| Snap visualization (rotation ticks) | No (varies) | Yes (visuals) | **Partially aligned** | Fret renders rotate tick marks when snapping is active. `draw_rotate_feedback`. Translation/scale snap visuals are not implemented. |
| Bounds snap | Yes (`boundsSnap`) | No | **Aligned (basic)** | `GizmoConfig::bounds_snap_step` snaps bounds scaling to per-axis extent steps (ImGuizmo-style), gated by `GizmoInput.snap` (`ecosystem/fret-gizmo/src/gizmo.rs`). |

### E) Interaction lifecycle and "editor feel"

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| Hover highlight | Yes | Yes | **Aligned** | `GizmoState.hovered` + per-handle color selection in draw helpers. |
| Active handle lock (no retarget mid-drag) | Yes | Yes | **Aligned** | Hover picking is disabled while `state.active.is_some()`. |
| Drag start threshold | Partial | Yes (subgizmo pick gating) | **Aligned** | `drag_start_threshold_px` arms drag and emits `Begin` after movement. |
| Begin/Update/Commit phases | No (implicit) | Partial | **Aligned (Fret-specific)** | `GizmoPhase::{Begin,Update,Commit,Cancel}` enables undo boundaries cleanly. |
| Cancel interaction (Escape) | External | External | **Aligned (Fret-specific)** | `GizmoInput.cancel` → `GizmoPhase::Cancel`, restore is host-owned. |
| Picking priority heuristics (avoid mis-click) | Yes (tuned) | Yes (subgizmo ordering) | **Partially aligned** | Fret has a translate picking ladder (`pick_translate_handle`) and Universal bias (`pick_universal_handle`), but still lacks a unified, configurable priority policy across translate/rotate/scale. |

#### Picking priority ladders (explicit per-mode audit)

| Mode | Reference behavior (high level) | Fret status | Evidence / notes |
| --- | --- | --- | --- |
| Translate | Center/view-plane > plane interior > axis (avoid axis stealing near origin). | **Partially aligned** | Center handle early-out exists in `pick_translate_handle`; plane-vs-axis priority is still heuristic-based. |
| Translate | Center/view-plane > plane interior > axis (avoid axis stealing near origin). | **Aligned (basic)** | Explicit early-outs exist for center handle and plane interior: `translate_center_handle_wins_near_origin`, `translate_plane_inside_wins_over_axis_when_both_hit` in `ecosystem/fret-gizmo/src/gizmo.rs`. |
| Rotate | Prefer the ring the user aims at; disambiguate view ring vs axis rings; avoid "wrong ring" when rings overlap in screen space. | **Aligned (basic)** | `pick_rotate_axis` has explicit view-ring vs axis-ring disambiguation (axis "strong hit" wins), backed by `rotate_view_ring_does_not_steal_axis_ring_when_both_hit`. Axis rings also fade out (and become unpickable) when edge-on: `rotate_ring_fade_hides_edge_on_axis_ring`. |
| Scale | Prefer axis end boxes when cursor is on the shaft; prefer center uniform only when close to origin; avoid fighting Universal overlays. | **Aligned (basic)** | Plane scale (XY/XZ/YZ) is implemented in `Scale` mode, and bounds handles win when overlapping scale axis end boxes: `scale_prefers_bounds_face_handle_over_axis_end_box_when_overlapping`. |
| Universal | Protect translate center/planes and scale end boxes; otherwise disambiguate rotate vs scale vs translate deterministically. | **Aligned (with known gaps)** | `pick_universal_handle` enforces a priority ladder (translate center/plane interior wins; scale end boxes win; otherwise tie-break rotate > scale > translate). Known gaps: more coverage for orthographic + near-plane + behind-camera overlap cases. |

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
| Left-handed vs right-handed | N/A (depends) | Yes (detects) | **Not implemented** | Fret assumes a `Mat4` VP and a depth convention (`DepthRange`), but does not explicitly model handedness. |
| Behind-camera culling / stability | Yes | Yes | **Partially aligned** | `project_point` rejects behind-camera points (`clip.w <= 0`) to avoid unstable picking; basic regression tests exist. Remaining gaps: broader coverage for rotate/scale and near-plane clipping behavior. |
| Numeric stability at large scales | Mixed | Better (f64) | **Partially aligned** | Fret uses `glam` f32 types; consider f64 internal math if large-world support becomes a requirement. |

### G) Rendering, styling, and customization

| Feature | ImGuizmo | transform-gizmo | Fret status | Evidence / notes |
| --- | --- | --- | --- | --- |
| Constant pixel size | Yes | Yes | **Aligned** | `axis_length_world(...)` maps desired pixel size to world length. |
| Configurable line thickness | Yes (`Style`) | Yes (`stroke_width`) | **Aligned** | `line_thickness_px` used by demo shader. |
| Per-axis colors + hover color | Yes | Yes | **Aligned** | `GizmoConfig::{x_color,y_color,z_color,hover_color}`. |
| Occluded feedback | N/A (overlay) | N/A (overlay) | **Aligned (Fret enhancement)** | `DepthMode::Ghost` + `show_occluded` and `occluded_alpha`. |
| Depth-tested gizmo geometry | No | No | **Intentional divergence / enhancement** | Fret expects engine-pass depth testing (ADR 0139). |
| Axis flip, axis masking, axis/plane fade limits | Yes | Partial | **Aligned (with known gaps)** | Fret supports `allow_axis_flip`, `axis_mask`, `axis_fade_px`, `plane_fade_px2` in `GizmoConfig`. Rotate rings use a separate view-angle fade window (`rotate_ring_fade_dot`) to avoid edge-on rings stealing interaction. |
| View gizmo (camera cube) | Yes (`ViewManipulate`) | No | **Not implemented** | Separate feature; can be built as another tool using similar math. |
| Grid draw helper | Yes (`DrawGrid`) | No | **Not implemented** | Could be added as a separate "debug draw" tool (not necessarily in `fret-gizmo`). |

## What to align next (recommended order)

This is a suggested sequence for reaching "mature editor" parity without over-design.

### P0 (correctness + UX baseline)

1. **Stability + picking UX audit (lock-in editor feel)**
   - Motivation: most user pain comes from drift/overshoot/mispicks, not missing modes.
   - Outcome: add targeted tests for translate/rotate/scale drag stability + explicit picking priority ladder.
2. **Universal: scale semantics + picking**
   - Fret already supports axis scale in `Universal` (via `GizmoConfig::universal_includes_scale`).
   - Decide whether `Universal` should also include uniform scale, and tighten picking rules so scale doesn't fight translate planes / rotate rings.
3. **Picking priority ladder**
   - Explicit priority ordering (e.g. active > hovered; center/plane/axis; rotate view ring) with tunable bias.
4. **Projection edge-case audit**
   - Validate orthographic, near-plane clipping, behind-camera cases with tests (project/unproject + pick stability).

### P1 (feature breadth parity)

1. **Arcball rotation**
   - Bring parity with transform-gizmo's `Arcball` (trackball) option.
2. **Plane scaling (XY/XZ/YZ)**
   - Parity with transform-gizmo; useful for non-uniform edits without axis-only drags.
3. **ImGuizmo-style behavior knobs**
   - Axis flip, axis mask, axis/plane fade limits (these are feel multipliers in dense scenes).

### P2 (ImGuizmo-specific extras)

1. **Bounds / box scaling (basic in place)**
   - Fret already implements a bounds-style box with corner + face handles, and supports `GizmoTarget3d::local_bounds`
     as the ImGuizmo `localBounds` equivalent input surface.
   - Next alignment targets: tighter multi-selection scaling semantics (pivot/anchor/axis constraints), and
     more mature visuals (thickness/AA/consistent occlusion feedback).
2. **View gizmo (camera cube)**
   - A separate tool; can be layered on the same math/picking substrate.

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
