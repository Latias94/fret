# ADR 0017: Multi-Window, Displays, and DPI Semantics

Status: Accepted

## Context

Fret must support:

- multiple OS windows (tear-off docks),
- multi-monitor setups,
- per-monitor DPI scaling (including moving a window between monitors),
- future wasm (single canvas) where “windows” may be logical, not OS-level.

Incorrect early assumptions about coordinate spaces and persistence lead to costly rewrites:

- layout drift on DPI changes,
- wrong input mapping in viewports,
- broken layout persistence across machines/monitors.

References:

- Core coordinate system contract (logical pixels):
  - ADR 0002
- Platform boundary and winit runner:
  - ADR 0003
- Zed/GPUI (non-normative):
  - display “visible bounds” excluding taskbar/dock and window default placement cascade:
    `repo-ref/zed/crates/gpui/src/platform.rs` (`PlatformDisplay::visible_bounds`),
    `repo-ref/zed/crates/gpui/src/window.rs` (`default_bounds`)
  - Windows DPI change handling (WM_DPICHANGED) and suggested-rect placement:
    `repo-ref/zed/crates/gpui/src/platform/windows/events.rs`

## Decision

### 1) Logical pixels are the core UI coordinate space

All UI geometry and input coordinates in `fret-core` are expressed in **logical pixels**.

Platform backends provide:

- `scale_factor` changes,
- window resize in logical dimensions.

The renderer converts logical pixels to physical pixels for surfaces.

### 2) Window state has two layers: logical layout + platform placement

Persistence is split:

- dock/layout graph is platform-agnostic and stored in logical terms (ADR 0013),
- OS window placement (position, size, monitor association) is stored separately and treated as best-effort.

Implementation note:

- The dock layout schema allows storing placement as optional metadata per logical window (`DockLayoutWindow.placement`)
  without making the dock graph depend on platform geometry.

### 3) Cross-monitor movement is supported by explicit scale factor events

When a window’s scale factor changes, the platform backend emits a scale-factor event and the UI:

- invalidates layout/paint,
- updates viewport input mapping and clip/scissor conversions accordingly.

## Consequences

- UI behavior remains consistent across multi-monitor setups.
- Layout persistence stays portable across machines, even when monitors differ.
- wasm can map multiple logical “windows” into a single canvas without changing core layout logic.

## Future Work

- Define an optional “monitor identity” field for best-effort window placement restoration.
- Add explicit policies for fractional scaling and snapping/pixel alignment.

## Implementation Notes

Prototype implementation (desktop runner + demo):

- Window move events are forwarded as data: `crates/fret-launch/src/runner/mod.rs`
- Best-effort window placement orchestration is available via effects:
  - `Effect::Window(WindowRequest::SetOuterPosition { .. })` (screen-space logical pixels)
  - capability gate: `ui.window_set_outer_position` (ADR 0054)
- Demo persists/restores `DockLayoutWindow.placement`: implemented in `apps/fret-demo` (entrypoints evolve; search for `DockLayoutWindow` usage).
