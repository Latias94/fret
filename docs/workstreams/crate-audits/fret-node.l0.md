# Crate audit (L0) — `fret-node`

## Crate

- Name: `fret-node`
- Path: `ecosystem/fret-node`
- Owners / adjacent crates: `fret-canvas`, `fret-ui`, `fret-runtime`, `fret-ui-kit`, `fret-authoring` (optional), app shells under `apps/fret-*`
- Current “layer”: ecosystem substrate + editor-grade UI integration (declarative-first, retained compat optional)

## 1) Purpose (what this crate *is*)

- Provides a long-lived, serializable node-graph document model (`Graph` + stable IDs) with editor-grade invariants (typed connections, deterministic import closure, reversible edits, diagnostics).
- Provides policy-light headless primitives (rules + profile-driven validation/connection planning) that can be reused without `fret-ui`.
- Provides an optional UI integration surface (behind `fret-ui`) that is intended to be a **teaching/reference surface** for how complex editor UIs should be shipped in the Fret ecosystem.

Evidence anchors:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/core/*`
- `ecosystem/fret-node/src/rules/*`
- `ecosystem/fret-node/src/profile/*`

## 2) Public contract surface

- Key exports / stable types:
  - headless model: `Graph`, `Node`, `Edge`, `Port`, `Group`, `StickyNote`, `Symbol` and ID types (re-exported from `core`)
  - rules/diagnostics: `ConnectPlan`, `Diagnostic`, `DiagnosticSeverity`
  - typing: `TypeDesc`, `TypeVarId`
  - UI (behind `fret-ui`): `NodeGraphController`, `node_graph_surface`, presenter/skin/style registry types
    - `NodeGraphController` intentionally mirrors a subset of XYFlow-style “instance” affordances (`set_viewport*`, `fit_view_nodes*`, connection queries) without copying the React/DOM API.
- Feature flags and intent:
  - default: `["fret-ui", "kit"]` (UI + convenience surfaces are on by default in-tree)
  - `headless`: explicitly builds without `fret-ui`
  - `compat-retained-canvas`: retained-bridge-backed UI surfaces (delete-planned escape hatch)
- “Accidental” exports to consider removing (L0 hypothesis):
  - (Resolved) `NodeGraphSurfaceProps` is now controller-first only:
    `controller: NodeGraphController` is required and the `store` fallback is removed.

Evidence anchors:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/Cargo.toml`
- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`

## 3) Dependency posture

- Backend coupling risks:
  - Headless surfaces depend only on portable crates (`fret-core` + serde/uuid/etc.).
  - UI surfaces depend on `fret-ui`, `fret-runtime`, `fret-canvas`, `fret-ui-kit` (expected for ecosystem editor UI).
- Layering policy compliance:
  - `kit` is explicitly headless-safe and must remain free of `fret-ui` (keep this invariant).
- Compile-time / complexity hotspots (by file size):
  - `ecosystem/fret-node/src/ui/declarative/paint_only.rs` (~6.2k LOC): high churn risk; continues to benefit from submodule splits.
  - retained conformance test harness files are large, but serve as regression gates for behavior parity.

Evidence anchors:

- `ecosystem/fret-node/Cargo.toml`
- `python tools/audit_crate.py --crate fret-node`
- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`

## 4) Module ownership map (internal seams)

- Headless document model + invariants
  - Files: `ecosystem/fret-node/src/core/*`, `ecosystem/fret-node/src/schema/*`, `ecosystem/fret-node/src/types/*`
- Edit planning + determinism
  - Files: `ecosystem/fret-node/src/rules/*`, `ecosystem/fret-node/src/profile/*`, `ecosystem/fret-node/src/ops/*`
- Runtime store + controlled-mode integration
  - Files: `ecosystem/fret-node/src/runtime/*`, `ecosystem/fret-node/src/io/*`
- UI integration (declarative-first, retained compat)
  - Declarative orchestration: `ecosystem/fret-node/src/ui/declarative/*`
  - Controller facade: `ecosystem/fret-node/src/ui/controller.rs`
  - Style/skin/presenter/registry: `ecosystem/fret-node/src/ui/{style,skin,presenter,registry,presets}.rs`
  - Retained engine (compat only): `ecosystem/fret-node/src/ui/canvas/*` and `ecosystem/fret-node/src/ui/{overlays,portal,editor,editors}/*` behind `compat-retained-canvas`

## 5) Refactor hazards (what can regress easily)

- Transaction-safety drift in declarative surfaces
  - Failure mode: pointer/keyboard interactions commit by mutating `Graph` directly or bypass history/diagnostics.
  - Existing gates: extensive `nextest` coverage in `ui/declarative/paint_only` tests.
  - Missing gate to add (L0): at least one diag-script suite that drives a declarative node-drag + undo/redo path end-to-end (host integration).
- Portal + overlay anchoring determinism
  - Failure mode: frame-lagged bounds cause visible jitter; portal culling/limits break hover overlays or fit-view.
  - Existing gates: retained conformance tests + paint-only focused gates.
  - Missing gate to add (L0): a minimal scripted repro that asserts stable anchoring under pan/zoom + portal throttling.
- “Two canonical ways to integrate” drift (controller vs store)
  - Failure mode: ecosystem/app code copies a store-first seam that later becomes delete-planned, while controller-first remains the intended teaching surface.
  - Existing gates: workstream docs + examples.
  - Missing gate to add (L0): an explicit “golden path” example that only uses `NodeGraphController` + `node_graph_surface` (no raw store wiring in app code).

Evidence anchors:

- `ecosystem/fret-node/src/ui/controller.rs`
- `ecosystem/fret-node/src/ui/declarative/mod.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/README.md`

## 6) Code quality findings (Rust best practices)

- Warnings hygiene: ecosystem warnings exist in baseline across the repo; consider “zero warnings for `fret-node`” as an L1+ goal once the major refactor slices settle.
- Surface clarity: `ui/mod.rs` re-exports a wide set of types (useful for discoverability), but increases the risk of downstream code depending on non-teaching surfaces; keep the “advanced transport” namespace strategy consistent (`ui::advanced::*`).

Evidence anchors:

- `ecosystem/fret-node/src/ui/mod.rs`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/design.md`

## 7) Recommended refactor steps (small, gated)

1. Publish a single “golden path” integration contract:
   - prefer `NodeGraphController` + `node_graph_surface`, treat raw store/queue seams as advanced/compat only — gate: `cargo nextest run -p fret-node --features compat-retained-canvas`
2. Continue shrinking `paint_only.rs` by responsibility-based submodules:
   - keep pointer session reducers, pointer move, keydown, and portal hosting in separate files under `ui/declarative/paint_only/` — gate: existing `nextest` coverage for the declarative module.
3. Collapse `NodeGraphSurfaceProps` to one canonical runtime binding input:
   - (Done) controller-first only (avoid `controller + store` ambiguity) — gate: `cargo nextest run -p fret-node`.
4. Add at least one diagnostics-driven gate for portal/overlay anchoring:
   - capture a minimal scripted scenario to lock the cross-frame bounds/portal throttling behavior — gate: `fretboard diag` (plus existing unit tests).

## 8) Open questions / decisions needed

- Should `fret-node` default features continue to include UI (`fret-ui`) by default, or should headless be the default with an opt-in `ui` feature (tradeoff: ergonomics vs portability)?
- Where should XYFlow-style “nodes as element subtrees in a world layer” live long-term:
  - in `fret-canvas` as a general mechanism/policy recipe (current direction in `docs/workstreams/xyflow-gap-analysis.md`), or
  - as a node-editor-specific portal strategy inside `fret-node`?
