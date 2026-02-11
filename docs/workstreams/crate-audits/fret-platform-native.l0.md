# Crate audit (L0) — `fret-platform-native`

## Crate

- Name: `fret-platform-native`
- Path: `crates/fret-platform-native`
- Owners / adjacent crates: `fret-platform` (portable contracts), `fret-core` (tokens/options/events), runner crates that host native backends
- Current “layer”: native platform implementations (non-wasm)

## 1) Purpose (what this crate *is*)

- Native (non-wasm) implementations of the `fret-platform` contracts:
  - clipboard (`arboard`)
  - file dialogs (`rfd`)
  - open-url (`webbrowser`)
  - token-based external drop and file selection payload reading (filesystem-backed)
- This crate is intentionally *not* portable: it is expected to depend on native APIs and real paths.

Evidence anchors:

- `crates/fret-platform-native/src/lib.rs`
- `crates/fret-platform-native/Cargo.toml`

## 2) Public contract surface

- Key exports / stable types:
  - `NativeClipboard` / `DesktopClipboard`
  - `NativeExternalDrop` / `DesktopExternalDrop`
  - `NativeFileDialog` / `DesktopFileDialog`
  - `NativeOpenUrl` / `DesktopOpenUrl`
- “Accidental” exports to consider removing (L0 hypothesis):
  - The `Desktop*` type aliases are convenient; keep them stable if apps already use them, but avoid adding more alias indirection unless it helps portability.

Evidence anchors:

- `crates/fret-platform-native/src/lib.rs`

## 3) Dependency posture

- Direct deps:
  - Workspace: `fret-platform`, `fret-core`
  - External (native-only): `arboard`, `rfd`, `webbrowser`
- Layering risks:
  - Ensure this crate remains the only place those native deps appear (do not leak into `fret-platform`).

Evidence anchors:

- `crates/fret-platform-native/Cargo.toml`
- `pwsh -NoProfile -File tools/check_layering.ps1`

## 4) Module ownership map (internal seams)

- Clipboard backend wrapper
  - Files: `crates/fret-platform-native/src/clipboard.rs`
- Open-url backend wrapper
  - Files: `crates/fret-platform-native/src/open_url.rs`
- File dialog (UI) + selection token storage + payload read logic
  - Files: `crates/fret-platform-native/src/file_dialog.rs`
- External drop token storage + payload read logic
  - Files: `crates/fret-platform-native/src/external_drop.rs`

## 5) Refactor hazards (what can regress easily)

- Token-backed payload reading and resource release ordering
  - Failure mode: leaked payload maps, reads after release, incorrect limit enforcement, unbounded memory growth.
  - Existing gates: unit tests for `read_paths` limit enforcement in this crate.
  - Missing gate to add: integration-level tests that exercise the full “allocate token → set payload → read_all → release” flow.
- UI-thread blocking work
  - Failure mode: synchronous filesystem reads (`std::fs::read`) can block if performed on the UI thread.
  - Recommendation: keep reads behind runner-owned effects/dispatchers; avoid calling `read_paths` from render/layout loops.
- Error classification stability
  - Failure mode: mapping errors to `*ErrorKind::{Unavailable, Unsupported, BackendError}` drifts and breaks app behavior.
  - Existing gates: trait surface in `fret-platform` + unit tests can be added as needed if apps start depending on specific kinds.

Evidence anchors:

- `crates/fret-platform-native/src/external_drop.rs` (`read_paths`)
- `crates/fret-platform-native/src/file_dialog.rs` (`read_paths`)

## 6) Code quality findings (Rust best practices)

- The native wrappers are small and readable; `cfg(target_arch = "wasm32")` fallbacks keep the crate compiling in wasm contexts without pulling native deps.
- The payload-read logic is duplicated between external drops and file dialogs; consider extracting a shared helper if behavior needs to stay identical (but only after adding a shared set of fixtures/gates).

Evidence anchors:

- `crates/fret-platform-native/src/external_drop.rs`
- `crates/fret-platform-native/src/file_dialog.rs`

## 7) Recommended refactor steps (small, gated)

1. Add a fixture-driven suite for payload read limits (max_files/max_file_bytes/max_total_bytes) shared by both external drops and file dialogs — outcome: prevent drift — gate: `cargo nextest run -p fret-platform-native`.
2. Add an integration-level gate that covers the token lifecycle (store → read_all → release) — outcome: avoid resource leaks/regressions — gate: `cargo nextest run -p fret-platform-native`.
3. Decide and document the async policy for filesystem reads (effects/dispatcher vs direct) — outcome: prevent UI-thread stalls — gate: add a lint or documentation gate if needed.

## 8) Open questions / decisions needed

- Should `read_all` be allowed to partially succeed (some files read, some errors) as a stable contract? Today it does; confirm apps want that behavior.
- Do we want deterministic error message strings (for tests), or should we avoid asserting exact strings and keep only structured error kinds in contracts?

