# Dependency Policy (Workspace-Level)

This document exists to keep early dependency decisions aligned with Fret’s long-term goals:
desktop-first now, wasm/WebGPU later, and a clean split between UI contracts and backend implementations.

## Principles

- Prefer workspace-managed dependencies (`[workspace.dependencies]`) to keep versions consistent.
- Keep `fret-core` minimal and backend-agnostic:
  - no `wgpu`, no `winit`, no layout engines (e.g. no `taffy`).
- Keep platform/render dependencies out of UI component crates:
  - `fret-components-*` should not depend on `fret-platform-*` or `fret-render-*`.
- Avoid forcing a global async runtime:
  - use `pollster` (or equivalent) at runner boundaries for wgpu init,
  - background work is app-owned and communicates via messages/effects.

## Pinning Policy

- Pin toolchain when required by upstream crates (e.g. wgpu MSRV). See `rust-toolchain.toml`.
- Pin fast-moving foundational crates in `[workspace.dependencies]`:
  - `wgpu`
  - `winit`
- When reading upstream references, prefer pinned `repo-ref/*` checkouts. See `docs/repo-ref.md`.

## Error/Logging Dependencies

- `thiserror`: typed errors in library crates.
- `tracing` (+ optional `tracing-subscriber` in binaries/demos): structured logs/spans.

## Persistence / File Formats

- `serde`: shared derive for versioned on-disk formats (layout/keymap/settings).
- `serde_json`: JSON I/O at app/demo boundaries (aligns with ADR 0014).

## Platform Capabilities

- Clipboard access is implemented at the platform backend boundary (not in UI/core):
  - `fret-platform` defines the portable clipboard contract,
  - `fret-platform-native` uses `arboard` to implement desktop clipboard I/O,
  - apps/runners drain clipboard effects via `Effect`s (see ADR 0041).

## Drift Checks

- Validate crate layering and forbid backend leakage: `python3 tools/check_layering.py`
- Validate ADR ID uniqueness: `python3 tools/check_adr_numbers.py`
