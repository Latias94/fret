# P3 Child Region Readiness - 2026-05-06

Status: readiness audit; no follow-on opened yet
Last updated: 2026-05-06

## Decision

Do not widen `fret-ui-kit::imui` child-region public API from this source-audit lane.

Current Fret proof surfaces already validate a useful child-region seam:

- `ecosystem/fret-ui-kit/src/imui/child_region.rs` builds a keyed scroll-area-backed region.
- `ecosystem/fret-ui-kit/src/imui/options/containers.rs` exposes chrome, layout, scroll handles,
  scroll axes, scrollbar visibility, and diagnostic ids.
- `ecosystem/fret-imui/src/tests/composition.rs` verifies stacked content, forwarded scroll handle
  options, nested menu/popup hosting, and framed versus bare chrome.
- `apps/fret-examples/src/workspace_shell_demo.rs` proves nested shell-mounted child regions in an
  editor-style pane.
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs` proves an app-owned scrollable
  asset collection with box select, keyboard owner state, context menu, inline rename, and zoom.

That is enough for current editor proofs, but it is not full Dear ImGui `BeginChild()` parity.

## Dear ImGui Reference Axes

The relevant `repo-ref/imgui` behavior axes are:

- `BeginChild(...)` can return false for collapsed or fully clipped child windows, allowing callers
  to skip expensive content submission while still matching `EndChild()`.
- `ImGuiChildFlags_ResizeX` and `ImGuiChildFlags_ResizeY` add axis-specific manual resize behavior.
- `ImGuiChildFlags_AutoResizeX`, `AutoResizeY`, and `AlwaysAutoResize` change measurement and
  clipping tradeoffs.
- `ImGuiChildFlags_FrameStyle` is a framed visual mode comparable to Fret's existing
  `ChildRegionChrome::Framed`, but Dear ImGui ties it into child-window sizing semantics.
- `ImGuiChildFlags_NavFlattened` changes focus-scope and keyboard navigation across child borders.

## Current Fret Read

- Confident: Fret already has the common "bounded scrollable child area" use case.
  Evidence: `child_region_helper_stacks_content_and_forwards_scroll_options`,
  `child_region_helper_can_host_menu_bar_and_popup_menu`, and
  `child_region_helper_can_switch_between_framed_and_bare_chrome`.
- Confident: the current app proofs intentionally keep collection and shell behavior app-owned.
  Evidence: `workspace_shell_demo` states that nested child regions stay app-composed, and the
  collection proof keeps selection, rename, zoom, and context-menu behavior local.
- Likely: the next child-region follow-on should be behavior-specific, not a broad flag mirror.
  The strongest candidate is `ResizeY` because `repo-ref/imgui/imgui_demo.cpp` uses it repeatedly
  for scrollable lists, baskets, trees, debug routes, and metrics panes.
- Unclear: clipping-return semantics need a Fret-native expression. Fret's declarative element
  construction does not naturally match ImGui's "return false but still call EndChild" grammar, so
  this should not be copied without a concrete performance or diagnostics proof.
- Unclear: nav flattening belongs to focus-scope policy, not to the current child-region chrome
  helper by default. It needs a keyboard-navigation repro before API design.

## Follow-On Threshold

Open a narrow child-region follow-on only when one of these targets is named with a runnable repro
and gate:

1. **Resizable child region**: `ResizeY` first, with persisted or app-owned size state, explicit
   min/max constraints, pointer hit testing, cursor feedback, and a focused smoke test.
2. **Auto-resize child region**: one axis only, with a proof that it does not defeat scroll-region
   intent or create layout feedback loops.
3. **Visibility / clipping budget**: a performance or diagnostics proof where submitting all child
   content is measurably wrong and a Fret-native "visible content gate" can be tested.
4. **Nav-flattened child region**: a keyboard/focus proof where tab/arrow navigation must cross
   parent and child region boundaries as one scope.

## Recommended Next Slice

Keep `child-region depth` below `diagnostics/devtools` and `collection helper` unless a concrete
editor proof asks for one of the behavior targets above.

If it becomes active, start with `imui-child-region-resize-y-v1` rather than a generic
`BeginChild()` parity lane.

Suggested first gate:

```powershell
cargo nextest run -p fret-imui child_region --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke --no-fail-fast
cargo check -p fret-demo --bin workspace_shell_demo
```

## Gate Results

2026-05-06 local results:

- `cargo nextest run -p fret-imui child_region --no-fail-fast` passed: 3 child-region composition
  tests passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke --no-fail-fast`
  passed: 1 public API smoke test passed.
- `cargo check -p fret-demo --bin workspace_shell_demo` passed.
