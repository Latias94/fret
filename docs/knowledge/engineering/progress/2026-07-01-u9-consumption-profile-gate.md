---
type: Work Progress
title: U9 consumption profile gate hardening
tags: fret,u9,consumption-profiles,facade,pre-release
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

The U9 consumption profile checker now covers the batteries-included `fret` app facade explicitly.
`tools/check_consumption_profiles.py` adds `cargo check -p fret --locked` and
`cargo check -p fret --locked --features batteries`, removes the dependency tree depth cap from
backend-free tree scans, and expands the banned backend/render package set to include web platform
and runner crates plus the concrete wgpu renderer crate.

`tools/pre_release.py` now includes the consumption profile checker near the other architecture
policy gates, and `tools/test_check_consumption_profiles.py` locks the banned dependency-tree helper
with positive/native-backend/web-backend fixture cases. `docs/repo-structure.md` now maps
`fret-framework` to the core/manual assembly facade role and `ecosystem/fret` to the app facade role.

# Verification

- `python3 -m unittest tools/test_check_consumption_profiles.py`
- `python3 -m py_compile tools/check_consumption_profiles.py tools/test_check_consumption_profiles.py tools/pre_release.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `git diff --check`
- `rg -n "Consumption profiles|check_consumption_profiles" tools/pre_release.py tools/check_consumption_profiles.py`

# Known External Blocker

`python3 tools/pre_release.py --skip-fmt --skip-clippy --skip-nextest --skip-icons --skip-release-closure --skip-portable-time --skip-diff-check`
currently stops before the new consumption-profile step because the existing ADR ID uniqueness gate
finds duplicate ID `0324` in `docs/adr/0324-a11y-state-description-semantics-v1.md` and
`docs/adr/0324-window-input-hit-testing-and-passthrough-v1.md`. This slice does not renumber ADRs.

# Next Action

Continue U9 facade modularization by splitting the view runtime cluster (`ViewWindowState`,
`AppUiRenderRootState`, `view_init_window`, `view_view`, `render_root_with_app_ui`, and
`view_record_engine_frame`) into a private `view/runtime.rs` module while preserving the same public
root/prelude aliases and authoring-surface tests.
