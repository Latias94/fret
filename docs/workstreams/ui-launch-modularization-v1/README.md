# UI + Launch Modularization v1 (Fearless Refactor, Track B)

Status: Draft

## Context

Fret’s architecture intentionally separates:

- **Mechanisms / contracts** (portable runtime substrate): `crates/fret-ui`, `crates/fret-runtime`, `crates/fret-core`
- **Policy / components** (Radix/shadcn-style behaviors and defaults): `ecosystem/fret-ui-kit`, `ecosystem/fret-ui-shadcn`
- **Backend wiring / app glue**: `crates/fret-launch`, plus app kits like `ecosystem/fret-kit`

This workstream targets **Track B**: improve internal modularity **without changing crate boundaries**.

Motivation:

- `crates/fret-ui` and `crates/fret-launch` contain very large modules, which increases coupling and “hidden contracts”.
- We want clearer seams so future Track C (optional crate splitting) becomes straightforward and low-risk.

## Goals

1. Reduce “god files” by splitting responsibilities into focused modules.
2. Make mechanism vs policy boundaries harder to violate by accident.
3. Keep public APIs stable (or change them only with explicit review + migration notes).
4. Preserve build/test behavior; avoid Cargo workspace reshuffles in this track.

## Non-goals

- No crate splits/renames in v1 (that is Track C).
- No intentional behavior changes.
- No new external dependencies.

## Guardrails

- `fret-ui` stays backend-free: no `wgpu`, no `winit`, no platform IO.
- Ecosystem component crates stay backend-free: no `fret-render-*`, `fret-runner-*`, `fret-platform-*`.
- `fret-launch` remains glue, but should not become an “everything crate”.

Recommended gates:

- `python tools/check_layering.py`
- `cargo fmt`
- `cargo nextest run -p fret-ui`
- `cargo nextest run -p fret-launch` (when touching launch wiring)

## Scope

### 1) `crates/fret-ui`: split by responsibility

Primary target: the `tree/` subsystem.

Desired structure (illustrative; naming can evolve):

- `tree/mod.rs`: module root only (types + re-exports), minimal implementations
- `tree/state/*`: tree storage, IDs, stable invariants
- `tree/mount/*`: declarative mount/diff/build logic
- `tree/dispatch/*`: event routing, capture/bubble, hit-test integration seams
- `tree/layout/*`: layout passes, invalidation, scheduling integration points
- `tree/paint/*`: scene emission boundaries (mechanism only)
- `tree/diag/*`: diagnostics, tracing helpers, test-only scaffolding

Policy outcomes (Radix/shadcn) stay in `ecosystem/*`.

### 2) `crates/fret-launch`: split by wiring topology

Primary target: the desktop runner module (winit + wgpu wiring).

Desired structure:

- `runner/common/*`: shared helpers within launch (not UI/runtime)
- `runner/desktop/*`: window lifecycle + event loop + surface/present + OS integration
- `runner/web/*`: wasm wiring + RAF scheduling + web effects
- `runner/diag/*`: screenshot/diag IO (separated from hot path)

Rule of thumb: if a helper is reusable policy, move it to ecosystem; if it’s backend-specific, move it to the
backend crate. Avoid accumulating ad-hoc utilities in launch.

## Acceptance Criteria

- `crates/fret-ui/src/tree/*` no longer has single-file “god modules”.
  - Initial target: < 2k lines per file; module roots are “thin”.
- No new layering violations; `tools/check_layering.py` remains green.
- No `fret-ui` public surface creep driven by component policy.
- Tests pass for touched crates (at least `nextest` for `fret-ui` / `fret-launch`).

## Baseline (2026-02-12)

Captured via `tools/audit_crate.ps1`:

- `fret-ui` top files:
  - `crates/fret-ui/src/tree/mod.rs` (~7723 lines)
  - `crates/fret-ui/src/elements/cx.rs` (~3680 lines)
  - `crates/fret-ui/src/tree/dispatch.rs` (~3627 lines)
  - `crates/fret-ui/src/tree/layout.rs` (~2800 lines)
- `fret-launch` top files:
  - `crates/fret-launch/src/runner/desktop/mod.rs` (~6654 lines)

## Track C (Follow-up, optional)

After Track B stabilizes seams, consider:

- splitting `fret-launch` into `fret-launch-desktop` and `fret-launch-web`, with `fret-launch` as a thin facade,
- or renaming runner/backends for clearer user cognition.
