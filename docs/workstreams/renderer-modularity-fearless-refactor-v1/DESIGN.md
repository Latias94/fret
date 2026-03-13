# Renderer Modularity (Fearless Refactor v1) — Design

Status: Draft

## Context

Fret's architecture explicitly treats rendering as a distinct layer with two supported hosting
topologies:

- editor-hosted: Fret creates the GPU context and can share device/queue outward,
- engine-hosted: the embedding engine creates the GPU context and passes it to Fret.

That contract is already stated in `docs/architecture.md`. The modularity problem is that the
current public surface still leaks too much of the default backend shape, and the backend's
internal state ownership is still too concentrated in a few large modules.

This design aims to improve modularity without destabilizing render semantics.

## Source of Truth

### Local references

- `docs/architecture.md`
- `crates/fret-render/src/lib.rs`
- `crates/fret-render-core/src/lib.rs`
- `crates/fret-render-wgpu/src/lib.rs`
- `crates/fret-render-wgpu/src/renderer/mod.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan.rs`
- `crates/fret-render-wgpu/src/renderer/services.rs`
- `crates/fret-render-wgpu/src/text/mod.rs`

### Related workstreams

- Render-plan semantics guardrails:
  - `docs/workstreams/renderer-render-plan-semantics-audit-v1/renderer-render-plan-semantics-audit-v1.md`

## Problem Statement

### What is wrong today

The current renderer stack has three kinds of drift:

1. Facade drift
   - `crates/fret-render` behaves like a backend dump, not a curated compatibility surface.
2. Surface drift
   - `crates/fret-render-wgpu` publicly exports some types that do not appear to be essential
     stable contracts.
3. Ownership drift
   - `Renderer` still owns too many subdomains directly, making focused refactors harder than they
     need to be.

### Why this matters

Without a tighter surface and better internal seams:

- backend-specific details become de facto public API,
- moving responsibilities between modules becomes a compatibility problem instead of an internal
  cleanup,
- and host-provided GPU integration remains more awkward than it should be.

That is exactly the kind of drift that turns normal cleanup into a risky rewrite.

## Invariants

The following must hold throughout v1:

1. `crates/fret-render` remains the stable default renderer facade.
   - Callers may continue to depend on it.
   - The facade should become more curated, not disappear.

2. `crates/fret-render-core` owns portable render-facing value contracts.
   - Backend-neutral metadata belongs there.
   - Backend-specific objects do not.

3. Render semantics must remain stable while modularization is in flight.
   - No silent changes to pass ordering, scissor rules, degradation behavior, text paint semantics,
     clip/mask behavior, or output color handling.

4. Engine-hosted GPU topology must remain first-class.
   - The public API must not require `WgpuContext` as the only ergonomic entrypoint.

5. Internal domains should become independently understandable and testable.
   - Text, SVG, plan compilation, execution, diagnostics, and GPU resource management should not
     require touching the same state owner for routine evolution.

6. Component policy must stay out of the renderer layer.
   - This workstream is about render contracts and modularity, not authoring policy.

## Locked v1 Decisions

These decisions are intentionally locked before code refactors begin:

1. No new renderer crates in v1.
   - We reduce risk inside the existing crate layout first.
2. `crates/fret-render` remains the stable default facade.
   - The change is curation, not removal.
3. `crates/fret-render-core` stays portable and value-oriented.
   - It does not become a backend bootstrap crate.
4. `WgpuContext` remains a supported convenience surface.
   - It is not the only ergonomic path we teach.
5. Host-provided GPU topology closure is a P0 contract seam.
   - Capability/bootstrap helpers should accept direct GPU objects where appropriate.
6. Render-plan semantics are frozen inputs to v1 modularization.
7. `text/mod.rs` is the first large internal breakup target.
8. `renderer/shaders.rs` is not a first-wave extraction target unless a real ownership boundary
   requires it.

## Target v1 Architecture

### 1. Curated default facade

`crates/fret-render` should expose a small, deliberate set of surfaces:

- stable render-facing contracts,
- the default backend's stable entrypoints,
- diagnostics/value types we explicitly want ecosystem callers to rely on.

It should stop re-exporting the entire backend crate wholesale.

Recommended shape:

- keep the short-path exports most callers already use,
- optionally add an explicit backend namespace if needed later,
- avoid wildcard re-export as the long-term contract.

### 2. Clear portable vs backend-specific boundary

`crates/fret-render-core` should remain the home for portable value types such as:

- render-target metadata,
- color encoding descriptors,
- ingest strategy metadata,
- other backend-neutral render-facing protocol values.

`crates/fret-render-wgpu` should own:

- `wgpu` bootstrap and surface setup,
- shader/pipeline creation,
- wgpu-only capability discovery,
- GPU execution details,
- backend-specific diagnostics snapshots.

### 3. First-class host-provided GPU seam

The backend should support both:

- convenience paths built around `WgpuContext`,
- and direct paths built from `Instance` / `Adapter` / `Device` / `Queue` / `Surface`.

In practice, v1 should close the seam by making capability and bootstrap helpers available without
requiring `WgpuContext` as the only front door.

Examples:

- capability discovery should work from adapter/device data directly,
- surface helpers should stay usable with host-provided GPU objects,
- docs/examples should teach both topologies explicitly.

### 4. Domain-oriented internal extraction

The main internal decomposition target is not "split files randomly." The target is to make the
following domains explicit:

- bootstrap and topology
  - backend init
  - surface configuration
  - capabilities
- scene planning
  - scene encoding
  - render-plan compilation
  - degradation decisions
  - effect-step helpers vs multi-step chain orchestration should remain explicit subdomains
    inside render-plan effect planning
- execution
  - uploads
  - pass recording
  - command encoding
- render services
  - text service implementation
  - path service implementation
  - SVG service implementation
  - material/custom-effect service implementation
- diagnostics
  - perf stores
  - render-plan dumps
  - backend init snapshots

The point is to move from "one large owner with many helper files" toward "explicit domains with
owned boundaries."

For `render_plan_effects`, that means the long-term boring shape is:

- effect-specific apply/build helpers stay grouped by effect family,
- padded/unpadded chain orchestration, raw-source selection, and final commit semantics live behind
  explicit chain-level helpers instead of staying inline in `apply_chain_in_place(...)`.
- chain-start resource preparation (budget evidence, scratch inventory, clip-mask charging) should
  also live with chain orchestration rather than in the top-level driver body.
- masked and unmasked multi-step chain dispatch should converge on explicit helper surfaces rather
  than leaving one path inline in the top-level module.
- shared chain-local utility helpers such as scratch-target discovery, custom-step detection, and
  backdrop-source-group decomposition should live with chain orchestration instead of remaining as
  top-level cross-module helpers.
- once chain dispatch is extracted, the top-level module should mostly retain shared budgeting /
  utility helpers plus curated wrapper entrypoints.

### 5. Tighten public exports after evidence exists

We should not shrink exports blindly. The order matters:

1. inventory actual consumers,
2. confirm which exports are truly needed,
3. add migration or compatibility notes,
4. then reduce the public surface.

Low-value candidate exports to review early:

- cache types that look internal,
- registries that are not used outside the backend crate,
- thin aliases that duplicate contract types already owned elsewhere.

The working assumption for v1 is conservative:

- cache and registry types are not stable contracts by default,
- backend diagnostics stores need explicit justification to stay in the default facade,
- and broad public visibility must be earned by consumers, not by accident.

## Options Considered

### Option A: Rewrite the renderer around a new architecture

Rejected for v1.

Why:

- current semantics are already protected by a strong regression net,
- a rewrite would mix architectural cleanup with semantic risk,
- and it would discard the leverage we already have from the existing tests.

### Option B: Keep the public surface broad and only split files

Rejected as the final direction.

Why:

- file splits alone do not fix compatibility drift,
- and they leave the "everything is public because it happened to be re-exported" problem intact.

### Option C: Surface-first, semantics-preserving modularization

Recommended.

Why:

- it gives us a smaller stable contract,
- preserves current behavior,
- and lets us refactor internals behind a stronger boundary.

## Migration Strategy

### Stage 0: Lock semantics and baseline

- Treat render-plan semantics as fixed for this refactor slice.
- Keep existing conformance/unit tests green.
- Record the current broad public surface before shrinking it.

### Stage 1: Close the topology seam

- Add direct capability/bootstrap helpers that do not require `WgpuContext`.
- Keep `WgpuContext` as a convenience path, not the only ergonomic path.

### Stage 2: Curate the facade

- Replace wildcard re-export in `crates/fret-render` with explicit exports.
- Keep compatibility for high-value entrypoints.
- Move obviously portable contract types behind `fret-render-core` where applicable.

### Stage 3: Extract internal domains

Prioritize the highest-value extractions:

1. text system breakup (`text/mod.rs`)
2. renderer state/domain breakup (`Renderer`)
3. shader source organization only where it improves ownership, not just line counts

### Stage 4: Shrink public surface

- Review backend exports one by one.
- Downgrade internal-only types from public to crate-private where possible.
- Update first-party callers if the facade surface changes.

### Stage 5: Re-evaluate crate boundaries

Only after the above is stable, decide whether further crate splits are worth it.

Examples of possible future work, explicitly out of scope for v1:

- a separate backend bootstrap crate,
- a separate backend diagnostics crate,
- a separate text/GPU text integration crate beyond the current `fret-render-text`.

## Regression Strategy

Minimum gates for each landed slice:

- `cargo fmt`
- `cargo nextest run -p fret-render-wgpu`
- `python3 tools/check_layering.py`

Recommended additional checks for facade or topology changes:

- targeted consumer build/tests for `crates/fret-launch`
- at least one engine-hosted style demo or smoke path
- render-plan/conformance tests when touching execution or planning internals

## ADR Trigger

This workstream document is enough for staged modularization planning.

An ADR should be added or updated before we do any of the following:

- change the stable meaning of `crates/fret-render` as the public default renderer facade,
- change the architecture contract for host-provided GPU topology,
- move a type across crates in a way that changes the long-term public contract,
- split backend semantics into new crates that affect published boundaries.
