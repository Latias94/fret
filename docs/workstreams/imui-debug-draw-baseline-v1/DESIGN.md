# ImUi Debug Draw Baseline v1

Status: Closed narrow product follow-on
Last updated: 2026-05-04

This lane adds the first reusable IMUI debug-draw surface by exposing a thin canvas-backed helper
from `fret-ui-kit::imui`. It is intended for diagnostics overlays, editor tooling, and small
immediate-mode inspection surfaces.

## Ownership

- `fret-ui-kit::imui` owns the facade API and the canvas-backed command translation.
- `crates/fret-ui` owns the underlying `Canvas` and scene emission mechanism.
- `fret-imui` stays thin and does not grow a renderer-specific debug API.

## Must-Be-True Outcomes

- Callers can describe a small debug-draw sequence in immediate-mode style.
- The helper can emit line, rect, filled-rect, and text primitives.
- The helper stays declarative by lowering into `Canvas`.
- Smoke tests keep the new facade surface and the existing adapter boundary green.

## Non-Goals

- No full Dear ImGui `DrawList` parity yet.
- Shape primitives are covered by `imui-debug-draw-shape-primitives-v1`.
- Stroke cap/join/dash policy is covered by `imui-debug-draw-stroke-style-v1`.
- Clip rect stack support is covered by `imui-debug-draw-clip-stack-v1`.
- No image overlays or custom blend/compositing policy.
- No runtime API changes.
