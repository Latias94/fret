# `fret-dnd`

Headless drag-and-drop primitives for Fret component ecosystems.

This crate is intentionally UI-agnostic: it depends only on `fret-core` geometry and IDs, and
provides reusable policy primitives (activation constraints, collision detection, modifiers, and
auto-scroll request computation).

## Status

Experimental learning project (not production-ready).

## What it provides

- Pointer sensors + activation constraints
- Collision strategies (closest-center, pointer-within, etc.)
- Sortable insertion helpers
- Auto-scroll request computation

