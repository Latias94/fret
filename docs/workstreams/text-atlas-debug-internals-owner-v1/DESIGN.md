# Text Atlas Debug Internals Owner v1 - Design

Status: Closed
Last updated: 2026-05-18

## Problem

`text/atlas.rs` and `text/atlas_runtime_state.rs` still carry native-only debug lookup helpers and
dimension accessors inline with the main atlas/runtime implementations. That keeps dump-only
inspection logic mixed into the general text atlas surface.

## Target State

Native-only atlas debug internals live in sibling `debug.rs` modules owned by the same parent
module, while the main atlas/runtime files keep only production behavior and module wiring.

## Scope

- `crates/fret-render-wgpu/src/text/atlas.rs`
- `crates/fret-render-wgpu/src/text/atlas/debug.rs`
- `crates/fret-render-wgpu/src/text/atlas_runtime_state.rs`
- `crates/fret-render-wgpu/src/text/atlas_runtime_state/debug.rs`

## Non-Goals

- No atlas allocation or cache policy changes.
- No dump schema changes.
- No public API expansion.
- No wasm debug enablement.

## Architecture Direction

Use native-only sibling modules for dump-specific internals so the main text atlas modules stop
owning inspection helpers that are only needed by renderer dump code.
