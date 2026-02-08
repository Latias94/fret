# `fret-runner-web`

Web/wasm runner glue for Fret.

At the moment, this crate primarily re-exports `fret-platform-web` services on `wasm32` and keeps
native builds explicit via a stub error type.

## Module ownership map

- `src/lib.rs`: cfg-based re-exports.
- `src/native.rs`: non-wasm stub types (compile-time explicitness).

## Refactor gates

- Formatting: `cargo fmt`
- Build/test: `cargo nextest run -p fret-runner-web --no-tests pass`

