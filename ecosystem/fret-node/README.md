# `fret-node`

Node-graph foundation and integration surfaces for Fret editor workflows.

This crate provides headless graph model/schema building blocks plus optional UI integration
surfaces for node-graph editors.

## Status

Experimental learning project (not production-ready).

## Features

- `ui` / `fret-ui`: enable `crates/fret-ui` integration helpers (canvas widget, styling surfaces)
- `imui`: optional immediate-mode authoring adapters (builds on `fret-authoring`)
- `canvas-rstar`: opt into an R-tree spatial index backend for large graphs
- `app-integration`: optional `fret-app` helpers (commands/default bindings)
- `headless`: build headless-only graph model surfaces

