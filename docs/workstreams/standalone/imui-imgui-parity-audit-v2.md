# imui ↔ Dear ImGui Parity Audit (v2)

Status: current audit snapshot (not an ADR)
Last updated: 2026-04-12

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
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
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

1. No generic immediate multi-select collection controller
   - Dear ImGui now exposes `BeginMultiSelect()` and demonstrates it in large editor-style asset
     views.
   - Fret has multi-selection in domain-specific surfaces (for example node graph logic and some
     recipe-specific selection models), but not yet as one generic `imui` collection primitive.
2. No public immediate child-region/window helper comparable to `BeginChild()`
   - Fret has explicit scroll and virtual-list containers, which is architecturally cleaner.
   - But there is still no single immediate "framed child region with coarse clipping and focus
     scope expectations" helper that matches common editor-pane authoring needs.
3. No immediate tab-bar/menu-bar family in the generic `imui` layer
   - Those behaviors exist elsewhere in the repo (`fret-workspace`, shell demos, shadcn/menu
     surfaces), but the generic immediate layer cannot yet express them as first-class helpers.
4. No immediate shortcut/key-ownership convenience layer comparable to `Shortcut()`,
   `SetNextItemShortcut()`, or `SetItemKeyOwner()`
   - Fret has a stronger app-wide command/keymap architecture.
   - What is missing is the thin immediate authoring seam that ties item-local affordances and
     shortcut hints to that command system without dropping into lower layers.
5. Partial item-status parity
   - `ResponseExt` covers a useful subset of hover, click, drag, context-menu, and nav-highlight
     behavior.
   - It still does not expose the broader Dear ImGui-style status vocabulary around
     activation/deactivation/edit lifecycle and key ownership.

Owner:

- `ecosystem/fret-imui`
- `ecosystem/fret-ui-kit::imui`
- `ecosystem/fret-ui-editor::imui`

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

- multi-select block/controller,
- child-region/pane helper,
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
   multi-select, child-region, and shortcut/key-owner ergonomics.
3. `crates/fret-ui` should resist parity-driven growth unless an ADR-backed mechanism hole is
   proven.
4. The historical v1 parity audit remains useful as archive evidence, but this v2 note is the
   current comparison snapshot for active `imui` / editor-grade parity discussions.
