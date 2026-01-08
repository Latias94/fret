# ADR 0111: User-Facing Crate Surfaces and the Golden Path

Status: Accepted

## Context

Fret already has a clear kernel/backends/apps split (ADR 0093) and an ecosystem bootstrap/tools story (ADR 0108).
However, “which crates should I depend on?” is still easy to get wrong in practice:

- users should be able to build a working UI app by depending on a small, memorable set of crates,
- advanced users should still be able to assemble everything manually without being forced into defaults,
- “assets” is overloaded (UI render assets vs editor asset pipeline; ADR 0004 vs ADR 0026),
- dev workflows (wasm build, hotpatch) should not leak toolchain complexity into libraries.

We also want to align our external story with the reference repositories we study:

- GPUI (Zed): small portable core, a clear runtime boundary, policy-heavy components above the kernel.
- Dioxus: strong developer tooling story (“one command” workflows), with hot reload/hotpatch remaining optional.

## Goals

- Define the smallest *recommended* dependency surface (“golden path”) for end users.
- Make the role of each public crate unambiguous (kernel vs ecosystem vs tooling).
- Clarify what “resource system” we do and do not provide.
- Ensure the decisions do not conflict with existing ADRs (especially ADR 0004 / 0026 / 0093 / 0108 / 0107).

## Non-goals

- Building an editor-grade project asset pipeline inside the Fret framework (ADR 0026 remains app-owned).
- Replacing ADR 0093’s kernel/backends/apps boundaries.
- Forcing a single “everything crate” as the only entry point (ADR 0108 rejects this).

## Decision

### 1) We standardize three user-facing layers

1. **Kernel (portable framework contracts)**: `crates/*` (ADR 0093)
2. **Ecosystem (policy + defaults + convenience)**: `ecosystem/*` (ADR 0108)
3. **Tooling (dev workflows)**: `apps/fretboard` (ADR 0108)

Hard rules remain:

- `crates/*` must not depend on `ecosystem/*`.
- Component/policy crates in `ecosystem/*` must not depend on backend crates.
- Tooling may shell out to external toolchains; libraries must not.

### 2) The “golden path” dependency set for typical users

For a typical *native* UI app, the recommended set is:

- `fret` (facade; portable re-exports)
- one component surface (e.g. `fret-ui-kit`, `fret-ui-shadcn`)
- `fret-bootstrap` (optional, recommended): settings + icons + budgets + dev toggles
- `fret-ui-assets` (optional, recommended if you show images/icons): UI render-asset caches and helpers

The user should not need to understand `winit`, `wgpu`, effects draining, or cache budgets to get their first app running.

Advanced users may choose the “manual assembly” route:

- depend on `fret-launch` directly (or backend crates directly) and configure everything themselves,
- still respect ADR 0004’s resource ownership and handle-based IDs.

### 3) “Resource system”: we provide UI render assets, not a project asset pipeline

We explicitly separate concerns:

- **UI render assets** are supported as an ecosystem-level convenience:
  - caches map “keys” (URLs, embedded bytes, icon names) → decode/load → register via effects flush → stable IDs (ADR 0004),
  - budgeting/eviction/stats are part of the ecosystem layer (`fret-ui-assets`).
- **Editor project assets** (GUID identity, import graphs, derived artifacts) remain *out of scope* for the framework kernel
  and must live in applications or separate editor tooling (ADR 0026).

This allows us to give users a practical “resource system” for UI without drifting into an engine/editor architecture.

### 4) Dev workflow entry points live in tooling, not in library crates

We adopt the Dioxus lesson: dev UX matters, but it must be kept out of core libraries.

- `fretboard` is the canonical “one command” entry point for:
  - `dev native` (run a selected demo/app),
  - `dev web` (shell out to `trunk serve` or equivalent),
  - enabling dev-only hotpatch/hotreload wiring by setting environment variables and feature flags.
- `fret-bootstrap` may expose small helpers for dev env wiring (feature-gated), but must not become a toolchain manager.

### 5) `fret-app-kit` is transitional and must not be required for the golden path

`fret-app-kit` historically mixed concerns (app defaults + UI asset conveniences).
To reduce cognition and dependency ambiguity:

- UI render-asset conveniences live in `fret-ui-assets`.
- App defaults / “starter semantics” (settings load, icon packs, budgets, dev toggles) live in `fret-bootstrap`.
- `fret-app-kit` may remain temporarily as a compatibility shim with re-exports, but new examples and docs must not depend on it.

## Recommended “User Story” (What we want people to do)

### A) Native app

- Depend on `fret`, a component crate, plus `fret-bootstrap`.
- Build your app using `FnDriver` as the primary authoring surface when you care about dev hotpatch (ADR 0107).

### B) Web (wasm32)

- Use `fretboard dev web` (tooling) to run the wasm harness via `trunk`.
- Library crates do not embed `trunk`, file servers, or websocket dev tooling.

## Alternatives Considered

### A) Put the golden path into `crates/fret` (facade)

Rejected.

The facade must remain backend-agnostic (ADR 0093). Pulling ecosystem defaults or backends into it would collapse
layering and make “minimal portable dependency” impossible.

### B) Provide a single “everything crate” for maximum ergonomics

Rejected (ADR 0108).

This increases compile times, blurs boundaries, and makes long-term evolution of defaults risky.

### C) Build an engine/editor asset pipeline as part of the framework

Rejected.

It conflicts with ADR 0026’s explicit scope separation and would pull non-UI concerns into the framework kernel.

## Consequences

### Benefits

- Users get a small, consistent dependency story: `fret` + components (+ `fret-bootstrap`).
- “Resources” are clarified as UI render assets, aligned with ADR 0004, without forcing a full asset pipeline.
- Tooling can iterate quickly without destabilizing core contracts.

### Costs

- We must maintain a clear migration path away from `fret-app-kit` (re-exports/deprecations).
- Documentation must be kept current as crates move between “recommended” and “advanced” buckets.

## Migration Plan

1. Ensure all examples/demos use `fret-bootstrap` as the default startup path.
2. Move remaining UI render-asset convenience APIs out of `fret-app-kit` into `fret-ui-assets` (keep a shim temporarily).
3. Update docs to present:
   - “Golden path” (recommended),
   - “Manual assembly” (advanced).
4. Expand `fretboard` to support stable “dev native/web” workflows (without moving toolchain concerns into libraries).

## References

- Kernel/backends/apps layering: `docs/adr/0093-crate-structure-core-backends-apps.md`
- Ecosystem bootstrap and tooling: `docs/adr/0108-ecosystem-bootstrap-ui-assets-and-dev-tools.md`
- Golden-path app driver/pipelines: `docs/adr/0112-golden-path-ui-app-driver-and-pipelines.md`
- Resource ownership boundary: `docs/adr/0004-resource-handles.md`
- Editor asset pipeline (out of scope): `docs/adr/0026-asset-database-and-import-pipeline.md`
- Dev hotpatch integration + action hooks policy: `docs/adr/0107-dev-hotpatch-subsecond-and-hot-reload-safety.md`
- Ecosystem integration guidance (non-binding): `docs/adr/0113-ecosystem-integration-contracts.md`
