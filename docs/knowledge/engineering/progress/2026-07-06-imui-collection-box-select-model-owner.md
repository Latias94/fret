---
type: Work Progress
title: IMUI collection box-select model-owner tightening
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/imui-collection-host-model-owner
tags: fret,imui,examples,model-owner,box-select
---

# Summary

The IMUI editor proof collection box-select runtime no longer writes shared models directly through
`UiActionHostExt::update_model(...)`. Pointer capture/release and event routing remain in the
runtime owner, pure pointer session transitions stay in `box_select/session.rs`, and shared
selection/keyboard publication routes through `ProofCollectionModelOwner`.

# Changes

- Added the runtime-local `ProofCollectionBrowserScopeBoxSelectModelOwner` for box-select session
  model transactions.
- Routed pointer down/move/up/cancel session state through that owner.
- Routed threshold selection publication through `ProofCollectionModelOwner::apply_navigation(...)`.
- Routed non-append click-up selection clearing through `ProofCollectionModelOwner::apply_navigation(...)`.
- Updated source-surface tests to require owner calls instead of raw `host.update_model(...)`.

# Verification

- `cargo nextest run -p fret-examples imui_editor_proof_demo_routes_collection_proof_through_demo_local_module imui_editor_proof_demo_keeps_collection_command_package_app_owned_and_explicit --no-fail-fast`
- `rg -n "host\\.update_model\\(|\\.update_model\\(" apps/fret-examples/src/imui_editor_proof_demo/collection -g '*.rs'` returned no matches.
- `rg -n "models_mut\\(\\)\\s*\\.\\s*update|models_mut\\(\\)\\.update|ModelStore::update|update_any|update_f64_model" apps/fret-examples/src/imui_editor_proof_demo -g '*.rs'` returned no matches.
