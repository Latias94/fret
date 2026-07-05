---
type: Work Progress
title: API Workbench Lite model owner cleanup
timestamp: 2026-07-06T00:00:00Z
git_branch: feat/ui-framework-public-surface-api-workbench-lite-state
tags: fret,ui-framework,public-surface,api-workbench,raw-model,local-state
---

# Summary

`api_workbench_lite_demo` now keeps its required shared-model access behind a local
`ApiWorkbenchModelOwner`. The view still teaches app-facing `LocalState<T>` fields and explicit
shadcn facade imports, while the mutation/query runtime bridge is isolated to one owner boundary.

# Decisions

- Do not add a new framework-level multi-mutation action helper in this slice.
- Keep the existing `cx.actions().models(...)` / `payload_models(...)` mechanisms for the few
  handlers that coordinate mutation/query models, but route their bodies through
  `ApiWorkbenchModelOwner`.
- Keep `fret_runtime::ModelStore` named once via the local owner alias so future cleanup can find
  the remaining raw boundary without scattering it through action functions.
- Keep the owner operation-specific. It should not expose a generic `LocalStateTxn` pass-through
  that lets future handlers bypass the boundary without naming the operation they are performing.

# Verification

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test api_workbench_lite_demo_surface --no-fail-fast`
- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test app_import_surface examples_src_keeps_local_state_raw_bridges_out app_state_demos_use_app_local_state_imports --no-fail-fast`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Follow-Up

- Revisit a framework-level multi-mutation submit helper only if another first-contact app needs
  the same "one LocalState-built input fans out to multiple mutation handles" pattern.
- Until then, prefer demo-local owner boundaries over widening `fret::app` for one-off mutation
  orchestration.
