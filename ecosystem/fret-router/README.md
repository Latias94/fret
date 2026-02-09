# `fret-router`

Typed message routing and router composition utilities for Fret applications.

This crate is intentionally lightweight in v1: it provides portable parsing/helpers and leaves
policy-heavy routing behavior in app/ecosystem layers.

## Status

Experimental learning project (not production-ready).

## Features

- `web-history`: wasm32 adapter for browser history-based navigation
- `hash-routing`: wasm32 adapter for hash-based navigation
- `query-integration`: integrates with `fret-query` for route-change invalidation/prefetch planning

