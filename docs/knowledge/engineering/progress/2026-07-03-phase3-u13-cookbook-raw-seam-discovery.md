---
type: Work Progress
title: Phase 3 U13 cookbook raw seam discovery gate
tags: fret,phase3,u13,cookbook,source-policy,quarantine
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 fourth slice turns cookbook advanced/raw seam discovery into a source-policy gate.

`tools/check_surface_policy.py` now scans `apps/fret-cookbook/examples` for high-risk public
example seams that are not covered by default-clean or advanced/manual surface records. The first
high-risk set is intentionally narrower than `RAW_SEAM_PATTERNS` and targets seams that make an
example public-looking but runtime/manual in practice:

- `fret::advanced`, `advanced::prelude::*`, and `advanced::raw`;
- `KernelApp` and `AppWindowId`;
- `FnDriver` and direct `fret_launch::` imports;
- raw `Model<...>`, `ModelStore`, and `LocalState::new_in`;
- direct `UiTree`.

The slice also adds exact cookbook quarantine records for the currently advanced cookbook examples.
Each record has an owner, reason, retirement condition, and exact `allowed_raw_seams`. The existing
`advanced-surface-unused-allowed-raw-seam` and `advanced-surface-unlisted-raw-seam` checks now keep
those records shrinking as examples migrate.

# Verification

Passed on 2026-07-03:

- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `cargo check -p fret-cookbook --all-targets`
- `cargo fmt --all --check`
- `git diff --check`

# Remaining U13 Work

The discovery gate currently covers cookbook examples first. `apps/fret-examples/src` still has a
large legacy/manual demo surface and should be handled in a separate U13 slice with either precise
per-file/per-directory classifications or a generated inventory that fails on new unclassified
high-risk seams.

Likely next steps:

- add discovery coverage for `apps/fret-examples/src` without adding a broad root quarantine;
- split the broad `apps/fret-examples-imui/src` quarantine into per-file records after raw seam
  categories are available;
- replace no-new-API raw helper cases before designing new helper APIs.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [U13 low-risk cookbook app surface migration](2026-07-03-phase3-u13-low-risk-cookbook-app-surface.md)
- [U13 advanced facade audits](../subagents/2026-07-03-phase3-u13-advanced-facade-audits.md)
