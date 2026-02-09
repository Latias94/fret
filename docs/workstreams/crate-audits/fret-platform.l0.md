# Crate audit (L0) — `fret-platform`

## Crate

- Name: `fret-platform`
- Path: `crates/fret-platform`
- Owners / adjacent crates: `fret-core` (tokens/events/options), `fret-platform-native`, `fret-platform-web`, runner crates that host backends
- Current “layer”: portable platform contracts

## 1) Purpose (what this crate *is*)

- A backend-agnostic set of platform contracts (clipboard, file dialogs, open-url, external drops).
- This crate intentionally avoids platform- or backend-specific dependencies (no `winit`, no `wgpu`, no OS bindings).
- Backend implementations are expected to live in dedicated crates (native/web).

Evidence anchors:

- `crates/fret-platform/src/lib.rs`
- `crates/fret-platform/Cargo.toml`

## 2) Public contract surface

- Key exports / stable types:
  - clipboard: `Clipboard`, `ClipboardError`, `ClipboardErrorKind`
  - external drop: `ExternalDropProvider`, `ExternalDropReadLimits`
  - file dialog: `FileDialogProvider`, `FileDialogSelection` (via `fret-core`), `FileDialogReadLimits`, `FileDialogError*`
  - open-url: `OpenUrl`, `OpenUrlError*`
- “Accidental” exports to consider removing:
  - none observed (explicit re-export list).

Evidence anchors:

- `crates/fret-platform/src/lib.rs`
- `crates/fret-platform/src/clipboard.rs`
- `crates/fret-platform/src/external_drop.rs`
- `crates/fret-platform/src/file_dialog.rs`
- `crates/fret-platform/src/open_url.rs`

## 3) Dependency posture

- Direct deps: `fret-core` only.
- Layering policy: expected (portable contract crate).
- Hotspots / heavy deps: none.

Evidence anchors:

- `crates/fret-platform/Cargo.toml`
- `pwsh -NoProfile -File tools/audit_crate.ps1 -Crate fret-platform`

## 4) Module ownership map (internal seams)

- Clipboard contract
  - Files: `crates/fret-platform/src/clipboard.rs`
- External drop contract (token-based payload retrieval)
  - Files: `crates/fret-platform/src/external_drop.rs`
- File dialog contract (token-based read model)
  - Files: `crates/fret-platform/src/file_dialog.rs`
- Open-url contract
  - Files: `crates/fret-platform/src/open_url.rs`

## 5) Refactor hazards (what can regress easily)

- Token/lifetime semantics between `fret-core` tokens and backend provider implementations
  - Failure mode: backend leaks resources by not honoring `release(token)`; runtime reads after release.
  - Existing gates: compile-time surface gate in `fret-platform` plus backend-specific tests in implementation crates.
  - Missing gate to add: per-backend integration tests that exercise `read_all` and `release` ordering under cancellation/timeouts.
- Contract drift by accidental dependency additions
  - Failure mode: backend deps leak into the portable crate and break WASM portability or layering.
  - Existing gates: layering check.
  - Missing gate to add: consider enforcing “no external deps” (besides `fret-core`) as a policy if this crate grows.

Evidence anchors:

- `crates/fret-platform/src/lib.rs`
- `pwsh -NoProfile -File tools/check_layering.ps1`

## 6) Code quality findings (Rust best practices)

- Simple, idiomatic traits + small error enums; good portability posture.
- Recommendation: keep contracts minimal and avoid adding policy (timeouts, threading, retries) here; policy belongs in app/runner layers.

Evidence anchors:

- `crates/fret-platform/src/*`

## 7) Recommended refactor steps (small, gated)

1. Keep re-export surface stable and explicit — outcome: avoid downstream churn — gate: `cargo nextest run -p fret-platform`.
2. Add backend integration gates in `fret-platform-native` / `fret-platform-web` once those contracts start being relied on by apps — outcome: catch resource-release ordering regressions — gate: backend-specific `nextest` runs + diag as needed.

## 8) Open questions / decisions needed

- Should any of these contracts be `async` in the portable layer, or should the portable layer remain synchronous and rely on runner-owned async/effects for long work (recommended by current architecture)?

