---
type: Work Progress
title: IMUI editor surface model owner cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/imui-editor-surface-model-owner
tags: fret,ui-framework,public-surface,imui,raw-model
---

# Summary

This slice completes the `host.models_mut().update(...)` cleanup for
`imui_editor_proof_demo/*`. Material and Advanced inspector surfaces now route numeric reset and
edit-outcome writes through `EditorProofModelOwner`.

# Decisions

- Delete the local `update_f64_model(...)` helper from `editor_advanced/surface.rs`; it was only a
  thin raw-write wrapper.
- Keep Material/Advanced surface files responsible for UI row composition, reset callback wiring,
  and visibility filtering.
- Let `EditorProofModelOwner` own numeric scalar writes and edit-outcome string writes shared by
  object, material, and advanced editor proof surfaces.

# Verification

- `cargo nextest run -p fret-examples editor_proof_model_owner_records_numeric_resets_and_drag_outcomes imui_editor_proof_demo_routes_collection_proof_through_demo_local_module --no-fail-fast`
- `rg -n "models_mut\\(\\)\\s*\\.\\s*update|models_mut\\(\\)\\.update|ModelStore::update|update_any|update_f64_model" apps/fret-examples/src/imui_editor_proof_demo -g '*.rs'`

# Follow-Up

- Collection still has intentional `UiActionHostExt::update_model(...)` host-facing action paths in
  keyboard, inline-rename outcome, and browser input runtime. Those should be evaluated as a
  separate host-action owner slice, not as raw `models_mut().update(...)` leakage.
