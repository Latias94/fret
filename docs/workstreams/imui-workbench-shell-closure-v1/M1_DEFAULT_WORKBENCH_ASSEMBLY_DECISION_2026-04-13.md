# M1 Default Workbench Assembly Decision - 2026-04-13

Status: active decision note

Related:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-13.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/WORKSPACE_SHELL_STARTER_SET.md`

## Question

After the P1 proof roster and shell smoke floor are frozen, should the repo now promote a thinner
first-party workbench-shell helper/recipe, or should the default answer remain explicit
example-local assembly over the frozen shell starter set?

## Audited evidence

- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-ui-gallery/src/driver/render_flow.rs`
- `apps/fret-ui-gallery/src/driver/chrome.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `tools/diag-scripts/suites/diag-hardening-smoke-workspace/suite.json`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/WORKSPACE_SHELL_STARTER_SET.md`

## Findings

### 1) The reusable shell primitives already exist at the right layer

The current reusable starter set already covers the stable shell building blocks:

- `WorkspaceFrame`
- `WorkspaceTopBar`
- `WorkspaceStatusBar`
- `workspace_pane_tree_element_with_resize`
- `WorkspaceTabStrip`
- `WorkspaceCommandScope`
- `WorkspacePaneContentFocusTarget`

That means the current gap is not "we have no shell vocabulary."
The current gap is whether the repo has enough repeated *high-level assembly* pressure to justify a
promoted helper above that starter set.

### 2) The current shell consumers do not repeat one stable high-level recipe shape

The current first-party shell consumers are intentionally different:

- `workspace_shell_demo`
  - broader P1 product proof
  - pane tree, shell command scope, dirty-close policy, file-tree liveness, shell-mounted editor
    rail, and tab/pane stress behavior
- UI Gallery workspace shell
  - shell-chrome reference/exemplar
  - `WorkspaceFrame` with top bar, optional status bar, sidebar/content split, and
    `WorkspaceCommandScope`
- `editor_notes_demo`
  - minimal shell-mounted rail proof
  - `WorkspaceFrame.left/right(...)` around app-local center content

These surfaces share the starter set, but they do not yet imply one stable promoted helper shape.

### 3) A promoted helper would be either too thin or too opinionated right now

If the repo promoted a helper today, it would likely fall into one of two failure modes:

1. Too thin:
   - only wraps a few obvious `WorkspaceFrame` + `WorkspaceCommandScope` calls,
   - reduces very little real decision-making,
   - and does not justify a new public/default-path noun.
2. Too opinionated:
   - bakes in app-local product choices such as file-tree rails, dirty-close UX, editor-rail
     content, or pane-tree posture,
   - and starts pulling product policy into `fret-workspace` or another first-party owner too
     early.

## Decision

Keep the default workbench answer as explicit example-local assembly over the frozen starter set.

Concretely:

1. Do not introduce `WorkspaceWorkbenchShell`, `EditorWorkbenchShell`, or a similar promoted helper yet.
2. Keep the reusable shell substrate in the existing `fret-workspace` starter set.
3. Keep the current owner for this slice in app/example composition above that starter set.
4. Keep `workspace_shell_demo` as the broader P1 product proof.
5. Keep UI Gallery workspace shell as a shell-chrome reference/exemplar, not as a competing P1
   product proof.

## Extraction threshold for a future helper

Revisit helper promotion only when all of these are true:

1. There is a second full-shell consumer beyond the current mixed proof surfaces that repeats the
   same assembly posture.
2. The repeated wiring is substantial enough that keeping it explicit is now the larger cost.
3. One focused source-policy gate and one launched gate can name the promoted helper surface
   directly.
4. The helper can stay above the starter set without absorbing app-specific protocols or
   runner/backend behavior.

## Immediate execution consequence

This lane should not spend its next slice extracting a new helper.

Instead:

- treat the current verdict as a no-new-helper-yet decision,
- keep the workbench answer explicit in first-party proof surfaces,
- and only reopen extraction if fresh repeated shell assembly pressure exceeds this note.
