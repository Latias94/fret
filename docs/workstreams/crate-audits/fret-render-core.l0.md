# Crate audit (L0) — `fret-render-core`

## Crate

- Name: `fret-render-core`
- Path: `crates/fret-render-core`
- Owners / adjacent crates: renderer crates (`fret-render`, `fret-render-wgpu`), `fret-ui` (render-facing types only)
- Current “layer”: renderer contract (portable)

## 1) Purpose (what this crate *is*)

- A tiny, portable set of render-facing contract types intended to be shared by multiple renderer backends.
- Must remain backend-agnostic (no `wgpu`, no platform/window concepts).

Evidence anchors:

- `crates/fret-render-core/src/lib.rs`

## 2) Public contract surface

- Key exports / stable types:
  - `RenderTargetColorSpace`
- Feature flags and intent:
  - none
- “Accidental” exports to consider removing:
  - none observed (crate is intentionally minimal)

Evidence anchors:

- `crates/fret-render-core/src/lib.rs`

## 3) Dependency posture

- Backend coupling risks: none (direct dependency is `serde` only).
- Layering policy compliance: expected (portable contract crate).
- Compile-time hotspots / heavy deps: none.

Evidence anchors:

- `crates/fret-render-core/Cargo.toml`
- `pwsh -NoProfile -File tools/audit_crate.ps1 -Crate fret-render-core`

## 4) Module ownership map (internal seams)

- Crate root — portable contract types
  - Files: `crates/fret-render-core/src/lib.rs`

## 5) Refactor hazards (what can regress easily)

- Serialization format drift for contract enums
  - Failure mode: persisted configs/goldens drift when renaming variants or changing `serde` casing policy.
  - Existing gates: unit test covering snake_case serialization.
  - Missing gate to add: if this enum becomes persisted in settings/layout, add an integration-level roundtrip gate in the owning persisted-format crate.

Evidence anchors:

- `crates/fret-render-core/src/lib.rs`

## 6) Code quality findings (Rust best practices)

- Simple, idiomatic, portable code.
- Recommendation: keep this crate intentionally small; prefer adding new portable types here only when multiple renderer backends need them.

Evidence anchors:

- `crates/fret-render-core/src/lib.rs`

## 7) Recommended refactor steps (small, gated)

1. Keep contract enums serialization-stable (snake_case) — outcome: avoid accidental golden/persisted drift — gate: `cargo nextest run -p fret-render-core`.
2. If/when adding more contract types, split into a directory module (`src/contract/*`) — outcome: preserve a stable facade while keeping internal organization flexible — gate: `cargo nextest run -p fret-render-core`.

## 8) Open questions / decisions needed

- Should `RenderTargetColorSpace` be `#[non_exhaustive]` (publicly forward-compatible) or remain exhaustive for internal stability? Decide once downstream crates start matching exhaustively across public APIs.

