# Closeout Audit - 2026-04-22

Status: closed closeout record

This audit records the final closeout read for `imui-child-region-depth-v1`.

Goal:

- verify whether the M1 target-surface freeze plus the landed M2 chrome slice still leave an
  active generic child-region design queue,
- separate the shipped chrome answer from pressures that still belong to shell/product owners or
  future narrower lanes,
- and decide whether this lane should remain active or close.

## Audited evidence

Lane docs:

- `docs/workstreams/imui-child-region-depth-v1/DESIGN.md`
- `docs/workstreams/imui-child-region-depth-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-child-region-depth-v1/M1_TARGET_SURFACE_FREEZE_2026-04-22.md`
- `docs/workstreams/imui-child-region-depth-v1/M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md`
- `docs/workstreams/imui-child-region-depth-v1/TODO.md`
- `docs/workstreams/imui-child-region-depth-v1/MILESTONES.md`
- `docs/workstreams/imui-child-region-depth-v1/EVIDENCE_AND_GATES.md`

Umbrella and reference docs:

- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`

Implementation / gate anchors:

- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-imui/src/tests/composition.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `ecosystem/fret-workspace/src/panes.rs`
- `apps/fret-examples/src/lib.rs`

Validation run used for closeout:

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_child_region_depth_follow_on --no-fail-fast`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast`
- `cargo nextest run -p fret-imui child_region_helper_stacks_content_and_forwards_scroll_options child_region_helper_can_host_menu_bar_and_popup_menu child_region_helper_can_switch_between_framed_and_bare_chrome --no-fail-fast`
- `cargo nextest run -p fret-examples --test workspace_shell_pane_proof_surface --test editor_notes_editor_rail_surface --no-fail-fast`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-child-region-depth-v1/WORKSTREAM.json > /dev/null`
- `git diff --check`

## Findings

### 1. The lane has now landed the only generic child-depth slice justified by first-party proof

M1 froze the candidate set and explicitly narrowed the likely generic admission to chrome posture.
M2 then landed the bounded answer:

- `ChildRegionOptions.chrome`
- default `ChildRegionChrome::Framed`
- opt-in `ChildRegionChrome::Bare`

The focused proof package now locks that outcome at both the contract seam and the IMUI
composition seam.

Conclusion:

- the lane no longer has an unresolved generic child-chrome question.

### 2. The remaining `BeginChild()`-scale pressure still belongs to other owners or future narrower lanes

What still remains after the chrome slice is real, but it is not unfinished work for this folder:

1. axis-specific resize
   - remains shell-owned in `fret-workspace` / product layout owners.
2. auto-resize / always-auto-resize
   - still lacks stronger first-party proof than the current pane-first demos.
3. focus-boundary flattening
   - still lacks a focused keyboard proof for generic admission.
4. `BeginChild() -> bool` visibility/return posture
   - remains a bad fit for the current declarative helper shape.
5. broader pane/workbench behavior
   - still belongs to shell/product owners rather than generic `fret-ui-kit::imui`.

Conclusion:

- keeping this folder active would only blur a closed verdict back into a standing backlog.

### 3. The existing proof surfaces remain sufficient; no new proof demo promotion is warranted

The current proof roster still matches the shipped answer:

- `workspace_shell_demo` remains the pane-first proof surface,
- `editor_notes_demo` remains the supporting pane-rail proof,
- the targeted `fret-imui` composition floor now covers scroll forwarding, embedded menu
  composition, and chrome posture switching,
- and `imui_adapter_seam_smoke` keeps the exported option/default seam reviewable.

Conclusion:

- this lane does not need a new dedicated demo or a broader diagnostics surface to stay credible.

## Decision from this audit

Treat `imui-child-region-depth-v1` as:

- closed for the current generic child-region depth question,
- a closeout record for the landed `ChildRegionChrome::{Framed, Bare}` slice,
- and not the place to continue adding resize / auto-resize / focus-boundary flattening /
  begin-return posture by default.

## Immediate execution consequence

From this point forward:

1. keep the current pane-first proof pair and focused child-region test floor as the shipped
   evidence package,
2. keep `ChildRegionChrome::{Framed, Bare}` as the bounded generic answer for this cycle,
3. do not widen generic `child_region` here with resize, auto-resize, focus-boundary flattening,
   or a `BeginChild() -> bool` contract,
4. keep shell/product pane behavior in shell/product owners,
5. and start a different narrower follow-on only if stronger first-party proof shows the current
   chrome slice is insufficient.
