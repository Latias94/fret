# Renderer vNext Fearless Refactor v1

Status: Draft (workstream notes only; ADRs remain the source of truth)

Tracking files:

- `docs/workstreams/renderer-vnext-fearless-refactor-v1-todo.md`
- `docs/workstreams/renderer-vnext-fearless-refactor-v1-milestones.md`

Current status (as of 2026-02-15):

- WebGPU/Tint uniformity closure landed (browser smoke verified).
- Quad shader now uses bounded pipeline variants (WGSL `override` constants) to recover perf after uniformity fixes.
- A cheap headless perf gate exists and has a checked-in baseline:
  - `python3 tools/perf/headless_svg_atlas_stress_gate.py`
  - `docs/workstreams/perf-baselines/svg-atlas-stress-headless.windows-local.v1.json`

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
  - `crates/fret-render-wgpu/tests/mask_image_conformance.rs`
  - `crates/fret-render-wgpu/tests/image_sampling_hint_conformance.rs`
  - `crates/fret-render-wgpu/tests/composite_group_conformance.rs`
  - `crates/fret-render-wgpu/tests/materials_conformance.rs`
  - `crates/fret-render-wgpu/tests/materials_sampled_conformance.rs`

When a new contract is added, extend this list with the smallest conformance gate that proves:

- ordering is preserved,
- the fallback path is deterministic,
- and the wasm/mobile story is explicit.

WebGPU guardrail (portability sanity):

- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`

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
cargo nextest run -p fret-render-wgpu --test mask_image_conformance
cargo nextest run -p fret-render-wgpu --test image_sampling_hint_conformance
cargo nextest run -p fret-render-wgpu --test composite_group_conformance
cargo nextest run -p fret-render-wgpu --test materials_conformance
cargo nextest run -p fret-render-wgpu --test materials_sampled_conformance
```

Windows note:

- If you hit `os error 206` (“filename or extension too long”) while building tests, set a shorter
  target directory, for example:

```powershell
$env:CARGO_TARGET_DIR = 'F:\ct'
```

Fallback (no nextest):

```bash
cargo test -p fret-render-wgpu --test affine_clip_conformance
```

### 2.2.3 Renderer perf snapshot baseline (deterministic stress)

Record perf snapshots using the deterministic SVG atlas stress harness (prints `renderer_perf:` /
`headless_renderer_perf:` lines). Suggested baseline capture:

```bash
cargo run -p fret-svg-atlas-stress --release -- --headless --frames 600
```

Notes:

- PowerShell:
  - `$env:FRET_RENDERER_PERF_PIPELINES=1; cargo run -p fret-svg-atlas-stress --release -- --headless --frames 600`
- bash/zsh:
  - `FRET_RENDERER_PERF_PIPELINES=1 cargo run -p fret-svg-atlas-stress --release -- --headless --frames 600`
- Keep the run duration and flags stable (e.g. 600 frames) so future diffs are meaningful.
- Capture both `renderer_perf:` and `renderer_perf_pipelines:` lines if enabled.

Headless gate (stable counters + thresholds):

```powershell
python3 tools/perf/headless_svg_atlas_stress_gate.py
```

Baseline: `docs/workstreams/perf-baselines/svg-atlas-stress-headless.windows-local.v1.json`

### 2.2.4 Quad/material headless gate (variants + dash)

This gate is focused on the quad shader hot paths (paint kinds + dash), and is intended to keep the
pipeline-variant keyspace bounded and observable.

```powershell
python3 tools/perf/headless_quad_material_stress_gate.py
```

Baseline: `docs/workstreams/perf-baselines/quad-material-stress-headless.windows-local.v1.json`

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

#### 3.1.1) RenderPlan IR (draft)

This section is intentionally concrete. The goal is to make refactors landable without repeatedly
touching the core render loop.

**Inputs (compile context):**

- `SceneEncoding` (ordered draws + marker stream derived from `SceneOp`)
- target format + viewport size (px)
- scale factor
- intermediate budget bytes + per-feature budgets (materials, masks)
- backend capabilities (format features, feature flags, limits)

**Outputs:**

- `RenderPlan { segments, passes, resources, degradations, stats }`

Where:

- `segments` preserve ordering and define cacheable “execution slices”.
- `passes` is an in-order list of concrete GPU work items (`SceneDrawRange`, `MaskPass`,
  `CompositePass`, postprocess steps, `ReleaseTarget`).
- `resources` is a small set of plan-local target descriptors (size/origin/format/sample count),
  expressed via `PlanTarget` handles (no backend types leak).
- `degradations` is a deterministic record of any quality/capability/budget fallbacks applied at
  compile time.
- `stats` is a compile-time summary used for telemetry and perf triage (pass count, estimated peak
  intermediate bytes, etc).

#### 3.1.2) Sequence points (ordering boundaries)

“Sequence points” are explicit boundaries where the renderer must flush all pending draws before
continuing, because later ops depend on the exact prior composited result.

**Required sequence points (v1):**

- effect scopes: `PushEffect/PopEffect` (including backdrop effects),
- mask scopes: `PushMask/PopMask`,
- compositing groups: `PushCompositeGroup/PopCompositeGroup`,
- clip-path scopes: `PushClipPath/PopClipPath`.

**Also treated as sequence points (pragmatic):**

- embedded viewport boundaries (if a viewport op implies a separate surface submission),
- any pass that reads from the current color target (read-after-write safety).

We do not allow cross-kind reordering across these boundaries. The only allowed optimization is
adjacency-preserving batching inside a segment.

#### 3.1.3) Segments (cacheable slices)

A segment is a contiguous slice of encoded draws plus a stable “start snapshot” for replay.
Segments are the unit of “no semantic change” caching: if a segment key matches, we can replay it
into any compatible plan target.

Draft shape:

- `RenderPlanSegment { draw_range, start_snapshot, flags }`
- `flags` indicates whether the segment contains:
  - path draws requiring special pipelines (MSAA + resolve),
  - embedded viewports,
  - or other pipeline breaks that should be scheduled as dedicated passes.

**Why segments exist:**

- to keep plan compilation free to reschedule *targets and pass boundaries* without re-encoding
  geometry each time,
- to make it possible to “compare old vs new plan compilers” on a fixed set of segments (guardrail
  for fearless refactors).

#### 3.1.4) State snapshots (small, replayable)

A “state snapshot” is the minimal, renderer-owned representation needed to interpret the subsequent
draw slice correctly.

Draft content:

- transform stack root (or an encoding-time handle to the transform stack),
- clip stack root (including clip-path prepared handles when present),
- mask stack root (including mask source + bounds),
- opacity multiplier (non-isolated) and any group alpha state (isolated opacity),
- scissor/viewport metadata required to bound computation.

Implementation detail: on wgpu today, many of these are already flattened into uniform indices on
individual draws. The snapshot is still valuable as a cache key boundary and a future seam for
“segment replay into different targets/origins”.

#### 3.1.5) Passes (linear by default; DAG-ready)

The initial representation remains a linear list (ADR 0116), but we keep it “DAG-ready” in the
sense that:

- passes explicitly name their `src`/`dst` targets,
- passes carry scissor + origin/size metadata,
- and any early-release opportunity is explicit (`ReleaseTarget`).

The plan compiler is responsible for selecting:

- `PlanTarget` identity (Output vs IntermediateN vs MaskN),
- per-scope target sizes (viewport-sized vs scissor-sized),
- and any resolve/copy passes required by backend hazards.

### 3.2) Budgets and deterministic degradation move to compile-time

During plan compilation:

- estimate intermediate requirements (bytes and pass count),
- apply deterministic degradations (quality ladder),
- record the applied degradations in telemetry for diagnostics and perf triage.

#### 3.2.1) Determinism rule

Execution must be “dumb”: no quality decisions at encode time, and no budget decisions at execute
time. All fallbacks are decided during `RenderPlan` compilation and recorded as part of the plan.

This is the only way to keep wasm/mobile behavior stable and testable.

#### 3.2.2) Budget inputs (draft)

At minimum:

- `intermediate_budget_bytes` (global budget for pooled textures),
- per-feature sub-budgets (optional, but strongly recommended):
  - mask texture bytes (clip-path + image-mask),
  - material paint budgets (per-frame distinct/material count),
  - effect chain maximum passes (to cap worst-case accidental complexity).

#### 3.2.3) Quality ladders (draft)

Each scope that can allocate intermediates must map to a small, fixed ladder of quality states.
The ladder is contract-owned (ADR), and the compiler only selects from it.

Examples:

- group/effect offscreen targets: `1x` → `1/2x` → `1/4x` → `disabled` (fallback to identity / non-isolated)
- clip-path / image-mask baked targets: `1x` → `1/2x` → `cached reuse` → `disabled` (fallback must be deterministic)

The exact ladders for each feature are owned by their ADRs (e.g. ADR 0118, ADR 0273).

#### 3.2.4) Compile-time selection algorithm (draft)

We want a deterministic algorithm that is simple enough to reason about and to conformance-test.

Proposed algorithm:

1) Scan markers to build an in-order list of “computation scopes” (effects, groups, clip-paths, masks),
   each with explicit bounds (`ScissorRect` / computation `bounds`).
2) For each scope, compute candidate target sizes per quality tier and estimate required bytes.
3) Walk scopes in-order (stable tie-breaking):
   - when pushing a scope, attempt to allocate its preferred tier,
   - if the allocation would exceed budgets, downgrade the tier until it fits,
   - if no tier fits, apply the contract-defined disabled fallback for that scope.
4) Emit `degradations` records:
   - `{ scope_kind, reason: Budget|Capability, from_tier, to_tier }`
5) Emit a plan with fully-specified targets, pass list, and early-release points.

The important part is that the compiler’s choices are purely a function of:

- scene + compile context,
- budgets + capabilities,
- and fixed ladders from ADRs.

No backend-dependent “best effort” behavior is allowed.

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

---

## Appendix A — Invariants checklist (preflight)

Use this checklist before starting any renderer-internal refactor step.

### A1) Semantic invariants (never break)

- [ ] Scene op order remains authoritative (no cross-kind reordering; no reordering across sequence points).
- [ ] All stacks preserve capture semantics (clip entries captured at push; later transforms do not retroactively affect clips).
- [ ] Computation `bounds` never act as implicit clips (outside bounds coverage is identity; ADR 0273).
- [ ] Masks remain paint-only by default (no accidental “mask affects hit-test” regression; ADR 0239/0273).
- [ ] Degradation remains deterministic under budget/capability pressure (no backend-specific “best effort” divergence).

### A2) Performance invariants (keep budgets honest)

- [ ] Any offscreen/intermediate allocation is bounded by explicit computation `bounds` and participates in budgets.
- [ ] Any new state surface is bounded (few enums, no unbounded sampler/material combinations).
- [ ] Any new slow path has a fast path preserved (e.g. rect clip stays scissor; avoid per-pixel eval for common cases).

### A3) Portability invariants (wasm/mobile first-class)

- [ ] Capability gating and fallback behavior are documented (ADR + workstream capability matrix update when decided).
- [ ] Missing resources degrade deterministically (e.g. missing images/materials → no-op/solid fallback), not per-backend.
- [ ] WebGPU uniformity rule is respected (Tint): derivative ops and sampling are not gated by non-uniform control flow.
  - If a shader path needs derivatives, prefer pipeline variants with bounded keys (kind/tile mode) over branching on instance data.

### A4) Guardrails (always run)

- [ ] `python3 tools/check_layering.py` passes.
- [ ] Renderer conformance anchors still pass (workstream fixed scene set).
- [ ] Record any new degradations/telemetry counters in the milestone log (peak bytes, pass counts, degradations applied).

---

## Appendix B — Paint support inventory (current)

This appendix is a living inventory for M4 (`Paint/Material evolution`). It answers:

- which scene primitives accept `Paint` today,
- which are still solid-color only,
- and where portability/fallback behavior is already conformance-gated.

### B1) Scene primitive paint surfaces

| Scene op | Surface | Current support | Notes / evidence |
| --- | --- | --- | --- |
| `SceneOp::Quad` | fill/background | `Paint` (solid + gradients + `Paint::Material`) | Renderer conformance: `crates/fret-render-wgpu/tests/paint_gradient_conformance.rs`, `crates/fret-render-wgpu/tests/materials_conformance.rs`, `crates/fret-render-wgpu/tests/materials_sampled_conformance.rs`. |
| `SceneOp::Quad` | border | `Paint` (solid + gradients + `Paint::Material`) | Same gates as fill; border paint is a `Paint` in the contract. |
| `SceneOp::StrokeRRect` | stroke | `Paint` (solid + gradients + `Paint::Material`) | Contract: `crates/fret-core/src/scene/mod.rs` (`StrokeRRect.stroke_paint: Paint`). |
| `SceneOp::Text` | glyph color | solid `Color` only | Paint expansion would need an explicit contract (e.g. gradient text, material text) and dedicated conformance scenes. |
| `SceneOp::Path` | fill color | solid `Color` only | Contract expansion candidate tracked as `REN-VNEXT-paint-002`. |
| `SceneOp::{Image,ImageRegion}` | texture sample | image + opacity | Sampling hints are tracked as M5; mask-image uses `Mask::Image` (ADR 0273), not `Paint`. |
| `SceneOp::MaskImage` / `SceneOp::SvgMaskIcon` | alpha mask tint | solid `Color` tint + opacity | Coverage is expected in red for `MaskImage`. |
| `SceneOp::SvgImage` | RGBA sample | image + opacity | Rasterization policy is renderer-owned; does not accept `Paint`. |
| `SceneOp::ViewportSurface` | embedded target | target + opacity | Portability depends on platform/runner; ordering invariants are covered by renderer conformance. |

### B2) Known gaps / next decisions

- `SceneOp::Path` being solid-only is the largest “paint surface discontinuity” today.
  - If we expand it to accept `Paint`, we must define: tiling semantics, local coordinate mapping
    (origin + transform), and deterministic fallbacks for wasm/mobile (ADR + conformance gate).
- `SceneOp::Text` paint expansion is deferred until text pipeline constraints are settled; keep the
  v1 contract simple and predictable.

Decision (v1 for this workstream):

- Keep `SceneOp::Path` **solid `Color` only** for now.
  - Rationale: the current path pipeline bakes transforms on CPU and does not carry local-space
    coordinates into the shader, so “painted paths” would either (a) require a new per-draw paint
    binding surface (state explosion risk) or (b) force an offscreen mask+fullscreen paint path
    (budget pressure). We defer this until we have:
    - stronger path conformance gates (fill rules / self-intersection / bounds),
    - and a clear, bounded shader binding shape for per-path paints/materials.

---

## Appendix C — Material capability notes (wgpu today)

This appendix is a staging note for `REN-VNEXT-mat-001/002`.

### C1) Registration-time capability gating (deterministic)

The default wgpu renderer performs capability gating at **material registration** time:

- `MaterialBindingShape::ParamsOnly`: supported (baseline).
- `MaterialBindingShape::ParamsPlusCatalogTexture`: supported only when the adapter reports:
  - `TextureFormat::Rgba8Unorm` is `FILTERABLE`, and
  - the format allows `TEXTURE_BINDING | COPY_DST` usages.

If the required feature set is not present, registration fails deterministically with
`MaterialRegistrationError::Unsupported`.

Evidence:

- `crates/fret-render-wgpu/src/renderer/services.rs` (`impl MaterialService for Renderer`).

### C2) Draw-time fallbacks (deterministic)

Draw-time material fallbacks must remain deterministic:

- unknown/unregistered `MaterialId` must degrade to a safe paint (currently transparent).
- budget-driven degradation must be observable (telemetry counters + conformance scenes).

The detailed “Must/May/Degrade” per-target policy remains tracked as `REN-VNEXT-mat-002`.

### C3) Per-target policy (v1 decisions)

The v1 policy is intentionally simple and deterministic:

| Material surface | Native (wgpu) | Web (wasm/WebGPU) | Mobile (wgpu) | Deterministic fallback |
| --- | --- | --- | --- | --- |
| `ParamsOnly` materials | **Must** | **Must** | **Must** | Registration must not fail; unknown ids degrade at draw time (transparent). |
| `ParamsPlusCatalogTexture` (sampled) | **May** | **May** | **May** | If unsupported, registration fails with `Unsupported` and callers must select a non-sampled alternative. |
