# ImUi Child Region Depth v1 - Evidence & Gates

Goal: keep the child-region depth lane tied to one current repro set, one explicit gate floor, and
one bounded evidence set before any helper widening is admitted.

## Evidence anchors (current)

- `docs/workstreams/imui-child-region-depth-v1/DESIGN.md`
- `docs/workstreams/imui-child-region-depth-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-child-region-depth-v1/M1_TARGET_SURFACE_FREEZE_2026-04-22.md`
- `docs/workstreams/imui-child-region-depth-v1/M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md`
- `docs/workstreams/imui-child-region-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-child-region-depth-v1/TODO.md`
- `docs/workstreams/imui-child-region-depth-v1/MILESTONES.md`
- `docs/workstreams/imui-child-region-depth-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
- `docs/adr/0217-scroll-offset-children-transform-and-scrollhandle-invalidation-v2.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
- `ecosystem/fret-ui-kit/src/style/layout.rs`
- `ecosystem/fret-imui/src/tests/composition.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `ecosystem/fret-workspace/src/panes.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/tests/workspace_shell_pane_proof_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui.cpp`

## First-open repro surfaces

Use these before reading older historical `imui` notes in depth:

1. Current pane-first proof surface
   - `cargo run -p fret-demo --bin workspace_shell_demo`
2. Current supporting pane-rail proof surface
   - `cargo run -p fret-demo --bin editor_notes_demo`
3. Current focused child-region composition floor
   - `cargo nextest run -p fret-imui child_region_helper_stacks_content_and_forwards_scroll_options child_region_helper_can_host_menu_bar_and_popup_menu child_region_helper_can_switch_between_framed_and_bare_chrome --no-fail-fast`
4. Current `fret-ui-kit` adapter seam floor
   - `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast`

Current status summary:

- the helper can already host editor-like pane content and embedded menu composition,
- M2 has now landed `ChildRegionChrome::{Framed, Bare}` as the only admitted generic child-depth
  slice,
- and the generic `child_region` surface still keeps resize, auto-resize, focus-boundary
  flattening, and begin-return posture out of scope until stronger first-party proof appears in a
  different narrow lane.

## Current focused gates

### Lane-local source-policy gate

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_child_region_depth_follow_on --no-fail-fast`

This gate currently proves:

- the repo keeps the child-region depth closeout explicit,
- the lane stays separate from the closed collection/pane proof lane,
- the landed M2 chrome slice and the closeout state remain machine-visible,
- and the first-open repo indexes point to the right closeout record.

### `fret-ui-kit` adapter seam floor

- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast`

This floor currently proves:

- the exported IMUI adapter/module seam remains contract-only,
- the new `ChildRegionOptions` default posture still compiles through the adapter surface,
- and the landed `ChildRegionChrome` enum stays reviewable as a bounded option/default change.

### Focused child-region composition floor

- `cargo nextest run -p fret-imui child_region_helper_stacks_content_and_forwards_scroll_options child_region_helper_can_host_menu_bar_and_popup_menu child_region_helper_can_switch_between_framed_and_bare_chrome --no-fail-fast`

This floor currently proves:

- the current helper already forwards scroll-handle and viewport test-id behavior through the
  existing scroll substrate,
- the current helper can host `menu_bar` + popup composition inside child content,
- the landed chrome slice can switch between framed and bare posture without widening runtime
  contracts,
- the lane never needed a child-specific menu mechanism just to justify deeper work,
- and the remaining non-chrome pressure no longer justifies keeping this folder active.

### Pane-proof surface floor

- `cargo nextest run -p fret-examples --test workspace_shell_pane_proof_surface --test editor_notes_editor_rail_surface --no-fail-fast`

This floor currently proves:

- `workspace_shell_demo` remains the pane-first proof surface,
- `editor_notes_demo` remains the supporting minimal pane-rail proof surface,
- and the lane does not silently replace those first-party proofs with a narrower dedicated demo
  just to justify the landed chrome slice.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-child-region-depth-v1/WORKSTREAM.json > /dev/null`

## Closeout posture

This folder is now closed.
Do not keep growing the gate package here by default.
If future pressure still targets generic child-depth behavior beyond the chrome slice, start a new
narrower follow-on with its own repro/gate/evidence package instead of widening this closeout
record.
