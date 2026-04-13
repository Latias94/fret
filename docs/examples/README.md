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
`use fret::advanced::AppUiRawModelExt;` plus `cx.raw_model::<T>()`; it is not part of the default
ladder.

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

## 0.1) Surface taxonomy

Use these labels consistently:

- **Default**: first-contact templates and stable cookbook lessons
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
- **Workspace shell** stays on `fret-workspace`.
  UI Gallery, `workspace_shell_demo`, and other editor-grade shells should compose explicit
  `fret_workspace::*` owners instead of routing that chrome back through `fret`.
- **In-window menubar** is only an optional bridge.
  If an example needs one, it should import `fret::in_window_menubar::*` explicitly rather than
  treating it as a synonym for workspace shell ownership.

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

- Golden pair:
  - `imui_action_basics` — generic/default immediate authoring on the app lane
  - `imui_editor_proof_demo` — editor-grade immediate proof on the intended generic/editor owner
    split
- Reference/smoke:
  - `imui_hello_demo` — tiny runnable facade smoke; useful, but not the main first-contact path
- Reference/contract proof:
  - `imui_response_signals_demo` — proof/contract surface for outward responses, helper lifecycle,
    and interaction queries
- Reference/product-validation:
  - `imui_interaction_showcase_demo` — presentable IMUI shell that keeps the immediate control
    flow story while using shadcn chrome for layout rhythm
  - `imui_shadcn_adapter_demo`
- Advanced/reference:
  - `imui_floating_windows_demo`
- Compatibility-only:
  - `imui_node_graph_demo`

Mounting rule for the immediate-mode lane:

- If your IMUI content already lives under an explicit layout host such as `Column`, `Row`, or
  `v_flex`, prefer `fret_imui::imui(cx, ...)`.
- If you are mounting IMUI directly at the view root or under a non-layout parent, prefer
  `fret_imui::imui_vstack(cx.elements(), ...)`.
- `imui_vstack(...)` is the explicit root-host bridge, not evidence that generic helper growth
  should reopen.
- `imui_action_basics` demonstrates the nested layout-host shape; `imui_hello_demo` remains the
  small smoke/reference proof of the explicit root-hosted shape.

Stable identity rule for the immediate-mode lane:

- For static lists whose order never changes, `ui.for_each_unkeyed(...)` is acceptable.
- For dynamic collections that insert, remove, reorder, or preserve per-row state, prefer
  `ui.for_each_keyed(...)` or `ui.id(key, ...)`.
- Rebuild rows each frame; do not treat element values as cloneable reusable UI.
- `imui_action_basics` is still the right generic/default proof even though it does not need keyed
  identity yet; `imui_editor_proof_demo` is the heavier proof where explicit stable identity is
  already visible.

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
  runnable facade check, but the generic/editor immediate teaching path should start from
  `imui_action_basics` and `imui_editor_proof_demo`.
- `imui_response_signals_demo` is an IMUI proof/contract surface. It validates response/query
  behavior and canonical helper outward responses rather than the default immediate teaching path.
- `imui_interaction_showcase_demo` and `imui_shadcn_adapter_demo` are IMUI product-validation
  surfaces. They validate polished shell composition and adapter/product layering rather than the
  default immediate teaching path.
- `imui_floating_windows_demo` is an IMUI overlap/floating proof surface. It validates IMUI
  interaction contracts and diagnostics affordances rather than the retained-mode onboarding lane.
- `imui_node_graph_demo` is an IMUI compatibility-only proof. It exists to keep the retained-bridge
  node-graph path auditable and should not be treated as the default downstream immediate path.

Start from the “Examples redesign” workstream for the intended product surface:

- [docs/workstreams/example-suite-fearless-refactor-v1/design.md](../workstreams/example-suite-fearless-refactor-v1/design.md)
