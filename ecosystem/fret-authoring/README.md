# `fret-authoring`

Shared authoring contracts for ecosystem-level UI frontends.

This crate defines small, policy-light traits that allow ecosystem crates to expose authoring
helpers without coupling to a specific frontend (e.g. an immediate-mode authoring layer).

## Status

Experimental learning project (not production-ready).

## Key types

- `UiWriter`: minimal surface for immediate-style composition that still mounts declarative
  elements into `UiTree`.
- `Response`: compact interaction result (hovered/pressed/focused/clicked/changed + optional rect).

## Features

- `query`: optional integration helpers for `fret-query`
- `selector`: optional integration helpers for `fret-selector`

