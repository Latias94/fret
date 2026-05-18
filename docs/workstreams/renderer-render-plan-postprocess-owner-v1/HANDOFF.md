# Renderer Render Plan Postprocess Owner v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This lane is moving debug postprocess lowering helpers out of `render_plan.rs` and into a sibling
postprocess module. The intended code change is an ownership split only.

## Important Invariant

Do not reshape the render-plan pass model, compiler pipeline, or postprocess behavior in this lane.
Only the existing postprocess helper block belongs here.

## Future Work

If postprocess behavior expands, continue in a new follow-on unless it is a direct maintenance slice
for this owner.
