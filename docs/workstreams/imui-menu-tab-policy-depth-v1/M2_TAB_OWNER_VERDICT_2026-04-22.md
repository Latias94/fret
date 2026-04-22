# M2 Tab Owner Verdict - 2026-04-22

Purpose: decide whether Dear ImGui-style richer tab behavior should widen generic
`fret-ui-kit::imui::tab_bar`, or remain owned by the editor/workspace shell surface.

## Evidence reviewed

Lane docs:

- `docs/workstreams/imui-menu-tab-policy-depth-v1/DESIGN.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`

Implementation and proof anchors:

- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-workspace/src/tab_strip/mod.rs`
- `ecosystem/fret-workspace/src/tab_strip/interaction.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `ecosystem/fret-workspace/tests/tab_strip_overflow_menu_lists_overflowed_tabs.rs`
- `ecosystem/fret-workspace/tests/tab_strip_focus_restore_after_close_command.rs`
- `ecosystem/fret-workspace/tests/tab_strip_pinned_boundary_has_test_id.rs`
- `ecosystem/fret-workspace/tests/tab_strip_keyboard_roving_arrow_activates_tab.rs`

Upstream comparison anchors:

- `repo-ref/imgui/imgui_demo.cpp`

## Findings

### 1. Dear ImGui's richer tab story is editor/workspace chrome, not generic in-page navigation

The Dear ImGui demo path around `BeginTabBar()` includes behavior such as:

- reorderable tabs,
- close buttons,
- tab-list popup fallback,
- fitting-policy flags,
- and leading/trailing `TabItemButton()` action tabs.

Those outcomes are closer to editor/workspace chrome than to the current generic Fret
`tab_bar[_with_options]` helper, which intentionally targets simple immediate selection + panel
switching.

### 2. Fret already has a first-party owner for the editor-grade tabstrip problem

`fret-workspace::WorkspaceTabStrip` already owns the shell-heavy policy that Dear ImGui's richer
tab demo resembles:

- overflow menu policy,
- pinned-row and pinned-boundary behavior,
- close commands and close-adjacent actions,
- MRU-aware focus restore after close,
- and drag/drop integration with pane/docking workflows.

That ownership is not hypothetical:

- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
  already classifies `workspace_shell_demo` and `workspace-tabstrip-editor-grade-v1` as the
  current shell/tab owner surface,
- `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md` explicitly freezes tab strip as
  editor/workspace chrome above runtime mechanisms,
- and `ecosystem/fret-workspace/src/tab_strip/mod.rs` already carries shell-only policy such as
  `overflow_menu_active_policy(...)`, `separate_pinned_row(...)`, MRU-based focus restore, close
  affordances, and overflow/drop orchestration.

### 3. Widening generic IMUI tabs now would duplicate an existing owner and blur the layer split

If generic `imui::tab_bar` grows overflow / reorder / close / pinned / trailing-action behavior
now, the repo would end up with:

- one generic immediate tab story in `fret-ui-kit::imui`,
- plus one editor/workspace tabstrip story in `fret-workspace`,
- with no second non-workspace first-party consumer proving the overlap is worth it.

That would regress the same ownership discipline the repo has been trying to preserve:

- generic immediate helpers stay compact,
- shell/product chrome stays in shell/product owners,
- and runtime mechanisms stay untouched unless the policy truly needs a lower layer.

## Verdict

Do not widen generic `fret-ui-kit::imui::tab_bar` with editor-grade overflow / reorder / close /
pinned / action-tab behavior in this lane.

Keep that behavior owned by `fret-workspace::WorkspaceTabStrip` and the workbench shell proof
surfaces.
Reopen generic IMUI tab growth only if a different first-party consumer, outside the workspace
shell, proves that the same policy belongs in a shared immediate helper.

## Immediate execution consequence

From this point forward:

1. remove tab overflow / scroll / reorder / close from the remaining generic IMUI backlog for this
   lane,
2. treat `ecosystem/fret-workspace/src/tab_strip/mod.rs` and
   `apps/fret-examples/src/workspace_shell_demo.rs` as the current owner path for editor-grade
   tabstrip behavior,
3. keep the active generic IMUI question focused on richer submenu-intent tuning and any possible
   roving/mnemonic posture,
4. start a different shell/workspace lane, not a generic IMUI helper lane, if the next work is
   about editor-grade tabs.
