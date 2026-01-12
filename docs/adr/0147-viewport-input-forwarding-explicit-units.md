# ADR 0147: Viewport Input Forwarding — Explicit Units, Mapping Geometry, and Scale Factor

Status: Proposed

## Context

Fret’s core coordinate-space contract is **logical pixels** (ADR 0017). The renderer converts logical
pixels to physical pixels using the window `scale_factor` (ADR 0002 / ADR 0017).

Viewports embed an engine-owned render target into the UI (`SceneOp::ViewportSurface`, ADR 0007) and
forward input to app/editor tooling via effects. The legacy contract (ADR 0025) forwards
`Effect::ViewportInputLegacy(ViewportInputEventLegacy)`.

The legacy event shape, `ViewportInputEventLegacy`, carries:

- `window: AppWindowId`
- `target: RenderTargetId`
- `uv: (f32, f32)`
- `target_px: (u32, u32)`
- `kind: ViewportInputKind` (buttons/modifiers/wheel)

This is sufficient for engines that only need **texture-space** (UV / target pixel) input.

However, editor-grade tooling (gizmos, selection, snapping, overlays, camera navigation) routinely
needs all of the following at the same time:

1. the **original cursor position in UI space** (logical pixels),
2. the **viewport mapping geometry** (content rect vs draw rect; fit mode),
3. the **target pixel size** (physical pixels),
4. the **window scale factor** (pixels-per-point / pixels-per-logical-pixel),
5. a clear, non-ambiguous statement of what “px” means at each layer.

Without these, consumers must “reconstruct” missing context (or reach into unrelated global state),
which leads to common, expensive-to-fix editor bugs:

- inconsistent gizmo sizes/thickness/pick radii across DPI settings,
- mismatched cursor units between UI input (logical) and engine targets (physical),
- double-scaling or missing scaling when composing with viewport fit modes (`Contain` / `Cover`),
- drift between tools because each re-derives mapping differently.

Zed/GPUI is a useful non-normative reference: it strongly distinguishes logical vs device pixels
(`Pixels` vs `DevicePixels`) and performs explicit conversions via `scale_factor`
(`repo-ref/zed/crates/gpui/src/geometry.rs`).

References:

- Core DPI semantics: `docs/adr/0017-multi-window-display-and-dpi.md`
- Viewport input forwarding v1: `docs/adr/0025-viewport-input-forwarding.md`
- Viewport gizmo boundary: `docs/adr/0139-viewport-gizmos-engine-pass-and-ui-overlay-boundary.md`

## Goals

1. Make the viewport input contract **unit-explicit** and self-contained enough for editor tooling.
2. Align viewport input semantics with ADR 0017 (logical pixels as the UI/input source of truth).
3. Preserve the existing `uv` + `target_px` fields as the engine-facing “texture space” affordance.
4. Enable gizmo/tool ecosystems (`ecosystem/*`) to consume viewport input without ad-hoc DPI glue.
5. Provide a migration path that does not require an immediate “flag day” refactor of all apps.

## Non-goals

- Defining editor tool policy (capture routing, snapping hotkeys, etc). That remains app/eco scope
  (ADR 0027 / ADR 0049 / ADR 0139).
- Replacing `ViewportMapping` math or the viewport fit modes; we reuse the existing mapping type.
- Introducing new `SceneOp` primitives.

## Decision

Introduce an explicit-units viewport input event (`ViewportInputEvent`) that carries explicit
mapping geometry and scale information, and forward it in parallel with a legacy event during a
transition period.

### 1) Add an explicit-units event type in `fret-core`

Add:

```rust
pub struct ViewportInputGeometry {
    /// The viewport widget bounds in window-local **logical pixels**.
    pub content_rect_px: Rect,
    /// The mapped draw rect in window-local **logical pixels** after applying `fit`.
    pub draw_rect_px: Rect,
    /// The backing render target size in **physical pixels**.
    pub target_px_size: (u32, u32),
    pub fit: ViewportFit,
    /// Pixels-per-point (a.k.a. window scale factor). Used to convert logical px → physical px.
    pub pixels_per_point: f32,
}

pub struct ViewportInputEvent {
    pub window: AppWindowId,
    pub target: RenderTargetId,
    pub geometry: ViewportInputGeometry,
    /// Cursor position in window-local **logical pixels** as reported by the UI event system.
    ///
    /// Note: `uv`/`target_px` may be clamped when pointer capture is active; this raw position is
    /// intentionally *not* clamped so tools can decide how to interpret off-viewport drags.
    pub cursor_px: Point,
    pub uv: (f32, f32),
    pub target_px: (u32, u32),
    pub kind: ViewportInputKind,
}
```

Normative unit rules:

- `Rect`/`Point` values in `geometry` and `cursor_px` are **logical pixels** (ADR 0017).
- `target_px_size` and `target_px` are **physical pixels** for the engine target.
- `pixels_per_point` is the conversion factor from logical → physical pixels.

### 2) Add a legacy effect variant in `fret-runtime`

Add:

- `Effect::ViewportInput(fret_core::ViewportInputEvent)`
- `Effect::ViewportInputLegacy(fret_core::ViewportInputEventLegacy)`

Rationale: adding these fields to the legacy event is a breaking change for any code constructing it
by struct literal (e.g. viewport widgets in ecosystem crates). Keeping a legacy variant allows a
controlled migration.

### 3) Add a legacy driver hook in `fret-launch`

Add a new optional hook (default no-op):

- `WinitAppDriver::viewport_input(&mut self, app: &mut App, event: ViewportInputEvent)`
- `WinitAppDriver::viewport_input_legacy(&mut self, app: &mut App, event: ViewportInputEventLegacy)`

Rationale: engine/editor integrations should be able to opt into the explicit-units event without
needing to parse the legacy shape or query global window metrics.

### 4) Update viewport widgets to emit the explicit-units event (alongside legacy during migration)

Viewport widgets that currently enqueue `Effect::ViewportInput` should be updated to also enqueue
`Effect::ViewportInput`.

The widget is the correct place to construct the explicit-units event because it has:

- the viewport bounds (`content_rect_px`),
- the mapped draw rect (via `ViewportMapping::map()`),
- the window-local cursor position from `PointerEvent`,
- the target pixel size (`target_px_size`),
- access to `WindowMetricsService` (via `UiHost::global`) to read `pixels_per_point`.

### 5) Recommended consumption pattern for tooling (non-normative)

Tools (e.g. `ecosystem/fret-gizmo`) should treat `ViewportInputEvent` as the canonical input:

- use `cursor_px` and `geometry.draw_rect_px` for UI-space deltas and hit testing,
- use `target_px` when interacting with engine-side pixel buffers,
- use `pixels_per_point` to keep “pixel-sized” affordances stable across DPI.

## Migration Plan

1. Implement explicit-units types and forwarding hooks in core/runtime/runner.
2. Update in-tree viewport widgets (e.g. `ecosystem/fret-plot3d`) to emit explicit-units events.
3. Update demos (e.g. `apps/fret-examples`) to consume explicit-units events for gizmo input and
   DPI-stable visuals.
4. Keep v1 forwarding for at least one cycle; add a deprecation note to ADR 0025 and/or to the v1
   types once migration is complete.

## Affected APIs / Surface Area

Core/runtime:

- `crates/fret-core/src/input.rs`: add `ViewportInputEvent`, `ViewportInputEventLegacy`, `ViewportInputGeometry`.
- `crates/fret-runtime/src/effect.rs`: add `Effect::ViewportInputLegacy(...)`.

Runner:

- `crates/fret-launch/src/runner/common.rs`: add `WinitAppDriver::viewport_input_legacy(...)`.
- `crates/fret-launch/src/runner/desktop/mod.rs` and `crates/fret-launch/src/runner/web.rs`:
  forward the new effect variant to the new driver hook.

Ecosystem / apps (opportunities to simplify):

- `ecosystem/fret-plot3d`: emit explicit-units input from `Plot3dCanvas` and stop forcing consumers to reconstruct
  cursor coordinates from `uv * target_px_size`.
- `ecosystem/fret-gizmo`: provide helpers that build `GizmoInput`/viewport rects from
  `ViewportInputEvent`, removing ad-hoc DPI scaling glue in demos.

## Consequences

Pros:

- Eliminates unit ambiguity at the viewport boundary; aligns tool math with ADR 0017.
- Makes DPI-correct gizmo sizing/picking a first-class outcome (aligned with transform-gizmo’s
  `pixels_per_point` surface).
- Reduces duplicated mapping logic in apps/ecosystem tooling.

Cons:

- Adds a parallel legacy surface that must be carried during migration.
- Slightly larger events (but still small and copyable).

## Alternatives Considered

1) **Keep v1 and require consumers to query `WindowMetricsService`**

- Pros: no new event type.
- Cons: leaks global coupling into every consumer; still leaves mapping geometry implicit; fails
  portability for hosts that do not expose window metrics consistently.

2) **Add fields to v1**

- Pros: single type.
- Cons: breaking change for struct-literal construction; forces a large immediate migration.

3) **Make all viewport input be physical pixels**

- Pros: simpler for engines.
- Cons: conflicts with ADR 0017 and forces UI/layout to operate in device pixels; harder to keep
  consistent across platforms and multi-window.
