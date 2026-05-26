# `fret-node` UI Mirror Inventory - 2026-05-26

Status: first FNRS-040 inventory

## Purpose

This inventory identifies long-lived UI-side graph/view/editor-config mirrors that can drift from
the authoritative runtime store if sync seams are bypassed.

Runtime/store correctness is now closed enough to start this cleanup:

- `FNRS-010`: `NodeGraphChanges` no longer silently drops node/edge metadata operations.
- `FNRS-020`: `NodeGraphLookups` stays fresh for lookup-affecting dispatch operations.
- `FNRS-030`: `NodeGraphStore` commit finalization is centralized.

## Mirror Map

| Surface | Mirror | Owner | Current reason | Risk | FNRS-040 action |
| --- | --- | --- | --- | --- | --- |
| `NodeGraphSurfaceBinding` | `graph`, `view_state`, `editor_config` models | App/declarative binding | Compatibility with external app-owned models and examples that still inspect bound mirrors | These can be mistaken for authoritative runtime state | Quarantine behind private `NodeGraphSurfaceMirrors` and keep public accessors explicit |
| `NodeGraphController` sync helpers | Caller-provided graph/view/editor-config models | Advanced app sync seams | Controlled-mode and explicit mirror-owned app integration | Repeated sync calls can grow accidental mirror write paths | Keep for now; future slices should route new app-facing work through binding/store first |
| Retained `NodeGraphCanvas` | graph/view/editor-config models | Retained compatibility island | Existing retained tests and compatibility surfaces still need explicit models | Largest remaining retained-state drift risk | Do not edit in the first FNRS-040 slice; requires retained compatibility gate selection |
| Declarative `paint_only` tests | fixture graph/view mirrors | Test-only fixtures | Prove store-vs-mirror boundaries and external sync behavior | Tests can normalize mirror reads unless source-policy guard remains | Keep source-policy test that runtime files use `binding.store_model()` |

## First Slice Decision

The first FNRS-040 implementation slice should quarantine `NodeGraphSurfaceBinding` mirrors without
changing public accessors:

- keep `graph_model()`, `view_state_model()`, and `editor_config_model()` for compatibility,
- keep `sync_from_store*` behavior unchanged,
- move private fields into `NodeGraphSurfaceMirrors`,
- update the surface-policy test to assert the explicit mirror container,
- leave retained canvas model ownership untouched until a retained compatibility gate is chosen.

This is a safe first step because it reduces the chance of adding new mirror fields directly to the
binding while preserving all public behavior.

