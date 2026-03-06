# `fret-launch`

Runner and launch glue for Fret apps and advanced integrations.

This crate sits “above” the UI runtime and renderer and provides the stable runner-facing entry
points for starting apps (native and web) while keeping platform/runner details out of higher-level
application facades.

Related workstream: `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/`

## Choosing the right layer

- App authors: start with `fret`
- Manual assembly with curated re-exports and without ecosystem defaults: start with `fret-framework`, then add `fret-bootstrap` as needed
- Advanced host/runner integration: use `fret-launch`

## Choosing a driver surface

- Recommended advanced surface: `fret_launch::FnDriver` and `fret_launch::FnDriverHooks`
- Compatibility-only surface: `fret_launch::WinitAppDriver`
- Native entrypoint wiring: `fret_launch::run_app`, `fret_launch::run_app_with_event_loop`, `fret_launch::WinitAppBuilder`

Migration posture:

- Existing `WinitAppDriver` integrations remain supported.
- New docs/examples should prefer `FnDriver`.
- If you still want bootstrap defaults, pair `fret-launch::FnDriver` with `fret_bootstrap::BootstrapBuilder::new_fn(...)` or `fret::run_native_with_fn_driver(...)`.

## Module ownership map

- `src/runner/`: the primary winit/web runner integration surfaces and helpers.
- `src/runner/common/`: shared runner contracts and support types (driver traits, contexts, config).
- `src/runner/web/`: wasm32 runner implementation (winit-web + WebGPU) split into focused submodules (`app_handler`, `gfx_init`, `render_loop`, `effects`, `streaming_images`, `ime_mount`).
- `src/error.rs`: runner error types.
- `src/stacksafe_config.rs`: stack-safety configuration (env-driven).
- `src/lib.rs`: public facade re-exports (`run_app*`, contexts, config types, driver surfaces).

## Public surface

Prefer importing long-lived core entry points from `fret_launch` crate-root re-exports in
`src/lib.rs`. The `runner/` module is internal plumbing and is expected to evolve as backend seams
firm up.

Prefer crate-root imports for long-lived core entry points:

- `fret_launch::FnDriver`
- `fret_launch::FnDriverHooks`
- `fret_launch::WinitRunnerConfig`
- `fret_launch::WinitAppBuilder`
- `fret_launch::run_app`

Prefer dedicated specialized modules for advanced interop/media helpers:

- `fret_launch::imported_viewport_target::ImportedViewportRenderTarget`
- `fret_launch::native_external_import::NativeExternalTextureFrame`
- `fret_launch::media::windows_mf_video`
- `fret_launch::media::apple_avfoundation_video`
- `fret_launch::shared_allocation::dx12`

Treat these as compatibility-oriented unless a specific hook gap forces them:

- `fret_launch::WinitAppDriver`

## Refactor gates

- Formatting: `cargo fmt -p fret-launch`
- Build/test: `cargo nextest run -p fret-launch`
- Layering: `python tools/check_layering.py`
