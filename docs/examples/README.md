# Examples (Index)

This repo has multiple “example-shaped” entry points. This page is the shortest path to choosing
the right one.

Default reading rule:

- start on the **Default** ladder,
- use **Comparison** surfaces only when you are intentionally reviewing ergonomics,
- treat **Advanced** surfaces as reference/product-validation layers, not as first-contact teaching material.

Productization note:

- the repo intentionally keeps this taxonomy small and repetitive; if a page/example does not clearly
  fit **Default**, **Comparison**, or **Advanced**, treat that as a docs bug rather than as a cue to
  infer a fourth category.

## 0) Boring ladder (recommended)

These are intentionally stable and should be your default onboarding path:

They all teach the same small authoring model first: `LocalState` for view-owned state,
`cx.actions().locals_with((...)).on::<A>(|tx, (...)| ...)` for coordinated LocalState writes,
`cx.actions().local(&local).set::<A>(...)` / `.update::<A>(...)` / `.toggle_bool::<A>()` for
single-local writes, keyed-row payload binding via `.action_payload(...)`, and
`cx.actions().local(&rows_state).payload_update_if::<A>(...)` as the default row-write path,
`cx.actions().transient(...)` for App-bound effects, and widget-local `.action(...)` /
`.action_payload(...)` / `.listen(...)` only when a control truly needs the activation bridge.
Drop down to `cx.actions().models(...)` when coordinating shared `Model<T>` graphs.
The only raw-model escape hatch is the explicit advanced import
`use fret::advanced::raw::AppUiRawModelExt;` plus `cx.raw_model::<T>()`; it is not part of the default
ladder.
This is the only blessed first-contact local-state story. For dynamic lists/subtrees, teach keyed
identity first (`ui::for_each_keyed(...)` or `ui.id(key, ...)`); keep unkeyed iteration as the
explicit static-list exception.

Installed/public template spelling below uses `fretboard new ...`.
In this repository, the public-surface equivalent is `cargo run -p fretboard -- new ...`.
`cargo run -p fretboard-dev -- new ...` remains the repo-local maintainer variant and writes under
`local/` by default.

1. `hello` (template) — smallest runnable UI surface.
   - Generate: `fretboard new hello --name hello-world`
2. `simple-todo` (template) — view runtime + typed actions + keyed lists (no selectors/queries;
   the current default path is `LocalState<Vec<_>>` + payload row actions for view-owned lists).
   - Generate: `fretboard new simple-todo --name my-simple-todo`
3. `todo` (template) — richer third rung once you need selectors + queries; generated as a
   product baseline with deletable selector/query slices, not as the default starter scaffold.
   - Generate: `fretboard new todo --name my-todo`
   - Read: [docs/examples/todo-app-golden-path.md](./todo-app-golden-path.md)
   - Note: this template opts into `fret` feature `state` (selector/query helpers), and its
     generated README calls out the first deletable slices if you want to collapse back toward
     `simple-todo`.
4. `workbench-lite` (template) — second-hour app slice after the default ladder. It proves command
   palette integration, a settings dialog, content pane, status bar, and a simulated submit flow
   without importing raw runtime seams.
   - Generate: `fretboard new workbench-lite --name my-workbench`
   - Note: this template keeps `use fret::app::prelude::*` as the app surface and imports style
     nouns explicitly from `fret::style`. It enables the command-palette feature but intentionally
     does not enable selector/query/mutation features.
   - Diagnostics: `tools/diag-scripts/public-app/workbench-lite-settings-dialog.json` covers
     settings draft/save/cancel, Escape, focus containment, and focus restore with stable
     `workbench_lite.*` selectors.
5. `mutation-workbench` (template) — async second-hour app slice when the simulated submit flow is
   too small. It proves mutation submit, retry, toast feedback, and query invalidation through the
   public `AppUi` facade.
   - Generate: `fretboard new mutation-workbench --name my-mutation-workbench`
   - Note: this template keeps `use fret::app::prelude::*` as the app surface, imports explicit
     `fret::mutation` / `fret::query` nouns for async work, and avoids framework-internal crates,
     raw element erasure, retained tree mechanisms, host adapters, and model-store plumbing.
   - Diagnostics: `tools/diag-scripts/public-app/mutation-workbench-flow.json` covers submit,
     success, forced error, editable input preservation, retry, query refresh, and toast feedback
     with stable `mutation_workbench.*` selectors.

## 0.1) Surface taxonomy

Use these labels consistently:

- **Default**: first-contact templates and stable cookbook lessons
- **Second-hour**: copyable public app slices such as `workbench-lite` and `mutation-workbench`
- **Comparison**: evidence-oriented side-by-side samples that help evaluate ergonomics, not onboarding
- **Advanced**: gallery, interop, renderer, docking, and maintainer-oriented surfaces

## 0.2) Shell split in examples

Examples in this repo intentionally teach three different shell layers. Do not collapse them into
one generic `AppShell` mental model.

- **Window bootstrap** lives on the startup builder lane.
  Templates and runnable examples should set initial title/size there, and should add
  `.window_min_size(...)`, `.window_position_logical(...)`, or `.window_resize_increments(...)`
  when that behavior is part of the user-facing product surface.
- **Page shell** stays app-owned.
  Templates, cookbook lessons, and ordinary demos may use centered cards, docs scaffolds, or
  responsive page wrappers, but those helpers are teaching surfaces local to the app/example, not
  stable framework contracts.
- **Workspace policy** stays on `fret-workspace`; ordinary startup stays app-facing.
  Editor-grade shells compose pane/tab/focus policy and typed commands from `fret_workspace::*`,
  then use `fret::workspace::WorkspaceApp` for command registration, menus, launch, frame
  lifecycle, and diagnostics wiring.
- **In-window menubar** is only an optional bridge.
  If an example needs one, it should import `fret::in_window_menubar::*` explicitly rather than
  treating it as a synonym for workspace shell ownership.

## 0.3) Real app probes

Use these probes to evaluate framework ergonomics after the default ladder. They are evidence
surfaces, not first-contact teaching surfaces.

| Probe | Start from | What it proves | Do not copy first |
| --- | --- | --- | --- |
| Editor notes workbench | `apps/fret-examples/src/editor_notes_demo.rs` | `InspectorTextFieldBinding` over explicit `LocalState` handles, including clean/dirty, commit/cancel, focus, and semantic-label diagnostics | Lower-level draft-controller or raw-model ownership when the binding is sufficient |
| Workspace shell / IDE-lite | `apps/fret-examples/src/workspace_shell_demo/` | `fret::workspace::WorkspaceApp`, typed `fret_workspace::commands::act` identities, dirty-close policy, command traces, keyboard focus, semantics, layout, and screenshots | The remaining app-specific overlay and virtual-list composition internals |
| Data-heavy admin surface | `apps/fret-examples/src/datatable_demo.rs` | `DataTableRecipe` with caller-visible state/output/columns/row keys and stable debug-id behavior | Low-level headless table composition when the standard recipe is enough |
| Node graph / canvas editor | `apps/fret-examples/src/node_graph_demo.rs` | Deep node-graph surface mounting, graph model setup, optional diagnostics hooks | Treating the current paint-oriented demo as proof of command/searcher authoring |

For the GPUI comparison baseline and implemented refactor, see
[docs/audits/gpui-ergonomics-boundary-audit-2026-07.md](../audits/gpui-ergonomics-boundary-audit-2026-07.md).
`datatable_demo` is now a default-clean recipe probe. `workspace_shell_demo` remains explicitly
advanced only for its residual app-specific policy internals; its launch, frame, typed-command,
and diagnostics paths are no longer temporary raw allowances.

Workspace evidence is intentionally split by owner: `ui-gallery-workspace-shell` validates shared
chrome layout, semantics, keyboard focus, and command traces, while `workspace-shell-app-facing`
is the real `WorkspaceApp` gate for its frame-stage trace, pane split/move behavior, and dirty-close
lifecycle. The suite definition is reproducible evidence only after its diagnostics run succeeds.

Copy the bounded public slices from
[Authoring Golden Path - Second-hour canonical slices](../authoring-golden-path.md#second-hour-canonical-slices),
then use the probe paths above for complete context. In particular, a production
`WorkspaceWindowState` must persist every requested dirty document in
`save_workspace_dirty_close` and return `true` only after that succeeds. The hook defaults to
`false`, so `SaveAndClose` otherwise keeps the prompt open and the workspace layout unchanged.

## 1) In-tree Cookbook (small, focused lessons)

Cookbook examples live under [apps/fret-cookbook/examples/](../../apps/fret-cookbook/examples/).
Each file is intended to be one lesson. They are runnable and designed for copy/paste.

Shell note:

- cookbook page framing is intentionally cookbook-owned; helpers such as the centered page scaffold
  keep lessons visually consistent without turning that page shell into a shared framework API.

Run one via the tooling runner (recommended):

```bash
cargo run -p fretboard-dev -- dev native --example simple_todo
```

Note: you can also run cookbook examples directly via Cargo, but some higher-ceiling examples are
feature-gated (see [apps/fret-cookbook/README.md](../../apps/fret-cookbook/README.md)).

Recommended starting points (Official; stable + onboarding-friendly):

- `hello`, `simple_todo`, `hello_counter`
- `overlay_basics`, `text_input_basics`, `commands_keymap_basics`
- `virtual_list_basics` (virtualization + keyed identity + reordering)
- `effects_layer_basics` (EffectLayer + EffectChain: pixelate/blur)
- `theme_switching_basics` (shadcn theme switching)

Lab / higher-ceiling examples (feature-gated; opt-in):

- `mutation_toast_feedback_basics` (explicit submit + Sonner feedback projection)
- `query_basics` (queries)
- `router_basics` (routing)
- `icons_and_assets_basics` (assets)
- `docking_basics`, `embedded_viewport_basics`, `external_texture_import_basics` (interop)

Tip: feature-gated examples and their `--features ...` hints are discoverable via:

```bash
cargo run -p fretboard-dev -- list cookbook-examples --all
```

Tip: when running cookbook examples via `fretboard-dev dev native --example <name>`, `fretboard-dev` will
auto-enable required cookbook features for known Lab examples and print what it enabled.

Immediate-mode sidecar (when you intentionally want the IMUI lane):

- First-party authoring policy: use the root `fret::imui` lane (`use fret::imui::prelude::*;` or
  `use fret::imui::{kit::..., prelude::*};` for kit-focused teaching surfaces, or
  `use fret::imui::{kit::..., editor, prelude::*};` for editor-grade teaching surfaces.
  No first-party IMUI example uses the retained node-graph canvas.

- Golden pair:
  - `imui_action_basics` — generic/default immediate authoring on the app lane
  - `imui_editor_controls_basics` — editor-grade first-contact controls through
    `fret::imui::editor`
- Debug draw proof:
  - `imui_debug_draw_basics` — canvas-backed draw-list authoring and metadata through
    `fret::imui::kit`
- Plot adapter proof:
  - `imui_plot_basics` — opt-in plotting through `fret_plot::imui` while the host surface stays on
    the root `fret::imui` lane
- Product workbench:
  - `cargo run -p fret-demo --bin imui_editor_workbench_demo`
  - discover product proofs with `cargo run -p fretboard-dev -- list native-demos --all`
  - `imui_editor_workbench_demo` is the canonical editor workbench route; it mounts the
    editor-notes workflow directly while the older proof demos remain smaller supporting surfaces
  - use it when you need the first-open editor-grade IMUI workbench path; drop to
    `imui_editor_proof_demo` only when you need the older dense panel proof directly
- Reference/smoke:
  - `imui_hello_demo` — tiny runnable facade smoke; useful, but not the main first-contact path
    - this is the smallest IMUI text/control smoke; use it when checking that visible text,
      button semantics, and checkbox semantics still work
    - `fretboard` / `fretboard-dev` need an explicit package here because both
      `fret-demo` and `fret-examples-imui` both define `imui_hello_demo`
    - maintainer wrapper:
      `cargo run -p fretboard -- dev native --package fret-demo --bin imui_hello_demo`
    - standalone fast-path:
      `cargo run -p fretboard -- dev native --package fret-examples-imui --bin imui_hello_demo`
- Reference/contract proof:
  - `imui_response_signals_demo` — proof/contract surface for outward responses, helper lifecycle,
    and interaction queries
- Reference/product-validation:
  - `imui_interaction_showcase_demo` — presentable IMUI shell that keeps the immediate control
    flow story while using shadcn chrome for layout rhythm
  - `imui_shadcn_adapter_demo`
- Advanced/reference:
  - `imui_floating_windows_demo`

Mounting rule for the immediate-mode lane:

- On the explicit `fret::imui` lane, `imui(...)` is now the safe default: it adds the stacked host
  needed for view roots and non-layout parents.
- If your IMUI content already lives under an explicit layout host such as `Column`, `Row`, or
  `v_flex`, and you explicitly want bare sibling emission, use `imui_raw(cx, ...)` from
  `use fret::imui::prelude::*;`.
- `imui_raw(...)` is the advanced seam, not the default first-open teaching surface.
- `imui_action_basics` demonstrates the default app-facing `imui_in(...)` shape with
  `LocalState<String>` text inputs; `imui_hello_demo` remains the small smoke/reference proof of the
  root-hosted shape.
  - the public CLI must choose a package explicitly for this binary name collision

Stable identity rule for the immediate-mode lane:

- For static lists whose order never changes, `ui.for_each_unkeyed(...)` is acceptable.
- For dynamic collections that insert, remove, reorder, or preserve per-row state, prefer
  `ui.for_each_keyed(...)` or `ui.id(key, ...)`.
- Rebuild rows each frame; do not treat element values as cloneable reusable UI.
- `imui_action_basics` is still the right generic/default proof even though it does not need keyed
  identity yet; `imui_editor_controls_basics` is the editor-control first-contact proof,
  `imui_plot_basics` is the opt-in plot adapter proof, and `imui_editor_workbench_demo` is the
  canonical product workbench route, while
  `imui_editor_proof_demo` remains the supporting dense panel proof where explicit stable identity
  is already visible.

Comparison / still-evolving examples (not recommended for onboarding) are labeled in the cookbook index:

- `simple_todo_v2_target` — comparison target for denser payload-row / root-handler keyed-list authoring on the same `LocalState<Vec<Row>>` baseline; it is intentionally evidence-oriented, not the default tutorial surface.
- [apps/fret-cookbook/EXAMPLES.md](../../apps/fret-cookbook/EXAMPLES.md)

Historical MVU removal inventory (applies to maintainer demos, not cookbook):

- [docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md](../workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md)

Cookbook curation (recommended order + feature-gated examples):

- [apps/fret-cookbook/README.md](../../apps/fret-cookbook/README.md)

Diagnostics scripts for cookbook examples live under:

- [tools/diag-scripts/cookbook/](../../tools/diag-scripts/cookbook/)
- [tools/diag-scripts/suites/](../../tools/diag-scripts/suites/) (cookbook suite manifests)
- New to diagnostics? Start with the `hello` walkthrough in
  [apps/fret-cookbook/README.md#diagnostics-optional](../../apps/fret-cookbook/README.md#diagnostics-optional).

Note: cookbook examples are separate binaries today, so scripts are per-example (not one “mega suite”).

## 2) UI Gallery (component catalog + conformance)

The UI gallery is a larger, multi-page app intended for component discovery and parity testing.

Taxonomy: this is an **Advanced** surface.

Shell note:

- UI Gallery is not the default small-app shell pattern. It combines a docs/page scaffold with
  editor-grade workspace chrome from `fret-workspace`, and keeps optional in-window menubar wiring
  explicit.

Use it when:

- you want to inspect component recipes and parity,
- you need a conformance/regression target,
- you are validating product polish after learning the default ladder.

Do not use it as the first place to learn the authoring model.

- Native (lite by default): `cargo run -p fret-ui-gallery`
- Full catalog: `cargo run -p fret-ui-gallery --features gallery-full`
- Dev/unfinished pages (opt-in): `cargo run -p fret-ui-gallery --features gallery-dev`
- Material 3 (in progress, opt-in): `cargo run -p fret-ui-gallery --features gallery-material3`
- Web: `cargo run -p fretboard-dev -- dev web --demo ui_gallery`
- Diagnostics (lite smoke): `cargo run -p fretboard-dev -- diag suite ui-gallery-lite-smoke --launch -- cargo run -p fret-ui-gallery`
- Details: [apps/fret-ui-gallery/README.md](../../apps/fret-ui-gallery/README.md)

## 3) Labs / maintainer harnesses

Some demos are intentionally “high ceiling” (docking arbitration, renderer effects, node graph
stress). They are useful for maintainers and advanced users but are not the onboarding path.

Explicit advanced/reference roster:

- `first_frame_smoke_demo` is a runner bootstrap / first-present smoke. It intentionally paints only
  a full-window quad and closes itself after several frames; it is not a text-rendering smoke.
- `custom_effect_v1_demo`, `custom_effect_v2_demo`, and `custom_effect_v3_demo` are renderer/effect
  reference surfaces. They keep explicit effect/runtime ownership because the point is validating
  effect ABI, bounded custom-effect authoring, and diagnostics behavior.
- `postprocess_theme_demo` and `liquid_glass_demo` are renderer/product-validation surfaces. They
  keep explicit renderer/theme or renderer-capability ownership because they validate high-ceiling
  post-process and glass/warp behavior rather than the default app lane.
- `genui_demo` is a generator/editor integration reference surface. It keeps explicit model
  ownership because the point is catalog/runtime/validation integration, not first-contact app
  authoring.
- `imui_hello_demo` is a tiny IMUI smoke/reference surface. It remains useful for the smallest
  runnable facade check and the smallest text/control rendering check, but the generic/editor
  immediate teaching path should start from `imui_action_basics` and `imui_editor_controls_basics`;
  use `imui_plot_basics` when the missing piece is the optional `fret_plot::imui` adapter, use
  `imui_editor_workbench_demo` when you need the canonical editor workbench route, and use
  `imui_editor_proof_demo` only for the supporting dense editor panel proof.
  - when launched through `fretboard`, select `fret-demo` or `fret-examples-imui` explicitly:
    `cargo run -p fretboard -- dev native --package fret-demo --bin imui_hello_demo`
    or
    `cargo run -p fretboard -- dev native --package fret-examples-imui --bin imui_hello_demo`
- `imui_response_signals_demo` is an IMUI proof/contract surface. It validates response/query
  behavior and canonical helper outward responses rather than the default immediate teaching path.
- `imui_interaction_showcase_demo` and `imui_shadcn_adapter_demo` are IMUI product-validation
  surfaces. They validate polished shell composition and adapter/product layering rather than the
  default immediate teaching path.
- `imui_floating_windows_demo` is an IMUI overlap/floating proof surface. It validates IMUI
  interaction contracts and diagnostics affordances rather than the retained-mode onboarding lane.
Start from the “Examples redesign” workstream for the intended product surface:

- [docs/workstreams/example-suite-fearless-refactor-v1/design.md](../workstreams/example-suite-fearless-refactor-v1/design.md)
