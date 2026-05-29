# Jellyflow Package Split v1

Status: Active
Last updated: 2026-05-29

## Problem

`fret-node` currently means both reusable node graph substrate and Fret UI integration. That makes
the headless story harder to teach, and it makes future standalone extraction harder than it needs
to be.

## Target State

Jellyflow is the reusable node/flow graph engine brand. `fret-node` remains the Fret adapter and
compatibility facade.

The intended package stack is:

1. `jellyflow-core`: graph document model, IDs, type descriptors, and interaction policy value
   types.
2. `jellyflow-runtime`: store/history/apply/callback/controlled-mode substrate.
3. `jellyflow-geometry`: geometry, spatial, path, and hit-test substrate once those seams are ready.
4. `fret-node`: Fret UI adapter, declarative surface, overlays, portals, diagnostics, app
   integration, and compatibility re-exports.

## In Scope

- New in-workspace Jellyflow crates under `ecosystem/`.
- Compatibility re-exports from `fret-node`.
- Dependency gates proving headless crates stay off UI, renderer, platform, and runner crates.
- ADR and usage-guide updates for the package ownership story.

## Out Of Scope

- Moving Jellyflow to a separate repository in this slice.
- Renaming every public `NodeGraph*` type.
- Moving declarative Fret UI, overlays, portals, or retained compatibility code into Jellyflow core.
- Adding a Jellyflow-owned wgpu renderer.

## First Slice

JF-001 creates `jellyflow-core`, moves `core`, `types`, and `interaction` into it, and leaves
`fret-node` wrapper modules so existing `fret_node::{core,types,interaction}` paths keep compiling.
