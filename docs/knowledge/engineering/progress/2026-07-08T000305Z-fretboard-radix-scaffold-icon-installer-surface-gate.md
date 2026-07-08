---
type: "Work Progress"
title: "Fretboard Radix scaffold icon installer surface gate"
description: "Work Progress for Fretboard Radix scaffold icon installer surface gate."
timestamp: 2026-07-08T00:03:05Z
tags: ["fretboard", "icons", "public-surface", "scaffold"]
status: "done"
verified_by: "cargo nextest run -p fretboard radix_icon_pack_templates_use_explicit_app_install_surface"
---

# Summary

The `fretboard new` scaffold template tests now gate the Radix icon pack installer surface across
all templates that emit an `install_app` hook. The templates were already generating
`fret_icons_radix::app::install(app);`; this slice broadens the regression test beyond the todo
template so future starter surfaces do not drift back to root-level or advanced installer spelling.

# Details

- Added `assert_radix_icon_app_install_surface(...)` in `crates/fretboard/src/scaffold/templates.rs`.
- Covered `todo`, `simple_todo`, `hello`, `workbench_lite`, and `mutation_workbench`.
- The gate requires `fret_icons_radix::app::install(app);`.
- The gate rejects `fret_icons_radix::install(app);`,
  `fret_icons_radix::install_app(app);`, `fret_icons_radix::advanced::install_with_ui_services`,
  `fret_icons_lucide::`, and Lucide action-icon helper imports in Radix templates.

# Next Action

Continue the public-surface follow-up lane by scanning for remaining default-lane examples or docs
that teach raw/advanced icon, shadcn, or conversion surfaces without an explicit escape-hatch reason.

# Citations

- `crates/fretboard/src/scaffold/templates.rs`
- `cargo nextest run -p fretboard radix_icon_pack_templates_use_explicit_app_install_surface`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `cargo fmt --all --check`
