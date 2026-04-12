# P1 Workbench Proof Matrix - 2026-04-12

Status: focused P1 matrix / decision note

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`

## Purpose

P1 needs one reviewable answer to a narrow question:

> which current first-party surfaces together prove the editor workbench shell, and which owner is
> responsible when that shell still feels incomplete?

This note freezes that proof roster before any implementation-heavy shell follow-on starts.

## Audited evidence

- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`

## Assumptions-first resume set

1. Confident: `workspace_shell_demo` is the broadest current first-party workbench shell proof.
   Evidence:
   - `WorkspaceFrame`
   - `WorkspaceTabStrip`
   - `workspace_pane_tree_element_with_resize`
   - `WorkspaceCommandScope`
   Consequence if wrong:
   - P1 would freeze the wrong primary proof and blur owner responsibilities too early.
2. Confident: `editor_notes_demo` is the clean minimal shell-mounted rail proof.
   Evidence:
   - `WorkspaceFrame::new(center)`
   - explicit `.left(left_rail)` / `.right(right_rail)`
   - `InspectorPanel::new(None)`
   Consequence if wrong:
   - P1 would lose the smallest proof that separates shell slots from editor content ownership.
3. Likely: `imui_editor_proof_demo` is important supporting evidence for docking/editor integration,
   but not the best default workbench-shell proof.
   Evidence:
   - strong docking and editor-immediate proof
   - shell is centered on `dock_space_with(...)` and immediate-mode editor surfaces
   Consequence if wrong:
   - P1 might overfit shell guidance to the immediate/docking-heavy proof instead of the clearer
     workspace-shell baseline.
4. Confident: missing shell closure should stay split between `fret-workspace`,
   `fret-docking`, `fret-ui-editor`, and app/example composition rather than reopening generic
   `imui` growth.
   Evidence:
   - prior lane closeouts already froze generic immediate ownership
   - current shell surfaces already show explicit `WorkspaceFrame` vs `InspectorPanel` boundaries
   Consequence if wrong:
   - P1 would regress into the same owner confusion that earlier `imui` lanes already closed.

## Proof matrix

| Surface | Classification | What it proves | Primary owner signal | What it does **not** prove |
| --- | --- | --- | --- | --- |
| `apps/fret-examples/src/workspace_shell_demo.rs` | Primary P1 workbench-shell proof | Coherent shell composition with pane tree, tab strip, shell-mounted inspector rail, command scope, dirty-close prompt, overlay coordination, and file-tree/content split | `fret-workspace` owns outer shell / pane / tab / command-scope composition; `fret-ui-editor` supplies inspector composites; app/example owns center content and data | Does not prove multi-window hand-feel parity by itself |
| `apps/fret-examples/src/editor_notes_demo.rs` | Secondary minimal shell-mounted rail proof | Smallest clear proof that `WorkspaceFrame` owns outer shell slots while `fret-ui-editor` owns reusable inspector content | `fret-workspace` for left/right rail slots and shell frame; `fret-ui-editor` for `InspectorPanel` / `PropertyGrid`; app/example for scene-specific center content | Does not prove pane trees, tab strips, docking arbitration, or multi-window behavior |
| `apps/fret-examples/src/imui_editor_proof_demo.rs` | Supporting docking/editor proof | Editor-grade immediate proof with docking runtime, dock-space composition, floating shells, and editor-owned immediate composites | `fret-docking` owns dock graph/runtime integration; `fret-ui-editor::imui` owns editor nouns; `fret-ui-kit::imui` owns generic immediate vocabulary | Should not be treated as the default P1 workbench-shell proof |
| `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md` | Supporting kernel/behavior contract proof | Tab-strip kernel, overflow, pinned boundary, close/reorder/drop contracts, and shell-level regression posture | `fret-workspace` owns tab-strip policy/kernel above runtime mechanisms | Does not prove the whole editor workbench shell alone |
| `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md` | Supporting runner/backend parity proof | Tear-off, hover-behind, transparent payload, close/merge, and cross-window hand-feel ownership | runner/backend + `fret-docking` own multi-window docking parity | Does not define shell slots, editor rails, or tab-strip product composition |

## Frozen P1 reading order

When reopening P1, read the current shell proof set in this order:

1. `apps/fret-examples/src/workspace_shell_demo.rs`
   - primary coherent workbench shell proof
2. `apps/fret-examples/src/editor_notes_demo.rs`
   - minimal shell-mounted rail proof
3. `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`
   - tab-strip behavior kernel / shell chrome contract
4. `apps/fret-examples/src/imui_editor_proof_demo.rs`
   - docking/editor immediate support proof
5. `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
   - runner/backend hand-feel parity proof

This order is intentional:

- shell composition first,
- shell-mounted editor content second,
- docking-heavy and multi-window parity proofs after that.

## Owner split to use for future P1 gaps

Map missing closure like this by default:

1. `fret-workspace`
   - outer shell slots,
   - pane tree composition,
   - tab strip behavior,
   - shell command scope,
   - dirty-close shell policy,
   - workspace-level focus restore
2. `fret-docking`
   - dock graph integration,
   - shell-aware docking choreography,
   - panel move/re-dock semantics,
   - shell/drop arbitration at the docking layer
3. `fret-ui-editor`
   - inspector/property/editor composites,
   - editor-specific field controls,
   - reusable editor content surfaces mounted inside shell slots
4. app/example or recipe layer
   - scene-specific center content,
   - selection/domain models,
   - product-local panel recipes that are not generic shell policy
5. runner/backend
   - hovered-window quality,
   - tear-off follow,
   - transparent payload / peek-behind,
   - mixed-DPI and cross-window hand-feel

## Decision

From this point forward:

1. `workspace_shell_demo` is the primary P1 coherent workbench-shell proof,
2. `editor_notes_demo` is the minimal secondary proof for shell-mounted rails,
3. `imui_editor_proof_demo` remains important supporting evidence, but should not define the
   default workbench-shell reading order,
4. and shell-level missing pieces should stay out of the generic `imui` backlog unless fresh
   evidence shows a real generic immediate contract gap.

## Immediate execution consequence

For this lane, the next durable outcome is simple:

- treat the proof roster above as the current P1 source of truth,
- evaluate future shell gaps against this owner split first,
- and start a narrower P1 follow-on only when implementation work starts to cluster under one owner
  strongly enough to justify its own lane.
