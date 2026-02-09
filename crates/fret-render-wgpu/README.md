# `fret-render-wgpu`

WGPU backend implementation for Fret rendering.

This crate provides the `wgpu`-based renderer implementation used by the Fret workspace. It is
responsible for turning `fret-core::scene::Scene` recordings into GPU draw calls, including text
and SVG integration used by higher layers.

## Status

Experimental learning project (not production-ready).

## When to use

- If you are building a runner/backend integration and need direct access to the wgpu renderer.

Most applications should depend on the higher-level facade crate `fret-render` (or `fret-kit`)
instead of depending on this crate directly.

