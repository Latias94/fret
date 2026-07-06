---
type: Work Progress
title: IMUI collection model owner cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/imui-collection-model-owner
tags: fret,ui-framework,public-surface,imui,raw-model
---

# Summary

`imui_editor_proof_demo/collection` is an editor-grade proof lane with an intentional shared
`Model<T>` graph. The collection browser spans command buttons, context menus, keyboard commands,
inline rename, focus restore, and browser-scope pointer runtime, so this is not a mechanical
`LocalState` migration.

This slice removes app-side raw `models_mut().update(...)` writes from the collection command,
context-menu, asset-grid, and rename-start paths by routing them through the demo-local
`ProofCollectionModelOwner`.

# Decisions

- Keep selection derivation, delete/duplicate result computation, menu/button chrome, and focus
  timer policy in their existing owner modules.
- Let `ProofCollectionModelOwner` own only already-derived shared-model mutations:
  duplicate/delete command transactions, inline rename start, active focus target publish, active
  asset activation, context-menu request state, and context-menu anchor clearing.
- Delete `proof_collection_begin_inline_rename_in_app(...)`; it was a legacy free helper that made
  the old raw-write boundary look reusable.
- Do not touch `UiActionHostExt::update_model(...)` paths in keyboard, inline-rename outcome, and
  browser input runtime yet. They should be handled as follow-up host-action owner slices.

# Verification

- `cargo nextest run -p fret-examples imui_editor_proof_demo_routes_collection_proof_through_demo_local_module imui_editor_proof_demo_keeps_collection_command_package_app_owned_and_explicit imui_editor_proof_demo_keeps_collection_context_menu_app_owned_and_explicit imui_editor_proof_demo_keeps_collection_inline_rename_app_owned_and_explicit proof_collection_model_owner_applies_command_transactions proof_collection_model_owner_applies_rename_and_tile_state --no-fail-fast`

# Follow-Up

- Route collection `UiActionHostExt::update_model(...)` paths through the same owner vocabulary or
  narrower host-action owners, especially `collection/keyboard/actions.rs` and
  `collection/asset_grid/inline_rename/actions.rs`.
- Continue the non-collection IMUI proof cleanup with `asset_ref.rs`, `editor_text_assist.rs`,
  `editor_material/surface.rs`, and `editor_advanced/surface.rs`.
