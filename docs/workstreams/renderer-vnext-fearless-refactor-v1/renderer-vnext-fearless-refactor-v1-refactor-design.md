# Renderer vNext Fearless Refactor v1 — Refactor Design

Status: Draft (living design notes; the workstream and ADRs remain the source of truth)

Related:

- Workstream: `docs/workstreams/renderer-vnext-fearless-refactor-v1/renderer-vnext-fearless-refactor-v1.md`
- TODO tracker: `docs/workstreams/renderer-vnext-fearless-refactor-v1/renderer-vnext-fearless-refactor-v1-todo.md`
- Milestones: `docs/workstreams/renderer-vnext-fearless-refactor-v1/renderer-vnext-fearless-refactor-v1-milestones.md`
- Internal-plan substrate: `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
- Internal refactor gates: `docs/adr/0201-renderer-internals-modularization-and-gates-v1.md`

## 0) Goal

Make `crates/fret-render-wgpu`’s renderer internals **fearlessly evolvable** while preserving all
public scene semantics.

We optimize for:

- reviewable diffs (clear subsystem ownership),
- reversible steps (small, landable increments),
- always-run regression protection (conformance + WebGPU validation),
- and explicit evidence anchors per milestone.

## 1) Non-negotiable invariants (contract-preserving)

1. `SceneOp` order remains authoritative (no reordering across primitive kinds).
2. Budget decisions and degradations remain deterministic and observable.
3. Color/compositing rules remain linear + premultiplied, with explicit output encoding.
4. wasm/WebGPU remains supported (Naga + optional browser/Tint validation).
5. No new crate boundary violations (keep `wgpu` out of contract crates).

## 1.1) Slice points (what we refactor vs freeze)

This workstream is “fearless” in implementation, but conservative in contract.

We intentionally slice the problem along stable seams:

- **Contract seam (frozen without an ADR):**
  - `crates/fret-core` scene semantics (`SceneOp`, clip/mask/effect stacks, `Paint`/`MaterialId` budgets + degradations).
  - Any semantic change must be introduced via a small ADR and guarded by the smallest possible conformance test.
- **Backend surface seam (frozen for apps):**
  - Public renderer entrypoints and config flags remain stable; refactors happen behind them.
- **Internal structure seam (free to refactor):**
  - `crates/fret-render-wgpu` internals may be reorganized aggressively (encode/plan/execute, GPU globals/buffers/caches),
    as long as gates pass and observable output remains equivalent.
- **Evidence seam (perf-driven changes):**
  - Key-space growth (pipeline variants, bind-group layout shape changes, new per-draw payloads) is only allowed when a
    reproducible perf bundle shows the hotspot and the mitigation is bounded.

## 2) Target internal decomposition (wgpu backend)

The `Renderer` type remains the public surface for the backend crate, but delegates to internal
subsystems.

Proposed module ownership (names illustrative):

- `renderer::encode` — `Scene` → `SceneEncoding`
- `renderer::plan` — `SceneEncoding` → `RenderPlan`
- `renderer::execute` — `RenderPlan` → `wgpu::CommandBuffer`
- `renderer::gpu::globals` — stable GPU handles used everywhere (samplers, catalog views, layouts)
- `renderer::gpu::buffers` — buffer capacity growth, ring rotation, upload helpers
- `renderer::gpu::bind_groups` — bind group caches (images/render targets/mask overrides)
- `renderer::caches` — intermediate pool, clip-path mask cache, SVG cache, path cache
- `renderer::obs` — perf + diagnostics capture (snapshots, counters, trace spans)

The intent is to reduce “cross-cutting field edits” during refactors and make it explicit where a
change belongs.

## 3) Migration plan (staged, landable)

### Stage 1 — GPU globals + cacheable handles (no behavior change)

Motivation:

- Many code paths recreate the same `wgpu::Sampler` / `TextureView` “just to rebuild a bind group”.
- Centralizing stable GPU handles reduces churn and makes ownership explicit.

Deliverables:

- Store “stable” GPU handles on the renderer (e.g. material catalog D2Array view + sampler).
- Ensure bind group rebuild paths reuse these handles.
- Keep all existing tests and perf snapshots consistent (no semantic changes).

Gates:

- `cargo test -p fret-render-wgpu --lib`
- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`

### Stage 2 — Consolidate buffer lifecycle in `gpu::buffers`

Motivation:

- Buffer capacity growth logic exists in multiple places (uniforms, render space, clip/mask stacks).

Deliverables:

- A single “buffer manager” responsible for:
  - capacity tracking,
  - (re)creating buffers,
  - rebuilding dependent bind groups,
  - clearing caches that depend on bind group identity.

Gates:

- Anchor conformance tests + WebGPU validation (ADR 0201).

### Stage 3 — Bind group caches as explicit services

Motivation:

- `image_bind_groups`, `viewport_bind_groups`, `uniform_mask_image_bind_groups` have similar patterns.

Deliverables:

- Extract a `BindGroupCache` layer with explicit keying (revision IDs + sampling mode).
- Make cache invalidation explicit and local to the service.

### Stage 4 — Execute loop isolation (RenderPlan executor)

Motivation:

- The pass-recording loop is large and hard to review.

Deliverables:

- Move `RenderPlanPass` execution into a dedicated module, keeping the minimal set of shared inputs:
  - GPU globals,
  - GPU buffers,
  - pipelines,
  - caches/pools.

### Stage 5 — Optional deeper splits (only if it reduces churn)

Examples:

- pipeline registry (per-format lazy creation),
- effect pass implementations (blur, warp, color adjust) as per-pass recorders,
- shader source plumbing organization.

## 4) Evidence discipline (what to record)

For each stage:

- Evidence anchors (1–3) pointing to key code paths.
- Commands used for gates.
- If perf is relevant, record a small headless baseline diff (same flags/frames).
