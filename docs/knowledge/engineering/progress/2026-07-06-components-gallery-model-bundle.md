---
type: "Work Progress"
title: "Components gallery model bundle"
description: "Work Progress for components gallery model bundle cleanup."
timestamp: 2026-07-06T18:35:00Z
tags: ["fret", "examples", "components-gallery", "public-surface", "raw-model"]
git_branch: "refactor/components-gallery-model-owner"
verified_by: "cargo nextest run -p fret-examples --test components_gallery_surface --no-fail-fast"
---

# Summary

`components_gallery.rs` now keeps startup model allocation behind a private
`ComponentsGalleryModelBundle`.

# Details

- Added `ComponentsGalleryModelBundle::new(...)` to own the gallery's initial `ModelStore::insert`
  calls.
- Changed `ComponentsGalleryDriver::build_ui(...)` so it constructs a named bundle instead of
  scattering `app.models_mut().insert(...)` across window setup.
- Strengthened `components_gallery_surface` so production source must keep update/set writes behind
  `ComponentsGalleryModelOwner` and startup inserts behind `ComponentsGalleryModelBundle`.

# Verification

- `cargo fmt --all --check`
- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test components_gallery_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

Continue with workspace-shell or plot/chart binding cleanup rather than broadening the gallery
refactor.

# Citations

- [components_gallery.rs](../../../../apps/fret-examples/src/components_gallery.rs)
- [components_gallery_surface.rs](../../../../apps/fret-examples/tests/components_gallery_surface.rs)
