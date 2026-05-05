# ImUi Debug Draw Triangle Mesh v1 Milestones

Status: Closed.

## M0 - Scene Contract

Exit criteria:

- `SceneOp` remains `Copy`.
- A fixed vertex-color triangle and textured triangle op exist.
- Validation and fingerprinting account for all per-vertex data.

Result: Complete.

## M1 - Renderer Wiring

Exit criteria:

- WGPU encodes vertex-color triangles through the vertex-color pipeline.
- WGPU encodes image triangles through the image pipeline.
- Focused tests verify three encoded vertices with expected position, UV, color, opacity, and image.

Result: Complete.

## M2 - IMUI Authoring

Exit criteria:

- IMUI exposes a `DebugDrawVertex` authoring type.
- Indexed triangle mesh helpers record one command and lower to scene triangles at paint time.
- Smoke tests prove the public API compiles.

Result: Complete.

## M3 - Evidence

Exit criteria:

- ADR 0002 and implementation alignment describe the triangle primitive surface.
- Workstream/audit indexes record the closed follow-on and residual raw DrawList gaps.
- Layering and source gates remain green.

Result: Complete.
