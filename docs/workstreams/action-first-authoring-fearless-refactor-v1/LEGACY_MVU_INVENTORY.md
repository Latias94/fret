# Legacy MVU Inventory (Action-First Authoring v1)

Last updated: 2026-03-06

This document tracks remaining in-tree usage of legacy MVU/message-routing helpers after the
action-first + view runtime v1 adoption.

Scope:

- “Legacy MVU” refers to the former `fret::mvu` / `fret::mvu_router` surfaces (and their
  `MessageRouter`/`MvuProgram` authoring surface), plus any remaining in-tree references to them.
- This is an inventory, not a deletion plan. Deletion/quarantine is tracked by `AFA-clean-062`.

Note:

- MVU surfaces were removed from `ecosystem/fret` (feature gate + `mvu` / `mvu_router` / `legacy`
  modules) as part of M9.
- In-tree demos no longer carry a separate demo-level opt-in feature.
- Any remaining MVU references should now be limited to historical documentation only; code
  surfaces are guarded by `tools/gate_no_mvu_in_tree.py` and `tools/gate_no_mvu_in_cookbook.py`.

## Why this inventory exists

- Keep docs/templates aligned to one boring golden path (View runtime + typed actions).
- Prevent slow drift where new code keeps reintroducing per-frame command routing in places that are
  intended to be action-first.
- Provide an explicit map for incremental migrations (don’t “fearlessly refactor” 200 files at once).

Policy:

- MVU is not an available authoring path in-tree. Do not reintroduce it.
- See: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MVU_POLICY.md`

Removal-track note:

- If the repo goal becomes “fully migrated, then hard delete MVU”, track that as milestone M9 and
  keep this inventory as the only source of truth for “what is left”.

## Already converged (golden path)

- `fretboard` scaffolds generate action-first + view runtime by default:
  - `apps/fretboard/src/scaffold/templates.rs` (`hello_template_main_rs`, `simple_todo_template_main_rs`, `todo_template_main_rs`)
- Docs index points to action-first authoring as available now:
  - `docs/README.md`
- Cookbook example migrated to view runtime + actions:
  - `apps/fret-cookbook/examples/async_inbox_basics.rs`
  - `apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs`
  - `apps/fret-cookbook/examples/hello_counter.rs`
  - `apps/fret-cookbook/examples/effects_layer_basics.rs`
  - `apps/fret-cookbook/examples/markdown_and_code_basics.rs`
  - `apps/fret-cookbook/examples/payload_actions_basics.rs`
  - `apps/fret-cookbook/examples/simple_todo.rs`
  - `apps/fret-cookbook/examples/text_input_basics.rs`
  - `apps/fret-cookbook/examples/theme_switching_basics.rs`
  - `apps/fret-cookbook/examples/undo_basics.rs`
  - `apps/fret-cookbook/examples/virtual_list_basics.rs`
  - `apps/fret-cookbook/examples/icons_and_assets_basics.rs`

## Remaining legacy MVU usage (as of 2026-03-05)

### 1) Cookbook examples (still MVU)

These examples implement `MvuProgram` and/or use `MessageRouter`:

- None (as of 2026-03-05). All cookbook examples now use the view runtime + typed actions.

Recommendation:

- Keep any future MVU cookbook examples explicitly labeled as legacy/compat.
- Avoid introducing new MVU usage in the cookbook unless we need a specific payload-routing teaching sample.

### 2) `apps/fret-examples/*` demos (legacy MVU copies removed)

Current status:

- None (as of 2026-03-05). All `apps/fret-examples` demos are view runtime + typed actions.
- Legacy MVU demo copies (`*_legacy.rs`) were deleted as part of M9.
- The former demo-level opt-in feature and `apps/fret-demo` legacy routing were removed.

Recommendation:

- Avoid adding new MVU-based demos. If MVU must be used temporarily (before M9 completes), keep it
  explicitly labeled as legacy/compat and do not document it as a golden path.

### 3) UI gallery legacy glue (`mvu_router`)

Current status:

- No remaining uses of `fret::mvu_router::KeyedMessageRouter` (removed).

Recommendation:

- Keep the legacy table preview explicitly labeled as legacy.
- Avoid introducing new `mvu_router` usage in non-legacy gallery pages.

### 4) Scaffold internal legacy references

Current status:

- None (as of 2026-03-05). Internal legacy MVU scaffolding sources were removed from
  `apps/fretboard/src/scaffold/templates.rs`.

Recommendation:

- Keep scaffold templates limited to the view runtime + typed actions golden path.

## Suggested next migrations (low-risk, high-value)

1) Update docs/templates to stop mentioning MVU as an available authoring path (keep a short history
   note instead).
2) Add a lightweight gate that fails if MVU identifiers reappear (grep-based).
