# Renderer vNext Fearless Refactor v1

Status: Draft (workstream notes only; ADRs remain the source of truth)

Tracking files:

- `docs/workstreams/renderer-vnext-fearless-refactor-v1-todo.md`
- `docs/workstreams/renderer-vnext-fearless-refactor-v1-milestones.md`

## 0) Why this workstream exists

Fret’s renderer must satisfy a hard set of constraints simultaneously:

- strict in-order compositing across mixed primitives and embedded engine viewports,
- predictable performance under multi-window editor workloads,
- portability to wasm/WebGPU and mobile GPUs,
- and a clean mechanism vs policy split (no component policy in the renderer contract layer).

We already have a stable public contract surface for ordered drawing (`fret-core::SceneOp`) plus
explicit stacks (transform/clip/mask/effect/compositing groups) with deterministic degradation
requirements.

This workstream exists to make the renderer implementation **fearlessly evolvable**:

- keep the public scene semantics stable,
- refactor the renderer internals to a plan/compile/execute architecture,
- and unlock new semantics via small, ADR-driven contract additions.

## 1) Scope

### In scope

- A renderer-internal refactor from “interpret and encode ops serially” toward:
  - `SceneOp` → `RenderPlan` (compile) → execute,
  - with explicit **sequence points** for effect/mask/compositing groups,
  - and deterministic budget/degradation decisions.
- New or clarified scene semantics where parity and portability demand it:
  - isolated opacity / `saveLayer(alpha)` (group alpha),
  - more general clip/mask sources (clip path, image mask) with bounded cost.
- “Paint/Material” evolution work that reduces long-term churn:
  - keep `Paint` and `MaterialId` as the controlled extensibility seam (Tier B),
  - ensure capability-gated fallbacks remain deterministic on wasm/mobile.

### Out of scope (v1)

- A general “user-provided WGSL” contract for components/plugins (Tier A/B trust model remains per ADR).
- A rewrite of `crates/fret-ui` authoring, layout, or event routing.
- Replacing the ordered `SceneOp` contract with a sorting-based scene model.

## 2) Invariants (do not break)

1. **Scene operation order is authoritative**
   - No cross-op reordering across primitive kinds.
   - Only adjacency-preserving batching is allowed.

2. **Transform + clip semantics are stable**
   - Clip entries are captured at push time; later transforms do not retroactively affect clips.

3. **All multi-pass features are bounded**
   - Anything that allocates intermediates must be bounded by explicit computation `bounds`,
     participate in budgets, and degrade deterministically.

4. **Wasm/mobile remain first-class**
   - Capability gating must have deterministic, documented fallbacks.
   - No “best effort” behavior that silently diverges per backend.

## 2.1) Always-run gates (pragmatic)

This workstream is intentionally “fearless but gated”. Before and after each milestone step:

- Crate layering: `python3 tools/check_layering.py`
- Renderer conformance anchors (GPU readback when available):
  - `crates/fret-render-wgpu/tests/affine_clip_conformance.rs`
  - `crates/fret-render-wgpu/tests/viewport_surface_metadata_conformance.rs`
  - `crates/fret-render-wgpu/tests/paint_gradient_conformance.rs`
  - `crates/fret-render-wgpu/tests/mask_gradient_conformance.rs`
  - `crates/fret-render-wgpu/tests/composite_group_conformance.rs`
  - `crates/fret-render-wgpu/tests/materials_conformance.rs`
  - `crates/fret-render-wgpu/tests/materials_sampled_conformance.rs`

When a new contract is added, extend this list with the smallest conformance gate that proves:

- ordering is preserved,
- the fallback path is deterministic,
- and the wasm/mobile story is explicit.

## 2.2) Baseline runbook (copy/paste)

This section describes a minimal, reproducible baseline capture for the refactor.

### 2.2.1 Layering gate

```bash
python3 tools/check_layering.py
```

### 2.2.2 Renderer conformance (fixed scene set)

Prefer `cargo nextest` when available:

```bash
cargo nextest run -p fret-render-wgpu --test affine_clip_conformance
cargo nextest run -p fret-render-wgpu --test viewport_surface_metadata_conformance
cargo nextest run -p fret-render-wgpu --test paint_gradient_conformance
cargo nextest run -p fret-render-wgpu --test mask_gradient_conformance
cargo nextest run -p fret-render-wgpu --test composite_group_conformance
cargo nextest run -p fret-render-wgpu --test materials_conformance
cargo nextest run -p fret-render-wgpu --test materials_sampled_conformance
```

Fallback (no nextest):

```bash
cargo test -p fret-render-wgpu --test affine_clip_conformance
```

### 2.2.3 Renderer perf snapshot baseline (deterministic stress)

Record perf snapshots using the deterministic SVG atlas stress harness (prints `renderer_perf:` /
`headless_renderer_perf:` lines). Suggested baseline capture:

```bash
cargo run -p fret-svg-atlas-stress -- --headless --frames 600
```

Notes:

- PowerShell:
  - `$env:FRET_RENDERER_PERF_PIPELINES=1; cargo run -p fret-svg-atlas-stress -- --headless --frames 600`
- bash/zsh:
  - `FRET_RENDERER_PERF_PIPELINES=1 cargo run -p fret-svg-atlas-stress -- --headless --frames 600`
- Keep the run duration and flags stable (e.g. 600 frames) so future diffs are meaningful.
- Capture both `renderer_perf:` and `renderer_perf_pipelines:` lines if enabled.

## 3) Proposed internal architecture (implementation, not contract)

## 3.0) Cost model checklist (design-time, not contracts)

These are reminders for keeping wasm/mobile viable while expanding semantics:

- **Isolated opacity** is inherently offscreen:
  - require computation `bounds`,
  - scissor to bounds,
  - and degrade deterministically by quality/downsample/budget rules.
  - default remains non-isolated `PushOpacity` for the zero-cost path.
- **Clip paths / image masks** are inherently “slow path” compared to rect scissor:
  - preserve rect/rrect scissor fast paths,
  - prefer cached mask textures for expensive shapes rather than per-pixel dynamic evaluation
    over large regions,
  - keep hit-testing semantics explicit: clip affects hit-testing; masks are paint-only by default.
- **Stroke upgrades** (dash/join/cap) must be prepared and cached:
  - put high-entropy stroke decomposition behind prepared handles (cache keys include style),
  - defer “constant screen-pixel stroke width” until snapping/AA rules and conformance are in place.
- **Sampling hints** mostly cost batching state splits:
  - keep the hint surface small (few enums),
  - and keep ecosystem defaults consistent (opt-in nearest for pixel-art/canvas scenarios).
- **Shadows** are a trade-off:
  - multi-quad approximations are stable and single-pass (but higher op counts),
  - true blur shadows are offscreen and must participate in budgets like effects/groups.

### 3.1) Compile to a RenderPlan

Compile the ordered `SceneOp` stream into a renderer-owned `RenderPlan` with:

- a sequence of execution segments that preserve `Scene.ops` order,
- explicit pass boundaries at:
  - `PushEffect/PopEffect`,
  - `PushMask/PopMask`,
  - `PushCompositeGroup/PopCompositeGroup`,
- a compact “state snapshot” representation (clip stack, transform stack, opacity multiplier),
  suitable for caching and replay.

### 3.2) Budgets and deterministic degradation move to compile-time

During plan compilation:

- estimate intermediate requirements (bytes and pass count),
- apply deterministic degradations (quality ladder),
- record the applied degradations in telemetry for diagnostics and perf triage.

### 3.3) Caching and reuse are explicit

Prefer stable reuse seams that do not change semantics:

- reuse of encoded segments keyed by scene fingerprint + target generation,
- reuse of cached mask textures for clip paths / image masks (bounded by budgets),
- and strict invalidation rules (no “implicit” cache correctness assumptions).

## 4) Contract roadmap (what we may add, ADR-driven)

The public scene contract should evolve via **small, composable additions**:

- isolated opacity via a group alpha (prefer descriptor extension over new ops when possible),
- clip paths as a new clip-stack entry type (with captured transform semantics),
- image masks as a new mask source (paint-only by default),
- and (later) sampling hints kept as a small enumerated surface to limit batching fragmentation.

Each addition must have:

- an ADR (contract + invariants + degradation rules),
- at least one conformance gate (GPU readback test when feasible),
- and evidence anchors (paths + tests + perf snapshot hooks).

Portability closure (paint/material) is tracked as a dedicated ADR so wasm/mobile behavior stays
explicit and testable:

- `docs/adr/0274-paint-and-material-portability-closure-v1.md`

## 5) References (contracts and guardrails)

- Ordered display list and batching: `docs/adr/0002-display-list.md`, `docs/adr/0009-renderer-ordering-and-batching.md`
- Transform + clip semantics: `docs/adr/0078-scene-transform-and-clip-composition.md`
- Renderer plan/compile substrate (internal architecture guidance):
  - `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
  - `docs/adr/0088-renderer-architecture-v2-scene-compiler.md`
- Effects + budgets: `docs/adr/0117-effect-layers-and-backdrop-filters-scene-semantics-v1.md`,
  `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- Mask semantics (paint-only by default): `docs/adr/0239-mask-layers-and-alpha-masks-v1.md`
- Compositing groups: `docs/adr/0247-compositing-groups-and-blend-modes-v1.md`
- Renderer extensibility tiers: `docs/adr/0123-renderer-extensibility-materials-effects-and-sandboxing-v1.md`

## 6) Capability matrix (draft)

This section is a planning aid for keeping portability explicit while evolving renderer internals.
It is not a contract by itself; normative behavior remains in ADRs.

Targets:

- **Native**: `wgpu` backend (desktop).
- **Web**: wasm + WebGPU.
- **Mobile**: iOS/Android via `wgpu` backends (downlevel policies may apply).

Legend:

- **Must**: expected to work as specified (or with bounded deterministic degradation per ADR).
- **May**: allowed to be capability-gated; callers must have a deterministic fallback.
- **Degrade**: supported only via deterministic degradation (quality downsample / disable).
- **TBD**: not decided in this workstream yet.

| Surface | Native | Web | Mobile | Notes |
| --- | --- | --- | --- | --- |
| `Scene` ordering (ADR 0009) | Must | Must | Must | No reordering across primitive kinds. |
| Rect clip (`PushClipRect`) | Must | Must | Must | Scissor fast path preserved. |
| RRect clip (`PushClipRRect`) | Must | Must | Must | Soft clip required; bounded by clip stack semantics. |
| Clip path (`PushClipPath`) | TBD | TBD | TBD | Proposed v1 in ADR 0273; must have deterministic fallback. |
| Mask gradients (`PushMask` + gradients) | Must | Must | Must | Paint-only by default (ADR 0239). |
| Mask image (`Mask::Image`) | TBD | TBD | TBD | Proposed v1 in ADR 0273; bounded cost and deterministic fallback required. |
| Effects (`PushEffect`) | Must | May | May | Offscreen; budgets + deterministic degradation (ADR 0117/0118). |
| Compositing groups (`PushCompositeGroup`) | Must | May | May | Offscreen; budgets + deterministic degradation (ADR 0247/0118). |
| Isolated opacity (group alpha) | TBD | TBD | TBD | Proposed v1 in ADR 0272; must have a zero-cost non-isolated default. |
| `Paint` solid/gradients | Must | Must | Must | Deterministic sanitize/degrade behavior (ADR 0233). |
| `Paint::Material` (params-only) | Must | May | May | Capability-gated registration (ADR 0235/0122). |
| Sampled materials (catalog textures) | May | May | May | Fixed binding shapes; registration fails deterministically if unsupported (ADR 0242). |
| Telemetry / degradations reporting | Must | Must | Must | Budgets + degradations must be observable (ADR 0118/0036). |
