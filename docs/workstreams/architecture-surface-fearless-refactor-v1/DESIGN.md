# Architecture Surface Fearless Refactor v1

Status: Active
Last updated: 2026-05-17

## Why This Lane Exists

Fret's hard layering gates are currently healthy, but the public and ecosystem surfaces have grown
wide enough that the intended architecture is harder to consume than it should be. The most important
friction is not a direct `wgpu`/`winit` leak into contract crates; it is that several high-level
modules now require callers to understand backend wiring, bootstrap policy, component policy,
state/action authoring, and advanced interop at the same time.

This lane records a pre-release fearless refactor program for narrowing those surfaces. Compatibility
with old in-repo interfaces is not a goal. When the correct design is to delete aliases, wrappers,
or redundant modules, delete them and migrate first-party callers.

## Relevant Authority

- ADRs:
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `docs/adr/0092-crate-structure-core-backends-apps.md`
  - `docs/adr/0106-ecosystem-bootstrap-ui-assets-and-dev-tools.md`
  - `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
  - `docs/adr/0110-golden-path-ui-app-driver-and-pipelines.md`
  - `docs/adr/0154-ecosystem-crate-taxonomy-glue-and-ui-kit-split-v1.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/architecture.md`
  - `docs/dependency-policy.md`
  - `docs/golden-architecture.md`
  - `docs/repo-structure.md`
- Related workstreams:
  - `docs/workstreams/framework-modularity-fearless-refactor-v1/`
  - `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/`
  - `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/`
  - `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/`
  - `docs/workstreams/menu-surfaces-alignment-v1/`
  - `docs/workstreams/renderer-modularity-fearless-refactor-v1/`

## Problem

### 1. The `fret` minimal consumption path is not actually minimal

`cargo tree -p fret --no-default-features -e normal --depth 3` still pulls `fret-launch`,
`fret-render`, `wgpu`, `winit`, native platform crates, and the renderer stack. That contradicts the
feature-level mental model where backend selection should be explicit (`desktop`, `native-wgpu`,
manual assembly, or direct launch dependencies).

### 2. `fret-bootstrap` has become heavier than its ADR role

ADR 0106 describes `fret-bootstrap` as a thin ecosystem startup composition layer. In practice,
`fret-bootstrap --no-default-features` still depends on `fret-launch` and `fret-render`, so callers
that only want bootstrap plans, defaults, settings, assets, or icon setup inherit backend knowledge.

### 3. The `fret` facade mixes too many caller lanes

`ecosystem/fret/src/lib.rs` and `ecosystem/fret/src/view.rs` are both large and public-facing. The
facade currently mixes app entry, view runtime, local state, typed actions, selector/query/mutation,
router, assets, IMUI, advanced interop, and lower-level component authoring helpers. Many of these
surfaces are valuable, but the interface is too wide for the Golden Path.

### 4. Ecosystem taxonomy is only partially closed

ADR 0154 points toward a split between headless engines, UI primitives, kit helpers, and recipe
surfaces. The current code still routes many callers through `fret-ui-kit` and compatibility
re-exports. `fret-ui-headless` and `fret-authoring` exist, while the broader primitives split is
still not finalized.

### 5. Menu/select component policies are repeated in giant recipe files

`select.rs`, `dropdown_menu.rs`, `context_menu.rs`, and related shadcn surfaces are large enough that
overlay policy, roving focus, typeahead, submenu intent, and dismissal behavior can drift across
files. That policy belongs behind a shared interaction module, with recipe files acting as adapters.

### 6. The `fret-render` facade is either shallow or under-specified

`fret-render` currently behaves as a default wgpu facade. If only one backend exists, it may be a
shallow compatibility module. If it is meant to be the renderer interface, it needs a stronger
contract around capabilities, validation, perf snapshots, and backend selection.

## Target State

- `fret --no-default-features` is a backend-free app-authoring facade, or the package exposes no
  misleading backend-free mode at all. The feature table and `cargo tree` gates agree.
- `fret-bootstrap` can express backend-free bootstrap plans and policy defaults without pulling in
  launch/render stacks. Concrete launch adapters are explicitly feature-gated or moved.
- `fret` keeps a narrow Golden Path prelude. Advanced interop and lower-level authoring escape
  hatches remain available only through explicit modules or direct crate dependencies.
- Ecosystem taxonomy is concrete:
  - `fret-ui-headless`: deterministic headless behavior engines and state machines.
  - `fret-ui-primitives` or an equivalent finalized module/crate: UI-runtime adapters for shared
    primitive behavior.
  - `fret-ui-kit`: composition helpers and design-system infrastructure.
  - recipe crates (`fret-ui-shadcn`, `fret-ui-material3`, editor surfaces): visual/component
    adapters, not owners of shared interaction policy.
- Menu/select shared behavior is owned by one interaction module with focused conformance tests.
- The renderer facade is either deleted/collapsed as a shallow compatibility module, or deepened into
  the real renderer interface with at least one gate proving its value.

## In Scope

- Cargo feature and dependency reshaping for `fret`, `fret-bootstrap`, `fret-framework`,
  `fret-launch`, and related app-authoring surfaces.
- Public surface narrowing, including hard deletion of compatibility aliases and redundant wrappers
  when first-party callers can be migrated.
- Ecosystem crate/module taxonomy cleanup around headless, primitives, kit, and recipe layers.
- Shared menu/select policy extraction when it reduces drift and creates a deeper module.
- Renderer facade decision and implementation plan.
- Gate updates: cargo tree checks, layering checks, targeted nextest gates, and docs alignment.

## Out Of Scope

- Preserving downstream compatibility for pre-release names.
- Large visual redesigns of shadcn/material/editor components unless needed to move policy to the
  correct module.
- Reopening accepted ADRs unless implementation evidence shows the accepted contract is wrong.
- Full workspace-wide formatting or broad mechanical cleanup unrelated to this lane.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Layering rules are currently mostly enforced. | High | `python tools/check_layering.py` passed on 2026-05-17. | If false, prioritize hard dependency fixes before surface cleanup. |
| Consumption profile gates exist but do not cover the `fret` no-default path. | High | `python tools/check_consumption_profiles.py` passed, while `cargo tree -p fret --no-default-features` still showed backend deps. | Add or adjust profile gates before refactoring public surfaces. |
| Fret is still pre-release enough to delete incorrect public-looking surfaces. | High | User explicitly approved fearless refactor with no compatibility burden. | If release policy changes, split compatibility migration into a separate lane. |
| `fret-ui` should stay mechanism-only. | High | ADR 0066 and golden architecture. | If a shared behavior truly needs runtime mechanisms, write/update an ADR first. |
| Giant shadcn recipe files hide shared policy repetition. | Medium | File size scan plus menu/select family ownership overlap. | If extraction reveals little duplication, keep recipe ownership and only add tests. |
| `fret-render` needs either deletion or deeper contract. | Medium | Current facade has one default backend; renderer modularity workstreams exist. | If future backend requirements are imminent, deepen instead of collapse. |

## Architecture Direction

Use the deletion test aggressively. A module earns its place only if deleting it would spread
complexity across callers. If deleting a facade simply removes an indirection, delete it. If a facade
is retained, make it deep by placing a meaningful interface behind it: feature selection, capability
reporting, validation, default policy, or adapter orchestration.

The default flow should be:

1. App authors depend on `fret` for a small Golden Path.
2. Framework integrators depend on `fret-framework`, `fret-launch`, or direct backend crates.
3. Component authors depend on `fret-ui`, `fret-runtime`, `fret-ui-headless`, and the finalized
   primitive/kit layers they actually need.
4. Recipe crates adapt shared primitive behavior to a design-system taxonomy.

When a caller currently uses a compatibility path, migrate it to the target surface and delete the
compatibility path in the same task whenever possible.

## Closeout Condition

This lane can close when:

- the `fret` and `fret-bootstrap` feature/dependency story matches documented consumption profiles,
- `cargo tree` gates prevent backend leakage in minimal app-authoring profiles,
- the `fret` Golden Path prelude and advanced escape hatches are intentionally separated,
- the headless/primitives/kit/recipe taxonomy is shipped or a narrower follow-on owns the remaining
  taxonomy work,
- at least one shared menu/select behavior module replaces duplicated recipe-local policy, or the
  extraction is explicitly rejected with evidence,
- the renderer facade decision is recorded and implemented or split into a renderer-specific lane,
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md` and relevant docs reflect shipped behavior.
