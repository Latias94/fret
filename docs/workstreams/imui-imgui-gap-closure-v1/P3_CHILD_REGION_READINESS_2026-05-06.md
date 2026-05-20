# P3 Child Region Readiness - 2026-05-06

Status: readiness audit; partially superseded by closed resize follow-ons
Last updated: 2026-05-20

Status note (2026-05-16): this audit remains the child-region gap map, but the manual resize
targets have since landed in `imui-child-region-resize-y-v1` and
`imui-child-region-resize-x-v1`. Read older references to "`ResizeY` first" as historical
sequencing, not current TODO state.

Status note (2026-05-18): a focused `fret-imui` composition gate now proves the Fret-native
AutoResizeY-equivalent posture: a `child_region` with width but no explicit height auto-sizes to
its measured content and pushes following siblings down. That evidence does not add an
`AutoResizeY` flag mirror; explicit fixed-height scroll regions and manual resize remain the
bounded-scroll paths.

Status note (2026-05-20): the same Fret-native posture is now locked for the width axis. A
`child_region` with height but no explicit width auto-sizes to measured content, keeps the
unbounded viewport auto-width, and pushes following siblings to the right. This is layout evidence,
not an `AutoResizeX` flag mirror.

## Decision

Do not widen `fret-ui-kit::imui` child-region public API from this source-audit lane. Use narrow
proof-led follow-ons for concrete behavior targets.

Current Fret proof surfaces already validate a useful child-region seam:

- `ecosystem/fret-ui-kit/src/imui/child_region.rs` builds a keyed scroll-area-backed region.
- `ecosystem/fret-ui-kit/src/imui/options/containers.rs` exposes chrome, layout, scroll handles,
  scroll axes, scrollbar visibility, diagnostic ids, and optional axis-specific manual resize.
- `ecosystem/fret-imui/src/tests/composition.rs` verifies stacked content, forwarded scroll handle
  options, nested menu/popup hosting, and framed versus bare chrome.
- `ecosystem/fret-ui-kit/tests/imui_child_region_smoke.rs` verifies the public resize option and
  response surfaces.
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
- Confident: the manual resize follow-ons should stay closed after landing axis-specific
  `ResizeY` and `ResizeX` helpers.
  Evidence: `docs/workstreams/imui-child-region-resize-y-v1/CLOSEOUT_AUDIT_2026-05-15.md` and
  `docs/workstreams/imui-child-region-resize-x-v1/CLOSEOUT_AUDIT_2026-05-16.md`.
- Confident: the Fret-native AutoResizeY-equivalent behavior is already represented by leaving the
  child-region height unconstrained. Evidence:
  `child_region_without_height_constraint_auto_sizes_to_content` proves a width-constrained,
  height-auto child region contains its measured content and pushes the next sibling below it.
  Keep this as layout evidence, not as a public `AutoResizeY` flag mirror.
- Confident: the Fret-native AutoResizeX-equivalent behavior is already represented by leaving the
  child-region width unconstrained. Evidence:
  `child_region_without_width_constraint_auto_sizes_to_content` proves a height-constrained,
  width-auto child region contains its measured content, keeps its viewport auto-width, and pushes
  the next sibling to the right. Keep this as layout evidence, not as a public `AutoResizeX` flag
  mirror.
- Unclear: clipping-return semantics need a Fret-native expression. Fret's declarative element
  construction does not naturally match ImGui's "return false but still call EndChild" grammar, so
  this should not be copied without a concrete performance or diagnostics proof.
- Unclear: nav flattening belongs to focus-scope policy, not to the current child-region chrome
  helper by default. It needs a keyboard-navigation repro before API design.

## Follow-On Threshold

Open a narrow child-region follow-on only when one of these targets is named with a runnable repro
and gate:

1. **Auto-resize child region**: only reopen if a proof needs behavior beyond the current
   unconstrained-axis auto-size layout gates, such as always-auto-resize feedback-loop control.
2. **Visibility / clipping budget**: a performance or diagnostics proof where submitting all child
   content is measurably wrong and a Fret-native "visible content gate" can be tested.
3. **Nav-flattened child region**: a keyboard/focus proof where tab/arrow navigation must cross
   parent and child region boundaries as one scope.

## Recommended Next Slice

Keep `child-region depth` below `diagnostics/devtools` and `collection helper` unless a concrete
editor proof asks for one of the behavior targets above.

Manual resize is already covered by `imui-child-region-resize-y-v1` and
`imui-child-region-resize-x-v1`, and the current unconstrained-axis child-region path covers the
basic AutoResizeY-equivalent and AutoResizeX-equivalent layout postures. Future work should start
with a named visibility gate, nav-flattening follow-on, or a more specific auto-resize behavior
than unconstrained-axis layout rather than a generic `BeginChild()` parity lane.

Suggested first gate:

```powershell
cargo nextest run -p fret-imui child_region --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui child_region_resize --no-fail-fast
cargo check -p fret-demo --bin workspace_shell_demo
```

## Gate Results

2026-05-06 local results:

- `cargo nextest run -p fret-imui child_region --no-fail-fast` passed: 3 child-region composition
  tests passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke --no-fail-fast`
  passed: 1 public API smoke test passed.
- `cargo check -p fret-demo --bin workspace_shell_demo` passed.

2026-05-16 resize follow-on results:

- `imui-child-region-resize-y-v1` closed with vertical resize policy and app-owned height state.
- `imui-child-region-resize-x-v1` closed with horizontal resize policy and app-owned width state.

2026-05-18 auto-height composition result:

- `cargo nextest run -p fret-imui child_region_without_height_constraint_auto_sizes_to_content
  --no-fail-fast` passed. This proves that a child region with explicit width and no explicit height
  auto-sizes to measured content, keeps the unbounded viewport auto-height, and pushes following
  siblings below the measured region.

2026-05-20 auto-width composition result:

- `cargo nextest run -p fret-imui child_region_without_width_constraint_auto_sizes_to_content
  --no-fail-fast` passed. This proves that a child region with explicit height and no explicit
  width auto-sizes to measured content, keeps the unbounded viewport auto-width, and pushes
  following siblings to the right.
