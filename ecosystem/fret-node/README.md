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

## Upstream references (non-normative)

Node-graph editors have a lot of established interaction vocabulary. These projects are useful for
design intent and parity targets:

- XyFlow (React Flow): https://github.com/xyflow/xyflow
- egui-snarl (Rust node graph editor): https://github.com/zakarumych/egui-snarl
