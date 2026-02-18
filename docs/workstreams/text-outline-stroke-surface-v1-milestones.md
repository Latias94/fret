# Text Outline/Stroke Surface v1 — Milestones

Status: Draft (workstream tracker)

Tracking files:

- `docs/workstreams/text-outline-stroke-surface-v1.md`
- `docs/workstreams/text-outline-stroke-surface-v1-todo.md`
- `docs/workstreams/text-outline-stroke-surface-v1-milestones.md`

## M0 — Design lock (bounded + portable)

Exit criteria:

- The contract surface is explicit:
  - where the API lives (`SceneOp::Text` vs dedicated op),
  - what outline vocabulary is supported (width + join/cap/miter, bounded),
  - and what it means under transforms and scale factors.
- Deterministic degradation rules are documented and testable.
- The chosen implementation strategy is stated and justified (vector path vs SDF/MSDF).

Evidence anchors:

- `docs/workstreams/text-outline-stroke-surface-v1.md`
- `docs/workstreams/text-outline-stroke-surface-v1-todo.md` (completed M0 items)

## M1 — Contract plumbing (core)

Exit criteria:

- `fret-core` exposes the contract types.
- Validation + fingerprinting are deterministic (no NaN divergence).

Evidence anchors:

- `crates/fret-core/src/scene/mod.rs`
- `crates/fret-core/src/scene/validate.rs`
- `crates/fret-core/src/scene/fingerprint.rs`

## M2 — Renderer implementation (wgpu)

Exit criteria:

- wgpu renderer renders outlines when supported.
- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu` is green.
- Fallback behavior is deterministic when the outline path is unsupported or budgeted.

Evidence anchors:

- `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs`
- `crates/fret-render-wgpu/src/renderer/shaders.rs`

## M3 — Conformance + perf gates

Exit criteria:

- GPU readback conformance test exists and is deterministic across scale factors.
- Perf gate exists only if needed to prevent a regression cliff.

Evidence anchors:

- `crates/fret-render-wgpu/tests/*_conformance.rs`
- `docs/workstreams/perf-baselines/*` (if perf gate is added)

## M4 — Adoption

Exit criteria:

- One real consumer uses outlined text (to validate ergonomics and discover missing semantics).
