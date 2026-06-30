---
type: Work Progress
title: Engineering Memory Log
tags: fret,engineering-memory
timestamp: 2026-06-30
---

# Engineering Memory Log

## 2026-06-30

- Created the first engineering memory bundle for the Fret architecture convergence goal.
- Recorded that the repository had no dirty changes to commit before planning began.
- Synthesized four audit lanes into `docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md`: repository boundaries, GPUI/Zed comparison, framework consumer/DX, and performance architecture.
- Key conclusion: Fret should continue toward `ViewId`, `ViewBoundary`, `notify`, prepaint frame products, stable handles, dirty graph, scene chunks, and source-policy gates rather than inventing a second runtime.
- Started execution on branch `feat/ui-framework-convergence`.
- Completed U1 contract freeze in commit `020bb34a37 docs(architecture): freeze ui convergence contract`; verified JSON, workstream catalog, layering, and whitespace checks.
- U2 source-policy checker exploration completed in subagent `019f15f9-d73a-7983-adf1-22de5ccc58b8`: add a heuristic `tools/check_surface_policy.py` with focused tests rather than broad repo-wide `fret_ui` / `fret_core` denies.
- Completed U2 in commit `84f60d8355 feat(tools): add ui surface policy gate`; added `tools/check_surface_policy.py`, focused unit tests, pre-release wiring, dependency-policy docs, and removed a raw `fret_ui::AnyElementIterExt` first-hour snippet.
- U2 verification passed: `python3 -m unittest tools/test_check_surface_policy.py`, `python3 tools/check_surface_policy.py`, `python3 tools/check_layering.py`, `python3 tools/check_consumption_profiles.py`, `python3 -m py_compile tools/check_surface_policy.py tools/test_check_surface_policy.py`, and `git diff --check`.
- U3 scaffold/app-ladder exploration completed in subagent `019f15fa-144d-7043-a574-2409e52848d1`: first slice should add/promote a public `workbench-lite` scaffold from the API workbench prototype, add a focused settings dialog recipe, and document existing `commands_keymap_basics`, `data_table_basics`, and `mutation_toast_feedback_basics`; workspace/canvas/node graph should stay advanced until public wrappers exist.
- Closed read-only audit subagents `019f1440-5447-75e2-b450-6700e61df8c6` and `019f1440-7fc5-7330-83d1-b51dae3f56f3`. Their reusable findings are: U4/U7/U8 should address layout dirty scans, subtree invalidation allocations, paint-cache rebase costs, text/layout cache and atlas churn; U6/U9 should address over-wide shadcn raw surface, giant menu/select files, table state/test monoliths, docking diagnostics vocabulary in runtime, and large docking/node/chart/plot tests or state objects.
- Completed U3 first slice locally: added `fretboard new workbench-lite`, generated a public app-facade workbench with command palette button, settings dialog, content pane, status bar, simulated submit flow, and stable `TEST_ID_*` selectors. Updated `docs/first-hour.md`, `docs/examples/README.md`, `docs/crate-usage-guide.md`, and `apps/fret-examples/README.md`.
- U3 verification passed after one transient crates.io `xml-rs` download failure was retried successfully: `cargo fmt --all --check`, `cargo nextest run -p fretboard scaffold`, `python3 tools/check_surface_policy.py`, `python3 tools/check_layering.py`, `python3 tools/check_consumption_profiles.py`, and `git diff --check`.
- U4 read-only explorer `019f1627-edfc-7001-be3e-8ef1d98c076e` recommends starting with observability, not `StableNodeHandle`: add counters for identity seeded/fallback resolve, parent pointer repair, GC reachability, dirty frontier breadth, dispatch snapshot cache hit/miss/build/invalidation, and observation churn; keep fallback scans temporarily but gate them toward zero after warmup.
