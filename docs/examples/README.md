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
`cx.actions().locals(...)` for coordinated LocalState writes, `cx.actions().transient(...)` for
App-bound effects, and local `on_activate*` only when widget glue truly needs it. Drop down to
`cx.actions().models(...)` when coordinating shared `Model<T>` graphs.

1. `hello` (template) — smallest runnable UI surface.
   - Generate: `cargo run -p fretboard -- new hello --name hello-world`
2. `simple-todo` (template) — view runtime + typed actions + keyed lists (no selectors/queries;
   the current default path is `LocalState<Vec<_>>` + payload row actions for view-owned lists).
   - Generate: `cargo run -p fretboard -- new simple-todo --name my-simple-todo`
3. `todo` (template) — “best practice baseline” (selectors + queries).
   - Generate: `cargo run -p fretboard -- new todo --name my-todo`
   - Read: [docs/examples/todo-app-golden-path.md](./todo-app-golden-path.md)
   - Note: this template opts into `fret` feature `state` (selector/query helpers).

## 0.1) Surface taxonomy

Use these labels consistently:

- **Default**: first-contact templates and stable cookbook lessons
- **Comparison**: evidence-oriented side-by-side samples that help evaluate ergonomics, not onboarding
- **Advanced**: gallery, interop, renderer, docking, and maintainer-oriented surfaces

## 1) In-tree Cookbook (small, focused lessons)

Cookbook examples live under [apps/fret-cookbook/examples/](../../apps/fret-cookbook/examples/).
Each file is intended to be one lesson. They are runnable and designed for copy/paste.

Run one via the tooling runner (recommended):

```bash
cargo run -p fretboard -- dev native --example simple_todo
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
cargo run -p fretboard -- list cookbook-examples --all
```

Tip: when running cookbook examples via `fretboard dev native --example <name>`, `fretboard` will
auto-enable required cookbook features for known Lab examples and print what it enabled.

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

Use it when:

- you want to inspect component recipes and parity,
- you need a conformance/regression target,
- you are validating product polish after learning the default ladder.

Do not use it as the first place to learn the authoring model.

- Native (lite by default): `cargo run -p fret-ui-gallery`
- Full catalog: `cargo run -p fret-ui-gallery --features gallery-full`
- Dev/unfinished pages (opt-in): `cargo run -p fret-ui-gallery --features gallery-dev`
- Material 3 (in progress, opt-in): `cargo run -p fret-ui-gallery --features gallery-material3`
- Web: `cargo run -p fretboard -- dev web --demo ui_gallery`
- Diagnostics (lite smoke): `cargo run -p fretboard -- diag suite ui-gallery-lite-smoke --launch -- cargo run -p fret-ui-gallery`
- Details: [apps/fret-ui-gallery/README.md](../../apps/fret-ui-gallery/README.md)

## 3) Labs / maintainer harnesses

Some demos are intentionally “high ceiling” (docking arbitration, renderer effects, node graph
stress). They are useful for maintainers and advanced users but are not the onboarding path.

Start from the “Examples redesign” workstream for the intended product surface:

- [docs/workstreams/example-suite-fearless-refactor-v1/design.md](../workstreams/example-suite-fearless-refactor-v1/design.md)
