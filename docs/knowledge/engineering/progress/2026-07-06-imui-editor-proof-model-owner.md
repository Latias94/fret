---
type: Work Progress
title: IMUI editor proof model owner cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/imui-editor-proof-model-owner
tags: fret,ui-framework,public-surface,imui,raw-model
---

# Summary

The non-collection IMUI editor proof still intentionally uses shared `Model<T>` handles for
editor-grade text fields, asset reference actions, and inspector readouts. This slice removes the
first low-risk raw write cluster from `asset_ref.rs` and `editor_text_assist.rs` without changing
the editor-control contracts.

`EditorProofModelOwner` now owns these shared string-model mutations:

- asset reference choose/reveal/clear action status and optional asset-slot replacement;
- text-assist accepted label recording;
- text-field edit outcome recording.

# Decisions

- Keep `AssetRefField` as a caller-owned UI shell; do not introduce asset-cache or asset-resolution
  semantics into the proof.
- Keep `record_text_field_outcome(...)` as the public helper used by object text fields, but move
  the write into `EditorProofModelOwner`.
- Do not migrate these fields to `LocalState` in this slice; the text/asset controls still expose
  model-bound contracts.

# Verification

- `cargo nextest run -p fret-examples editor_proof_model_owner_records_asset_ref_actions editor_proof_model_owner_records_text_assist_and_outcomes imui_editor_proof_demo_mounts_asset_ref_field_as_ui_shell imui_editor_proof_demo_routes_collection_proof_through_demo_local_module --no-fail-fast`

# Follow-Up

- Move `editor_material/surface.rs` and `editor_advanced/surface.rs` reset/outcome writes behind
  `EditorProofModelOwner` or narrower surface-specific owner methods.
