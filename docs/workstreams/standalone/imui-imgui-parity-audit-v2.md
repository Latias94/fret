# imui ↔ Dear ImGui Parity Audit (v2)

Status: current audit snapshot (not an ADR)
Last updated: 2026-04-22

## Purpose

This note refreshes the older parity audit after:

- the `imui-stack-fearless-refactor-v2` closeout,
- the `imui-editor-grade-product-closure-v1` P3 runner/package freeze,
- and the latest multi-window docking parity fixes and diagnostics hardening on 2026-04-12.

It answers a narrower question than the historical v1 note:

> what is still materially missing before Fret feels "Imgui-class" for editor-grade work, and
> which of those gaps are actually runtime gaps versus runner/shell/ecosystem gaps?

The goal is not API cloning.
The goal is to avoid the wrong refactor:

- do not widen `crates/fret-ui` when the real problem is runner/backend follow behavior,
- do not reopen generic `imui` helper growth when the real problem is workbench shell composition,
- and do not mistake explicit Fret design choices for accidental parity regressions.

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Dear ImGui: https://github.com/ocornut/imgui

Local reference snapshot used for this audit:

- `repo-ref/imgui` at `2dc64f99b`

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

## Scope

Compared surfaces:

1. Immediate authoring facade and interaction result surface.
2. Editor/workbench product surface (tabs, panes, menus, inspectors, shell composition).
3. Multi-window runner/backend behavior (hover-behind, follow, transparent payload, mixed-DPI).
4. Diagnostics/devtools ergonomics for closing parity gaps safely.

Out of scope:

- recreating Dear ImGui's exact API grammar,
- treating every missing `ImGuiWindowFlags_*` bit as a required Fret surface,
- moving docking/viewports policy into `imui`,
- or using this note to justify a second runtime beside `fret-ui`.

## Current evidence set

Fret implementation and planning anchors:

- `docs/architecture.md`
- `docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
- `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/DESIGN.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_TAB_OWNER_VERDICT_2026-04-22.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-ui-kit/src/primitives/menu/sub_trigger.rs`
- `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/docking_arbitration_demo.rs`

Dear ImGui anchors:

- `repo-ref/imgui/docs/BACKENDS.md`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui.cpp`
- `repo-ref/imgui/imgui_tables.cpp`
- `repo-ref/imgui/imgui_demo.cpp`

Recent launched proof snapshot (2026-04-12):

- `fix(diag): bind transparent payload pointer moves to last seen window` (`d8f2b2848`)
- `fix(docking): keep pending drags through left-window cancels` (`d21fa656e`)
- `fix(diag): harden docking arbitration suite inputs and gates` (`720bbfcaf`)
- `target/fret-diag/imui-p3-multiwindow-parity-after-pending-fix/campaigns/imui-p3-multiwindow-parity/1776006065940/campaign.result.json`
- `target/fret-diag/docking-arbitration-after-pending-fix-full-v3/suite.summary.json`

## Executive read

The most important conclusion from this refresh is:

Fret is **not** primarily blocked by missing immediate-mode runtime mechanics.

The remaining gap to "Imgui-class editor feel" is dominated by:

1. runner/backend multi-window reliability,
2. shell/workbench composition and product proof surfaces,
3. a small set of ecosystem-level immediate authoring conveniences for editor collections.

The wrong move would be a large runtime rewrite to imitate Dear ImGui's internals.
The current declarative-on-retained-substrate architecture is good enough to reach the target, as
long as the remaining work stays in the correct owners.

## Gap matrix

### 1) Runtime / mechanism substrate

Status: aligned enough; not the primary blocker

Why:

- Fret already has a documented mechanism/policy split that Dear ImGui does not attempt to
  formalize: `crates/fret-ui` stays mechanism-only while interaction policy lives in ecosystem
  crates.
- Stable identity, overlays, docking ops, diagnostics, and cross-window drag sessions are all
  treated as explicit contracts.
- The immediate frontend compiles down to the declarative element tree instead of creating a
  parallel runtime.

Implication:

- Do not chase Dear ImGui by collapsing back into a monolithic "everything in one context" stack.
- If a parity gap appears painful, assume runner/shell/policy first and only widen runtime
  contracts when an ADR-backed proof says the mechanism is insufficient.

### 2) Multi-window runner/backend hand-feel

Status: partial, but closing fast

What is now materially aligned:

- overlapped-window hover selection is tracked as runner-owned,
- moving payload windows can "peek behind" during follow,
- transparent payload routing has explicit diagnostics coverage,
- `PointerCancel(LeftWindow)` no longer drops pending dock drags too early,
- and the current named `docking-arbitration` suite can run with platform-split manifests instead
  of accidentally pulling Windows-only gates into non-Windows runs.

Why this is still the top remaining gap:

- Dear ImGui's multi-viewport hand-feel is primarily a backend/platform responsibility
  (`ImGuiBackendFlags_HasMouseHoveredViewport`, `AddMouseViewportEvent()`,
  `ImGuiViewportFlags_NoInputs`).
- Fret now has the same owner split in principle, but platform closure is not uniformly complete.
- The remaining fragility is about cross-window follow behavior, overlap resolution, release/cancel
  paths, z-order posture, and mixed-DPI transitions, not about missing button/hover widgets.

Still open:

- mixed-DPI follow across real monitor boundaries is still only partially locked,
- Linux/Wayland remains a deliberate degradation story rather than full parity,
- window-decoration and initial placement polish remain backend-specific,
- and launched proof must keep exercising the exact overlap/release/cancel paths that recently
  regressed.

Owner:

- `crates/fret-launch`
- backend integrations
- `ecosystem/fret-docking`

Not the owner:

- `crates/fret-ui`
- generic `fret-imui` / `fret-ui-kit::imui` helper growth

### 3) Immediate authoring surface breadth

Status: better than the old docs implied, but still below Dear ImGui's editor convenience layer

Already present in Fret's current immediate stack:

- explicit identity scopes (`id`, `push_id`, `for_each_keyed`, `for_each_unkeyed`)
- horizontal / vertical / grid / scroll containers
- table and virtual-list helpers
- buttons, menu items, selectables, combos, tree nodes, collapsing headers
- popup menu / popup modal / tooltip helpers
- floating areas and floating windows
- typed drag source / drop target helpers
- immediate editor adapters for the main `fret-ui-editor` control set

The real remaining gaps are narrower:

1. First-cut generic immediate multi-select collection primitive now exists, and the current
   first-party collection proof is real, but the breadth is still narrower than Dear ImGui
   - `fret-ui-kit::imui` now exposes a reusable `ImUiMultiSelectState<K>` model plus
     model-backed `multi_selectable[_with_options]` helpers with plain click, primary-modifier
     toggle, and shift-range selection semantics.
   - `apps/fret-examples/src/imui_editor_proof_demo.rs` and the closed
     `docs/workstreams/imui-collection-pane-proof-v1/` lane now freeze the current asset-browser /
     file-browser style proof, so the remaining gap is breadth rather than first-party proof
     absence.
   - What still remains is Dear ImGui-class collection depth: no marquee/box-select bridge, no
     lasso/drag-rectangle story, and no richer keyboard-owner story around the collection helper.
2. First-cut immediate child-region helper now exists, and the current first-party pane proof is
   real, but the helper depth is still intentionally narrow
   - `fret-ui-kit::imui` now exposes a keyed `child_region[_with_options]` helper that wraps a
     framed scroll surface with default vertical item flow and coarse clipping.
   - `apps/fret-examples/src/workspace_shell_demo.rs`, `apps/fret-examples/src/editor_notes_demo.rs`,
     and the closed `docs/workstreams/imui-collection-pane-proof-v1/` lane now freeze the current
     pane-first proof that exercises nested toolbar / tabs / inspector / status composition.
   - The remaining gap is depth rather than ownership: there is still no `BeginChild()`-scale
     child-flag surface, no richer menu-bar-in-child story, and no Dear ImGui-style axis-specific
     resize / auto-resize behavior on the generic helper surface.
3. First-cut immediate menu/tab family now includes click-open menus, top-level menubar
   hover-switch, keyboard-open on `ArrowDown` / `ArrowUp`, open-menu left/right switching,
   submenu hover-open / sibling hover-switch with a basic grace corridor, and a thin tab-bar seam,
   but richer depth is still open
   - `fret-ui-kit::imui` now exposes a small `menu_bar[_with_options]` container plus
     `begin_menu[_with_options]` and `begin_submenu[_with_options]` trigger/helper seams for
     click-open top-level and nested menus, alongside `tab_bar[_with_options]` +
     `begin_tab_item[_with_options]` for simple immediate tab selection and panel switching.
   - The current generic floor is materially better than the older audit snapshot implied:
     `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
     now records shipped top-level menubar hover-switch plus submenu hover-open / sibling
     hover-switch with a basic grace corridor in generic IMUI.
   - The remaining gap is now narrower than the older audit implied:
     richer submenu-intent tuning and reverse-direction top-level focus arbitration remain open in
     generic IMUI, while outer-scope active-menubar mnemonic / roving posture now has a stronger
     shell-first owner in `fret::in_window_menubar`.
   - Richer tab-bar policy now has a stronger first-party owner elsewhere:
     `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_TAB_OWNER_VERDICT_2026-04-22.md` keeps
     Dear ImGui-like overflow / reorder / close / action-tab behavior in
     `fret-workspace::WorkspaceTabStrip`, so generic IMUI should not grow that surface by default.
   - This gap now has a dedicated narrow owner:
     `docs/workstreams/imui-menu-tab-policy-depth-v1/`.
4. First-cut immediate command metadata seam now exists, and the current lane explicitly closed on
   a no-new-surface verdict for broader key-owner APIs
   - `fret-ui-kit::imui` now exposes `menu_item_command[_with_options]` and
     `button_command[_with_options]`, which resolve command title, enabled state, and menu-item
     shortcut hints from Fret's command/keymap layer without widening `crates/fret-ui`.
   - `ButtonOptions`, `SelectableOptions`, `CheckboxOptions`, `SwitchOptions`,
     `CollapsingHeaderOptions`, `TreeNodeOptions`, `TabItemOptions`, `MenuItemOptions`,
     `BeginMenuOptions`, `BeginSubmenuOptions`, `ComboOptions`, and `ComboModelOptions` also
     expose a focused `activate_shortcut` seam for exact item-local `KeyChord` activation without
     widening the runtime's global shortcut ownership model.
   - Representative `fret-imui` proof now also locks `shortcut_repeat` as opt-in on focused direct
     pressables, popup items, menu/menu-submenu triggers, and combo/combo-model triggers, so held
     chords do not retrigger by default.
   - The closed key-owner lane now freezes the current verdict:
     do not add a generic `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale facade yet.
   - The remaining gap is therefore no longer "missing admission." It is "missing stronger first-
     party proof": there is still no broader item-local shortcut registration seam beyond focused
     button/selectable/checkbox/switch/disclosure/tab/menu/combo pressables, and no evidence yet
     that generic IMUI, rather than product/shell owners, needs a broader key-owner contract.
5. Partial item-status parity
   - `ResponseExt` now covers hover, click, drag, context-menu, nav-highlight, and a first
     lifecycle slice around activation/deactivation/edit sequencing for direct pressables, menu
     items, boolean controls, slider, input text, textarea, combo, and combo-model helpers.
   - The remaining gap is no longer basic lifecycle presence. It is breadth and owner separation:
     broader Dear ImGui-style status vocabulary, richer menu/tab policy, and any future key-owner
     surface still live in separate narrow lanes.

Owner:

- `ecosystem/fret-imui`
- `ecosystem/fret-ui-kit::imui`
- `ecosystem/fret-ui-editor::imui`
- `ecosystem/fret-ui-kit/src/imui/multi_select.rs`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-ui-kit/src/command.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`
- `ecosystem/fret-imui/src/tests/composition.rs`

Not the owner:

- `crates/fret-ui`

### 4) Porting ergonomics

Status: intentional divergence, but still a real cost center

Dear ImGui optimizes for:

- cursor-based layout flow,
- `SameLine()` / item-width nudges,
- `PushID()` plus `##` / `###` naming tricks,
- and window-scoped local authoring convenience.

Fret intentionally prefers:

- explicit layout containers,
- explicit keyed identity,
- and a smaller shared authoring contract.

Architecturally, Fret's choice is correct.
Productively, it means direct Dear ImGui ports still require more rewrites than they should for
some editor surfaces.

The right response is **not** to move cursor-layout state into `crates/fret-ui`.
If proof surfaces show a real need, add thin compatibility helpers at the facade level:

- optional `same_line`-style sugar,
- optional label/ID translation helpers,
- optional item-width convenience wrappers,
- optional "child region" sugar over explicit containers.

These should live in `fret-imui` or `fret-ui-kit::imui`, stay opt-in, and be justified by
real first-party proof surfaces instead of hypothetical ports.

### 5) Workbench-shell composition

Status: the biggest practical product gap

Dear ImGui feels complete not because every API is clever, but because the demo and common editor
patterns compose:

- menu bars,
- dockspace + document tabs,
- left rails / trees / inspectors,
- rename/save/popups,
- multi-select collections,
- and persistent workbench panes.

Fret now has the right ownership split for those pieces:

- `fret-workspace` for shell slots and workbench rails,
- `fret-docking` for docking choreography,
- `fret-ui-editor` for editor controls/composites,
- examples and recipes for app-owned center content.

What is still missing is one stronger shell-first golden path where those pieces feel like one
system instead of adjacent proofs.

Today the repo already has:

- `workspace_shell_demo`
- `editor_notes_demo`
- `docking_arbitration_demo`
- `imui_editor_proof_demo`

But the user-facing "this is the default editor-grade composition story" is still more fragmented
than Dear ImGui's workbench feel.

This is not a runtime gap.
It is a product/shell integration gap.

### 6) Diagnostics / debug loop

Status: stronger than Dear ImGui in reproducibility, weaker in ambient ubiquity

Fret is already ahead on:

- scripted diagnostics,
- bundles,
- campaigns,
- regression summaries,
- launched proof artifacts,
- and shareable evidence for CI/review.

Dear ImGui is still ahead on:

- "always available" discoverability through `ShowDemoWindow()` / Metrics / Debugger culture,
- immediate visual inspection as the first reflex from inside the app.

Fret's P2 diagnostics work improved the first-open path significantly, but the day-to-day
"always-open metrics/demo" habit is still less universal than in Dear ImGui-based tools.

This should be solved through first-party devtools and example discoverability, not through runtime
contract churn.

## What is not actually missing

These items often look like parity gaps at first glance, but should not automatically trigger
refactors:

1. Full `ImGuiWindowFlags_*` mirroring
   - many Dear ImGui window flags are really theme, shell, or persistence policy in Fret.
2. A second immediate runtime
   - Fret's current immediate frontend over the declarative runtime is the right architecture.
3. Generic docking APIs in `imui`
   - docking/viewports remain runner + docking owned.
4. Reverting to label-suffix identity tricks by default
   - explicit keyed identity is a net improvement, even if opt-in compatibility sugar may be worth
     adding.
5. Expanding `fret-authoring::Response`
   - keep the shared contract minimal; richer status should remain facade-level.

## Recommended refactor order

### R1) Finish runner-owned multi-window closure first

Why first:

- it is the highest hand-feel leverage,
- it already caused real regressions,
- and recent fixes proved the remaining bugs are in cancel/release/routing choreography, not in
  generic widget APIs.

Do next:

- keep `imui-p3-multiwindow-parity` as the bounded launched proof package,
- keep `docking-arbitration` as the broader regression suite,
- finish mixed-DPI real-device/manual acceptance,
- and continue narrowing platform-specific follow-on work inside the docking parity lane.

### R2) Add a narrow ecosystem-level immediate collection package

Focus on the missing editor collection conveniences:

- shortcut/key-owner convenience seam,
- optional immediate tab/menu-bar helpers only if shell proofs justify them.

Constraint:

- no runtime widening unless a proof shows the mechanism is truly missing.

### R3) Strengthen the shell-first editor proof

Treat this as the main product maturity lane:

- `workspace_shell_demo` should stay the primary workbench proof,
- `editor_notes_demo` should remain the bounded shell-mounted editor-rail proof,
- `docking_arbitration_demo` should remain the multi-window stress surface,
- and the repo should keep converging those proofs into one clearer editor-grade composition story.

### R4) Only then consider opt-in porting sugar

If first-party proofs keep paying a tax for explicit layout/identity translation, add thin
compatibility helpers in the immediate facade.

Do not add them preemptively.
Do not add them to `crates/fret-ui`.

## Decision

From this audit forward:

1. The top remaining gap to Dear ImGui-grade editor feel is runner/backend and shell closure, not
   runtime architecture.
2. The next generic immediate work should be narrow:
   shortcut/key-owner ergonomics, optional tab/menu helpers, and stronger first-party pane proofs.
3. `crates/fret-ui` should resist parity-driven growth unless an ADR-backed mechanism hole is
   proven.
4. The historical v1 parity audit remains useful as archive evidence, but this v2 note is the
   current comparison snapshot for active `imui` / editor-grade parity discussions.
