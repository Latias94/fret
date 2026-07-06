---
type: Work Progress
title: IMUI collection runtime scalar model-owner tightening
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/imui-collection-host-model-owner
tags: fret,imui,examples,model-owner,browser-runtime
---

# Summary

The IMUI editor proof collection browser runtime no longer writes context-menu anchor, zoom extent,
or inline-rename pending focus state directly through `UiActionHostExt::update_model(...)`.

# Changes

- Added `ProofCollectionModelOwner::publish_context_menu_anchor(...)`.
- Added `ProofCollectionModelOwner::set_zoom_extent(...)`.
- Added `ProofCollectionModelOwner::take_inline_rename_focus_pending(...)`.
- Routed browser-scope right-click anchor publication through the model owner.
- Routed Primary+Wheel zoom extent updates through the model owner while keeping scroll offset
  updates in the runtime.
- Routed inline-rename focus timer pending-flag consumption through the model owner.

# Remaining Work

Collection host-facing `update_model(...)` remains only in the box-select runtime:

- Pointer session begin/move/up/cancel state.
- Threshold selection publication to selection/keyboard models.

# Verification

- `cargo nextest run -p fret-examples proof_collection_model_owner_applies_rename_and_tile_state imui_editor_proof_demo_routes_collection_proof_through_demo_local_module imui_editor_proof_demo_keeps_collection_command_package_app_owned_and_explicit imui_editor_proof_demo_keeps_collection_zoom_app_owned_and_explicit --no-fail-fast`
