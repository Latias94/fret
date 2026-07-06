---
type: Work Progress
title: IMUI collection keyboard model-owner tightening
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/imui-collection-host-model-owner
tags: fret,imui,examples,model-owner,keyboard
---

# Summary

The IMUI editor proof collection keyboard action helpers no longer write shared models directly
through `UiActionHostExt::update_model(...)`. Keyboard dispatch still owns key matching, snapshot
reads, and action notification, while shared-model mutation semantics route through
`ProofCollectionModelOwner`.

# Changes

- Added `ProofCollectionModelOwner::apply_select_all(...)` and
  `ProofCollectionModelOwner::apply_navigation(...)`.
- Routed keyboard delete, duplicate, begin-rename, select-all, and navigation action writes through
  `ProofCollectionModelOwner`.
- Updated source-surface tests so keyboard action helpers require owner calls and forbid raw
  `host.update_model(...)`.

# Remaining Work

Collection host-facing `update_model(...)` remains in the following categories:

- Inline rename outcome commit/invalid/cancel.
- Browser-scope context-menu anchor publication.
- Browser-scope zoom extent updates.
- Rename focus-pending timer consumption.
- Box-select pointer session lifecycle and threshold selection publication.

# Verification

- `cargo nextest run -p fret-examples proof_collection_model_owner_applies_command_transactions imui_editor_proof_demo_keeps_collection_keyboard_owner_app_owned_and_explicit imui_editor_proof_demo_routes_collection_proof_through_demo_local_module --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
