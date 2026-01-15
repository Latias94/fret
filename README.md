# Fret

<p align="center">
  <img src="assets/fret-icon.svg" width="128" height="128" alt="Fret icon" />
</p>

Fret is the precision fretboard for your Rust UI: a GPU-first framework that turns application logic into crisp, fluid interactions.

The primary demo in this repository is an editor-style app (Unity/Unreal/Godot-inspired) used to drive requirements: docking, tear-off windows, multiple viewports, and layered GPU rendering.

This repo focuses on the **core framework** (`fret-*` crates). Reusable UI components will live in a separate
repository (`fret-components`) per `docs/adr/0037-workspace-boundaries-and-components-repository.md`.

- Quick start (desktop, in this repo): `cargo run -p fretboard -- new todo --name my-todo`
- Start here: `docs/README.md`
