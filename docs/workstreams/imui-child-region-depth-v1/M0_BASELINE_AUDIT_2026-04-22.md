# M0 Baseline Audit — 2026-04-22

Purpose: justify why `imui-child-region-depth-v1` is a new narrow follow-on instead of reopening
the closed collection/pane proof lane or widening the umbrella backlog again.

## Evidence reviewed

- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
- `ecosystem/fret-imui/src/tests/composition.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui.cpp`

## Findings

### 1) The basic pane-proof question is already closed

The repo no longer lacks first-party evidence that `child_region` can host editor-like pane
composition.

The closed collection/pane lane already froze:

- `workspace_shell_demo` as the pane-first proof surface,
- `editor_notes_demo` as supporting pane-rail evidence,
- and a no-helper-widening verdict for the proof-breadth goal itself.

Conclusion:

- this new lane should not reopen "does the helper basically work?" or "do we need a new pane-only
  demo?" by default.

### 2) The remaining gap is real, but it is narrower than proof breadth

The current helper remains intentionally thin:

- `ChildRegionOptions` only exposes `layout`, `scroll`, `test_id`, and `content_test_id`,
- `child_region_element(...)` always builds one framed scroll surface with default vertical flow,
- and there is no generic admission yet for resize, auto-resize, or focus-scope posture.

By contrast, Dear ImGui `BeginChild()` explicitly carries:

- `ResizeX` / `ResizeY`,
- `AutoResizeX` / `AutoResizeY`,
- `AlwaysAutoResize`,
- `FrameStyle`,
- and `NavFlattened`,

with specific clipping and measurement tradeoffs.

Conclusion:

- the remaining question is child-region depth, not first-party pane proof absence.

### 3) Child-specific menu composition is no longer the leading blocker

The fresh focused proof
`child_region_helper_can_host_menu_bar_and_popup_menu`
already shows that the current helper can host:

- `menu_bar`,
- popup menu triggers,
- and child-local menu content

without a new child-specific menu mechanism.

Conclusion:

- the first M1/M2 focus should stay on child-region depth, not on reopening the generic menu
  family from a different angle.

### 4) The owner split is already clear enough to avoid runtime drift

The current evidence supports a stable split:

- `crates/fret-ui` stays fixed,
- `fret-ui-kit::imui` owns any additive child-region facade surface,
- `fret-imui` owns focused regression proof,
- `apps/fret-examples` owns the first-party pane proof surfaces,
- and `fret-workspace` continues to own shell-level workbench composition above the generic helper.

Conclusion:

- this lane should start from owner freeze and target-surface audit, not from runtime growth.

### 5) Not every Dear ImGui child flag should be cloned

The upstream reference is useful because it shows the kinds of pressure mature editor UIs create,
but this repo still has to preserve its own mechanism/policy split.

That means the lane should not assume:

- every `ImGuiChildFlags_*` bit belongs in generic IMUI,
- or that a single new flag bag is automatically the right Fret answer.

Conclusion:

- M1 should audit categories, not mechanically port upstream flags.

## Execution consequence

Use `imui-child-region-depth-v1` as the active narrow P1 follow-on for `BeginChild()`-scale
child-region depth.

From this note forward:

1. treat pane-first proof breadth as closed,
2. keep `workspace_shell_demo` and `editor_notes_demo` as the first-open pane proof surfaces,
3. keep `crates/fret-ui` unchanged unless stronger evidence appears,
4. start from target-surface freeze before any helper implementation,
5. and close or split again instead of turning this folder into a broad "remaining imgui parity"
   backlog.
