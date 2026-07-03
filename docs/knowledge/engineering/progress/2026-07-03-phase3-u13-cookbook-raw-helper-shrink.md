---
type: Work Progress
title: Phase 3 U13 cookbook raw helper shrink
tags: fret,phase3,u13,cookbook,app-facade,source-policy
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 fifth slice removes no-new-API raw helper usage from selected cookbook examples and
shrinks the source-policy quarantine records accordingly.

Changed examples:

- `apps/fret-cookbook/examples/commands_keymap_basics.rs`
- `apps/fret-cookbook/examples/text_input_basics.rs`
- `apps/fret-cookbook/examples/virtual_list_basics.rs`

`commands_keymap_basics` and `text_input_basics` no longer import
`fret::advanced::raw::LocalStateModelStoreExt`. Their command availability closures capture
frame-derived booleans instead of reopening `host.models_mut()`.

`virtual_list_basics` now stores its item collection in `LocalState<Arc<Vec<RowItem>>>` via
`app.local_state(...)`. Rotate, scroll-target, and scroll-jump actions now use
`cx.actions().locals_with(...)` / `LocalStateTxn` instead of `cx.actions().models(...)` and raw
`ModelStore` reads. The example still intentionally uses lower-level virtual-list mechanism types,
so its source-policy record remains but no longer lists `fret_runtime`.

# Verification

Passed on 2026-07-03:

- `cargo check -p fret-cookbook --example commands_keymap_basics`
- `cargo check -p fret-cookbook --example text_input_basics --features cookbook-state`
- `cargo check -p fret-cookbook --example virtual_list_basics --features cookbook-state`
- `cargo check -p fret-cookbook --all-targets`
- `cargo check -p fret-cookbook --all-targets --features cookbook-state`
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
- Static search over the three touched examples for `LocalStateModelStoreExt`, raw
  `read_in(host.models_mut())`, `models::<act::RotateItems>`, `models::<act::ScrollToTarget>`,
  `models::<act::ScrollJump>`, `use fret_runtime::Model`, `value_in_or(models`,
  `value_in_or_default(models)`, and `fret::advanced` returned no matches.

# Remaining U13 Work

The next narrow shrink candidates are the larger examples that still truly coordinate shared
runtime models: `form_basics`, `undo_basics`, `drag_basics`, and IMUI examples. Those likely need
small app-facing helper design or explicit advanced lane classification rather than purely local
rewrites.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Cookbook raw seam discovery gate](2026-07-03-phase3-u13-cookbook-raw-seam-discovery.md)
