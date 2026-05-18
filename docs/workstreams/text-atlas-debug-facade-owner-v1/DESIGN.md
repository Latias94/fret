# Text Atlas Debug Facade Owner v1 - Design

Status: Closed
Last updated: 2026-05-18

## Problem

`text/diagnostics.rs` mixed two separate responsibilities:

- general text diagnostics snapshots used by the renderer telemetry path,
- native-only atlas debug lookup facades used by renderer text JSON dumps.

That left method-level wasm/native `cfg` branches inside the general diagnostics module even though
the debug atlas facade is a dump-only native surface.

## Target State

General diagnostics stay target-neutral. Native-only atlas debug lookup and atlas-dimension facade
methods live in a native-only owner module:

- `crates/fret-render-wgpu/src/text/diagnostics_debug.rs`

The atlas runtime API and dump schema remain unchanged.

## Scope

- `crates/fret-render-wgpu/src/text/diagnostics.rs`
- `crates/fret-render-wgpu/src/text/diagnostics_debug.rs`
- `crates/fret-render-wgpu/src/text/mod.rs`
- `crates/fret-render-wgpu/src/renderer/render_text_dump.rs` as the consumer evidence anchor.

## Non-Goals

- No atlas allocation or cache behavior changes.
- No renderer text dump JSON schema changes.
- No wasm dump enablement.
- No public API changes.
- No deeper atlas/runtime private-field split in this slice.

## Architecture Direction

Keep module ownership aligned with runtime surface area. General diagnostics should own snapshots and
telemetry; target-specific debug dump facades should live behind target-specific module selection.
