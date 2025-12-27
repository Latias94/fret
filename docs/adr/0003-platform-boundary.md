# ADR 0003: Platform Boundary (winit runner)

Status: Accepted

## Context

We must support:

- Windows/macOS/Linux (winit),
- multiple OS windows and surfaces (tear-off docking),
- future wasm/WebGPU and potentially mobile.

We also want to avoid hardwiring a specific event loop implementation into UI core or renderer.

## Decision

The desktop implementation is split across:

- `fret-platform`: winit-facing helpers (window registry) and the accessibility bridge
  (`accesskit_winit`).
- `fret-runner-winit-wgpu`: the concrete desktop runner that owns the event loop, maps winit events
  into `fret-core` events, drains `App` effects, and drives presentation via `fret-render`.

Keep the runner boundary responsible for:

- window lifecycle (create/close, mapping OS ids to `AppWindowId`),
- translating OS events into `fret-core` events,
- owning presentable surfaces (swapchains) and calling the renderer,
- draining `App` effects and executing platform operations.

### Invariants

- `fret-ui` does not depend on `winit`.
- `fret-core` does not depend on `wgpu` or `winit`.
- `fret-demo` is a sample binary; long-term it should shrink to wiring code around the runner boundary.

## Consequences

- We can swap platform backends (web, mobile) without rewriting UI core.
- Multi-window features remain centralized (no ad-hoc window creation inside widgets).

## Future Work

- Move more logic from `fret-demo` into `fret-platform` once APIs stabilize.
- Add clipboard/IME/drag-and-drop as platform services exposed via effects.

