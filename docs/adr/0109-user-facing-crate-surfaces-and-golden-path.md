# ADR 0109: User-Facing Crate Surfaces and the Golden Path


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

Status note (2026-04-09): the CLI product split described here has been refined by
`docs/workstreams/fretboard-public-app-author-surface-v1/README.md` and its target-state
companions, and the implementation lane now lives in
`docs/workstreams/fretboard-public-dev-implementation-v1/README.md`. The current shipped public
`fretboard` surface is `new` + `assets` + `config` + project-facing `dev`. Repo-only
`fretboard-dev` currently owns `diag`, `hotpatch`, `list`, `theme`, and richer repo-local `dev`
shortcuts.

## Context

Fret already has a clear kernel/backends/apps split (ADR 0092) and an ecosystem bootstrap/tools story (ADR 0106).
However, “which crates should I depend on?” is still easy to get wrong in practice:

- users should be able to build a working UI app by depending on a small, memorable set of crates,
- advanced users should still be able to assemble everything manually without being forced into defaults,
- “assets” is overloaded (UI render assets vs editor asset pipeline; ADR 0004 vs ADR 0026),
- dev workflows (wasm build, hotpatch) should not leak toolchain complexity into libraries.

We also want to align our external story with the reference repositories we study:

- GPUI (Zed): small portable core, a clear runtime boundary, policy-heavy components above the kernel.
- Dioxus: strong developer tooling story (“one command” workflows), with hot reload/hotpatch remaining optional.

Zed/GPUI code anchors (non-normative):

- runtime substrate: `repo-ref/zed/crates/gpui`
- policy-heavy UI surfaces: `repo-ref/zed/crates/ui`
- settings/keymap and file-backed layering: `repo-ref/zed/crates/settings`

## Goals

- Define the smallest *recommended* dependency surface (“golden path”) for end users.
- Make the role of each public crate unambiguous (kernel vs ecosystem vs tooling).
- Clarify what “resource system” we do and do not provide.
- Ensure the decisions do not conflict with existing ADRs (especially ADR 0004 / 0026 / 0092 / 0106 / 0105).

## Non-goals

- Building an editor-grade project asset pipeline inside the Fret framework (ADR 0026 remains app-owned).
- Replacing ADR 0092’s kernel/backends/apps boundaries.
- Forcing a single “everything crate” as the only entry point (ADR 0106 rejects this).

## Decision

### 1) We standardize three user-facing layers

1. **Kernel (portable framework contracts)**: `crates/*` (ADR 0092)
2. **Ecosystem (policy + defaults + convenience)**: `ecosystem/*` (ADR 0106)
3. **Tooling (dev workflows)**:
   - public product: `crates/fretboard`
   - repo-only maintainer CLI: `apps/fretboard` (`fretboard-dev`)

Hard rules remain:

- `crates/*` must not depend on `ecosystem/*`.
- Component/policy crates in `ecosystem/*` must not depend on backend crates.
- Tooling may shell out to external toolchains; libraries must not.

### 2) The “golden path” dependency set for typical users

For a typical *native* UI app, the recommended set is:

- `fret` (ecosystem meta crate; batteries-included entry points)
  - wraps `fret-bootstrap` + the default component surface (`fret-ui-shadcn`)
- Optional ecosystem helpers (recommended defaults):
  - `fret-selector` (memoized derived state)
  - `fret-query` (async resource state + caching)

The user should not need to understand `winit`, `wgpu`, effects draining, or cache budgets to get their first app running.

Advanced users may choose the “manual assembly” route:

- depend on `fret-framework` + `fret-bootstrap` (or depend on `fret-launch` / backend crates directly) and configure everything themselves,
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
  - shipped today:
    - `new`
    - `assets`
    - `config`
    - project-facing `dev native`
    - project-facing `dev web`
  - future product targets:
    - reduced `diag` core
- repo-only `fretboard-dev` currently owns:
  - mono-repo demo/cookbook launchers and richer repo-only `dev` shortcuts
  - diagnostics maintainer tooling
  - hotpatch helpers and advanced transport flags
  - `theme import-vscode`
- `fret-bootstrap` may expose small helpers for dev env wiring (feature-gated), but must not become a toolchain manager.

### 5) Keep the kernel story unbundled (optional kit allowed)

To keep layering clear and keep portable dependencies explicit, the **kernel** story remains unbundled:

- UI render-asset conveniences live in `fret-ui-assets`.
- App defaults / "starter semantics" (settings load, icon packs, budgets, dev toggles) live in `fret-bootstrap`.

We provide an **ecosystem-level meta crate** (`fret`) for desktop-first quick starts.

## Recommended “User Story” (What we want people to do)

### A) Native app

- Depend on `fret`.
- Build your app using `FnDriver` as the primary authoring surface when you care about dev hotpatch (ADR 0105).
  - Manual-assembly escape hatch: depend on `fret-framework` + `fret-bootstrap` instead.

### B) Web (wasm32)

- Use `fretboard dev web --manifest-path ./Cargo.toml` for a project-facing Trunk workflow.
- In this mono-repo, `fretboard-dev dev web --demo ...` remains the repo demo-shell convenience.
- Library crates do not embed `trunk`, file servers, or websocket dev tooling.

## Alternatives Considered

### A) Put the golden path into `crates/fret-framework` (facade)

Rejected.

The facade must remain backend-agnostic (ADR 0092). Pulling ecosystem defaults or backends into it would collapse
layering and make “minimal portable dependency” impossible.

### B) Provide a single “everything crate” for maximum ergonomics

Rejected (ADR 0106) as the **only** user story / required entry point.

This increases compile times, blurs boundaries, and makes long-term evolution of defaults risky.

### C) Build an engine/editor asset pipeline as part of the framework

Rejected.

It conflicts with ADR 0026’s explicit scope separation and would pull non-UI concerns into the framework kernel.

## Consequences

### Benefits

- Users get a small, consistent dependency story: `fret` (plus optional `fret-query` / `fret-selector`).
- “Resources” are clarified as UI render assets, aligned with ADR 0004, without forcing a full asset pipeline.
- Tooling can iterate quickly without destabilizing core contracts.

### Costs

- Documentation must be kept current as crates move between “recommended” and “advanced” buckets.

## Migration Plan

1. Ensure all examples/demos use `fret` as the default startup path.
2. Update docs to present:
   - “Golden path” (recommended),
   - “Manual assembly” (advanced).
3. Land the frozen follow-on CLI work without reopening the product taxonomy:
   - keep the shipped public project-facing `dev native/web` surface aligned with docs
   - land a public reduced `diag` core follow-on
   - keep hotpatch repo-only until the public `dev` lane is stable
   - keep `theme import-vscode` out of the main public CLI

## References

- Kernel/backends/apps layering: `docs/adr/0092-crate-structure-core-backends-apps.md`
- Ecosystem bootstrap and tooling: `docs/adr/0106-ecosystem-bootstrap-ui-assets-and-dev-tools.md`
- Public CLI taxonomy freeze + closeout: `docs/workstreams/fretboard-public-app-author-surface-v1/FINAL_STATUS.md`
- Golden-path app driver/pipelines: `docs/adr/0110-golden-path-ui-app-driver-and-pipelines.md`
- Resource ownership boundary: `docs/adr/0004-resource-handles.md`
- Editor asset pipeline (out of scope): `docs/adr/0026-asset-database-and-import-pipeline.md`
- Dev hotpatch integration + action hooks policy: `docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md`
- Ecosystem integration guidance (non-binding): `docs/adr/0111-ecosystem-integration-contracts.md`
