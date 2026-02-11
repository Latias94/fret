# ADR 0003: Platform Boundary (winit runner)

Status: Superseded (see ADR 0090-platform-backends-native-web.md)

Note: This ADR reflected an early desktop-only split. The current implementation separates
portable contracts (`fret-platform`) from per-target backends (`fret-platform-native`,
`fret-platform-web`) while keeping winit glue in runner/backend crates.

## Context

We must support:

- Windows/macOS/Linux (winit),
- multiple OS windows and surfaces (tear-off docking),
- future wasm/WebGPU and potentially mobile.

We also want to avoid hardwiring a specific event loop implementation into UI core or renderer.

## Decision

The desktop implementation is split across:

- `fret-platform`: portable platform I/O contracts (clipboard, external drop payloads, file dialogs,
  open-url). This crate is intentionally backend-agnostic.
- `fret-runner-winit`: the winit glue (event mapping + optional AccessKit adapter).
- `fret-runner-winit`: the reusable winit platform adapter that maps `winit` events into `fret-core`
  events (shared by desktop and wasm/web backends). The concrete mapping surface is exposed as
  `fret_runner_winit::WinitPlatform`.
- `fret-launch`: the concrete desktop glue that owns the event loop, drains `App`
  effects, and drives presentation via `fret-render`.

Keep the runner boundary responsible for:

- window lifecycle (create/close, mapping OS ids to `AppWindowId`),
- translating OS events into `fret-core` events (delegating the pure mapping layer to `fret-runner-winit`),
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

- Move more logic from `fret-demo` into `fret-runner-winit` once APIs stabilize.
- Add clipboard/IME/drag-and-drop as platform services exposed via effects.
