---
type: Work Progress
title: IMUI collection inline rename model-owner tightening
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/imui-collection-host-model-owner
tags: fret,imui,examples,model-owner,inline-rename
---

# Summary

The IMUI editor proof collection inline-rename outcome actions no longer write shared models
directly through `UiActionHostExt::update_model(...)`. Outcome actions still own snapshot reads,
commit derivation, focus restoration, and redraw scheduling, while rename result writes route
through `ProofCollectionModelOwner`.

# Changes

- Added `ProofCollectionModelOwner::apply_inline_rename_commit(...)`.
- Added `ProofCollectionModelOwner::reject_inline_rename(...)`.
- Added `ProofCollectionModelOwner::cancel_inline_rename(...)`.
- Routed inline rename commit, invalid, and cancel writes through those owner transactions.
- Updated source-surface tests so inline rename actions require owner calls and forbid raw outcome
  model writes/readout formatting.

# Remaining Work

Collection host-facing `update_model(...)` remains in:

- Browser-scope context-menu anchor publication.
- Browser-scope zoom extent updates.
- Rename focus-pending timer consumption.
- Box-select pointer session lifecycle and threshold selection publication.

# Verification

- `cargo nextest run -p fret-examples proof_collection_model_owner_applies_inline_rename_outcomes imui_editor_proof_demo_keeps_collection_inline_rename_app_owned_and_explicit imui_editor_proof_demo_routes_collection_proof_through_demo_local_module imui_editor_proof_demo_keeps_collection_command_package_app_owned_and_explicit --no-fail-fast`
