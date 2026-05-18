# Renderer Debug Dump Gate v1 - Design

Status: Closed
Last updated: 2026-05-18

## Problem

Renderer debug dumps duplicated native-only frame gate logic in separate modules:

- render plan dumps parsed `FRET_RENDERPLAN_DUMP*`
- text dumps parsed `FRET_RENDER_TEXT_DUMP*`

Both copies implemented the same contract: an enable flag, optional exact frame, optional
after-frame threshold, optional periodic interval, default one-shot behavior, and optional dump
directory. Keeping the mechanism duplicated made future dump tooling easier to drift.

## Target State

The shared mechanism lives in one native-only renderer module. Each dump owner still owns:

- its environment variable prefix,
- its default directory name,
- its output filename,
- and its JSON schema.

## Scope

- `crates/fret-render-wgpu/src/renderer/debug_dump_gate.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan_dump_emit.rs`
- `crates/fret-render-wgpu/src/renderer/render_text_dump.rs`
- `crates/fret-render-wgpu/src/renderer/mod.rs`

## Non-Goals

- No JSON schema changes.
- No dump filename changes.
- No wasm dump enablement.
- No broader diagnostics API changes.

## Architecture Direction

`debug_dump_gate` is a renderer-internal mechanism module. It gives dump owners a small
`DumpFrameEnv` descriptor and a `should_emit_dump_frame` helper that preserves per-dump one-shot
state by accepting the caller-owned `AtomicBool`.

This keeps policy names local to each dump owner while centralizing the reusable gate behavior.
