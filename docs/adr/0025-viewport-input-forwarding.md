# ADR 0025: Viewport Input Forwarding Contract

Status: Superseded (replaced by ADR 0132)

## Context

Note: this ADR described a legacy forwarding shape that has been removed. The implemented contract
is ADR 0132 (`ViewportInputEvent` with explicit units + mapping geometry).

Fret embeds engine viewports as `SceneOp::ViewportSurface` (ADR 0007). A real editor also needs to
forward input from the UI viewport widget to the host app/engine:

- pointer move/down/up + wheel,
- modifiers and pressed buttons (for camera navigation / drag behaviors),
- mapping from window logical pixels to viewport UV and target pixel coordinates,
- focus + modal gating (only the active viewport should receive tool input),
- multi-window tear-off (same contract across windows).

This is **UI infrastructure**. Tool systems (selection, gizmos, snapping, picking, undo coalescing)
remain app-owned and are explicitly out of scope (see ADR 0027).

References:

- Viewport surfaces via `RenderTargetId`:
  - `docs/adr/0007-viewport-surfaces.md`
- Frame lifecycle and submission ordering (where to forward into the engine):
  - `docs/adr/0015-frame-lifecycle-and-submission-order.md`
- Focus + command routing and modal blocking:
  - `docs/adr/0020-focus-and-command-routing.md`
- Keyboard vs text input split:
  - `docs/adr/0012-keyboard-ime-and-text-input.md`

## Decision

### 1) Viewport input events are data-only and core-defined

Define a core event shape that is independent of the engine implementation:

- `fret-core::ViewportInputEvent`
- `fret-core::ViewportInputKind`

Events include:

- `window: AppWindowId`
- `target: RenderTargetId`
- `uv: (f32, f32)` and `target_px: (u32, u32)`
- kind-specific data (buttons/modifiers/button/wheel delta)

### 2) Pointer move carries buttons + modifiers

To avoid ad-hoc “current state” queries during drags, pointer move must include:

- currently pressed mouse buttons
- current modifiers

This is carried by `PointerEvent::Move { position, buttons, modifiers }`.

### 3) Viewport mapping is explicit and reusable

Viewport widgets map `PointerEvent` → `ViewportInputEvent` using an explicit mapping type
(`ViewportMapping`), so that:

- the mapping logic is shared across viewports/panels/windows,
- resizing and fit modes remain deterministic.

### 4) Forwarding is effect-driven, not a direct engine call from widgets

Viewport widgets enqueue `Effect::ViewportInput(ViewportInputEvent)`; the platform runner drains
effects and delivers them to the host integration callback (`WinitAppDriver::viewport_input`).

This preserves the platform boundary (ADR 0003) and avoids coupling widgets to engine objects.

### 5) Focus and modal state gate viewport forwarding

Viewport input forwarding occurs only when:

- the viewport widget (or its panel) is focused, and
- no modal overlay blocks input (ADR 0011 + ADR 0020).

## Consequences

- Multi-window viewport tooling stays possible without changing input contracts.
- Engines can implement picking/tools/camera control without Fret embedding editor policy.
- Input forwarding remains compatible with future wasm/WebGPU platform limitations.

## Future Work

- Relative mouse / pointer lock for FPS-style camera controls.
- High-resolution scroll and trackpad gesture semantics (platform-specific).
- Support richer pointer devices (pen, pressure) as additive optional fields.

