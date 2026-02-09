# `fret-ui-shadcn`

Shadcn/ui-inspired component set and recipes for Fret.

This crate is a **naming + taxonomy surface** intended to mirror shadcn/ui (v4) so users can
transfer knowledge and recipes directly.

## Status

Experimental learning project (not production-ready).

## When to use

- You want a productive, cohesive component surface for apps (forms, tables, overlays, layouts).
- You want shadcn-style mental models, but in a GPU-first Rust UI runtime (not HTML/CSS).

## Features

- `app-integration`: helpers that integrate with `fret-app` (optional)
- `state-selector` / `state-query`: opt into derived/async state helpers
- `state`: enables both selector + query integration

