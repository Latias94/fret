# Renderer Modularity (Fearless Refactor v1) — TODO

Status: In progress

Last updated: 2026-03-12

Related:

- Purpose: `docs/workstreams/renderer-modularity-fearless-refactor-v1/README.md`
- Design: `docs/workstreams/renderer-modularity-fearless-refactor-v1/DESIGN.md`
- Milestones: `docs/workstreams/renderer-modularity-fearless-refactor-v1/MILESTONES.md`
- Render semantics audit:
  - `docs/workstreams/renderer-render-plan-semantics-audit-v1/renderer-render-plan-semantics-audit-v1.md`

Tracking legend:

- `[ ]` open
- `[~]` in progress
- `[x]` done
- `[!]` blocked

ID format:

- `RMFR-{area}-{nnn}`

---

## A. Baseline and Surface Audit

- [x] RMFR-audit-001 Confirm the current facade shape.
  - `crates/fret-render` is currently a wildcard re-export of `fret-render-wgpu`.
- [x] RMFR-audit-002 Capture the current backend baseline gates.
  - `cargo nextest run -p fret-render-wgpu`
  - `python3 tools/check_layering.py`
- [x] RMFR-audit-003 Confirm the current topology seam.
  - `Renderer::new(adapter, device)` and `render_scene(device, queue, ...)` already permit
    host-provided GPU objects.
- [x] RMFR-audit-004 Inventory every public export from `crates/fret-render-wgpu/src/lib.rs` and
  classify it:
  - stable facade surface,
  - backend-only but intentionally public,
  - likely accidental public export.
- [x] RMFR-audit-005 Inventory all first-party consumers of `fret_render::*` and group them by
  dependency pattern:
  - bootstrap only,
  - diagnostics only,
  - renderer mutation/services,
  - external texture / viewport integration.

---

## A1. Locked v1 Decisions

- [x] RMFR-decisions-006 Lock v1 to the existing renderer crate layout.
- [x] RMFR-decisions-007 Lock `crates/fret-render` as the stable default facade.
- [x] RMFR-decisions-008 Lock `crates/fret-render-core` as the portable value-contract crate.
- [x] RMFR-decisions-009 Lock `WgpuContext` as supported convenience API, not sole first-class
  path.
- [x] RMFR-decisions-010 Lock host-provided GPU topology closure as P0.
- [x] RMFR-decisions-011 Lock render-plan semantics as frozen inputs for modularization.
- [x] RMFR-decisions-012 Lock `text/mod.rs` as the first high-value internal breakup target.
- [x] RMFR-decisions-013 Lock cache/registry-style exports into "review for shrink" status by
  default.

---

## B. Facade and Contract Closure

- [x] RMFR-facade-010 Replace wildcard re-export in `crates/fret-render` with an explicit export
  list.
- [ ] RMFR-facade-011 Decide the stable v1 facade surface for:
  - `Renderer`
  - `RenderSceneParams`
  - `SurfaceState`
  - `WgpuContext`
  - capability snapshots
  - perf/report stores
- [~] RMFR-facade-012 Decide which current `fret-render-wgpu` exports should stop being re-exported
  by the default facade.
- [ ] RMFR-facade-013 Move or alias portable value contracts from backend-owned exports to
  `fret-render-core` where that improves ownership clarity.
- [ ] RMFR-facade-014 Document the intended stable meaning of `crates/fret-render`.

---

## C. Host-Provided GPU Topology Closure

- [x] RMFR-topology-020 Add capability helpers that work from adapter/device inputs directly rather
  than requiring `WgpuContext`.
- [~] RMFR-topology-021 Review surface/bootstrap helpers and confirm they stay usable for
  engine-hosted integration.
- [ ] RMFR-topology-022 Add or update at least one smoke path that exercises the host-provided GPU
  topology explicitly.
- [ ] RMFR-topology-023 Update docs/examples so both topology entrypoints are visible:
  - editor-hosted convenience path,
  - engine-hosted path.

---

## D. Internal Domain Extraction

### D1. Text

- [~] RMFR-text-030 Split `crates/fret-render-wgpu/src/text/mod.rs` into explicit submodules.
  - Suggested first slices:
    - font catalog / fallback policy
    - shaping + measurement
    - atlas/cache bookkeeping
    - diagnostics / tests
  - Landed so far:
    - glyph atlas bookkeeping moved into `crates/fret-render-wgpu/src/text/atlas.rs`
    - `text/mod.rs` now goes through atlas accessors instead of touching atlas internals directly
    - diagnostics/debug snapshot code moved into `crates/fret-render-wgpu/src/text/diagnostics.rs`
    - `text/mod.rs` no longer owns atlas/debug/perf snapshot helper implementations directly
    - text quality state/gamma helpers moved into `crates/fret-render-wgpu/src/text/quality.rs`
    - `text/mod.rs` no longer owns text quality configuration/state internals directly
- [ ] RMFR-text-031 Keep `fret_render_text` as the low-level text contract crate and avoid moving
  backend-specific state there prematurely.
- [ ] RMFR-text-032 Add focused tests around any extracted text subdomain whose behavior was
  previously only covered through the monolithic module.

### D2. Renderer state owner

- [ ] RMFR-renderer-040 Identify the subdomain state that can move out of `Renderer` without
  changing behavior.
- [ ] RMFR-renderer-041 Extract cohesive domain owners for:
  - text
  - SVG
  - materials/custom effects
  - intermediate budgeting/pools
  - diagnostics state
- [ ] RMFR-renderer-042 Reduce cross-domain mutable coupling inside `Renderer`.
- [ ] RMFR-renderer-043 Keep service trait implementations readable after extraction.

### D3. Shaders and pipelines

- [ ] RMFR-shaders-050 Audit whether `renderer/shaders.rs` needs ownership-oriented splitting or
  only comment/index cleanup.
- [ ] RMFR-shaders-051 Avoid splitting shader source files purely for line count if no boundary
  benefit exists.
- [ ] RMFR-shaders-052 Keep WGSL validation tests aligned with any source reorganization.

---

## E. Public Export Tightening

- [~] RMFR-exports-060 Review cache/registry-style exports and remove public visibility where no
  real consumer exists.
- [ ] RMFR-exports-061 Decide whether backend-only diagnostics stores belong in the stable default
  facade or under a more explicit backend namespace.
- [ ] RMFR-exports-062 Confirm whether `WgpuContext` remains a stable convenience surface or should
  be demoted in guidance.
- [ ] RMFR-exports-063 Update first-party callers after any facade shrink.

---

## F. Gates and Evidence

- [x] RMFR-gates-070 Establish backend baseline gates before refactor work.
- [~] RMFR-gates-071 Add a surface snapshot note or test proving the intended `fret-render` export
  set after facade curation.
- [ ] RMFR-gates-072 Add targeted smoke coverage for host-provided GPU topology if absent.
- [ ] RMFR-gates-073 Keep render-plan semantics guardrails green for any planning/execution change.
- [ ] RMFR-gates-074 If facade docs/examples change, leave evidence anchors in the workstream docs.

---

## G. Docs and Contract Follow-up

- [x] RMFR-docs-080 Create this workstream doc set.
- [x] RMFR-docs-085 Capture first-pass surface inventory and consumer buckets.
- [ ] RMFR-docs-081 Update this tracker as refactor stages land.
- [ ] RMFR-docs-082 Add or update an ADR if the stable renderer facade contract changes.
- [ ] RMFR-docs-083 If an ADR is added, update `docs/adr/IMPLEMENTATION_ALIGNMENT.md`.
- [ ] RMFR-docs-084 Decide whether this workstream also needs:
  - `EVIDENCE_AND_GATES.md`
  - `OPEN_QUESTIONS.md`
  - `MIGRATION_MATRIX.md`

---

## H. Cleanup / Exit

- [ ] RMFR-cleanup-090 Finish migrating first-party callers to the curated facade surface.
- [ ] RMFR-cleanup-091 Remove or quarantine exports that are now explicitly internal-only.
- [ ] RMFR-cleanup-092 Re-check whether additional crate splits are still necessary after internal
  modularization.
- [ ] RMFR-cleanup-093 Make sure the final docs teach one boring renderer integration story for
  each supported topology.
