---
type: Work Progress
title: Phase 2 closeout
tags: fret,phase2,closeout,architecture,ce-work
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Phase 2 Closeout

## Summary

Phase 2 U1-U14 is closed with explicit retained bridges. The closeout document records the commit
evidence for identity, view-boundary, renderer, text, public authoring, source-policy, and `AppUi`
facade work, while avoiding an overclaim that every strong Definition of Done deletion is already
complete.

The important retained bridges are:

- parent-pointer repair still runs in normal mount/layout repair paths,
- retained-tree GC/liveness is still a required bridge,
- flat `Scene` remains the native/web launch input with diagnostic chunks,
- full-blob text helper calls remain in debug/test and parity contexts,
- non-quad partial upload is limited to resource-free `VertexColor` viewport vertices,
- advanced/manual source-policy quarantine records are still retirement records,
- `LocalState::new_in` and explicit raw bridge traits remain for manual/hybrid advanced code,
- historical observation-collapse perf keys remain for bundle compatibility.

## Verification

The final U14 code slice passed:

- `cargo check -p fret --all-targets`
- `cargo nextest run -p fret --lib --no-fail-fast`
- `cargo check -p fret-examples-imui --all-targets`
- `cargo check -p fret-cookbook --all-targets`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all`
- `cargo fmt --all --check`
- `git diff --check`

The closeout tail passed:

- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_workstream_catalog.py`
- `python3 tools/check_adr_numbers.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

## Next Action

Mark the Phase 2 goal complete after committing this closeout. Future work should open a narrow
follow-on for one retained bridge instead of reopening the broad Phase 2 plan.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [Phase 2 closeout](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-closeout.md)
- [Convergence workstream evidence](../../../workstreams/fearless-architecture-convergence-v1/EVIDENCE_AND_GATES.md)
- Subagent `019f25b1-6e8b-7c43-b448-dab143802e2c`
