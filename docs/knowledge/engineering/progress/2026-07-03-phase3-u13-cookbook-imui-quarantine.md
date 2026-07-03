---
type: Work Progress
title: Phase 3 U13 cookbook app-surface and IMUI quarantine tightening
tags: fret,phase3,u13,app-facade,source-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 second slice moves three low-risk cookbook examples out of
`advanced::prelude::*` and tightens source-policy quarantine accounting.

The migrated cookbook examples are:

- `apps/fret-cookbook/examples/app_owned_bundle_assets_basics.rs`
- `apps/fret-cookbook/examples/drop_shadow_basics.rs`
- `apps/fret-cookbook/examples/icons_and_assets_basics.rs`

They now use `fret::app::prelude::*`, `App`, `WindowId`, and
`IntoUiElement<App>` while keeping only their necessary component/asset/render helper imports.
`apps/fret-cookbook/src/lib.rs` now asserts these examples stay on the app surface and do not
regrow `advanced::prelude`, `advanced::raw`, or `use fret::advanced`.

# Source Policy

The `advanced::prelude::*` raw-trait split exposed implicit raw local-state trait use in
`apps/fret-examples-imui`. Those examples now import the required traits explicitly from
`fret::advanced::raw`.

`tools/check_surface_policy.py` now registers `apps/fret-examples-imui/src` as an
`advanced_manual` surface with owner `examples-imui`, a retirement note, and the currently used
raw seams:

- `fret::advanced`
- `fret_core`
- `fret_ui`
- `AnyElement`
- `ElementContext`

The gate also reports `advanced-surface-unused-allowed-raw-seam` when an advanced/manual quarantine
record lists a seam that no longer appears in the source. This keeps allowed seam records shrinking
as wrappers land instead of becoming permanent broad allowlists.

# Verification

Passed on 2026-07-03:

- `cargo check -p fret-examples-imui --all-targets`
- `cargo check -p fret-cookbook --all-targets`
- `cargo nextest run -p fret-cookbook --lib --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `cargo fmt --all --check`
- `git diff --check`
- Static search over the three migrated cookbook examples for `KernelApp`, `AppWindowId`,
  `advanced::prelude`, `advanced::raw`, `use fret::advanced`, and `IntoUiElement<KernelApp>`
  returned no matches.

# Remaining U13 Work

U13 is not fully closed. Remaining work should continue classifying and shrinking public-looking
advanced surfaces:

- Audit remaining cookbook `advanced::prelude::*` examples and migrate any that only need
  app/default, explicit `advanced::driver`, explicit `advanced::view`, or explicit
  `advanced::interop` lanes.
- Prefer deleting raw model/action usage when an app-facing helper exists; otherwise keep
  `advanced::raw` imports explicit and covered by source policy.
- Continue shrinking `ADVANCED_MANUAL_SURFACES` records as wrappers or generated starters replace
  raw seams.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [U13 advanced raw and driver split](2026-07-03-phase3-u13-advanced-raw-driver-split.md)
