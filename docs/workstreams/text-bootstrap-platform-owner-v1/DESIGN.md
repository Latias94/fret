# Text Bootstrap Platform Owner v1 - Design

Status: Closed
Last updated: 2026-05-18

## Problem

`text/bootstrap.rs` mixed two responsibilities:

- assembling `TextSystem` runtime state,
- selecting the platform-specific startup `ParleyShaper`.

The wasm/native shaper contract is important, but it is a platform policy detail rather than the
main text-system assembly path.

## Target State

`text/bootstrap.rs` owns text-system assembly. Platform-specific shaper startup lives in:

- `crates/fret-render-wgpu/src/text/bootstrap/platform.rs`

The existing contract is preserved:

- wasm uses bundled-only fonts because no truthful system-font capability exists there today,
- native keeps using the default `ParleyShaper::new()` constructor.

## Scope

- `crates/fret-render-wgpu/src/text/bootstrap.rs`
- `crates/fret-render-wgpu/src/text/bootstrap/platform.rs`

## Non-Goals

- No fallback-policy changes.
- No atlas bootstrap changes.
- No public API changes.
- No web system-font enablement.

## Architecture Direction

Keep bootstrap assembly declarative. Target-specific startup policy should live in a named owner
module so future platform changes do not clutter the `TextSystem` construction path.
