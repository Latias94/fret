---
type: "Work Progress"
title: "Query demo default surface migration"
description: "Work Progress for Query demo default surface migration."
timestamp: 2026-07-07T03:19:34Z
tags: ["ui-surface", "query", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
---

# Summary

Migrated `query_demo.rs` and `query_async_tokio_demo.rs` from raw `fret_ui` text-helper
signatures to the default app facade. The demos now use `AppRenderContext`, `impl UiChild`,
`fret::app::text::*`, `fret::style::Color`, and the new explicit `fret::time` facade instead of
teaching `ElementContext`, `UiHost`, `AnyElement`, `fret_core::time::Instant`, or direct
`fret_ui_kit` imports.

# Details

Changed files:

- `ecosystem/fret/src/lib.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Contract changes:

- Added `fret::time::{Duration, Instant, SystemTime, UNIX_EPOCH}` as an explicit app-facing
  cross-platform time module.
- Added both query demos to `DEFAULT_AUTHORING_SURFACES` and `PUBLIC_EXAMPLE_SCAN_ROOTS`.
- Added tests that keep the query demos out of `ADVANCED_MANUAL_SURFACES`.

Verification passed before commit:

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo check -p fret-examples`
- `cargo nextest run -p fret --lib --no-fail-fast`
- `cargo nextest run -p fret-examples --lib --no-fail-fast`
- `cargo fmt --all --check`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- `python3 -m py_compile tools/check_surface_policy.py tools/test_check_surface_policy.py`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`

Known warning: `cargo check -p fret-examples` and `cargo nextest run -p fret-examples --lib`
still report the pre-existing `fret-chart::visual_map_track_at` dead-code warning.

# Next Action

Continue from latest `main` with one of the remaining public-example raw-surface groups. Good next
candidates are `assets_demo.rs` (facade migration if app-facing asset wrappers cover the remaining
advanced service use) or the utility/window demos (advanced/manual classification if they remain
manual lifecycle proofs).

# Citations

- `docs/knowledge/engineering/subagents/2026-07-07-public-example-surface-followup-audit.md`
- `docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md`
