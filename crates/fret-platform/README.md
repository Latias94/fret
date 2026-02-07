# `fret-platform`

`fret-platform` defines **portable platform contracts** used by the Fret runtime and runners.

This crate is intentionally backend-agnostic:

- No `winit`
- No `wgpu`
- No web bindings (`web-sys`)

Concrete implementations live in:

- `crates/fret-platform-native` (desktop: Windows/macOS/Linux)
- `crates/fret-platform-web` (wasm32: browser APIs)

## Module ownership map

- `src/clipboard.rs`: clipboard read/write contracts.
- `src/external_drop.rs`: token-based external drop payload retrieval contracts.
- `src/file_dialog.rs`: file dialog contracts (token-based; ADR 0053).
- `src/open_url.rs`: open-url contracts.

## Public surface

Prefer importing from `fret_platform`’s re-exports in `src/lib.rs` rather than reaching into
submodules directly. This keeps call sites stable if internals are regrouped during refactors.

## Refactor gates

- Formatting: `cargo fmt`
- Build/test: `cargo nextest run -p fret-platform`

