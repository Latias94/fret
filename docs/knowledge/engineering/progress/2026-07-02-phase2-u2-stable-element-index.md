---
type: Work Progress
title: Phase 2 U2 stable element node index
tags: fret,ui,identity,phase2,ce-work
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
status: active
---

# Summary

Phase 2 U2 is partially implemented on `feat/ui-framework-phase2-refactor`.
The slice adds `StableNodeHandle`, a window-local `ElementNodeIndex`, identity-index frame stats,
bootstrap diagnostics fields, and `fret-diag` perf-key registry coverage.

The current implementation intentionally keeps the old fallback scan path for U3. Duplicate live
declarative ids are diagnostic and no longer silently pick a retained fallback node in the seeded
resolver. Indexed hits now revalidate retained attachment so a missed detach cleanup cannot turn a
stale handle into an authoritative live result.

# Verification

Passed:

- `cargo check -p fret-ui`
- Focused U2 identity nextest coverage for indexed hit, seeded stale-to-index, rebind, duplicate,
  removed-node seed, stale indexed detached handles, per-window isolation, and existing seeded
  live/reusable cases.
- `cargo check -p fret-bootstrap`
- `cargo check -p fret-diag`
- Focused `fret-diag` perf-key registry tests.
- `cargo nextest run -p fret-bootstrap --lib --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `git diff --check`

Failed:

- `cargo nextest run -p fret-ui --no-fail-fast`: 1155 passed, 23 failed.
- Failures cluster around scroll/layout/text/prepaint/scroll-into-view behavior, including
  `anchored_anchor_element_uses_scroll_transformed_visual_bounds`,
  `canvas_prepaint_can_prepare_text_scene_fragment_before_paint`, layout primitive harnesses,
  text wrap width tests, interactive resize wrapped-text tests, and scroll-into-view tests.

# Current Diagnosis

Main-thread isolation experiments found that the anchored scroll-transform failure still reproduced
after temporarily disabling the resolver index branch, seed duplicate recording, `set_node_element`
generation/indexing behavior, and structural `index_live_subtree` maintenance. A later reversible
baseline experiment also applied the tracked U2 diff in reverse against `HEAD` and still reproduced
`anchored_anchor_element_uses_scroll_transformed_visual_bounds`, proving this representative failure
is present without the U2 tracked changes.

Explorer `019f21e5-3622-7111-a55e-0d7cddb35d15` concluded that the 23 failures are best explained
as current-branch scroll/layout/text/prepaint regressions, not direct consequences of the U2
identity-index diff. It grouped the failures into scroll/layout extent, wrapped-text measurement,
canvas prepaint manifest fingerprint, and barrier solve-cache clusters. The explorer did identify
the U2 stale-index risk above; that has been fixed with focused coverage.

# Next Action

Commit U2 with focused identity evidence and record the full `fret-ui` gate as a known baseline
blocker. The next implementation slice should fix the scroll/layout extent cluster first, starting
from `scroll_rounds_scrollable_extent_up_to_next_pixel` or the layout primitives harness, before
attempting higher-level scroll-into-view or combobox retained-active failures.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- Explorer `019f21e5-3622-7111-a55e-0d7cddb35d15`
