---
type: "Work Progress"
title: "Shadcn app installer root surface gate"
description: "Work Progress for Shadcn app installer root surface gate."
timestamp: 2026-07-08T00:20:21Z
tags: ["shadcn", "public-surface", "app-integration"]
status: "done"
verified_by: "cargo nextest run -p fret-ui-shadcn --lib app_integration_stays_under_explicit_app_module"
---

# Summary

The `fret-ui-shadcn` app-integration policy test now rejects root-level installer teaching and
exports. The intended setup lane stays `fret_ui_shadcn::app::*`, while root `install(...)`,
`install_with(...)`, `install_with_theme(...)`, `install_app(...)`, and root
`install_with_ui_services(...)` spellings remain closed.

# Details

- Added `assert_readme_avoids_root_install_surface(...)` to `surface_policy_tests.rs`.
- Strengthened `app_integration_stays_under_explicit_app_module` to reject root `pub fn install*`
  exports and `pub use app::...` from `lib.rs`.
- The first attempted nextest command omitted `--lib` and started compiling broad integration test
  binaries; it was cancelled and replaced with the targeted lib-only gate listed below.

# Next Action

Continue the public-surface follow-up lane by checking remaining app-facing docs and examples for
stale raw, advanced, or root setup vocabulary that is not already covered by a focused policy gate.

# Citations

- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`
- `ecosystem/fret-ui-shadcn/README.md`
- `ecosystem/fret-ui-shadcn/src/lib.rs`
- `ecosystem/fret-ui-shadcn/src/app.rs`
- `cargo nextest run -p fret-ui-shadcn --lib app_integration_stays_under_explicit_app_module`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `cargo fmt --all --check`
