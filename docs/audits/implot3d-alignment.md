# ImPlot3D Alignment (Fret Plot3D)

This document tracks *conceptual* parity between:

- ImPlot3D (reference: `repo-ref/implot3d` @ `5981bc5`)
- `fret-plot3d` (3D viewport widget + input forwarding; currently minimal)

The goal is not API compatibility. The goal is to replicate the **core UX and capabilities** of an
application-grade 3D plotting surface (rotate/pan/zoom + common plot items) while keeping Fret's
architecture constraints:

- Plot3D renders into an engine-owned `RenderTargetId` (not new 3D `SceneOp`s).
- The UI side remains portable (data + input forwarding only).
- The engine/driver owns GPU work submission via the engine render hook pipeline.

## Current State (What We Already Have)

- Embedded viewport surface (ADR 0097):
  - `Plot3dCanvas` emits `SceneOp::ViewportSurface` targeting an engine-owned `RenderTargetId`.
  - Input is forwarded as `Effect::ViewportInput(ViewportInputEvent)` using `ViewportMapping`.
  - Evidence: `ecosystem/fret-plot3d/src/retained.rs`
- End-to-end runner hook:
  - The demo allocates a `ViewportRenderTarget` and records a wgpu render pass every frame.
  - Evidence: `apps/fret-examples/src/plot3d_demo.rs` (`record_engine_frame`)

This proves the *plumbing* (targets + input forwarding + submission ordering). It does **not**
implement plotting yet.

## ImPlot3D Feature Inventory (Reference)

ImPlot3D is an ImGui drawlist-based immediate-mode library. Key features to replicate:

- Interactive 3D camera: rotate / pan / zoom
  - Flags: `ImPlot3DFlags_NoRotate`, `ImPlot3DFlags_NoPan`, `ImPlot3DFlags_NoZoom`
  - Reference: `repo-ref/implot3d/implot3d.h`
- Plot items (not exhaustive):
  - Line / Scatter / Surface / Mesh / Text / Image
  - Reference: `repo-ref/implot3d/implot3d.h` (e.g. `PlotLine`, `PlotSurface`)
- Legend:
  - Toggle visibility per item, hover highlight policies
  - Reference: `repo-ref/implot3d/implot3d_internal.h` (legend state)
- Colormaps:
  - Built-in + user-defined, continuous sampling
  - Reference: `repo-ref/implot3d/implot3d.h` (colormap API)
- Axis constraints:
  - Lock min/max; zoom constraints
  - Reference: `repo-ref/implot3d/implot3d.h` (axis flags + zoom constraints)

Important constraint called out by ImPlot3D:

- 16-bit index pitfalls for dense meshes (recommend 32-bit indices).
  - Reference: `repo-ref/implot3d/README.md` ("Extremely Important Note")

## Alignment Matrix (Fret vs ImPlot3D)

| Capability | ImPlot3D | Fret today | Status | Notes / pointers |
| --- | --- | --- | --- | --- |
| Engine-owned render target | N/A (ImGui drawlist) | `RenderTargetId` + `SceneOp::ViewportSurface` | ✅ | `ecosystem/fret-plot3d/src/retained.rs` |
| Input forwarding | ImGui IO | `ViewportInputEvent` via `ViewportMapping` | ✅ | `crates/fret-core/src/input.rs`, `crates/fret-ui/src/retained_bridge.rs` |
| 3D camera (orbit/pan/zoom) | ✅ | ❌ | ❌ | Needs a Plot3D engine model + input mapping policy |
| Axes/grid/ticks/labels | ✅ | ❌ | ❌ | Needs a 3D axis layout + text overlays in the engine render target |
| Line plots | ✅ | ❌ | ❌ | Needs a Plot3D renderer path (lines) + LOD policies |
| Scatter plots | ✅ | ❌ | ❌ | Similar to line, but sprite/point rendering |
| Surface/mesh | ✅ | ❌ | ❌ | Requires triangle/mesh pipelines + 32-bit indices policy |
| Legend (toggle/highlight) | ✅ | ❌ | ❌ | Likely a UI overlay in `fret-plot3d` + shared policy with 2D plots |
| Colormaps | ✅ | ❌ | ❌ | Reuse/align with `fret-plot` and `delinea` visual mapping taxonomy |
| Picking / hover hit testing | ✅ | ❌ | ❌ | Needs GPU picking or CPU BVH; output contract should be deterministic |

## Recommended Fret Architecture (How To “Replica” ImPlot3D)

Instead of copying ImPlot3D's immediate-mode API, implement the same capabilities through Fret's
two-tier architecture:

1) **UI tier (`fret-plot3d`)**: portable model + input mapping + output contract.

- Extend `Plot3dModel` to include:
  - camera state (orbit center, distance, yaw/pitch),
  - axes config (labels, ranges, grid visibility),
  - items (stable IDs + typed payloads: line/scatter/surface/mesh),
  - interaction locks (NoRotate/NoPan/NoZoom equivalents).
- Define `Plot3dOutput` (hovered item, picked point, camera transform, etc.).

2) **Engine tier (driver-owned wgpu renderer)**: deterministic rendering into the target.

- Provide a reusable renderer type (e.g. `Plot3dRenderer`) that the app/driver calls from
  `record_engine_frame`.
- Keep large data work budgeted and cacheable (similar spirit to `delinea` / ADR 0194):
  - progressive mesh uploads,
  - pixel-bounded LOD for dense line/point clouds,
  - stable per-item GPU resources keyed by IDs + revision counters.

This keeps Plot3D portable and makes “real 3D” depth-correct rendering possible without changing
`SceneOp` contracts (ADR 0097 rationale).

## Proposed Milestones (Practical, Testable)

- P0: Camera + axes + a single plot item type (scatter or line) + hover hit testing output.
  - Demo: replace the animated clear in `apps/fret-examples/src/plot3d_demo.rs` with a plotted dataset.
- P1: Legend + multiple item types + colormaps.
- P2: Surface/mesh rendering + 32-bit index stress harness (ImPlot3D's known pitfall).

