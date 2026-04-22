# M2 Child Region Chrome Slice - 2026-04-22

Purpose: land the only bounded generic child-region slice admitted by the M1 target-surface
freeze.

## Evidence reviewed

- `docs/workstreams/imui-child-region-depth-v1/DESIGN.md`
- `docs/workstreams/imui-child-region-depth-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-child-region-depth-v1/M1_TARGET_SURFACE_FREEZE_2026-04-22.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0217-scroll-offset-children-transform-and-scrollhandle-invalidation-v2.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-imui/src/tests/composition.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `ecosystem/fret-workspace/src/panes.rs`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui.cpp`

## Findings

### 1) The current first-party proof only justifies chrome posture, not a wider child-flag surface

The current proof set already answers the broader "can pane-first child content exist at all?"
question:

- `workspace_shell_demo` and `editor_notes_demo` keep the pane-first proof explicit,
- sizing remains handled by `LayoutRefinement`,
- embedded menu composition already works inside child content,
- and shell/product resize already has a stronger owner in `ecosystem/fret-workspace/src/panes.rs`.

That leaves only one generic helper-local mismatch that the current first-party proof actually
exposes: the helper previously had one implicit framed card posture with no way to opt into a bare
surface.

Conclusion:

- keep generic admission limited to chrome posture only.

### 2) `ChildRegionChrome::{Framed, Bare}` is the smallest surface that fixes the current mismatch

The landed slice adds one bounded enum:

- `ChildRegionChrome::Framed`
- `ChildRegionChrome::Bare`

and threads it through `ChildRegionOptions`.

The helper behavior now becomes:

- `Framed` keeps the existing default posture:
  `.p_2()`, `.rounded_md()`, `.border_1()`, `bg(card)`, and `border_color(border)`.
- `Bare` removes that built-in frame/padding chrome while preserving:
  keyed identity,
  the scroll-area substrate,
  default vertical child flow,
  and existing `ScrollOptions` forwarding.

Conclusion:

- admit one bounded chrome enum instead of a broad `ChildRegionFlags` clone.
- Keep sizing on `LayoutRefinement`.

### 3) The slice now has focused proof at both the contract seam and the IMUI composition seam

The landed proof package is intentionally small:

- `adapter_seam_option_defaults_compile`
- `adapter_seam_module_stays_contract_only`
- `child_region_helper_stacks_content_and_forwards_scroll_options`
- `child_region_helper_can_host_menu_bar_and_popup_menu`
- `child_region_helper_can_switch_between_framed_and_bare_chrome`

The new composition proof locks the practical result:

- the framed variant produces a larger first-item inset than the bare variant,
- which proves the helper no longer hardcodes one chrome posture,
- without needing a new dedicated pane demo or a wider immediate runtime surface.

Conclusion:

- the chrome slice is now executable rather than purely documentary.

### 4) The M1 defer/reject list stays unchanged after landing the chrome slice

Nothing in the landed chrome option weakens the M1 owner split:

- do not clone Dear ImGui `size_arg`,
- keep axis-specific resize shell-owned,
- keep auto-resize / always-auto-resize deferred,
- keep focus-boundary flattening deferred,
- and reject a `BeginChild() -> bool` return contract for the declarative helper.

Conclusion:

- `ChildRegionChrome` is the only admitted generic child-depth addition in this lane.

## Verdict

M2 lands `ChildRegionChrome::{Framed, Bare}` as the only admitted generic child-region depth slice
for this lane.

What ships:

- `ChildRegionOptions.chrome`
- default `ChildRegionChrome::Framed`
- opt-in `ChildRegionChrome::Bare`

What still does **not** ship:

- axis-specific resize flags,
- axis-specific auto-resize / always-auto-resize flags,
- focus-boundary flattening,
- a `BeginChild() -> bool` return contract,
- or a cloned Dear ImGui child-flag bag.

## Immediate execution consequence

From this point forward:

1. use `chrome: ChildRegionChrome::Bare` only when shell/product code wants to supply its own
   frame/padding posture,
2. keep generic sizing on `LayoutRefinement`,
3. keep pane resize in shell/product owners such as `fret-workspace`,
4. keep resize / auto-resize / focus-boundary flattening / begin-return posture out of generic
   `child_region`,
5. and reopen child-depth growth only through a new narrower follow-on if stronger first-party
   proof exceeds the current chrome slice.
