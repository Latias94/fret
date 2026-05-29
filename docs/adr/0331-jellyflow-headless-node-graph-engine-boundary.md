# ADR 0331: Jellyflow Headless Node Graph Engine Boundary

Status: Accepted
Date: 2026-05-29

## Context

`fret-node` began as the in-tree node graph foundation plus Fret UI integration surface. As the
declarative path matures, the crate now carries two different responsibilities:

- a portable node/flow graph engine that should be useful without Fret UI, renderers, or platform
  runners,
- the Fret adapter that maps that engine into `fret-ui`, `fret-canvas`, overlays, portals, focus,
  diagnostics, and app integration.

Keeping both meanings under the `fret-node` name makes headless use and future extraction less
obvious. It also makes dependency drift harder to audit: a headless consumer should not need to know
which modules are safe to use under `--no-default-features`.

## Decision

Introduce **Jellyflow** as the neutral engine brand for the reusable node/flow graph substrate.
`fret-node` remains the Fret ecosystem adapter and compatibility facade.

The split starts inside this monorepo. We do not move Jellyflow to a separate repository until the
crate boundaries and compatibility story have survived focused compile and behavior gates.

The intended package direction is:

- `jellyflow-core`: headless graph document model, stable IDs, type descriptors, and interaction
  policy value types.
- `jellyflow-runtime`: future headless store/history/apply/callback/controlled-mode substrate.
- `jellyflow-geometry`: future headless geometry, spatial, path, and hit-test substrate when those
  contracts are ready to leave the Fret adapter.
- `jellyflow-fret` or `fret-node`: Fret UI adapter, controller/binding, declarative surface,
  overlays, portals, diagnostics, and compatibility re-exports.

`jellyflow-core` must stay free of `fret-ui`, `fret-runtime`, `fret-canvas`, `wgpu`, and `winit`.
Fret renderer integration belongs in the Fret adapter layer, not in the Jellyflow core.

## Consequences

- `fret-node` can keep existing user-facing paths by re-exporting Jellyflow modules during the
  transition.
- Headless consumers get a smaller compile target and a clearer dependency contract.
- Future repository extraction becomes mechanical only after the package boundaries stabilize.
- Some names will temporarily contain `NodeGraph*` even inside Jellyflow crates. Renaming those
  symbols is a separate API migration and should not block the first package split.

## Initial Implementation

The first slice creates `ecosystem/jellyflow-core` and moves the stable headless `core`, `types`,
and `interaction` modules there. `fret-node` now depends on `jellyflow-core` and keeps
compatibility wrapper modules for `fret_node::{core,types,interaction}`.

Evidence:

- `ecosystem/jellyflow-core/Cargo.toml`
- `ecosystem/jellyflow-core/src/lib.rs`
- `ecosystem/fret-node/src/{core,types,interaction}/mod.rs`
- `docs/workstreams/jellyflow-package-split-v1/`
