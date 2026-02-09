# Crate audit (L0) — `fret-render`

## Crate

- Name: `fret-render`
- Path: `crates/fret-render`
- Owners / adjacent crates: `fret-render-wgpu` (default backend), downstream apps/runners that historically depend on `fret-render`
- Current “layer”: renderer facade / compatibility shim

## 1) Purpose (what this crate *is*)

- A compatibility facade that keeps the historical `fret-render` crate name stable.
- Today it re-exports the default renderer backend (`fret-render-wgpu`) so downstream crates can depend on `fret-render` without knowing backend crate names.

Evidence anchors:

- `crates/fret-render/src/lib.rs`
- `crates/fret-render/Cargo.toml`

## 2) Public contract surface

- Key exports / stable types:
  - Re-export of `fret-render-wgpu` (full surface).
- Feature flags and intent:
  - none (L0 observation): this means `fret-render` always pulls the wgpu backend.
- “Accidental” exports to consider removing (L0 hypothesis):
  - If/when we introduce multiple backends, avoid `pub use *` across them; prefer a small facade surface with explicit exports.

Evidence anchors:

- `crates/fret-render/src/lib.rs`

## 3) Dependency posture

- Direct deps: only `fret-render-wgpu`.
- Transitive deps: the full wgpu/text/svg stack via `fret-render-wgpu`.
- Layering policy compliance: expected (renderer layer), but note that `fret-render` is not a “portable contract crate”.

Evidence anchors:

- `crates/fret-render/Cargo.toml`
- `pwsh -NoProfile -File tools/audit_crate.ps1 -Crate fret-render`

## 4) Module ownership map (internal seams)

- Crate root — re-export facade
  - Files: `crates/fret-render/src/lib.rs`

## 5) Refactor hazards (what can regress easily)

- Facade drift (re-export set changes silently)
  - Failure mode: downstream crates break unexpectedly if the facade stops re-exporting key types.
  - Existing gates: unit test that references a small set of “must exist” exported types.
  - Missing gate to add: if/when we narrow the facade, add a compatibility story + migration plan (and keep a temporary re-export allowlist).

Evidence anchors:

- `crates/fret-render/src/lib.rs`

## 6) Code quality findings (Rust best practices)

- Intentionally tiny and readable.
- Recommendation: keep this crate minimal and “boring”; move real implementation and policy decisions into backend crates.

Evidence anchors:

- `crates/fret-render/src/lib.rs`

## 7) Recommended refactor steps (small, gated)

1. Keep the facade stable and minimal — outcome: reduce churn for downstream crates — gate: `cargo nextest run -p fret-render`.
2. When multiple backends are introduced, switch to feature-gated backend selection (`default = ["wgpu"]`) and tighten the re-export surface — outcome: avoid pulling wgpu in contexts that don’t need it — gate: `pwsh -NoProfile -File tools/check_layering.ps1` + targeted `nextest` runs.

## 8) Open questions / decisions needed

- Do we want `fret-render` to remain “default backend alias” long-term, or should downstream move to explicit backend crates (with `fret-render` becoming a minimal trait/contract facade)?

