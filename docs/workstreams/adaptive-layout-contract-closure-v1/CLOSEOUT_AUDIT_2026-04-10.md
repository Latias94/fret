# Closeout Audit — 2026-04-10

Status: closed closeout record

Related:

- `docs/workstreams/adaptive-layout-contract-closure-v1/DESIGN.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/M1_CONTRACT_FREEZE_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/M2_PANEL_RESIZE_GATE_PROMOTION_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/M2_GALLERY_QUERY_AXIS_PROOF_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/M3_EDITOR_RAIL_COMPOSITION_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/TODO.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/MILESTONES.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/EVIDENCE_AND_GATES.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- `docs/known-issues.md`
- `docs/crate-usage-guide.md`
- `docs/workstreams/container-queries-v1/container-queries-v1.md`
- `docs/workstreams/environment-queries-v1/environment-queries-v1.md`

## Verdict

This lane is now closed.

It completed the broad cross-lane closure work it was opened for:

- the adaptive taxonomy is frozen,
- the public low-level vs policy-level import lanes are explicit,
- the first user-visible proof surfaces are active,
- and the editor-rail owner split is proven in running code.

What remains is no longer one broad "adaptive closure" queue.
It is family-specific follow-on work only if fresh evidence appears.

## What shipped

### 1) Query-axis ownership is now explicit in shipped docs and facade lanes

The repo no longer relies on one ambiguous "responsive" story.

The current shipped posture is:

- low-level reads stay explicit on `fret::env::{...}`,
- shared adaptive vocabulary is available on the explicit `fret::adaptive::{...}` lane,
- and recipe/documentation surfaces are expected to say whether behavior is viewport/device-driven
  or container/panel-driven.

Conclusion:

- the taxonomy goal that justified this lane is complete enough to stop treating it as an active
  execution tracker.

### 2) The proof surface set is no longer theoretical

This lane now leaves three distinct reviewable proof classes behind:

1. UI Gallery narrow-window proof,
2. fixed-window panel-resize proof,
3. explicit query-axis teaching proof on the Navigation Menu docs path.

Those proofs cover the central contract question this lane was opened to answer:

- panel width must stay distinct from viewport width,
- user-visible docs/examples must teach the difference,
- and regressions should be caught before another broad adaptive rewrite is proposed.

Conclusion:

- adaptive closure is now tied to reproducible gates rather than only naming guidance.

### 3) App-shell and editor-shell adaptive stories are no longer conflated

The lane also closes the owner question that kept leaking into adaptive discussions:

- shadcn `Sidebar` remains an app-shell / device-shell surface,
- editor-grade inner panel content stays on `fret-ui-editor`,
- and workspace-shell placement uses `WorkspaceFrame.left/right(...)`.

The `workspace_shell_demo` proof now makes that split executable instead of leaving it as a pure
audit verdict.

Conclusion:

- there is no remaining reason to keep this lane open as the place where editor rails and app-shell
  sidebars are debated together.

### 4) The older mechanism lanes remain valid reference docs and do not need extra status notes

This lane was opened because the mechanism lanes did not own authoring taxonomy or user-visible
proof closure, not because their mechanism contracts became obsolete.

Decision:

- keep `container-queries-v1` and `environment-queries-v1` as mechanism/reference docs,
- do not rewrite them into another authoring lane,
- and let the repo-level entrypoints point readers to this closeout record when they need the
  shipped cross-lane posture.

Conclusion:

- `ALC-051` resolves to "leave the mechanism lanes as reference docs; do not add churn-only
  status notes there."

### 5) The remaining backlog no longer justifies a broad follow-on lane today

The remaining pressure is narrower than this lane's original scope.

Examples of future work that may still happen:

- one component family with fresh device-vs-panel drift,
- a second real editor-rail consumer that justifies extraction,
- or a new adaptive strategy layer above today's `fret::adaptive` classification helpers.

Those are not unresolved v1 closure debt.

Conclusion:

- do not reopen `adaptive-layout-contract-closure-v1` for generic "responsive cleanup".

## Gates that define the closed surface

- `cargo nextest run -p fret-ui-gallery --test popup_menu_narrow_surface`
- `cargo nextest run -p fret-ui-gallery --test navigation_menu_docs_surface --no-fail-fast`
- `cargo nextest run -p fret-ui-shadcn --test navigation_menu_query_mode_reopen --no-fail-fast`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/navigation/ui-gallery-navigation-menu-md-breakpoint-query-source-toggle.json --dir target/fret-diag/adaptive-navigation-menu-query-axis --session-auto --pack --include-screenshots --launch target/release/fret-ui-gallery`
- `cargo run -p fretboard -- diag run tools/diag-scripts/container-queries-docking-panel-resize.json --dir target/fret-diag/adaptive-panel-resize-promote --session-auto --pack --include-screenshots --launch target/release/container_queries_docking_demo`
- `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --no-fail-fast`
- `cargo check -p fret-demo --bin workspace_shell_demo --message-format short`
- `git diff --check`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`

## Follow-on policy

Do not reopen this lane for:

- generic responsive cleanup,
- another broad naming pass without fresh proof pressure,
- or speculative public `PanelRail` / `InspectorSidebar` extraction.

If future work is needed, start a narrower follow-on such as:

1. one component-family adaptive parity lane,
2. one editor-rail extraction lane after a second real consumer exists,
3. or one higher-level adaptive strategy lane if `fret::adaptive` needs to grow beyond the current
   explicit classification helpers.
