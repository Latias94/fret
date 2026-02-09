# `fret`

Fret's kernel facade crate: a small, memorable entry point for **manual/advanced assembly**.

This crate re-exports selected workspace crates behind opt-in feature flags, without pulling in
ecosystem defaults (components, policy, tooling).

If you want the batteries-included experience (desktop-first templates, defaults, tooling), prefer:

- `fret-kit` (app entry points)
- `fretboard` (dev tooling)

## Status

Experimental learning project (not production-ready).

## Cargo features

Default features are intentionally minimal (`core` only).

Convenience bundles:

- `desktop` / `native-wgpu`: native desktop stack (winit + wgpu)
- `wasm` / `web`: wasm32 browser stack (WebGPU) via the web runner

Lower-level feature flags:

- `core`, `app`, `ui`, `runtime`, `render`, `fonts`
- `platform-contracts`, `platform-native`, `platform-web`
- `runner-winit`, `runner-web`, `launch`

## Quick start

Use a bundle feature:

```toml
[dependencies]
fret = { version = "0.1", features = ["desktop"] }
```

Or keep it explicit:

```toml
[dependencies]
fret = { version = "0.1", default-features = false, features = ["core", "ui", "runtime"] }
```

