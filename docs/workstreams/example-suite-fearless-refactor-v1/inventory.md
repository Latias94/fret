# Example Suite v1 — Current Inventory + Mapping

This appendix maps the **intended catalog** to the **current repo reality**.

It is deliberately pragmatic:

- it provides concrete “where is it today?” anchors,
- it marks migration intent (keep/move/replace),
- it gives maintainers a single place to update when files move.

Legend:

- **State**:
  - `Keep`: keep the current anchor as the canonical implementation (may still be polished).
  - `Move`: migrate into the cookbook crate (`examples/`) when created.
  - `Replace`: keep behavior but change the hosting/wiring (e.g. consolidate registry).
  - `Maint`: maintainer/stress harness (not in the onboarding catalog by default).

## Canonical ladder mapping

| ID | State | Current anchors | Run (today) | Notes |
|---|---|---|---|---|
| hello | Keep | `apps/fretboard/src/scaffold/templates.rs` | `fretboard new hello --name hello-world` | Template-generated; should stay boring/stable. |
| simple-todo | Keep | `apps/fretboard/src/scaffold/templates.rs` | `fretboard new simple-todo --name my-simple-todo` | Template-generated; “no selector/query” baseline. |
| todo | Keep | `apps/fretboard/src/scaffold/templates.rs`, `docs/examples/todo-app-golden-path.md` | `fretboard new todo --name my-todo` | Best-practice baseline (selectors + queries). |
| components_gallery | Replace | `apps/fret-examples/src/components_gallery.rs`, `apps/fret-demo/src/bin/components_gallery.rs` | `fretboard dev native --bin components_gallery` | Keep behavior, but the registry/discovery story should unify. |
| ui_gallery | Keep | `apps/fret-ui-gallery/src/*`, `apps/fret-demo/src/bin/ui_gallery.rs` | `cargo run -p fret-ui-gallery` | Treat as component catalog + conformance harness. |

## Reference apps (planned)

These are app-scale examples (Zed-like anchors). They are intentionally larger than cookbook
examples and should be treated as product surfaces.

| ID | State | Target anchors | Notes |
|---|---|---|---|
| workbench | Planned | `apps/workbench/` (TBD) | Editor-grade shell: docking + command palette + settings + file tree + doc surfaces. |
| viz-studio | Planned | `apps/viz-studio/` (TBD) | Viz workspace: charts/plot + canvas + node graph (optional) + perf-friendly virtualization. |
| shader-lab | Planned | `apps/shader-lab/` (TBD) | Renderer lab: built-in steps + CustomV1/V2/V3 tracks + budgets/capabilities surfaced. |

## Cookbook crate (current)

| ID | State | Current anchors | Run (today) | Notes |
|---|---|---|---|---|
| cookbook.hello | Keep | `apps/fret-cookbook/examples/hello.rs` | `cargo run -p fret-cookbook --example hello` | Minimal “hello” runnable. |
| cookbook.hello_counter | Keep | `apps/fret-cookbook/examples/hello_counter.rs` | `cargo run -p fret-cookbook --example hello_counter` | Small MVU + Model counter; stable `test_id` set. |
| cookbook.simple_todo | Keep | `apps/fret-cookbook/examples/simple_todo.rs` | `cargo run -p fret-cookbook --example simple_todo` | Minimal todo list (MVU + keyed rows) intended for copy/paste. |
| cookbook.overlay_basics | Keep | `apps/fret-cookbook/examples/overlay_basics.rs` | `cargo run -p fret-cookbook --example overlay_basics` | Minimal dialog example with stable `test_id` stamps. |
| cookbook.commands_keymap_basics | Keep | `apps/fret-cookbook/examples/commands_keymap_basics.rs` | `cargo run -p fret-cookbook --example commands_keymap_basics` | Command registration + default keybinding + availability gating. |
| cookbook.undo_basics | Keep | `apps/fret-cookbook/examples/undo_basics.rs` | `cargo run -p fret-cookbook --example undo_basics` | App-owned undo/redo history (`fret-undo`) wired to `edit.undo/edit.redo` commands. |
| cookbook.text_input_basics | Keep | `apps/fret-cookbook/examples/text_input_basics.rs` | `cargo run -p fret-cookbook --example text_input_basics` | Input submit/clear via commands (Enter/Escape) + numeric semantics gates. |
| cookbook.effects_layer_basics | Keep | `apps/fret-cookbook/examples/effects_layer_basics.rs` | `cargo run -p fret-cookbook --example effects_layer_basics` | Minimal effect layer example (Pixelate/Blur) with stable `test_id` stamps. |
| cookbook.icons_and_assets_basics | Keep | `apps/fret-cookbook/examples/icons_and_assets_basics.rs` | `cargo run -p fret-cookbook --example icons_and_assets_basics` | Icon packs (lucide/radix) + semantic `ui.*` ids + file-based SVG/images via `fret-ui-assets`. |
| cookbook.virtual_list_basics | Keep | `apps/fret-cookbook/examples/virtual_list_basics.rs` | `cargo run -p fret-cookbook --example virtual_list_basics` | Keyed virtualization + items_revision + scroll-to-item, with a minimal diag smoke script. |
| cookbook.async_inbox_basics | Keep | `apps/fret-cookbook/examples/async_inbox_basics.rs` | `cargo run -p fret-cookbook --example async_inbox_basics` | Portable async pattern: background task → inbox → runner drain, with cancellation + progress semantics gate. |
| cookbook.markdown_and_code_basics | Keep | `apps/fret-cookbook/examples/markdown_and_code_basics.rs` | `cargo run -p fret-cookbook --example markdown_and_code_basics` | Markdown rendering + fenced code blocks (code-view/syntax) + copy affordance, with a minimal diag smoke script. |
| cookbook.theme_switching_basics | Keep | `apps/fret-cookbook/examples/theme_switching_basics.rs` | `cargo run -p fret-cookbook --example theme_switching_basics` | Minimal theme switching (shadcn New York v4 Light/Dark) with stable `test_id` stamps. |

## Cookbook crate (planned v1 additions)

These are the next "small but high-signal" recipes to add, in easy -> hard order.

| ID | State | Target anchors | Notes |
|---|---|---|---|
| cookbook.docking_basics | Planned | `apps/fret-cookbook/examples/docking_basics.rs` | Minimal docking surface: core model ops + UI policy via `fret-docking`. |
| cookbook.canvas_pan_zoom_basics | Planned | `apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs` | `fret-canvas` pan/zoom + hit-testing + selection basics. |
| cookbook.chart_interactions_basics | Planned | `apps/fret-cookbook/examples/chart_interactions_basics.rs` | `fret-chart` or `fret-plot` interactions (hover/drag) with a minimal diag script. |
| cookbook.gizmo_basics | Planned | `apps/fret-cookbook/examples/gizmo_basics.rs` | `fret-gizmo` wiring + viewport-style transforms (native-first). |

## Interop + renderer “high ceiling” mapping

| ID | State | Current anchors | Run (today) | Notes |
|---|---|---|---|---|
| docking_arbitration | Maint | `apps/fret-examples/src/docking_arbitration_demo.rs`, `apps/fret-demo/src/bin/docking_arbitration_demo.rs` | `fretboard dev native --bin docking_arbitration_demo` | Editor-grade regression harness; keep out of onboarding. |
| embedded_viewport | Move | `apps/fret-examples/src/embedded_viewport_demo.rs`, `apps/fret-demo/src/bin/embedded_viewport_demo.rs` | `fretboard dev native --bin embedded_viewport_demo` | Candidate for Interop Track cookbook. |
| external_texture_import | Keep | `apps/fret-examples/src/external_texture_imports_demo.rs`, `apps/fret-demo/src/bin/external_texture_imports_demo.rs`, web: `apps/fret-examples/src/external_texture_imports_web_demo.rs` | native: `fretboard dev native --bin external_texture_imports_demo`; web: `fretboard dev web --demo external_texture_imports_web_demo` | Keep as canonical interop surface (native + web). |
| liquid_glass | Maint | `apps/fret-examples/src/liquid_glass_demo.rs`, `apps/fret-demo/src/bin/liquid_glass_demo.rs` | `fretboard dev native --bin liquid_glass_demo` | Renderer lab; likely stays native-first initially. |
| custom_effect_v1/v2/v3 | Maint | `apps/fret-examples/src/custom_effect_*`, `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/*` | native/web demos exist (see `apps/fret-demo-web/src/wasm.rs`) | Keep as “Labs”; gate by capabilities + budgets. |

## Web demo selection (current)

Web “demo IDs” live in:

- `apps/fretboard/src/demos.rs` (list of web demos)
- `apps/fret-demo-web/src/wasm.rs` (selection + wiring)

This duplication is a known drift risk; the v1 plan is to consolidate into a single source of truth
(see `catalog-source-of-truth.md`).
