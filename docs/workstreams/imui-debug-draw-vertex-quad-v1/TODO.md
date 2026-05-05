# ImUi Debug Draw Vertex Quad v1 TODO

Status: Closed.

## Completed

- [x] Add portable `SceneOp::VertexColorQuad` and `SceneOp::ImageQuad`.
- [x] Add `UvPoint` as the arbitrary per-corner UV vocabulary.
- [x] Update scene validation, fingerprinting, and stack conformance.
- [x] Add WGPU vertex-color draw encoding and pipeline support.
- [x] Extend image vertices to carry tint color while preserving existing image draws.
- [x] Add renderer encoding tests for vertex order, corner colors, UVs, tint, opacity, and premul flag.
- [x] Add IMUI debug-draw helpers for `add_rect_filled_multi_color` and image quads.
- [x] Update ADR 0002 and implementation alignment evidence.

## Future Follow-Ons

- [ ] DrawList channel splitting and command metadata.
- [ ] Callback/user draw commands with a safe renderer boundary.
- [ ] More direct Dear ImGui porting sugar if real app code proves friction.
