# `fret-platform-web`

Browser/wasm32 platform services used by the Fret runtime.

This crate is expected to be used from `crates/fret-runner-web` (or other wasm runners) to satisfy
`fret-runtime::Effect`s that require browser APIs (timers, file inputs, IME bridges, etc.).

It intentionally does **not** implement input/event mapping; use `winit` (or another runner) for
that layer.

## Module ownership map

- `src/wasm/mod.rs`: wasm32 implementation of `WebPlatformServices` and related helpers.
- `src/native/mod.rs`: non-wasm stub types to keep cross-target builds explicit.
- `src/lib.rs`: cfg-based module selection + public re-exports.

## Public surface

On `wasm32`, `fret_platform_web` re-exports the wasm implementation. On non-wasm targets, it
re-exports stub types that report “wasm32 only”.

## Refactor gates

- Formatting: `cargo fmt`
- Build/test: `cargo nextest run -p fret-platform-web`
