---
type: "Work Progress"
title: "Generated icon pack installer surface gate"
description: "Work Progress for Generated icon pack installer surface gate."
timestamp: 2026-07-07T23:52:38Z
tags: ["fret", "icons", "generator", "public-surface", "advanced", "app-install"]
---

# Summary

Added regression assertions to the generated icon-pack surface test so future generated packs keep
the explicit `app` / `advanced` installer split.

# Details

- Changed `crates/fret-icons-generator/src/lib.rs`.
- The existing `svg_directory_generation_emits_complete_pack_surface` test now verifies generated
  `src/lib.rs` exposes feature-gated `app` and `advanced` modules without root-level
  `install(...)`, `install_app(...)`, or `install_with_ui_services(...)` functions.
- The same test verifies generated README guidance teaches
  `<pack>::app::install(...)` plus `<pack>::advanced::install_with_ui_services(...)`, while keeping
  root installer names out of the generated docs.
- Generated `app.rs` and `advanced.rs` are now checked as separate seams: `app.rs` owns
  `install(...)`, and `advanced.rs` owns `install_with_ui_services(...)` as a thin bridge to
  `crate::app::install(app)`.
- Focused verification passed:
  `cargo nextest run -p fret-icons-generator svg_directory_generation_emits_complete_pack_surface`.

# Next Action

Run the standard format, surface, boundary, largest-file, wiki, and whitespace gates, then commit
and push `main` if clean.

# Citations

- `crates/fret-icons-generator/src/lib.rs`
- `docs/workstreams/ecosystem-integration-traits-v1/TODO.md`
- `docs/workstreams/ecosystem-integration-traits-v1/MIGRATION_MATRIX.md`
