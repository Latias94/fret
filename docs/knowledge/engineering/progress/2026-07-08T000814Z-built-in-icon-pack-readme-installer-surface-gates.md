---
type: "Work Progress"
title: "Built-in icon pack README installer surface gates"
description: "Work Progress for Built-in icon pack README installer surface gates."
timestamp: 2026-07-08T00:08:14Z
tags: ["icons", "public-surface", "readme"]
status: "done"
verified_by: "cargo nextest run -p fret-icons-lucide -p fret-icons-radix readme_keeps_app_installation_and_alias_policy_explicit"
---

# Summary

The built-in Lucide and Radix icon pack README gates now protect both sides of the installer
surface contract: the docs must keep teaching `app::install(...)` plus the explicit advanced
`install_with_ui_services(...)` hook, and must not reintroduce root-level `install(...)`,
`install_app(...)`, or root `install_with_ui_services(...)` examples.

# Details

- Added `assert_readme_avoids_root_install_surface(...)` to the Lucide and Radix icon pack tests.
- The gate rejects root installer spellings for `fret_icons_lucide` and `fret_icons_radix` while
  allowing the intended `fret_icons_*::app::install(...)` and
  `fret_icons_*::advanced::install_with_ui_services(...)` README guidance.
- This complements the existing public `lib.rs` checks that keep app integration under the explicit
  `app` module and advanced hooks under `advanced`.

# Next Action

Continue scanning default app-facing documentation and starter surfaces for stale root installer,
raw shadcn, or legacy conversion-trait vocabulary.

# Citations

- `ecosystem/fret-icons-lucide/src/lib.rs`
- `ecosystem/fret-icons-radix/src/lib.rs`
- `ecosystem/fret-icons-lucide/README.md`
- `ecosystem/fret-icons-radix/README.md`
- `cargo nextest run -p fret-icons-lucide -p fret-icons-radix readme_keeps_app_installation_and_alias_policy_explicit`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `cargo fmt --all --check`
