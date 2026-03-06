# `fret-launch`

Runner and launch glue for Fret demos/apps.

This crate sits “above” the UI runtime and renderer and is intended to provide a stable entrypoint
for starting apps (native and web) while keeping platform/runner details out of application code.

## Module ownership map

- `src/runner/`: the primary winit/web runner integration surfaces and helpers.
- `src/runner/common/`: shared runner contracts and support types (driver traits, contexts, config).
- `src/runner/web/`: wasm32 runner implementation (winit-web + WebGPU) split into focused submodules (`app_handler`, `gfx_init`, `render_loop`, `effects`, `streaming_images`, `ime_mount`).
- `src/error.rs`: runner error types.
- `src/stacksafe_config.rs`: stack-safety configuration (env-driven).
- `src/lib.rs`: public facade re-exports (`run_app*`, contexts, and runner config types).

## Public surface

Prefer importing from `fret_launch`’s re-exports in `src/lib.rs`. The `runner/` module is internal
plumbing and is expected to evolve as the backend seams firm up.

Compatibility note:

- Existing `fret_launch::runner::*` imports remain supported for now.
- Prefer crate-root imports for long-lived entry points and helper modules when available:
  - `fret_launch::FnDriver`
  - `fret_launch::WinitRunnerConfig`
  - `fret_launch::WinitAppBuilder`
  - `fret_launch::run_app`
  - platform helpers such as `fret_launch::windows_mf_video`,
    `fret_launch::apple_avfoundation_video`, and `fret_launch::dx12`

## Refactor gates

- Formatting: `cargo fmt`
- Build/test: `cargo nextest run -p fret-launch --no-tests pass` (until crate-local tests exist)
