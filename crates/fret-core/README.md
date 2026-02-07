# `fret-core`

Portable, backend-agnostic core vocabulary for the Fret workspace.

This crate is intentionally **small** and **dependency-light**. It defines:

- stable IDs (windows, nodes, pointers, resources),
- geometry and pixel units,
- the cross-platform input/event model,
- scene recording primitives (portable display list),
- portability/capability-facing service traits and snapshots (no OS bindings).

Non-goals:

- no `wgpu`, no `winit`, no platform backends,
- no async runtime coupling (Tokio, etc.),
- no UI policy (that lives in ecosystem crates).

## Module ownership map (v1)

This is a living “where should this code go?” map.

- `ids` — stable identity types (`NodeId`, `AppWindowId`, `PointerId`, `ImageId`, ...)
- `geometry` — `Px`, `Point`, `Rect`, `Size`, transforms, and geometry helpers
- `input` — portable input events (`Event`, pointer/key/IME), plus normalization helpers
- `scene` — portable scene recording (`Scene`, `SceneOp`) consumed by the renderer layer
- `semantics` — portable semantics snapshot types (A11y bridge contract surface)
- `dock` — docking model/ops/persistence vocabulary (policy lives in ecosystem)
- `viewport` — viewport mapping types used for Tier A embedding (`RenderTargetId`, mapping helpers)
- `services` — portable service traits (host-provided, backend-agnostic)
- `window` — window metrics and coordinate space vocabulary (no windowing backend here)
- `file_dialog`, `svg`, `text`, `image`, `streaming`, `render_text` — portable data/config types used across layers
- `time`, `utf`, `cursor`, `layout_direction`, `panels`, `vector_path` — small supporting modules

If you need to add something new:

1. Put it in the narrowest module that “owns” the concept.
2. Prefer a new submodule over growing an unrelated one.
3. Avoid re-exporting it from `lib.rs` unless it is part of the stable vocabulary.
