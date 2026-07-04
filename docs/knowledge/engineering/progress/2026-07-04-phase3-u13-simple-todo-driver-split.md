---
type: Work Progress
title: Phase 3 U13 simple todo driver split
tags: fret,phase3,u13,examples,surface-policy,wasm
timestamp: 2026-07-04
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 now splits the copyable simple todo example from its demo-shell runner glue.

- `apps/fret-examples/src/simple_todo_demo.rs` is a default-clean app-facing view file again. It no
  longer imports `fret_bootstrap`, `fret_runtime`, `fret_launch`, `fret::advanced`, or
  `fret_icons::IconRegistry`.
- `apps/fret-examples/src/simple_todo_demo/driver.rs` owns `build_app`,
  `build_runner_config`, `build_fn_driver`, native `run`, the wasm `run` stub, and the icon
  registry installation test.
- `tools/check_surface_policy.py` now classifies `simple_todo_demo.rs` as `default_app_clean` and
  classifies only `simple_todo_demo/driver.rs` as an internal harness allowed to touch launch,
  runtime capability, and advanced view-driver seams.
- The verification pass exposed unrelated web-demo drift in the same examples shell:
  `plot_declarative_demo::run` needed native-only cfg, `plot_image_demo` and `tags_demo` needed
  wasm-compatible `build_app/build_runner_config/build_fn_driver` exports, and the command gallery
  snippet needed an explicit `fret::advanced::raw::LocalStateModelStoreExt` import for its retained
  model-store bridge.

# Verification

- `cargo check -p fret-examples --lib`
- `cargo check -p fret-demo --bin simple_todo_demo`
- `cargo check -p fret-demo-web --target wasm32-unknown-unknown`
- `cargo nextest run -p fret-examples --test simple_todo_demo_surface --no-fail-fast`
- `python3 tools/test_check_surface_policy.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `git diff --check`

Known warnings during verification remain pre-existing: `fret-chart::visual_map_track_at` dead code
and wasm-path `fret-platform-native` clipboard dead code warnings.

# Next

Continue U13 by reassessing the remaining advanced cookbook/example surfaces. The next likely
migration candidate remains `gizmo_basics.rs`, but it should start with a small scoped design for
wheel input, vector/path canvas helpers, and local-state/command migration rather than a blind move
to default-clean.

# Citations

- [Phase 3 retained bridge deletion plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Simple todo view](../../../apps/fret-examples/src/simple_todo_demo.rs)
- [Simple todo driver](../../../apps/fret-examples/src/simple_todo_demo/driver.rs)
- [Surface policy gate](../../../tools/check_surface_policy.py)
- [Remaining surface audit](../subagents/2026-07-04-phase3-u13-remaining-surface-audit.md)
