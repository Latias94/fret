# ADR 0136: Pointer Click Count and Double-Click Semantics

Status: Proposed

## Context

Multiple Fret components need "double click" semantics with consistent behavior across desktop and
web (via winit):

- Plots: ImPlot-style `LMB double-click` to fit/reset the view.
- Text/editor widgets: future word/line selection, multi-click behaviors.
- Viewports/tools: potential double-click focus/fit behaviors.

If click counting is implemented ad-hoc in widgets, behavior tends to diverge and regress:

- Dragging can be misinterpreted as a click (especially with small pointer motion).
- Platform-specific click timings and thresholds leak into component policy.
- It becomes difficult to keep UX consistent across desktop and wasm.

Fret already centralizes platform normalization in runners (e.g. keyboard normalization in ADR 0018).
Pointer click counting follows the same philosophy.

## Decision

### 1) `PointerEvent::{Down,Up}` carries a normalized `click_count`

Add a `click_count: u8` field to:

- `fret-core::PointerEvent::Down { .. }`
- `fret-core::PointerEvent::Up { .. }`

Semantics:

- `click_count == 1` for a single click, `2` for double click, etc.
- The count only increments for **true clicks**: a press + release sequence that does not exceed a
  small movement threshold ("click slop").
- Drag/pan/box-select should not pollute click sequences.

### 2) The platform runner is responsible for click counting

The click count is computed by the runner (currently `fret-runner-winit`) because it has access to:

- event timestamps (or equivalent monotonic time),
- pointer motion between press and release,
- platform-specific pointer button identities.

Widgets should treat `click_count` as an input signal and should not implement their own timing logic.

Viewport forwarding (`ViewportInputKind::{PointerDown,PointerUp}`) also carries `click_count` so
engine-facing tools can share the same semantics.

### 3) Thresholds are defined in logical pixels

The runner uses logical pixels (ADR 0017) for all pointer geometry:

- `click_slop_px`: maximum pointer travel allowed for a press/release to qualify as a click.
- `multi_click_max_delay`: maximum time between consecutive clicks for the count to increment.

These thresholds are part of runner normalization policy and may be made configurable later if needed.

## Consequences

- Components can implement double-click UX without depending on platform time APIs.
- Drag interactions no longer conflict with click counting.
- Input behavior becomes consistent across desktop and web (winit-backed).
- Some code must be updated to account for the new fields (pattern matches and test event
  construction).

## Implementation Notes

- Event surface: `crates/fret-core/src/input.rs`
- Runner normalization: `crates/fret-runner-winit/src/lib.rs` (`WinitInputState` click tracker)
- Plot alignment: `ecosystem/fret-plot` uses `click_count == 2` to support ImPlot-style fit on
  `LMB double-click`.
 - Viewport forwarding: `ViewportInputKind::{PointerDown,PointerUp}` carries `click_count` for
   tools/embeds.

## Future Work

- Consider exposing an optional "input map" configuration for plots (ImPlot-style `InputMap`), where
  double-click fit is one of the bindable actions.
