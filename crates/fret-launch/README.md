# `fret-launch`

Runner and launch glue for Fret demos/apps.

This crate sits “above” the UI runtime and renderer and is intended to provide a stable entrypoint
for starting apps (native and web) while keeping platform/runner details out of application code.

## Module ownership map

- `src/runner/`: the primary winit/web runner integration surfaces and helpers.
- `src/error.rs`: runner error types.
- `src/stacksafe_config.rs`: stack-safety configuration (env-driven).
- `src/lib.rs`: public facade re-exports (`run_app*`, contexts, and runner config types).

## Public surface

Prefer importing from `fret_launch`’s re-exports in `src/lib.rs`. The `runner/` module is internal
plumbing and is expected to evolve as the backend seams firm up.

## Refactor gates

- Formatting: `cargo fmt`
- Build/test: `cargo nextest run -p fret-launch --no-tests pass` (until crate-local tests exist)

