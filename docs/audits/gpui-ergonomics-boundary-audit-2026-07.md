# GPUI Ergonomics and Boundary Audit (2026-07)

This audit compares Fret against the pinned Zed/GPUI reference under `repo-ref/zed` from the
perspective of a general-purpose Rust GUI framework. It is an audit note, not an ADR.

> Implementation status (2026-07-11): this document is the pre-refactor evidence baseline. The
> gaps below were executed through
> `docs/plans/2026-07-09-002-refactor-gpui-ergonomics-boundary-plan.md`. The current ordinary path
> uses `fret::workspace::WorkspaceApp`, typed workspace actions, `DataTableRecipe<T>`, and
> `InspectorTextFieldBinding`. The workspace probe also records the app-facing frame-stage harness
> through a stable diagnostics selector; residual raw workspace-demo seams remain explicitly
> classified as advanced app-specific policy internals. The suite definitions split expected
> coverage between `ui-gallery-workspace-shell` shared chrome and the real-app
> `workspace-shell-app-facing` path.
>
> U9 execution evidence (2026-07-12): `ui-gallery-workspace-shell` passed 4/4 in session
> `1783801365683-65846` under `target/fret-diag/u9-ui-gallery-workspace-shell-final-20260712` (the
> model-command proof is run `1783801375847`). This is a direct custom-driver
> `WorkspaceWorkbench` owner/trace gate for shared Gallery chrome; it does not substitute for a
> real `WorkspaceApp`. The required
> `workspace-shell-app-facing` suite passed 10/10 in session `1783801123285-47715` under
> `target/fret-diag/u9-workspace-shell-app-facing-final-rerun-20260712` (final run
> `1783801150929`). Both suite
> summaries have empty failure reason-code counts.

## Verdict

Fret's core architecture is sound for a general GUI framework:

- `fret-core` / `fret-runtime` / `fret-ui` / backend crates keep the major dependency direction
  clean.
- `fret::app::prelude::*`, `FretApp`, `View`, `AppUi`, `LocalState`, and typed actions form a
  coherent first-contact Interface.
- `fret-ui` is a real deep Module: event routing, focus, layout, paint, overlays, text, and
  virtualization sit behind a mechanism layer rather than leaking backend details.

The main ergonomic gap is not lack of capability. The gap is that realistic editor-grade examples
still cross too many advanced seams:

- `FnDriver`, `UiTree`, `RenderRootContext`, manual `UiFrameCx` staging, and diagnostics loops show
  up in app-shaped demos.
- Workspace commands often fall back to string `CommandId` dispatch rather than an app-facing
  typed/action-first command surface.
- Shared/controlled state in richer inspector workflows still makes app code touch `ModelStore`
  directly.

The next high-leverage work is to deepen app-facing Modules around frame driving, workspace shell
authoring, table recipes, and editor-inspector state binding. Do not flatten the core into a
GPUI-style `gpui::*` root; Fret's explicit policy/mechanism split is a strength.

Implementation plan:
`docs/plans/2026-07-09-002-refactor-gpui-ergonomics-boundary-plan.md`.

## GPUI Reference Shape

GPUI's authoring model is short and productive:

- Start an `Application`, open a window, create a root `Entity<T>`, implement `Render`.
- Store state outside the element tree in `Entity<T>`.
- Render with `div()` and fluent style/interaction builders.
- Use `Context<T>` as the primary state, event, async, listener, focus, and window access seam.

Evidence:

- GPUI's README defines the three authoring registers: `Entity`, `Render` view, and low-level
  `Element`: `repo-ref/zed/crates/gpui/README.md`.
- The crate root exposes a wide convenience surface: `repo-ref/zed/crates/gpui/src/gpui.rs`.
- `AppContext` and `VisualContext` concentrate entity/window operations:
  `repo-ref/zed/crates/gpui/src/gpui.rs`.
- `div()` is an all-purpose authoring Module:
  `repo-ref/zed/crates/gpui/src/elements/div.rs`.
- `uniform_list(...)` hides virtualization behind a compact Interface:
  `repo-ref/zed/crates/gpui/examples/data_table.rs`.

What to borrow:

- Deep, high-leverage ecosystem authoring Modules.
- Test entry points that exercise the same Interface app authors use.
- Short, memorable render and listener patterns.

What not to borrow:

- A broad root prelude for all layers.
- A core all-purpose `Div` that mixes mechanism, policy, and design-system assumptions.
- Making platform/window details part of the default first-hour Interface.

## Fret Assessment

### Strong Interfaces

- `FretApp::new(...).window(...).view::<V>()?.run()` is already close to the GPUI startup shape,
  with more policy separation.
- `AppUi` groups state/actions/data/effects and hides raw `UiTree`, `ElementContext`, action notify
  internals, and model-store plumbing from the default path.
- `LocalState<T>` is a good app-state Adapter for view-owned state.
- `fret-ui-kit` and `fret-ui-shadcn` are the right owners for policy-heavy components and recipes.
- `fret-platform` has real native/web Adapter seams and keeps platform traits portable.

### Boundary Strengths

- Layering checks are green as of this audit: `python3 tools/check_layering.py` exited 0.
- Backend dependencies are concentrated in runner/render/launch crates rather than contract crates.
- `fret-ui` root tests already guard against retained widget authoring leaking into the default
  surface.
- `fret::app::prelude::*` is curated and tested as a budget, not a dumping ground.

### Boundary and Ergonomic Risks

- `ecosystem/fret/src/lib.rs` is too large for a single root file and mixes facade, tests,
  advanced/raw lanes, builder glue, and docs policy.
- `fret-runtime` root re-exports many runner/window/diagnostic stores, making the portable runtime
  Interface harder to scan.
- App-shaped demos such as datatable and workspace shell still teach manual runtime staging.
- `Effect` mixes app/platform effects with diagnostic and docking-shaped details.
- Some mechanism names in `fret-ui` read like component policy (`Pressable`, `Spinner`,
  `ResizablePanelGroup`, ripple/state-layer paint helpers). Keep them explicit and avoid teaching
  them as component contracts.

## Real App Probe Findings

| Probe | Finding | Owner |
| --- | --- | --- |
| `editor_notes_demo.rs` | `FretApp`/`View` shell is good, but inspector draft and summary actions fall back to raw `ModelStore` and activation closures. | `fret-ui-editor`, `ecosystem/fret` |
| `workspace_shell_demo` | Editor-grade shell requires `FnDriver`, `UiTree`, raw models, string commands, manual diagnostics, and custom command routing. | `fret-workspace`, `fret-bootstrap`, `ecosystem/fret` |
| `datatable_demo.rs` | Headless table + shadcn table is capable, but the author call site passes many coordination details manually. | `fret-ui-kit`, `fret-ui-shadcn` |
| `node_graph_demo.rs` | The surface mounting Interface is strong and compact; the demo does not yet prove command/searcher/keyboard authoring. | `fret-node` |

## Friction Register

| Pri | Broken truth | Likely owner | Next move |
| --- | --- | --- | --- |
| P0 | An editor-grade app should start from a public app surface, not from `FnDriver`, `UiTree`, and manual diagnostics. | `ecosystem/fret`, `fret-workspace`, `fret-bootstrap` | Add a workspace-workbench template or `WorkspaceApp` builder that owns frame/diagnostics wiring. |
| P0 | Complex shell commands should have a typed/action-first Interface, not split between typed actions and string `CommandId` constants. | `fret-runtime`, `fret-workspace` | Design typed workspace command wrappers with diagnostics trace preservation. |
| P1 | Second-hour shared/controlled state should not force app authors to mutate `ModelStore` directly. | `fret-ui-editor`, `ecosystem/fret` | Add controller/local-state binding recipes for inspector draft workflows. |
| P1 | Normal admin tables should not require hand-wiring output state, columns, row keys, toolbar, pagination, and debug ids every time. | `fret-ui-kit`, `fret-ui-shadcn` | Add a compact `DataTable` recipe/builder with stable diagnostics ids. |
| P1 | Diagnostics should be a runner capability, not hand-coded in every custom driver render loop. | `fret-bootstrap`, `fretboard-dev diag` | Provide a diagnostics frame hook/helper for custom drivers and workspace shells. |
| P1 | App-facing tests should avoid raw `dispatch_event -> layout_all -> paint_all` staging unless the test is specifically mechanism-level. | `fret-ui`, `fret-bootstrap` | Add an app-facing frame harness and migrate representative behavior tests. |
| P2 | The examples taxonomy is correct, but real app probes are scattered. | docs/examples | Keep a probe routing table in the examples index. |
| P2 | Node graph proves surface mounting but not command/searcher authoring. | `fret-node` | Add a typed node-graph mini app with commands, searcher, and diag coverage. |

## Refactor Lanes

1. `WorkspaceApp` / workspace-workbench
   - Hide `FnDriver`, `UiTree`, `RenderRootContext`, `UiFrameCx`, diagnostics driving, and default
     workspace command routing behind one app-facing Module.
   - Gate with a public template and one diagnostics script.

2. App-facing frame harness
   - Provide a test/driver Interface that runs propagation, render, layout, paint, semantics, and
     diagnostics in the right order.
   - Keep raw `UiTree` staging for mechanism tests only.

3. Runtime effect envelope cleanup
   - Separate ordinary app/platform effects from diagnostics and docking-specific requests.
   - Keep source compatibility only through explicit advanced/compat modules.

4. Table recipe compaction
   - Move common `DataTable` column/output/toolbar/pagination/debug-id wiring into a recipe builder.
   - Keep lower-level headless table composition available for custom tables.

5. `fret` facade file split
   - Split `ecosystem/fret/src/lib.rs` into internal modules for app surface, component surface,
     advanced/raw surface, builder glue, assets, and text helpers.
   - Keep the public paths stable while reducing root-file coupling.

## KTD2 Falsification Gate

KTD2 keeps `FretApp`, `View`, `AppUi`, and the `UiAppDriver` frame pipeline as the app substrate.
This is a falsifiable decision, not a preference for preserving the current shape. Evaluate it
against the three independent second-hour probes used by this audit:

- `workspace_shell_demo` for frame lifecycle, commands, focus, semantics, and diagnostics;
- `datatable_demo` for data-heavy recipe composition and app-owned state/output; and
- `editor_notes_demo` for controlled inspector state and editor rail composition.

The evidence and stop thresholds are:

| Signal | Evidence that KTD2 still holds | Threshold that falsifies KTD2 |
| --- | --- | --- |
| Repeated mechanism leak | Each probe's ordinary module reaches its public app/recipe/controller module without importing `FnDriver`, `UiTree`, `RenderRootContext`, or `UiFrameCx`. | After one owner-layer adapter attempt, at least two of the three probes still require the same raw mechanism noun in ordinary app code. One probe alone is a local module-depth defect; two independent probes are substrate evidence. |
| Frame-pipeline fidelity | Workspace behavior runs through `UiAppDriver` ordering and the app-facing stage trace while preserving render, layout, paint, semantics, and diagnostics output. | A probe can preserve its behavior only by bypassing `UiAppDriver`, or by duplicating/reordering two or more public `UiAppFrameStage` stages. |
| Dependency direction | Generic frame hooks remain in `fret-bootstrap`; workspace/table/editor policy remains in its ecosystem owner; `fret-app` stays backend-free. | Making the probe work requires `fret-app` to depend on `winit`, `wgpu`, `fret-launch`, or an ecosystem policy crate, or requires workspace/table/editor policy to move into `fret-ui`. |
| Default-surface budget | `fret::app::prelude::*` remains free of raw driver/tree/frame nouns, while narrow app-facing modules carry the added depth. | The default prelude must add two or more of `FnDriver`, `UiTree`, `RenderRootContext`, `UiFrameCx`, `ElementContext`, or backend/window types, or a broad GPUI-style root re-export is required. |
| Observable behavior | The real workspace suite preserves source/scope command traces, keyboard focus outcomes, semantics, and stable selectors through `WorkspaceApp`. | The raw path passes while the app-facing path cannot preserve any one of command source/scope, focus restore, semantics roles/actions, or diagnostics snapshots after a bounded owner-layer fix. |

Crossing any threshold stops further local wrapping and requires a revised implementation plan.
An ADR is additionally required when the response would change frame-stage ordering, input/focus
semantics, diagnostics protocol, crate dependency direction, or the default root/prelude contract.
A deeper ecosystem module that preserves those contracts needs a revised plan only if scope grows;
it does not need a new ADR.

Current source and focused-test evidence does not falsify KTD2: the three probes use the existing
app substrate through owner-layer modules, the workspace frame-stage trace remains observable, and
the default app prelude stays free of mechanism nouns. The commands in
[`Gates For Future Code Refactors`](#gates-for-future-code-refactors) are the reproducible evidence
set; the U9 sessions recorded at the top of this audit supply the diagnostics execution evidence.

### Two-Layer Authoring-Surface Enforcement

The authoring boundary is enforced at two different failure points. Both layers are required:

| Layer | What it catches | Canonical gates | Known limit |
| --- | --- | --- | --- |
| Source-string policy | Direct imports, raw constructor/call-site nouns, broad advanced preludes, and unclassified raw model/driver ownership in default or public probe source. | `python3 tools/check_surface_policy.py`; `python3 tools/gate_examples_source_tree_policy.py`; focused `apps/fret-examples/tests/*_surface.rs`. | A local source scan can be evaded by a facade re-export or renamed alias that removes the forbidden spelling from the consumer. |
| Public API surface | Symbols smuggled through `pub use`, including `as` aliases, accidental app/component prelude growth, and public paths that do not compile as documented. | `ecosystem/fret/src/authoring_surface_policy_tests.rs`; `ecosystem/fret/tests/surface_policy/*`; public facade smoke tests such as `app_editor_public_facade_smoke` and `advanced_public_facade_smoke`. | A surface budget does not prove that a real app avoids a raw API at its call site; the source-string layer supplies that evidence. |

A change is not compliant when only one layer passes. New forbidden ordinary-path nouns belong in
the existing source-policy scanners and their negative fixtures; new facade/prelude boundaries
belong in alias-aware public-surface tests plus a positive compile smoke for the intended path.

## Workspace Keyboard And Semantics Matrix

This matrix is the review contract for the real `WorkspaceApp` shell. Stable selectors are part of
the contract because diagnostics must prove the same surface that an app author launches.

Ownership is intentionally split. `WorkspaceCommandScope` publishes the scoped focus registry and
the current tab-strip/content focus lane. `WorkspaceWorkbench` owns the default workspace model
transitions, dirty-close prompt transactions, and the post-transition focus request.
`WorkspaceApp` bridges those outcomes into deferred post-frame focus and calls the app's
persistence hook before a `SaveAndClose` transaction can commit.

Model commands therefore take the owner-first Workbench/window route; focus-only commands remain
on the `WorkspaceCommandScope` widget route. In UI Gallery, `WorkspaceWindowLayout` is the sole
tabs/dirty/active source of truth and `selected_page` plus router state are render projections.

| Surface | Stable semantics and selectors | Focus order / roving rule | Keyboard and command outcome | Restore and diagnostics evidence |
| --- | --- | --- | --- | --- |
| Pane tab strip | `workspace-shell-pane-<pane>-tab-strip` is `tab_list`; each `workspace-shell-pane-<pane>-tab-<tab>` is `tab`, exposes `invoke`, and reports `selected`. | `Ctrl+F6` enters the active pane's selected tab. Arrow Left/Right roves and activates; Home/End chooses the first/last tab without adding every tab to sequential Tab order. | `Ctrl+Tab` / `Ctrl+Shift+Tab` route typed next/previous actions; close, split, move, pin, and preview actions lower to canonical `workspace.*` command IDs. Oversized active tabs use a stable leading-edge reveal instead of alternating between impossible full-visibility alignments. | Escape or the focus-content command returns to the recorded pane content. `workspace-shell-demo-tabstrip-keyboard-roving-smoke`, `workspace-shell-demo-tab-cycle-commands-smoke`, and `workspace-shell-demo-tab-text-during-dock-drag-stability` assert roles, selection, focus, shortcut routing, source, scope, stable bounds, and drag-time pixels. |
| Center panes | `workspace-shell-pane-<pane>-root` is `panel`; `workspace-shell-pane-<pane>-content` is a focusable `text_field` named `Pane content`. | Pane content is the non-tabstrip return target. `WorkspaceCommandScope` records whether focus started in the active pane's tab-strip or content lane; after a model command changes the active pane/tab, Workbench requests the same lane in the new active context after the frame rebuild. | Typed pane focus, split, and move-active-tab actions use directional/next/previous commands. Default chords remain registry metadata; diagnostics assert the canonical command rather than hard-coding platform-specific display strings. | Tab-strip exit returns to the recorded content element. The pane-focus/move script contains cross-pane lane assertions; `WorkspaceWorkbench` and command-scope tests cover both active-tab-strip and active-pane-content fallbacks. |
| Left file-tree rail | `workspace-shell-file-tree-root` is `list`; `workspace-shell-file-tree-node-<id>` is a focusable `tree_item` with `focus` and `invoke`, plus `expanded`/`selected` where applicable. | The rail root is not a focus stop. Sequential Tab enters the first visible tree item and continues through visible rows; this retained file-tree currently uses sequential focus, not one-stop roving focus. | Enter/Space invokes the focused row, updating selection and expansion for branch rows. | `workspace-shell-demo-rails-keyboard-semantics-smoke` asserts root/row roles, non-collapsed bounds, first-row keyboard entry, next-row traversal, activation, selection, and expansion. |
| Right editor rail | `workspace-shell-editor-rail` is a stable, read-only generic container; `workspace-shell-editor-rail-header` bounds the header and the accessible `text` label is `Editor Rail`. | The current rail contains readouts only and therefore has no focus stop or keyboard entry target; sequential Tab skips it. This is an explicit read-only exception, not a hidden missing control. | No command originates from the read-only rail. If an interactive control is added, the same change must promote the rail/header to `region`/`heading`, give the first control a stable `test_id`, and add a Tab-entry assertion. | `workspace-shell-demo-rails-keyboard-semantics-smoke` locks the root/header selectors, accessible title, minimum bounds, and non-overlap with the left rail and center pane. |
| Dirty-close dialog | `workspace-shell-dirty-close-prompt` is `dialog`; `.cancel`, `.discard`, and `.save_and_close` are focusable buttons in that order. | Initial focus is Cancel. Tab cycles Cancel -> Discard -> Save & Close -> Cancel; Shift+Tab reverses inside the modal focus barrier. | Enter invokes the focused choice. Escape and Cancel share the cancel state machine; Discard resolves the Workbench transaction immediately. `SaveAndClose` resolves only after `WorkspaceWindowState::save_workspace_dirty_close` returns `true`; the default `false` keeps the prompt open and leaves the layout unchanged. | Cancel restores the triggering tab. Destructive close restores the surviving active tab. The dirty-close app-facing scripts contain assertions for roles, trapping, pointer/keyboard source, window scope, driver handling, hook-confirmed save, and restore outcomes. |
| Command diagnostics | Every typed workspace **model** action lowers to one canonical command ID and produces a final dispatch trace with source, source `test_id` when available, scope, handler, and domain outcome. UI-only focus actions use the same canonical identity and dispatch pipeline, but their current contract covers ID, source, scope, and handler only; no domain outcome is promised. | Shortcut focus origin remains observable; window-owner/driver handling must not erase the original `shortcut` source. During a modal, only direct pointer/keyboard model actions whose live source is inside the active barrier scope reach Workbench. | Widget-handled and driver-handled paths are asserted separately; blocked dirty close and applied save/discard remain distinguishable model outcomes. Shortcut, programmatic, stale, underlay, and window-close sources remain blocked by the modal route. | `workspace-shell-demo-tab-cycle-commands-smoke`, the dirty-close scripts, `ui-gallery-workspace-shell-tab-commands-smoke`, and the frame-stage selector define the assertions for source preservation, modal provenance, model outcome, and app-facing frame completion. |

The `workspace-shell-app-facing` suite is the required real-app gate for this matrix. The
`ui-gallery-workspace-shell` suite remains the shared-chrome gate; it is not a substitute for the
real `WorkspaceApp` shell. The U9 artifacts recorded above prove 10/10 and 4/4 passing runs,
respectively.

## Documentation Closeout

- The default onboarding ladder remains `hello` -> `simple-todo` -> `todo`.
- Realistic app probes are routed explicitly as probes rather than first-contact examples.
- `docs/ui-ergonomics-and-interop.md` links to this audit.
- `docs/examples/README.md` includes the real-app probe table.

## Gates For Future Code Refactors

- `cargo fmt`
- `python3 tools/check_surface_policy.py`
- `python3 tools/gate_examples_source_tree_policy.py`
- `cargo nextest run -p fret-ui`
- `cargo nextest run -p fret-app`
- `cargo nextest run -p fret-ui-shadcn`
- `python3 tools/check_layering.py`
- `tools/check_workspace_tab_drag_visual_stability.sh`
- For app-facing frame or workspace changes, run or update:
  `cargo run -p fretboard-dev -- diag suite ui-gallery-workspace-shell --launch -- cargo run -p fret-ui-gallery --release`
  and
  `cargo run -p fretboard-dev -- diag suite workspace-shell-app-facing --launch -- cargo run -p fret-demo --bin workspace_shell_demo --release`.
