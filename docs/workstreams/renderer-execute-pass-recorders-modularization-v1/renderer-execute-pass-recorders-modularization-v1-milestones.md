# Renderer Execute Pass Recorder Modularization v1 — Milestones

This workstream is intentionally “mechanical”: reorganize code while keeping semantics unchanged.

## Milestone 1 — Executor routing

Completion criteria:
- All `RenderPlanPass` variants are recorded through `RenderSceneExecutor`.
- No `ExecuteCtx`-style secondary context object is required.
- Conformance tests stay green.

## Milestone 2 — Shared helpers extracted

Completion criteria:
- Target selection (output/intermediate/mask) is centralized in `render_scene/helpers.rs`.
- Scissor mapping and bind-group picking are centralized (no duplicated logic in `execute.rs`).
- `execute.rs` focuses on orchestration and data uploads.

## Milestone 3 — Recorder parameter surface reduction

Completion criteria:
- `execute.rs` builds stable “input bundles” once per frame and passes references through.
- `RecordPassResources` exists and is used by `RenderSceneExecutor::record_pass`.
- Pass-specific args are grouped into structs where it reduces churn.

## Milestone 4 — Decision: `SceneDrawRange` ownership

Completion criteria:
- Choose one of:
  - keep `Renderer::record_scene_draw_range_pass` as the canonical impl (explicit args struct), or
  - migrate it into `render_scene/recorders/*` for uniformity
- Document the choice and rationale in the workstream TODO.

## Gates

Minimum gates (per change):
- `cargo check -p fret-render-wgpu`
- `cargo nextest run -p fret-render-wgpu --test viewport_surface_metadata_conformance`
- `cargo nextest run -p fret-render-wgpu --test mask_image_conformance`
- `cargo nextest run -p fret-render-wgpu --test composite_group_conformance`
- `cargo nextest run -p fret-render-wgpu --test clip_path_conformance`
- `cargo nextest run -p fret-render-wgpu --test materials_conformance`
- `cargo nextest run -p fret-render-wgpu --test text_paint_conformance`
- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`

