# IMUI Worktree Convergence Plan - 2026-05-26

Status: Active integration plan

## Context

Two worktrees currently contain overlapping IMUI work:

- `F:/SourceCodes/Rust/fret` is on `main` at `09e568ed` and is ahead of `origin/main` by six
  shadcn/parity commits.
- `F:/SourceCodes/Rust/fret-worktrees/imui-imgui-editor-grade-refactor` is on
  `imui-imgui-editor-grade-refactor` at `901aa6bdfd`, which is the merge-base with `main`.
- Both worktrees contain dirty IMUI changes that overlap in `fret-ui-kit/src/imui/*`,
  `fret-imui` tests, `fret-plot`, `tools/gate_imui_workstream_source.py`, and `Cargo.lock`.

## Integration Decision

Use `main` as the integration base because it already contains the six committed shadcn/parity
foundation commits that the IMUI branch is missing. Do not continue feature development in either
worktree until the worktree convergence is complete.

Merge content by topic rather than treating either worktree as globally authoritative:

- Keep `main` as the final branch and history base.
- Preserve the `fret-plot/imui` adapter from either side; the implementation is identical.
- Preserve the table owner split from either side for the identical files, resolving the small
  `render.rs` import drift mechanically.
- Prefer the `imui-imgui-editor-grade-refactor` facade organization for the final tree:
  `facade_core.rs`, `scope_methods.rs`, `container_methods.rs`, completed
  `container_wrappers.rs`, `layout_sugar.rs`, and the associated porting-sugar options.
- Prefer the `imui-imgui-editor-grade-refactor` ListBox organization where it follows the facade
  owner split, while keeping the same semantic scope: no selection model, filtering policy,
  command package, or active-descendant ownership in the ListBox container.
- Prefer the `imui-imgui-editor-grade-refactor` canonical workbench, Demo/Metrics/Debug, style/theme
  picker, and closeout docs because those topics do not exist in `main` and have focused evidence.
- Include `main`'s `facade_writer/image_items.rs` in the first convergence checkpoint only after it
  is completed with source-gate and workstream evidence. This completion was done before the
  checkpoint so the root facade can compile without partial staging.
- Rebuild `tools/gate_imui_workstream_source.py` from the `imui-imgui-editor-grade-refactor` gate
  coverage, then reconcile any `main`-only guards that remain relevant.

## Execution Order

1. Create a `main` checkpoint commit for the already closed `main` slices, including the completed
   `facade_writer/image_items.rs` owner split.
2. Create a checkpoint commit on `imui-imgui-editor-grade-refactor` for its current dirty state.
3. Merge `imui-imgui-editor-grade-refactor` into `main`.
4. Resolve conflicts by the topic decisions above.
5. Run the focused convergence gates before resuming feature development.
6. Continue IMUI development only from `F:/SourceCodes/Rust/fret` on `main` after convergence.

## Minimum Gates

Run these after conflict resolution, expanding only if a failure points to a broader surface:

```powershell
git diff --check
python tools\gate_imui_workstream_source.py
python tools\check_workstream_catalog.py
python -m py_compile tools\gate_imui_workstream_source.py tools\diag_gate_imui_p2_devtools_first_open.py tools\diag_gate_imui_product_chain.py
cargo fmt --check -p fret-ui-kit -p fret-imui -p fret-plot -p fret-ui-editor -p fret-examples -p fret-demo -p fretboard-dev -p fret-devtools -p fret-devtools-mcp
cargo check -p fret-ui-kit --features imui --lib
cargo check -p fret-plot
cargo check -p fret-plot --features imui
cargo check -p fret-ui-editor --features imui
cargo check -p fret-demo --bin imui_editor_workbench_demo
cargo nextest run -p fret-imui list_box_container_stamps_semantics_scroll_and_hosts_selectables table_sortable_header_reports_app_owned_trigger_without_sorting_rows table_resizable_header_reports_drag_response table_plain_header_left_click_does_not_activate_or_click --no-fail-fast
cargo nextest run -p fret-plot imui_adapter_stays_opt_in_and_declarative_only --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui editor_theme_preset_picker_stamps_listbox_options_and_selected_state editor_theme_preset_picker_click_updates_model_and_replays_reversible_preset --no-fail-fast
cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface --no-fail-fast
```

## Risk Notes

- `tools/gate_imui_workstream_source.py` is the highest-risk conflict because both worktrees use it
  as source-of-truth evidence for different slices.
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` should resolve toward the more complete owner
  split instead of re-expanding default method bodies.
- `apps/fret-examples/src/lib.rs` has line-ending warnings in the IMUI worktree. Do not normalize
  unrelated files during the merge.
- `facade_writer/image_items.rs` was completed with gate and evidence coverage before the first
  checkpoint; preserve that small owner split during convergence unless the later IMUI worktree
  facade organization replaces it with an equivalent owner.
