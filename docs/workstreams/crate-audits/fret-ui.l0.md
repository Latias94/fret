# Crate Audit (L0) — `fret-ui`

Status: L0 complete (quick scan; no deep dive yet)

## Purpose

UI substrate (mechanism-only): element tree, layout/dispatch/paint semantics, overlay mechanisms, deterministic routing and caching behavior.

## Snapshot (from `tools/audit_crate.ps1`)

- Largest files:
  - `crates/fret-ui/src/tree/mod.rs`
  - `crates/fret-ui/src/declarative/tests/layout.rs`
  - `crates/fret-ui/src/declarative/tests/interactions.rs`
  - `crates/fret-ui/src/declarative/tests/virtual_list.rs`
  - `crates/fret-ui/src/tree/dispatch.rs`
  - `crates/fret-ui/src/elements/cx.rs`
  - `crates/fret-ui/src/tree/layout.rs`
  - `crates/fret-ui/src/declarative/mount.rs`
  - `crates/fret-ui/src/element.rs`
  - `crates/fret-ui/src/theme/mod.rs`
- Direct deps (workspace): `fret-core`, `fret-runtime`
- Direct deps (external): `serde`, `serde_json`, `slotmap`, `smallvec`, `stacksafe`, `taffy`, `tracing`,
  `unicode-ident`, `unicode-segmentation`, `virtualizer`
- Kernel forbidden deps spot check: ok (no backend deps)

## Hazards (top candidates)

- Deterministic dispatch/layout/paint ordering (user-visible behavior).
- Hit testing / pointer occlusion / capture arbitration correctness.
- Cache reuse and invalidation correctness (stale paths, ghost interactions, perf cliffs).
- Overlay dismissal + focus restore contracts (mechanism-only, policy in ecosystem).
- Layout recursion / stack safety (infinite loops, overflow, unbounded recursion).

## Recommended next steps (L1 candidates)

1. Split `crates/fret-ui/src/tree/mod.rs` into a facade that re-exports focused submodules.
2. Convert huge conformance tests into data-driven harnesses where possible (ties to `BU-FR-guard-004`).
3. Identify 3–5 “always-run” diag scripts that cover top interaction hazards (ties to `BU-FR-guard-003`).

