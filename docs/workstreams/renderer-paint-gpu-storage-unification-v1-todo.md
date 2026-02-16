---
title: Renderer Paint GPU Storage Unification v1 — TODO
status: active
date: 2026-02-16
---

# Renderer Paint GPU Storage Unification v1 — TODO

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `RPGPU-{area}-{nnn}`

## M0 — Docs baseline

- [x] RPGPU-docs-010 Add workstream docs (purpose/TODO/milestones) and link from `docs/workstreams/README.md`.
  - Evidence: `docs/workstreams/renderer-paint-gpu-storage-unification-v1.md`
  - Evidence: `docs/workstreams/README.md`

## M1 — Introduce a reusable storage ring

- [x] RPGPU-ring-100 Introduce a small internal utility for per-frame storage buffers + bind groups.
  - Constraints:
    - does not change bind group indices (only internal plumbing),
    - does not require shader changes,
    - keeps the existing growth policy (power-of-two + doubling bump).
  - Evidence: `crates/fret-render-wgpu/src/renderer/buffers.rs` (`StorageRingBuffer`)

## M2 — Adopt for path/text paints

- [x] RPGPU-adopt-200 Replace path paint buffer management with the shared ring utility.
  - Evidence: `crates/fret-render-wgpu/src/renderer/pipelines/path.rs` and render upload path.
- [x] RPGPU-adopt-210 Replace text paint buffer management with the shared ring utility.
  - Evidence: `crates/fret-render-wgpu/src/renderer/pipelines/text.rs` and render upload path.
  - Evidence: `crates/fret-render-wgpu/src/renderer/resources.rs` (`path_paints`, `text_paints`)
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (paint buffer uploads)

## M3 — Gates + evidence

- [x] RPGPU-gate-300 Keep gates green and leave 1–3 evidence anchors:
  - `python3 tools/check_layering.py`
  - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
  - `cargo test -p fret-render-wgpu --test text_paint_conformance`
