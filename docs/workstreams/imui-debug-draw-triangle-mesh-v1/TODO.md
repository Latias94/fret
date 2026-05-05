# ImUi Debug Draw Triangle Mesh v1 TODO

Status: Closed.

## Completed

- [x] Add copyable `SceneMeshVertex`.
- [x] Add `SceneOp::VertexColorTriangle` and `SceneOp::ImageTriangle`.
- [x] Add validation, fingerprinting, and scene stack conformance coverage.
- [x] Add WGPU encoding for vertex-color and textured triangle ops.
- [x] Add renderer tests for per-vertex position, UV, color, opacity, and image routing.
- [x] Add IMUI `DebugDrawVertex` plus triangle mesh and image triangle mesh helpers.
- [x] Add compile smoke coverage for the public IMUI API.
- [x] Update ADR/audit/workstream indexes.

## Future Follow-Ons

- [ ] Batched renderer mesh resources if large editor overlays prove draw-call pressure.
- [ ] Per-command metadata / draw command introspection.
- [ ] Callback/user draw commands only if a contract-safe renderer extension point is designed.
