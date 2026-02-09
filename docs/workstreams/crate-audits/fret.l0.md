# Crate audit (L0) — `fret`

## Crate

- Name: `fret`
- Path: `crates/fret`
- Owners / adjacent crates: all kernel crates + launch/platform/runner crates (as optional deps)
- Current “layer”: user-facing facade (manual/advanced assembly)

## 1) Purpose (what this crate *is*)

- A small, memorable facade crate that re-exports selected workspace crates behind opt-in feature flags.
- Default feature set is intentionally minimal (`core` only); larger bundles are explicit (`desktop`, `wasm`).
- Intended as an “advanced assembly” entry point; batteries-included lives elsewhere (e.g. `fret-kit`, `fretboard`).

Evidence anchors:

- `crates/fret/src/lib.rs`
- `crates/fret/Cargo.toml`
- ADR 0111: `docs/adr/0111-user-facing-crate-surfaces-and-golden-path.md`

## 2) Public contract surface

- Key exports / stable types:
  - Namespaced module re-exports (feature-gated): `core::*`, `ui::*`, `runtime::*`, `render::*`, etc.
  - `prelude` provides a small ergonomic set for app glue (also feature-gated).
- “Accidental” exports to consider removing:
  - Low risk: the surface is intentionally explicit and namespaced; drift mainly comes from what upstream crates export.
- Feature flags and intent:
  - Optional deps + explicit bundles (`desktop`, `wasm`) are a good posture for compile time and dependency hygiene.

Evidence anchors:

- `crates/fret/src/lib.rs`
- `crates/fret/Cargo.toml`

## 3) Dependency posture

- External deps: none (workspace-only, optional deps).
- Layering policy:
  - This crate is a facade; it can point at backend crates via feature flags, but should not accidentally enable heavy stacks by default.
- Compile-time hotspots / heavy deps:
  - Risk is feature bundle sprawl (more combinations than CI checks), not code size.

Evidence anchors:

- `crates/fret/Cargo.toml`
- `pwsh -NoProfile -File tools/audit_crate.ps1 -Crate fret`

## 4) Module ownership map (internal seams)

- Namespaced re-export modules: `core`, `app`, `ui`, `runtime`, `render`, `fonts`, `platform*`, `runner*`, `launch`
  - Files: `crates/fret/src/lib.rs` only
- Ergonomic imports: `prelude`
  - Files: `crates/fret/src/lib.rs`

## 5) Refactor hazards (what can regress easily)

- Feature bundle correctness and portability
  - Failure mode: a feature combo compiles on one platform but not another (wasm32 vs native).
  - Existing gates: none specific.
  - Missing gate to add: `cargo check -p fret --features desktop` and `cargo check -p fret --features wasm --target wasm32-unknown-unknown` in CI.
- “Golden path” confusion (users pick `fret` when they should pick `fret-kit`)
  - Failure mode: pressure to add policy defaults into this crate, bloating the facade.
  - Existing gates: ADR 0111 documents intent.
  - Missing gate to add: keep `README`/docs pointers current; consider a short crate-level README excerpt in rustdoc.

## 6) Code quality findings (Rust best practices)

- Simple and idiomatic: pure re-export facade with clear feature gates.
- Main risk is governance (what belongs here) rather than implementation correctness.

Evidence anchors:

- `crates/fret/src/lib.rs`

## 7) Recommended refactor steps (small, gated)

1. Add compile gates for the main bundles (`desktop`, `wasm`) — outcome: feature-combo drift becomes obvious — gate: `cargo check -p fret --features desktop` + `cargo check -p fret --features wasm --target wasm32-unknown-unknown`.
2. Keep `prelude` intentionally small and stable — outcome: downstream churn is minimized — gate: `cargo nextest run -p fret` (smoke).

## 8) Open questions / decisions needed

- Should `fret` keep the current “namespaced modules only” posture, or should it also expose a small set of top-level re-exports for the most common types (risk: API drift)?

