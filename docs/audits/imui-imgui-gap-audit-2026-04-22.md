# IMUI vs Dear ImGui Gap Audit — 2026-04-22

Status: Snapshot audit (current-code evidence). This note is intended to complement, and in a few
places correct, older pre-reset parity notes.

Local Dear ImGui snapshot used for reference: `repo-ref/imgui` @ `d7b40ab9a`

## Scope

- Compare the current Fret immediate-mode lane against the local Dear ImGui snapshot.
- Focus on current code, current first-party teaching surfaces, and current executable proofs.
- Exclude:
  - the compatibility-only retained bridge lane (`imui_node_graph_demo`),
  - user-owned in-progress menu/tab work,
  - platform-specific multi-viewport validation that cannot currently be exercised on this machine.

## Inputs Reviewed

- `ecosystem/fret-imui/src/lib.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/floating_options.rs`
- `ecosystem/fret-ui-kit/src/imui/response/hover.rs`
- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-imui/src/tests/{floating.rs,interaction.rs,popup_hover.rs,models.rs}`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-examples/src/{imui_hello_demo.rs,imui_response_signals_demo.rs,imui_shadcn_adapter_demo.rs,imui_floating_windows_demo.rs,imui_editor_proof_demo.rs,workspace_shell_demo.rs,imui_node_graph_demo.rs}`
- `docs/examples/README.md`
- `docs/workstreams/imui-ecosystem-facade-v3/imui-ecosystem-facade-v3.md`
- `docs/workstreams/imui-compat-retained-surface-v1/BASELINE_AUDIT_2026-03-31.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui.cpp`

## Findings

### 1. The current layer split is correct and should not be "fearlessly refactored" away

`fret-imui` is already the thin frontend that Fret needs: it exports the immediate-mode mounting
primitives (`imui`, `imui_in`, `imui_build`, `imui_raw`, etc.) and intentionally keeps policy out
of the crate.

The imgui-like surface is intentionally hosted in `fret-ui-kit::imui`, while the app-facing root
lane is `fret::imui::{prelude::*, kit, editor, docking}`.

Conclusion:

- Do not fatten `fret-imui` into a widget/policy crate.
- Do not move interaction policy from `fret-ui-kit::imui` into `crates/*` or `fret-imui`.

Evidence anchors:

- `ecosystem/fret-imui/src/lib.rs`
- `ecosystem/fret/src/lib.rs`
- `docs/workstreams/imui-ecosystem-facade-v3/imui-ecosystem-facade-v3.md`

### 2. Current Fret IMUI parity is materially broader than some older parity notes still imply

The current code already covers a meaningful Dear ImGui subset:

- floating window behavior knobs, including explicit analogs for `NoBringToFrontOnFocus` and
  `NoInputs`,
- hovered query flags and nav-aware `hovered_like_imgui()`,
- `disabled_scope(...)`,
- the ImGui-aligned drag threshold default (`6px`),
- immediate wrappers for menus, tab bars, tables, virtual lists, combos, text input, and tooltips,
- typed drag-source / drop-target payload seams,
- keyed identity helpers (`ui.id(...)`, `ui.push_id(...)`, `for_each_keyed(...)`),
- multi-select helpers and editor-grade proof surfaces.

This means the current gap is no longer "basic immediate widgets are missing". Older notes that
still say "no immediate tables/tab bars/drag-and-drop API yet" are stale relative to the current
tree.

Evidence anchors:

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/floating_options.rs`
- `ecosystem/fret-ui-kit/src/imui/response/hover.rs`
- `ecosystem/fret-imui/src/tests/floating.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`

### 3. ID ergonomics are not the same as ID capability

Dear ImGui exposes `PushID()` / `GetID()` plus label suffix conventions such as `"##"` and
`"###"`.

Fret does not mirror the label-suffix parsing model, but it does already provide the core identity
capability through explicit keyed scopes:

- `ui.id(...)`
- `ui.push_id(...)`
- `ui.for_each_keyed(...)`
- `ui.for_each_unkeyed(...)` as an explicit opt-in for static-order collections

So the real gap is ergonomic sugar for ports from raw ImGui code, not missing stable identity
mechanics.

Conclusion:

- Do not invent a second hashing / ID runtime.
- If friction remains high for ImGui ports, add narrow sugar on top of the existing keyed story
  instead of copying the raw label-suffix model into the whole lane.

Evidence anchors:

- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `docs/examples/README.md`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui.cpp`

### 4. The biggest remaining parity gaps are now deeper than buttons, sliders, or menus

#### 4.1 Input-text parity is still shallow

Dear ImGui exposes a wide `ImGuiInputTextFlags_*` family (`ReadOnly`, `Password`,
`AutoSelectAll`, `NoUndoRedo`, completion/history callbacks, `AllowTabInput`, multiline-specific
flags, etc.).

Current `InputTextOptions` is intentionally small: `enabled`, `focusable`, accessibility labels,
placeholder, `submit_command`, and `cancel_command`.

Conclusion:

- The next serious "imgui-level editor UX" gap is text editing policy, not generic button chrome.
- This should be solved in `fret-ui-kit::imui` and `fret-ui-editor`, not by bloating `fret-imui`.

Evidence anchors:

- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `repo-ref/imgui/imgui.h`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

#### 4.2 There is still no immediate style-stack lane, and that is mostly the right decision

Older parity notes already framed style-stack APIs (`PushStyleVar`, `PushStyleColor`,
`SetNextItemWidth`, historical `same_line`) as intentionally not mirrored by the current lane.

That remains the correct default posture: Fret expresses visual policy through tokens, explicit
layout, and recipe/component layers rather than a parallel immediate styling runtime.

Conclusion:

- Do not reopen a generic style-stack API on `fret-imui`.
- If a repeated editor/tooling use case emerges, solve it with narrow policy helpers on
  `fret::imui::kit` or `fret-ui-editor`, not a global push/pop styling world.

Evidence anchors:

- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/DESIGN.md`

#### 4.3 Immediate draw-list parity is still absent from the IMUI lane

The repo has explicit draw-list concepts, but they live in specialized domains such as gizmos and
renderer overlays, not in a generic IMUI `DrawList` surface.

Conclusion:

- If Fret needs imgui-like debug overlays / custom immediate drawing, add a dedicated ecosystem
  adapter or debug-draw lane.
- Do not overload `fret-imui` with a generic drawing API just because Dear ImGui has one.

Evidence anchors:

- `ecosystem/fret-gizmo/src/gizmo/types.rs`
- `apps/fret-examples/src/gizmo3d_demo.rs`
- `docs/audits/gizmo-imguizmo-transform-gizmo-alignment.md`

#### 4.4 Multi-viewport / OS-window parity remains the hardest unresolved area

Fret already has meaningful in-window floating and docking proofs:

- `imui_floating_windows_demo`
- `imui_editor_proof_demo`
- `workspace_shell_demo`

But Dear ImGui's multi-viewport behavior goes further into OS-window lifecycle, viewport flags, and
backend cooperation. The current repo still treats that as a dedicated docking / multi-window lane,
not a solved general IMUI claim.

Conclusion:

- Keep this work docking-owned.
- Do not try to "finish imgui parity" by adding windowing policy into `fret-imui`.

Evidence anchors:

- `apps/fret-examples/src/imui_floating_windows_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `docs/workstreams/imui-ecosystem-facade-v3/imui-ecosystem-facade-v3.md`
- `repo-ref/imgui/imgui.cpp`

### 5. Test architecture is now a bigger refactor hazard than missing public API

`fret-imui` itself is tiny, but its verification surface is concentrated in very large test files:

- `src/tests/interaction.rs`
- `src/tests/models.rs`
- `src/tests/floating.rs`
- `src/tests/popup_hover.rs`

This is now one of the main reasons IMUI refactors stay risky: behavior coverage exists, but it is
expensive to navigate and review.

Conclusion:

- The next fearless refactor should prioritize test decomposition, not more top-level helper growth.
- A fixture-driven split by capability family would improve reviewability without widening contracts.

Evidence anchors:

- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-imui/src/tests/models.rs`
- `ecosystem/fret-imui/src/tests/floating.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`
- `tools/audit_crate.py --crate fret-imui`

### 6. First-party teaching surfaces are finally mostly aligned; compatibility exceptions should stay explicit

The first-party immediate-mode story now teaches the root `fret::imui` lane across cookbook/examples,
while `imui_node_graph_demo` remains the explicit compatibility-only retained-bridge exception.

This is a good place to stop API churn and use the examples as regression anchors instead of
continuing to move teaching surfaces around.

Evidence anchors:

- `docs/examples/README.md`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/imui_node_graph_demo.rs`

## What Not To Refactor Next

- Do not turn `fret-imui` into a fat widget crate.
- Do not move menu/tab/hover/floating policy down into `crates/*`.
- Do not reopen retained-bridge authoring as a normal first-party IMUI lane.
- Do not copy Dear ImGui's style-stack model into Fret just for API familiarity.
- Do not add a second identity/hash story when `ui.id(...)` / `ui.push_id(...)` already exists.

## Recommended Next Steps

1. Split the `fret-imui` mega-tests by capability family.
   - Outcome: safer refactors for hover/floating/menu/tab/text/drag lanes.
   - Likely tool: fixture-driven harnesses for repetitive response/state matrices.

2. Open an input-text parity lane in `fret-ui-kit::imui` plus `fret-ui-editor`.
   - Outcome: close the biggest remaining "editor-grade imgui feel" gap without bloating the
     frontend.
   - Focus: read-only/password/auto-select-all/undo policy/history/completion/multiline behavior.

3. Treat menu/tab depth as an explicit `fret-ui-kit::imui` policy lane.
   - Outcome: finish the difficult part of IMUI parity in the correct layer.
   - Constraint: do not route this through `fret-imui` or `crates/fret-ui`.

4. Refresh older parity notes after this audit, not before.
   - Outcome: prevent stale documents from driving the wrong refactor.
   - Specifically: archived notes that still claim immediate tables/tab bars/drag-drop are absent
     should be marked historical or updated with current evidence.

5. Decide separately whether Fret wants a dedicated immediate debug-draw adapter.
   - Outcome: either a clear non-goal or a narrow ecosystem lane for custom draw-list-like tools.
   - Constraint: keep it out of the minimal frontend unless the adapter story is proven.

## Bottom Line

Fret IMUI is no longer blocked by "missing the obvious imgui basics". The architecture is mostly in
the right place already:

- `fret-imui` is thin,
- `fret-ui-kit::imui` owns policy,
- first-party teaching surfaces now route through `fret::imui`,
- and the remaining parity work is concentrated in deeper policy and editor-grade UX.

If the goal is "reach imgui-level usefulness", the next wins are:

- text/input parity,
- menu/tab depth,
- better test architecture,
- and a deliberate decision on whether immediate debug-draw belongs in the ecosystem.

Not the right next move:

- another broad IMUI surface reset,
- or making `fret-imui` itself a large, stateful widget framework.
