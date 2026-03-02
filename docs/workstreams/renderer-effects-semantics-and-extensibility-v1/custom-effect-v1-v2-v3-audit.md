---
title: Custom Effects (V1/V2/V3) — Design Audit + Fearless Refactor Plan
status: draft
date: 2026-03-02
scope: renderer, effects, extensibility, contracts, diagnostics, wgsl
---

# Custom Effects (V1/V2/V3) — Design Audit + Fearless Refactor Plan

This note audits the current **CustomV1 / CustomV2 / CustomV3** design and implementation, with two goals:

1) Validate that the contract surfaces stay **bounded, deterministic, and budgetable** (escape hatch with a ceiling).
2) Produce a **3–8 PR** “fearless refactor” split plan that is landable (small diffs, clear gates, reversible).

Scope:

- Contract surface: `fret-core` (`EffectStep::*`, validation, fingerprinting).
- Reference backend: `fret-render-wgpu` (registration, WGSL stitching/validation, bind shapes, degradations).
- Authoring/demo evidence: `apps/fret-examples` + `tools/diag-scripts` (liquid glass + custom effect demos).

Non-goals:

- Redesigning the overall effect system (this is an audit + refactor plan).
- Turning the custom effect mechanism into an unbounded shader API.

## Summary judgment (high level)

The current CustomV1/V2/V3 design is **directionally correct** for Fret’s constraints:

- The core contract stays small and portable (no wgpu handles, fixed payload, bounded steps).
- Versioned ABIs keep bind shapes fixed, which preserves determinism and budgeting.
- The wgpu backend has conformance coverage and diagnostics vocabulary for V3 degradations.

The main issues are **implementation duplication** and a few **surface hygiene gaps** (capability discovery, cache
eviction consistency, authoring guardrails).

## Contract surface audit (`fret-core`)

### What is stable (portable, bounded)

- Program registration descriptors are portable and intentionally tiny:
  - `crates/fret-core/src/effects.rs` (`CustomEffectDescriptorV1/V2/V3`, `CustomEffectService`).
- Effect steps keep the “single pass + bounded params” model:
  - `crates/fret-core/src/scene/mod.rs` (`EffectStep::CustomV1/V2/V3`).
- Non-finite user input is sanitized at the contract layer:
  - `crates/fret-core/src/scene/mod.rs` (`EffectStep::sanitize`).
  - `crates/fret-core/src/scene/validate.rs` rejects non-finite ops in recorded scenes.
- Render-plan cache fingerprinting includes the custom effect step fields:
  - `crates/fret-core/src/scene/fingerprint.rs` (hashes `EffectId`, params, sample bounds, V2 input, V3 sources).

### Version differences (contract-only)

- **V1**: params-only + `src_texture` reads (no sampler in prelude; intended `textureLoad`).
- **V2**: adds one optional user image input (`CustomEffectImageInputV1`).
- **V3**:
  - adds `user0` + `user1` (two optional user image inputs),
  - adds renderer-provided sources selection (`CustomEffectSourcesV3`): `want_raw` and optional pyramid request.

Contract anchors:

- `crates/fret-core/src/scene/mod.rs`:
  - `CustomEffectImageInputV1`
  - `CustomEffectSourcesV3`
  - `CustomEffectPyramidRequestV1`
  - `EffectStep::CustomV1/V2/V3`

## Backend implementation audit (`fret-render-wgpu`)

### Registration and WGSL validation

The backend maintains a renderer-owned registry keyed by `(ABI version, WGSL source)`:

- `crates/fret-render-wgpu/src/renderer/services.rs`:
  - `register_custom_effect_v1/v2/v3`
  - `build_and_validate_custom_effect_wgsl_v1/v2/v3` (WGSL stitching + naga parse/validate)

Notes:

- The per-version `build_and_validate_*` functions are intentionally strict and bounded:
  - byte limit: `MAX_CUSTOM_EFFECT_WGSL_BYTES` (64 KiB)
  - parse+validate: naga
- There is duplication across v1/v2/v3 (same loop, different prelude generator), which is refactorable without touching
  public contracts.

### Fixed bind shapes (the “ceiling”)

V2 and V3 require filterable sampled user inputs (because their ABI includes filtering samplers):

- V2 bind group layout includes `input_texture` as `Float { filterable: true }` + filtering sampler:
  - `crates/fret-render-wgpu/src/renderer/pipelines/custom_effect.rs` (`ensure_custom_effect_v2_pipelines`)
- V3 bind group layout includes `user0` + `user1` textures as `Float { filterable: true }` + filtering samplers:
  - `crates/fret-render-wgpu/src/renderer/pipelines/custom_effect.rs` (`ensure_custom_effect_v3_pipelines`)

Per-step degradation for incompatible user images is deterministic:

- `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs`:
  - `record_custom_effect_v3_pass`: non-filterable formats fall back to a renderer-owned 1×1 transparent texture view.
  - (Analogous checks exist for V2.)

### Deterministic degradation + diagnostics (V3)

V3 has explicit counters + plan visibility for:

- raw requested/distinct/aliased
- pyramid requested/applied/degraded reasons
- backdrop source group outcomes
- whether CustomV3 was requested vs actually emitted as a render-plan pass

Triage hint codes (worst frame):

- `renderer.custom_effect_v3_requested_but_skipped` (requested by effect chain, but no passes emitted)
- `renderer.custom_effect_v3_raw_aliased_to_src` (raw snapshot unavailable; `src_raw` aliases)
- `renderer.custom_effect_v3_pyramid_degraded_to_one` (pyramid levels degraded under budget pressure)

Interpretation note:

- `renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes` will often be `0` when the pyramid is
  degraded to 1 level, because we only charge/report pyramid bytes when `levels >= 2` (mip storage allocated).
  Use triage evidence fields like `custom_effect_chain_headroom_after_mask_bytes` together with
  `custom_effect_v3_pyramid_required_bytes_levels2_est` to make “why did it degrade?” explainable from a bundle.

Anchors:

- Plan compile: `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs`
- Plan dump: `crates/fret-render-wgpu/src/renderer/render_plan_dump.rs`
- Conformance: `crates/fret-render-wgpu/tests/effect_custom_v3_conformance.rs`
- Triage: `crates/fret-diag/src/triage_json.rs`

## Liquid glass parity audit (demo authoring)

### What looks “complete enough” for a bounded single-pass ceiling

The `CustomV3` lens WGSL used by the liquid glass demo covers the core AndroidLiquidGlass reference behaviors:

- Rounded-rect SDF + gradient-derived refraction direction
- Rim-only refraction gate (“refraction height”)
- Circle-map taper
- Chromatic dispersion (cheap 3-tap and Android-like 7-tap option)
- Bevel lighting modulation (ported from AndroidLiquidGlass SDF shader)
- Raw + pyramid sampling path (`src_raw` + `src_pyramid`) for multi-scale “frost”

AndroidLiquidGlass reference anchors (for parity comparisons):

- Rounded-rect refraction + dispersion: `AndroidLiquidGlass/backdrop/src/main/java/com/kyant/backdrop/Shaders.kt`
  - `RoundedRectRefractionShaderString`
  - `RoundedRectRefractionWithDispersionShaderString`
- Lens wiring (uniform setup + negated amount): `AndroidLiquidGlass/backdrop/src/main/java/com/kyant/backdrop/effects/Lens.kt`
- Bevel modulation reference (SDF shader): `AndroidLiquidGlass/catalog/src/main/java/com/kyant/backdrop/catalog/utils/SdfShader.kt`

Parity notes (intentional / known drifts):

- Dispersion sign: Android’s dispersion intensity uses the signed `(centered.x * centered.y) / (half.x * half.y)` term.
  The demo WGSL currently uses `abs(...)`, which makes dispersion symmetric across quadrants. This is a look choice; if
  we want closer parity, drop `abs` (authoring-only change).
- Corner radii: Android’s rounded-rect shader assumes a per-corner radii vector; the demo uses a uniform radius.
  This avoids observable drift today, but we should keep the `radius_at(centered, radii)` helper correct for future
  non-uniform radii authoring.
- Sampling: Android uses `content.eval()` (hardware bilinear). Fret uses manual bilinear sampling on `textureLoad` so
  CustomV3 can remain bounded and support non-filterable internal formats deterministically.

Anchors:

- `apps/fret-examples/src/custom_effect_v3_wgsl.rs` (`CUSTOM_EFFECT_V3_LENS_WGSL`)
- `apps/fret-examples/src/liquid_glass_demo.rs` (CustomV3 chain + bevel controls + optional backdrop source group)
- Diagnostics suites:
  - `tools/diag-scripts/suites/liquid-glass-custom-v3/`
  - `tools/diag-scripts/suites/liquid-glass-custom-v3-degraded/`
  - `tools/diag-scripts/suites/liquid-glass-custom-v3-sources-degraded/`
  - `tools/diag-scripts/suites/perf-liquid-glass-custom-v3-steady/`

### Known gaps / intentional differences

These are not correctness bugs, but areas where “parity” may still drift from the Android reference:

- The demo is still a **single-pass** shader; Android implementations may effectively behave like multi-pass (or use
  different composition order). This is an intentional “bounded ceiling” tradeoff.
- Noise/grain uses a local hash in the demo WGSL; we may prefer switching to the renderer’s deterministic catalog noise
  helper for consistency across effects (`fret_catalog_hash_noise01`), but that is authoring/policy, not contract.
- “Exact look” depends on blur radius choices and pyramid level mapping; these are exposed as parameters, but need a
  locked baseline for tuning (the diag suites are the right vehicle).

## Hazards (what is likely to bite us later)

### H1 — Capability discovery may need finer granularity

`RendererCapabilities` exposes V3 support explicitly (`custom_effect_v3`). The remaining open question is whether we
need finer-grained flags (e.g. “pyramid sources supported”) in the future, or whether deterministic degradation plus
diagnostics hints are sufficient.

Anchor:

- `crates/fret-render-wgpu/src/capabilities.rs` (`RendererCapabilities`)

### H2 — Internal duplication increases drift risk

The `services.rs` custom effect registration path (hash reuse + refcounting + insertion) and WGSL stitch+validate code
used to be duplicated for v1/v2/v3. This is safe but tends to drift (logging wording, validation flags, max-size rules,
capability gating, etc.).

Anchor:

- `crates/fret-render-wgpu/src/renderer/services.rs`

### H3 — Cache eviction consistency (unregister)

When unregistering a custom effect, pipeline caches should be evicted consistently for v1/v2/v3. If one ABI’s cache is
not evicted, it can hold stale GPU objects until a broader clear happens.

Anchor:

- `crates/fret-render-wgpu/src/renderer/services.rs` (`unregister_custom_effect`)
- `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (per-ABI pipeline maps)
- Locked by unit test: `crates/fret-render-wgpu/src/renderer/services.rs` (`unregister_custom_effect_evicts_custom_effect_pipelines`)

### H4 — Authoring pitfalls remain easy to hit

Even with naga validation, authors can hit WGSL pitfalls that are not obvious from the snippet model (e.g. swizzle
assignment rules, derivative-uniformity pitfalls). We already have conformance tests and a smoke test, but they can be
extended to reduce regressions.

Anchors:

- `apps/fret-examples/tests/wgsl_smoke.rs`
- `crates/fret-render-wgpu/tests/effect_custom_v*_conformance.rs`

## Fearless refactor split plan (3–8 PRs)

Each PR below is intended to be small, reviewable, and reversible. “Gates” are the minimum regression artifacts to run
before landing.

### PR0 — Landed: CustomV3 observability + liquid-glass degradation suites

Goal:

- Make “CustomV3 requested but skipped” and “CustomV3 sources degraded” triageable from diag bundles.

Changes (landed):

- Add renderer perf counters: `custom_effect_v3_steps_requested`, `custom_effect_v3_passes_emitted`.
- Add triage hint: `renderer.custom_effect_v3_requested_but_skipped`.
- Stabilize liquid-glass demo automation selectors (`test_id`) used by suites.
- Add/curate liquid-glass suites:
  - `tools/diag-scripts/suites/liquid-glass-custom-v3-degraded/`
  - `tools/diag-scripts/suites/liquid-glass-custom-v3-sources-degraded/`

Gates:

- `cargo check -p fret-render-wgpu -p fret-diag -p fret-bootstrap`
- `cargo run -p fretboard -- diag suite liquid-glass-custom-v3-sources-degraded --dir target/fret-diag/lg-v3 --session-auto --launch -- .\\target\\debug\\liquid_glass_demo.exe`

Rollback:

- Revert the commit(s) that add counters/hints/suites.

### PR1 — Docs: audit + authoring guidance links

Status:

- Landed.

Goal:

- Make the current V1/V2/V3 semantics and known gaps discoverable from the workstream index.

Changes (landed):

- Add this audit note.
- Add links from the workstream README to the audit note and the liquid-glass-related diag suites.

Gates:

- None (docs-only).

Rollback:

- Revert the docs commit.

### PR2 — Refactor: dedupe WGSL stitch+validate in `services.rs` (no behavior change)

Status:

- Landed.

Goal:

- Remove v1/v2/v3 duplication in CustomEffect registration and `build_and_validate_custom_effect_wgsl_v*` without
  changing the public surface.

Changes (landed):

- Introduce internal helper `build_and_validate_custom_effect_wgsl_with_sources(...)`.
- Introduce internal helper `register_custom_effect_wgsl(...)` to share the hash reuse + insertion path.
- Introduce internal helper `custom_effect_sampled_user_image_supported(...)` to share V2/V3 capability gating.
- Keep per-ABI stitch generators as-is; route v1/v2/v3 through the shared helpers.

Risks:

- Accidental logging/label drift; accidental validation flag changes.

Gates:

- `cargo test -p fret-examples --test wgsl_smoke`
- `cargo nextest run -p fret-render-wgpu --tests effect_custom_v1_conformance effect_custom_v2_conformance effect_custom_v3_conformance`

Rollback:

- Revert the refactor (no contract changes).

### PR3 — Fix: unregister evicts custom-effect pipeline caches + add a unit test

Status:

- Landed (unit test exists in `crates/fret-render-wgpu/src/renderer/services.rs`).

Goal:

- Ensure `unregister_custom_effect` evicts all ABI pipeline caches consistently.

Changes:

- Evict `custom_effect_pipelines` (v1), `custom_effect_v2_pipelines`, and `custom_effect_v3_pipelines` in `unregister_custom_effect`.
- Add a focused unit test under `crates/fret-render-wgpu/src/renderer/services.rs` that:
  - registers v1/v2/v3 effects, forces pipeline creation, unregisters, and asserts each map entry is evicted.

Gates:

- `cargo nextest run -p fret-render-wgpu --tests`

Rollback:

- Revert the fix; behavior is internal (no contract surface change).

### PR4 — Capabilities: expose V3 support in `RendererCapabilities` (additive)

Status:

- Landed (`crates/fret-render-wgpu/src/capabilities.rs` exposes `custom_effect_v3`).

Goal:

- Make capability discovery explicit for V3 so apps can show “unsupported” state without probing registration.

Changes:

- Extend `crates/fret-render-wgpu/src/capabilities.rs` with one or more additive fields:
  - `custom_effect_v3: bool` (same conservative check as V2 user-image support), and optionally
  - `custom_effect_v3_pyramid: bool` (if we ever want to gate pyramid separately),
  - `custom_effect_v3_user_images: u8` (if we later want to vary the ceiling by backend).
- Update `apps/fret-examples/src/liquid_glass_demo.rs` and `custom_effect_v3_*_demo.rs` to use the capability flag for
  UI labeling.

Risks:

- Public surface expansion (additive) across crates; requires ensuring the capability struct is plumbed to the places
  that need it.

Gates:

- `cargo build -p fret-examples`

Rollback:

- Revert the additive fields and demo usage.

### PR5 — Guardrails: extend WGSL smoke gates to V1/V2 and unify “stitch then parse” helpers

Status:

- Landed (the smoke test covers v1/v2/v3 + the liquid-glass lens program).

Goal:

- Catch “snippet compiles alone but fails when stitched” earlier (especially for author demos and future recipes).

Changes (landed):

- Extend `apps/fret-examples/tests/wgsl_smoke.rs` to validate stitched WGSL for v1 and v2 in addition to v3.
- (Optional) Extract a small shared helper in `fret-render-wgpu` tests for stitching/parsing to avoid duplicating the
  prelude glue across tests.
- Keep `services.rs` validation warnings structured (log message stable; ABI/version reported via fields) to reduce
  wording drift in downstream log scrapers.

Gates:

- `cargo test -p fret-examples --test wgsl_smoke`

Rollback:

- Revert the smoke-test extension.

### PR6 — Diagnostics polish: tighten the “degraded and why” story for CustomV3 liquid glass

Status:

- Landed.

Goal:

- Make it faster to triage “looks wrong” reports: show whether we lost raw distinctness, pyramid levels, or group
  sharing (and why).

Changes:

- RenderPlan dumps:
  - Extend CustomEffectV3 summaries with `pyramid_levels_min` + `pyramid_levels_sum` to make “applied levels” visible even when degraded to 1.
  - Add `custom_effect_v3_diagnostics` to the dump root (CustomEffectV3 sources + BackdropSourceGroup degradation counters).
- Suites:
  - Pin a high intermediate budget for the BackdropSourceGroup liquid-glass suite to reduce cross-machine drift and make “known-good” runs more deterministic.
  - Add a post-run suite gate `--check-triage-hint-absent <code>` so suites can assert “no degradations” via triage hints.

Gates:

- `cargo test -p fret-render-wgpu --lib render_plan_dump`
- (Optional) `cargo run -p fretboard -- diag suite liquid-glass-custom-v3 --dir target/fret-diag/liquid-glass-custom-v3 --session-auto --check-triage-hint-absent renderer.custom_effect_v3_requested_but_skipped --check-triage-hint-absent renderer.custom_effect_v3_raw_aliased_to_src --check-triage-hint-absent renderer.custom_effect_v3_pyramid_degraded_to_one --check-triage-hint-absent renderer.backdrop_source_groups_raw_degraded --check-triage-hint-absent renderer.backdrop_source_groups_pyramid_degraded --launch -- .\\target\\debug\\liquid_glass_demo.exe`

Rollback:

- Revert the dump formatting/script changes.

### PR7 — Semantics: unify “requested vs effective” counters + triage codes across V1/V2/V3

Status:

- Landed.

Goal:

- Make it trivial to answer: “did we skip the whole pass?”, “did we degrade sources?”, “did we silently fall back?” across all CustomEffect ABIs.

Changes:

- Add per-ABI counters in renderer perf:
  - steps requested vs passes emitted (V1/V2 as well, mirroring V3).
- Add triage hint codes with stable evidence keys:
  - `renderer.custom_effect_v1_requested_but_skipped`
  - `renderer.custom_effect_v2_requested_but_skipped`
  - Keep `renderer.custom_effect_v3_requested_but_skipped` as-is.
- Add conformance tests to force “requested but skipped” under tight budgets (bounded, deterministic).

Gates:

- `cargo check -p fret-render-wgpu -p fret-diag -p fret-bootstrap`
- `cargo test -p fret-render-wgpu --test effect_custom_v3_conformance`
- (Optional) Add suite gates:
  - `cargo run -p fretboard -- diag suite <suite> --dir target/fret-diag/<suite> --session-auto --check-triage-hint-absent renderer.custom_effect_v1_requested_but_skipped --check-triage-hint-absent renderer.custom_effect_v2_requested_but_skipped --launch -- <demo exe>`

Rollback:

- Revert counters/hints/tests (no contract changes).

### PR8 — Budget model: make CustomEffect ABI scratch/budget charging consistent (no policy drift)

Goal:

- Remove “same scene, different ABI => different silent budget outcomes” drift and make budgets explainable.

Changes:

  - Status:
  - In progress (land as a small series of commits / local PR-sized steps).
  - Landed:
    - BackdropSourceGroup pyramid headroom accounting fix (avoid double-counting `raw_bytes`).
    - Centralize intermediate budget math helpers (effective budget + allocation checks) in the RenderPlan compiler.
    - Carry effect-chain intermediate budget evidence (effective budget min/max + other-live max) into perf snapshots + diag stats.
    - Centralize “base required bytes” helpers to replace ad-hoc `full * N` budget checks.
    - Centralize intermediate budget charging helpers (mask targets + CustomV3 pyramid bytes), so “budget available / required” evidence is computed from a single source of truth.
  - In progress:
    - (Done) Extend diagnostics evidence to report the computed CustomEffect budget model inputs for the worst frame (budget, base required, optional required, chosen plan).
- Landable steps (3–8, PR-sized):
  1) [Landed] Fix BackdropSourceGroup pyramid headroom accounting (avoid double-counting `raw_bytes`) and lock with a unit test.
  2) [Landed] Centralize “base required bytes” helpers used by CustomV1/V2/V3 (single-scratch, padded-chain work path, optional chain-raw) and use them everywhere instead of ad-hoc `full * N`.
  3) [Landed] Centralize intermediate budget charging helpers (mask targets + CustomV3 pyramid bytes), so “budget available / required” evidence is computed from a single source of truth.
  4) [Landed] Add a tiny table-driven conformance set for budget edges (budget=0 / insufficient / target exhausted) per ABI.
  5) [Landed] Extend diagnostics evidence to report the computed CustomEffect budget model inputs for the worst frame (budget, base required, optional required, chosen plan).

Notes:

- For triage evidence, prefer carrying byte-scaled context instead of relying on viewport/format inference:
  - `renderer_intermediate_full_target_bytes` (bytes for one full-viewport intermediate target),
  - `renderer_intermediate_budget_bytes`,
  - `renderer_render_plan_effect_chain_budget_samples`,
  - `renderer_render_plan_effect_chain_effective_budget_min_bytes`,
  - `renderer_render_plan_effect_chain_effective_budget_max_bytes`,
  - `renderer_render_plan_effect_chain_other_live_max_bytes`,
  - CustomEffect chain budget model evidence (new):
    - `renderer_render_plan_custom_effect_chain_budget_samples`
    - `renderer_render_plan_custom_effect_chain_effective_budget_min_bytes`
    - `renderer_render_plan_custom_effect_chain_effective_budget_max_bytes`
    - `renderer_render_plan_custom_effect_chain_other_live_max_bytes`
    - `renderer_render_plan_custom_effect_chain_base_required_max_bytes`
    - `renderer_render_plan_custom_effect_chain_optional_required_max_bytes`
    - `renderer_render_plan_custom_effect_chain_base_required_full_targets_max`
    - `renderer_render_plan_custom_effect_chain_optional_mask_max_bytes`
    - `renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes`
  - Triage-derived headroom (computed from the above, shown in hint evidence when available):
    - `custom_effect_chain_headroom_before_optional_bytes`
    - `custom_effect_chain_headroom_after_mask_bytes`
    - `custom_effect_chain_headroom_after_optional_bytes`
  - Triage-derived pyramid sizing estimates (worst-frame hint evidence):
    - `custom_effect_v3_pyramid_required_bytes_levels2_est`
    - `custom_effect_v3_pyramid_would_fit_levels2_est`
  - `renderer_intermediate_peak_in_use_bytes`.

Conformance:

- Target exhaustion: `crates/fret-render-wgpu/tests/effect_custom_v3_conformance.rs` includes a
  nested FilterContent scenario that forces CustomEffectV3 pass emission to be skipped due to
  lack of scratch targets (target exhausted).
- Budget zero + target exhaustion: conformance gates exist for CustomEffect V1/V2/V3 so budget
  edges remain deterministic across ABIs (requested step present, emitted pass absent, and the
  corresponding render plan degradation reason is visible in perf snapshots).
- Budget insufficient (non-zero): conformance gates exist for CustomEffect V1/V2/V3 so “needs >= 2
  full targets” requirements remain stable and produce a `BudgetInsufficient` render plan
  degradation that can be triaged from perf snapshots.

Remaining:

- None (PR8 step5 landed).

Gates:

- `cargo nextest run -p fret-render-wgpu --tests`
- `cargo test -p fret-examples --test wgsl_smoke`

Rollback:

- Revert refactors/tests; behavior should be unchanged unless explicitly called out by the tests.

### PR9 — Refactor: make CustomV3 source planning a small, testable unit

Status:

- Landed.

Goal:

- Reduce drift risk in CustomV3 by making “raw/pyramid choice + degradation + scissor ROI” explicit and unit-testable.

Changes (landed):

- Centralize CustomV3 source planning into a small helper + focused unit tests:
  - raw choice (distinct vs aliased),
  - pyramid level selection (budget vs group vs request),
  - pyramid build scissor ROI choice.
- Route all CustomV3 emission paths through the same planning code (padded chain, in-place single-scratch, and step
  compilation).

Anchors:

- `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` (`plan_custom_v3_sources_and_charge_budget`)
- `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` (unit tests: `custom_v3_sources_plan_*`)

Gates:

- `cargo test -p fret-render-wgpu --test effect_custom_v3_conformance`
- `cargo nextest run -p fret-render-wgpu --tests` (preferred)

Rollback:

- Revert the refactor commit; behavior should be unchanged.

### PR10 — Refactor: reduce V1/V2/V3 pass struct duplication

Status:

- Landed.

Goal:

- Reduce “same field list repeated in 3 places” drift in pass validation, dumping, and recording.

Changes (landed):

- Introduce an internal `CustomEffectPassCommon` (src/dst, sizes, scissor, mask, effect id, params, load).
- Re-home shared validation/dump helpers to operate on the common struct.

Anchors:

- `crates/fret-render-wgpu/src/renderer/render_plan.rs` (`CustomEffectPassCommon`, pass structs, validation helpers)
- `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` (pass construction)
- `crates/fret-render-wgpu/src/renderer/render_plan_dump.rs` (dump + summaries)
- `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs` (pass recording)
- `crates/fret-render-wgpu/src/renderer/render_scene/effect_pipelines.rs` (pipeline prewarming)
- `crates/fret-render-wgpu/src/renderer/render_scene/executor.rs` (write-epoch bumps)

Gates:

- `cargo check -p fret-render-wgpu`
- `cargo test -p fret-render-wgpu --test effect_custom_v3_conformance`

Rollback:

- Revert the refactor commit; no behavior change expected.

### PR11 — Refactor: split `render_plan_effects.rs` into modules (no behavior change)

Status:

- Landed.

Goal:

- Reduce merge conflict risk and make effect-chain planning readable (the file is a hotspot).

Changes (landed):

- Move chain planning (`apply_chain_in_place`, padding/work-target logic) into a dedicated module.
- Move CustomV1/V2/V3 emission helpers into a dedicated module.
- Keep existing tests and conformance behavior unchanged.

Anchors:

- `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` (module root + public helpers)
- `crates/fret-render-wgpu/src/renderer/render_plan_effects/custom.rs` (CustomV1/V2/V3 helpers + V3 sources plan)
- `crates/fret-render-wgpu/src/renderer/render_plan_effects/scissor.rs` (scissor mapping helpers)
- `crates/fret-render-wgpu/src/renderer/render_plan_effects/tests.rs` (unit tests moved out-of-line)

Gates:

- `cargo test -p fret-render-wgpu --test effect_custom_v3_conformance`
- `cargo nextest run -p fret-render-wgpu --tests` (preferred)

Rollback:

- Revert the move; no behavior change expected.

### PR12 — Authoring: make `max_sample_offset_px` easier to get right

Status:

- Landed.

Goal:

- Reduce “padding bugs” in demos and future recipes by standardizing how `max_sample_offset_px` is derived.

Changes (landed):

- Add a tiny authoring helper in `apps/fret-examples` that computes conservative `max_sample_offset_px` bounds from known shader parameters.
- Update liquid glass demo + CustomV3 lens demos to use the helper.

Anchors:

- `apps/fret-examples/src/effect_authoring.rs` (authoring helpers)
- `apps/fret-examples/src/liquid_glass_demo.rs` (CustomV2 + CustomV3 chains)
- `apps/fret-examples/src/custom_effect_v3_demo.rs` (native lens demo)
- `apps/fret-examples/src/custom_effect_v3_web_demo.rs` (web lens demo)

Gates:

- `cargo check -p fret-examples`
- Run `tools/diag-scripts/suites/perf-liquid-glass-custom-v3-steady/` (manual + scripted)

Rollback:

- Revert helper usage; behavior should be unchanged.

## Evidence anchors (most load-bearing)

- Contract:
  - `crates/fret-core/src/effects.rs`
  - `crates/fret-core/src/scene/mod.rs`
  - `crates/fret-core/src/scene/validate.rs`
  - `crates/fret-core/src/scene/fingerprint.rs`
- Backend:
  - `crates/fret-render-wgpu/src/renderer/services.rs`
  - `crates/fret-render-wgpu/src/renderer/pipelines/custom_effect.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs`
  - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs`
- Tests:
  - `crates/fret-render-wgpu/tests/effect_custom_v3_conformance.rs`
  - `apps/fret-examples/tests/wgsl_smoke.rs`
- Liquid glass demo + diag:
  - `apps/fret-examples/src/liquid_glass_demo.rs`
  - `tools/diag-scripts/suites/liquid-glass-custom-v3/`
  - `tools/diag-scripts/suites/perf-liquid-glass-custom-v3-steady/`
