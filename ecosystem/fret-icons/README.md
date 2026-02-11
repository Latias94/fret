# `fret-icons`

Shared icon surface and icon primitives for Fret UI ecosystems.

This crate is renderer-agnostic:

- Components depend on semantic icon IDs (`IconId`).
- Icon packs register assets as data (`IconSource`).
- Rendering and caching remain in the renderer layer.

## Status

Experimental learning project (not production-ready).

## Key types

- `IconId`: stable semantic IDs (e.g. `"ui.search"`)
- `IconRegistry`: registers SVG sources and aliases, resolves with loop/missing diagnostics

