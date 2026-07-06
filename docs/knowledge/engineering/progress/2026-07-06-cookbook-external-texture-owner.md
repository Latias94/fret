---
type: Work Progress
title: Cookbook external texture owner helper
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/cookbook-external-texture-owner
tags: fret,ui-framework,public-surface,cookbook,external-texture,raw-model,owner
---

# Summary

`apps/fret-cookbook/examples/external_texture_import_basics.rs` now routes its engine-frame target
metric writes through a demo-local `ExternalTextureImportBasicsModelOwner`.

The example remains an advanced/manual interop cookbook surface because it owns imported render
target updates, a manual `UiTree`, and low-level viewport presentation. The cleanup only removes
the copyable pattern of writing `target_w`, `target_h`, and `ingest` directly from
`record_engine_frame(...)`.

# Decision

Keep model allocation and low-level render-target ownership inside the advanced cookbook example.
Do not migrate this example to `LocalState<T>` or a default app recipe until external texture import
has a real app-facing contract. The local owner helper is a quarantine cleanup, not a public API.

# Evidence

- Red proof before implementation:
  `cargo nextest run -p fret-cookbook external_texture_import_basics_model_writes_stay_behind_owner_helper --no-fail-fast`
  failed because `ExternalTextureImportBasicsModelOwner` did not exist.
- The same test now requires `ExternalTextureImportBasicsModelOwner::set_target_metrics(...)` and
  forbids the old direct `record_engine_frame(...)` update calls.
- `cargo check -p fret-cookbook --all-targets` passes.

# Next

Apply this pattern to cookbook advanced/manual examples only when a raw write is owner-shaped and
the example should remain classified as interop or renderer mechanism documentation. For default
authoring cookbook examples, prefer app-facing facade lanes instead.
