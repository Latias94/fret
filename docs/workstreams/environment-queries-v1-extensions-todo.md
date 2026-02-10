---
title: Environment Queries (v1) — Preference Extensions — TODO
status: draft
date: 2026-02-10
---

# Environment Queries (v1) — Preference Extensions — TODO

- [x] Land ADR 1185 and add a task-jump entry in `docs/adr/README.md`.

## Runtime mechanism

- [x] Add committed snapshot fields + query keys:
  - [x] `text_scale_factor`
  - [x] `prefers_reduced_transparency`
  - [x] `accent_color`
- [x] Ensure new keys participate in:
  - [x] dependency tracking
  - [x] view-cache deps fingerprint
  - [x] diagnostics export (`debug.environment`)

## Runner sources (best-effort)

- [x] Web/wasm:
  - [x] `prefers_reduced_transparency` via `matchMedia` when available.
  - [x] `text_scale_factor` via computed root `font-size` (best-effort).
- [x] Desktop:
  - [x] Wire OS-specific sources (best-effort; leave `None` when unavailable).
  - Evidence: `crates/fret-launch/src/runner/desktop/mod.rs` (`read_desktop_text_scale_factor`, `read_desktop_prefers_reduced_transparency`, `read_desktop_accent_color`).

## Ecosystem adoption

- [x] Add at least one reduced-transparency gate in shadcn-ish recipes:
  - Prefer disabling blur/frosted-glass effects when `prefers_reduced_transparency` is true.
  - Evidence: `ecosystem/fret-ui-kit/src/declarative/glass.rs` (`glass_panel`) + `ecosystem/fret-ui-kit/src/recipes/glass.rs` (`glass_effect_chain_for_environment`).
