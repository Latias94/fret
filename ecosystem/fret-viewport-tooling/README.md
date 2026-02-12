# `fret-viewport-tooling`

Viewport integration helpers and editor tooling primitives for Fret.

This crate provides policy-light, unit-explicit glue for building editor-style viewport tools
(gizmos, selection, camera navigation, debug overlays) on top of viewport input events.

## Status

Experimental learning project (not production-ready).

## When to use

- You are building a canvas/viewport panel with embedded tools.
- You want shared input mapping helpers without coupling to a specific gizmo implementation.

