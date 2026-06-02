# P3 Porting Sugar Readiness - 2026-05-06

Status: narrow SameLine proof promoted; broad porting sugar remains candidate-only
Last updated: 2026-05-31

## Decision

Do not add broad Dear ImGui porting-sugar APIs to `fret-imui` or `fret-ui-kit::imui` from this
source-audit lane.

2026-05-31 follow-up: the existing closure-scoped `ui.same_line(...)` /
`ui.same_line_with_options(...)` helpers are now allowed in first-party teaching surfaces for dense
inline continuation rows. `apps/fret-cookbook/examples/imui_action_basics.rs` now uses
`ui.same_line_with_options(...)` for the payload action button row, backed by the existing
`fret-imui` layout token test. This is not a broad Dear ImGui mutable cursor surface, and
item-width and label-ID helpers remain candidate-only / explicit.

The current Fret proof surfaces already cover the common authoring pressure without needing a
public mirror of Dear ImGui's cursor and label grammar:

- `ecosystem/fret-imui/src/frontend.rs` exposes `row`, `column`, `id`, `push_id`, and keyed
  iteration for the minimal immediate-mode surface.
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` exposes typed container helpers such as
  `horizontal_with_options`, `vertical_with_options`, `grid_with_options`, `table_with_options`,
  `scroll_with_options`, and `child_region_with_options`; split owner modules under
  `ecosystem/fret-ui-kit/src/imui/facade_writer/`, including `container_wrappers.rs`, now own
  focused inherent wrappers without adding Dear ImGui-style mutable cursor sugar.
- `ecosystem/fret-ui-kit/src/imui/options/containers.rs` keeps horizontal layout explicit through
  `HorizontalOptions` (`layout`, `gap`, `justify`, `items`, `wrap`, `test_id`).
- `apps/fret-cookbook/examples/imui_action_basics.rs` uses `ui.same_line_with_options(...)` for a
  small inline command group with a stable row `test_id`.
- `apps/fret-cookbook/examples/imui_editor_controls_basics.rs` and
  `apps/fret-examples/src/imui_editor_proof_demo.rs` use explicit `id_source` / `test_id` fields
  on editor controls instead of label-string suffix parsing.
- `apps/fret-examples/src/imui_editor_proof_demo.rs` uses `PropertyGrid::row_with(...)` for
  editor-style label/value rows, which is the main place Dear ImGui authors would otherwise reach
  for `SameLine()` plus item-width tuning.

That is enough for current editor proofs, but it is not a "drop C++ Dear ImGui code into Fret"
compatibility surface.

## Dear ImGui Reference Axes

The relevant `repo-ref/imgui` behavior axes are:

- `SameLine(offset_from_start_x, spacing)` mutates the current window cursor so the next submitted
  item appears to the right of the previous item.
- `PushItemWidth`, `PopItemWidth`, `SetNextItemWidth`, and `CalcItemWidth` provide stack/next-item
  width defaults for "large item plus label" widgets.
- `PushID` / `PopID` and label suffixes (`##` / `###`) let authors decouple visible labels from
  hashed item identity.
- The Dear ImGui demo uses these together to build dense settings panes, help markers, metrics
  rows, log controls, ID-stack tools, and table-like inline groups.

These APIs are effective in Dear ImGui because it owns a mutable cursor, a per-window ID stack, and
a label parser. Fret's public surface should not copy that grammar unless a Fret-native proof shows
the same authoring tax in at least two places.

## Current Fret Read

- Confident: inline layout is covered for current teaching surfaces without a mutable cursor API.
  Evidence: `ui.row(...)`, `ui.horizontal(...)`, `horizontal_with_options(...)`, and the narrow
  closure-scoped `ui.same_line(...)` / `ui.same_line_with_options(...)` helpers cover the current
  cookbook and proof needs without exposing Dear ImGui's per-window cursor mutation model.
- Confident: editor label/value layout should stay component-owned for now.
  Evidence: `PropertyGrid::row_with(...)` expresses the "label plus control plus optional trailing
  affordance" shape directly, avoiding per-control item-width stack state.
- Confident: label and identity must stay explicit, not string-parsed.
  Evidence: Fret controls expose `id_source`, `test_id`, `a11y_label`, row/test id options, and
  `ui.push_id(...)`; current proofs use those fields repeatedly.
- Confident: `same_line` should stay closure-scoped sugar over Fret layout containers, not a
  free-standing "place the next item here" operation.
- Likely: a future item-width helper, if justified, should be an editor/property-row or control
  sizing option rather than a global stack/next-item API.
- Unclear: whether a migration shim for raw Dear ImGui snippets is worth supporting. No current
  first-party proof is trying to mechanically port a large Dear ImGui demo body into Fret.

## Follow-On Threshold

Open a narrow porting-sugar follow-on only when a repeated authoring tax appears in at least two
first-party proof surfaces with a runnable repro and gate.

Candidate follow-ons, in priority order:

1. **Further inline layout expansion**: only if two proof surfaces need behavior that the existing
   closure-scoped `same_line` helpers cannot express.
2. **Property-row width policy**: only if two editor/property surfaces need the same label width,
   control width, right-alignment, or trailing affordance policy and `PropertyGrid::row_with(...)`
   cannot express it cleanly.
3. **Control sizing preset**: only if multiple controls duplicate the same width/height options in
   app code; keep this as typed options, not an item-width stack.
4. **Label/identity helper**: only for a Fret-native naming helper that preserves explicit
   `label`, `id_source`, `test_id`, and accessibility fields. Do not parse `##` / `###` suffixes in
   public APIs by default.

## Rejection Criteria

Do not open this follow-on for any of these reasons alone:

- a single call site would be shorter with a Dear ImGui `SameLine()` cursor call,
- app code repeats `id_source` and `test_id` once per control,
- one proof has many `PropertyGrid::row_with(...)` calls,
- a Dear ImGui demo snippet would be shorter with `SameLine()`,
- a C++ API name exists upstream but lacks a Fret-native owner, repro, and gate.

## Recommended Next Slice

Keep broad `porting sugar readiness` candidate-only for now. The narrow SameLine teaching-surface
proof is closed by the existing closure-scoped helpers; item-width stacks, next-item width defaults,
and label-suffix identity parsing remain out of scope.

If this becomes active, start with the smallest named pain point:

- `imui-property-row-width-policy-v1` for a proven editor/property-grid sizing tax.

Do not start with a broad `imgui-porting-sugar` crate or a direct mirror of
`SameLine` / `PushItemWidth` / `SetNextItemWidth` / label suffix parsing.

Suggested readiness gates:

```powershell
rg -n "SameLine|PushItemWidth|SetNextItemWidth|CalcItemWidth|PushID|##|###" repo-ref/imgui/imgui.h repo-ref/imgui/imgui.cpp repo-ref/imgui/imgui_demo.cpp
rg -n "row\\(|horizontal\\(|horizontal_with_options|row_with|id_source|test_id|push_id" ecosystem/fret-imui/src/frontend.rs ecosystem/fret-ui-kit/src/imui/facade_writer.rs ecosystem/fret-ui-kit/src/imui/facade_writer ecosystem/fret-ui-kit/src/imui/options/containers.rs apps/fret-cookbook/examples/imui_action_basics.rs apps/fret-cookbook/examples/imui_editor_controls_basics.rs apps/fret-examples/src/imui_editor_proof_demo.rs
cargo check -p fret-demo --bin imui_editor_proof_demo
```

## Gate Results

2026-05-06 local results:

- `rg -n "SameLine|PushItemWidth|SetNextItemWidth|CalcItemWidth|PushID|##|###" repo-ref/imgui/imgui.h repo-ref/imgui/imgui.cpp repo-ref/imgui/imgui_demo.cpp`
  passed and confirmed the relevant Dear ImGui cursor, width, and identity axes.
- `rg -n "row\\(|horizontal\\(|horizontal_with_options|row_with|id_source|test_id|push_id" ecosystem/fret-imui/src/frontend.rs ecosystem/fret-ui-kit/src/imui/facade_writer.rs ecosystem/fret-ui-kit/src/imui/facade_writer ecosystem/fret-ui-kit/src/imui/options/containers.rs apps/fret-cookbook/examples/imui_action_basics.rs apps/fret-cookbook/examples/imui_editor_controls_basics.rs apps/fret-examples/src/imui_editor_proof_demo.rs`
  passed and confirmed the current Fret authoring/proof anchors across the root facade file and
  split owner modules.
- `cargo check -p fret-demo --bin imui_editor_proof_demo` passed.
