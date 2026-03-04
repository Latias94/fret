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

Canonical cookbook index (Bevy-style tables + feature gates):

- [`apps/fret-cookbook/EXAMPLES.md`](../../../apps/fret-cookbook/EXAMPLES.md)

| ID | State | Current anchors | Run (today) | Notes |
|---|---|---|---|---|
| cookbook.hello | Keep | `apps/fret-cookbook/examples/hello.rs` | `cargo run -p fret-cookbook --example hello` | Minimal “hello” runnable. |
| cookbook.hello_counter | Keep | `apps/fret-cookbook/examples/hello_counter.rs` | `cargo run -p fret-cookbook --example hello_counter` | Small MVU + Model counter; stable `test_id` set. |
| cookbook.simple_todo | Keep | `apps/fret-cookbook/examples/simple_todo.rs` | `cargo run -p fret-cookbook --example simple_todo` | Minimal todo list (MVU + keyed rows) intended for copy/paste. |
| cookbook.toast_basics | Keep | `apps/fret-cookbook/examples/toast_basics.rs` | `cargo run -p fret-cookbook --example toast_basics` | Official. Sonner/Toaster integration; diag smoke exists (`cookbook-toast-basics`). |
| cookbook.date_picker_basics | Keep | `apps/fret-cookbook/examples/date_picker_basics.rs` | `cargo run -p fret-cookbook --example date_picker_basics` | Official. Minimal DatePicker wiring; diag smoke exists (`cookbook-date-picker-basics`). |
| cookbook.form_basics | Keep | `apps/fret-cookbook/examples/form_basics.rs` | `cargo run -p fret-cookbook --example form_basics` | Official. Minimal validation pattern (no form registry dep); diag smoke exists (`cookbook-form-basics`). |
| cookbook.drag_basics | Keep | `apps/fret-cookbook/examples/drag_basics.rs` | `cargo run -p fret-cookbook --example drag_basics` | Official. Pointer capture drag; diag smoke exists (`cookbook-drag-basics`). |
| cookbook.overlay_basics | Keep | `apps/fret-cookbook/examples/overlay_basics.rs` | `cargo run -p fret-cookbook --example overlay_basics` | Minimal dialog example with stable `test_id` stamps. |
| cookbook.commands_keymap_basics | Keep | `apps/fret-cookbook/examples/commands_keymap_basics.rs` | `cargo run -p fret-cookbook --example commands_keymap_basics` | Command registration + default keybinding + availability gating. |
| cookbook.undo_basics | Keep | `apps/fret-cookbook/examples/undo_basics.rs` | `cargo run -p fret-cookbook --features cookbook-undo --example undo_basics` | Lab (feature-gated). App-owned undo/redo history (`fret-undo`) wired to `edit.undo/edit.redo` commands. |
| cookbook.text_input_basics | Keep | `apps/fret-cookbook/examples/text_input_basics.rs` | `cargo run -p fret-cookbook --example text_input_basics` | Input submit/clear via commands (Enter/Escape) + numeric semantics gates. |
| cookbook.effects_layer_basics | Keep | `apps/fret-cookbook/examples/effects_layer_basics.rs` | `cargo run -p fret-cookbook --example effects_layer_basics` | Minimal effect layer example (Pixelate/Blur) with stable `test_id` stamps. |
| cookbook.icons_and_assets_basics | Keep | `apps/fret-cookbook/examples/icons_and_assets_basics.rs` | `cargo run -p fret-cookbook --features cookbook-assets --example icons_and_assets_basics` | Lab (feature-gated). Icon packs (lucide/radix) + semantic `ui.*` ids + file-based SVG/images via `fret-ui-assets`. |
| cookbook.assets_reload_epoch_basics | Keep | `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs` | `cargo run -p fret-cookbook --features cookbook-assets --example assets_reload_epoch_basics` | Lab (feature-gated). File image + SVG icon with ViewCache-safe dev reload (`UiAssetsReloadEpoch`). |
| cookbook.image_asset_cache_basics | Keep | `apps/fret-cookbook/examples/image_asset_cache_basics.rs` | `cargo run -p fret-cookbook --features cookbook-image-assets,cookbook-renderer --example image_asset_cache_basics` | Lab (feature-gated). Low-level ImageAssetCache keyed upload + evict/reload (deterministic, screenshot-friendly). |
| cookbook.canvas_pan_zoom_basics | Keep | `apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs` | `cargo run -p fret-cookbook --features cookbook-canvas --example canvas_pan_zoom_basics` | Lab (feature-gated). Pan/zoom wiring (`fret-canvas/ui`) + a tiny app-owned drag tool for one item. |
| cookbook.virtual_list_basics | Keep | `apps/fret-cookbook/examples/virtual_list_basics.rs` | `cargo run -p fret-cookbook --example virtual_list_basics` | Keyed virtualization + items_revision + scroll-to-item, with a minimal diag smoke script. |
| cookbook.query_basics | Keep | `apps/fret-cookbook/examples/query_basics.rs` | `cargo run -p fret-cookbook --features cookbook-query --example query_basics` | Lab (feature-gated). Query-style async resource state (`fret-query`): invalidate + namespace invalidation + error mode, with a deterministic baseline script. |
| cookbook.router_basics | Keep | `apps/fret-cookbook/examples/router_basics.rs` | `cargo run -p fret-cookbook --features cookbook-router --example router_basics` | Lab (feature-gated). Minimal router wiring (`fret-router` + `fret-router-ui`): links + outlet rendering + back/forward, with a deterministic smoke script. |
| cookbook.data_table_basics | Keep | `apps/fret-cookbook/examples/data_table_basics.rs` | `cargo run -p fret-cookbook --features cookbook-table --example data_table_basics` | Lab (feature-gated). shadcn-style DataTable + headless TanStack-aligned state (sorting/filtering/pagination) with virtualized rows. |
| cookbook.compositing_alpha_basics | Keep | `apps/fret-cookbook/examples/compositing_alpha_basics.rs` | `cargo run -p fret-cookbook --features cookbook-renderer --example compositing_alpha_basics` | Lab (feature-gated). Renderer semantics: straight vs premultiplied alpha, verified via screenshot baseline. |
| cookbook.drop_shadow_basics | Keep | `apps/fret-cookbook/examples/drop_shadow_basics.rs` | `cargo run -p fret-cookbook --features cookbook-renderer --example drop_shadow_basics` | Lab (feature-gated). Renderer semantics: DropShadowV1 bounded multi-pass effect; toggle + screenshot baseline. |
| cookbook.async_inbox_basics | Keep | `apps/fret-cookbook/examples/async_inbox_basics.rs` | `cargo run -p fret-cookbook --features cookbook-async --example async_inbox_basics` | Lab (feature-gated). Portable async pattern: background task → inbox → runner drain, with cancellation + progress semantics gate. |
| cookbook.markdown_and_code_basics | Keep | `apps/fret-cookbook/examples/markdown_and_code_basics.rs` | `cargo run -p fret-cookbook --features cookbook-markdown --example markdown_and_code_basics` | Preview (feature-gated). Markdown rendering + fenced code blocks (code-view/syntax) + copy affordance, with a minimal diag smoke script. |
| cookbook.theme_switching_basics | Keep | `apps/fret-cookbook/examples/theme_switching_basics.rs` | `cargo run -p fret-cookbook --example theme_switching_basics` | Minimal theme switching (shadcn New York v4 Light/Dark) with stable `test_id` stamps. |
| cookbook.docking_basics | Keep | `apps/fret-cookbook/examples/docking_basics.rs` | `cargo run -p fret-cookbook --features cookbook-docking --example docking_basics` | Lab (feature-gated). Minimal docking surface: retained dock host + app-owned panel registry + runner `dock_op` wiring. |
| cookbook.chart_interactions_basics | Keep | `apps/fret-cookbook/examples/chart_interactions_basics.rs` | `cargo run -p fret-cookbook --features cookbook-chart --example chart_interactions_basics` | Lab (feature-gated). Minimal chart wiring (`fret-chart` + `delinea`): shared engine + retained canvas + app-driven zoom, with a deterministic diag smoke script. |
| cookbook.gizmo_basics | Keep | `apps/fret-cookbook/examples/gizmo_basics.rs` | `cargo run -p fret-cookbook --features cookbook-gizmo --example gizmo_basics` | Lab (feature-gated). `fret-gizmo` wiring + viewport-style transforms (native-first). |
| cookbook.embedded_viewport_basics | Keep | `apps/fret-cookbook/examples/embedded_viewport_basics.rs` | `cargo run -p fret-cookbook --features cookbook-interop --example embedded_viewport_basics` | Lab (feature-gated). Embedded viewport surface: offscreen render target + `ViewportInputEvent` forwarding. |
| cookbook.external_texture_import_basics | Keep | `apps/fret-cookbook/examples/external_texture_import_basics.rs` | `cargo run -p fret-cookbook --features cookbook-interop --example external_texture_import_basics` | Lab (feature-gated). Imported render target updates (`ImportedViewportRenderTarget`) + viewport presentation. |
| cookbook.customv1_basics | Keep | `apps/fret-cookbook/examples/customv1_basics.rs` | `cargo run -p fret-cookbook --features cookbook-customv1 --example customv1_basics` | Lab (feature-gated). Custom effect v1: register bounded WGSL at `on_gpu_ready` and apply `EffectStep::CustomV1`. |

## Migration tracker: `fret-demo` lesson-shaped → cookbook

This table tracks the “move lesson-shaped demos out of `fret-demo`” intent (Readiness M4) as a
subset of this workstream.

| Source bin (`apps/fret-demo/src/bin/*`) | Target cookbook example | Status | Notes |
|---|---|---|---|
| `sonner_demo.rs` | `toast_basics` | Done | Reimplemented as a smaller lesson + diag smoke (`cookbook-toast-basics`). |
| `form_demo.rs` | `form_basics` | Done | Reimplemented as a minimal pattern (no form registry dependency) + diag smoke (`cookbook-form-basics`). |
| `drag_demo.rs` | `drag_basics` | Done | Reimplemented as pointer-capture drag + diag smoke (`cookbook-drag-basics`). |
| `date_picker_demo.rs` | `date_picker_basics` | Done | Reimplemented as a minimal controlled-model example + diag smoke (`cookbook-date-picker-basics`). |
| `datatable_demo.rs` | `data_table_basics` | Done | Reimplemented as a smaller lesson-shaped example + baseline screenshot (`cookbook-data-table-basics`). |
| `assets_demo.rs` | `assets_reload_epoch_basics` | Done | Reimplemented as a file-based example + reload epoch button + diag smoke (`cookbook-assets-reload-epoch-basics`). |
| `image_upload_demo.rs` | `image_asset_cache_basics` | Done | Reimplemented as a low-level cache lesson + baseline screenshot (`cookbook-image-asset-cache-basics`). |
| `drop_shadow_demo.rs` | `drop_shadow_basics` | Done | Reimplemented as a deterministic toggle surface + screenshot baseline (`cookbook-drop-shadow-basics`). |
| `alpha_mode_demo.rs` | `compositing_alpha_basics` | Done | Reimplemented as a low-level renderer semantics lesson + screenshot baseline (`cookbook-compositing-alpha-basics`). |
| `query_demo.rs` | `query_basics` | Done | Reimplemented as an action-first view runtime lesson + deterministic baseline (`cookbook-query-basics`). |
| `router_query_demo.rs` | `router_basics` | Done | Canonical cookbook: [`apps/fret-cookbook/examples/router_basics.rs`](../../../apps/fret-cookbook/examples/router_basics.rs) (feature-gated: `--features cookbook-router`). Legacy `router_query_demo` was removed after migration. |

Cleanup note:

- Most of the thin `apps/fret-demo/src/bin/*` wrappers for the migrated items were removed to reduce
  duplication.
- A small number remain as maintainer-grade probe harnesses (referenced by ADRs/workstreams), e.g.
  `drag_demo.rs`, `assets_demo.rs`, `drop_shadow_demo.rs`.

## Interop + renderer “high ceiling” mapping

| ID | State | Current anchors | Run (today) | Notes |
|---|---|---|---|---|
| docking_arbitration | Maint | `apps/fret-examples/src/docking_arbitration_demo.rs`, `apps/fret-demo/src/bin/docking_arbitration_demo.rs` | `fretboard dev native --bin docking_arbitration_demo` | Editor-grade regression harness; keep out of onboarding. |
| embedded_viewport | Keep | `apps/fret-cookbook/examples/embedded_viewport_basics.rs` (cookbook), ref: `apps/fret-examples/src/embedded_viewport_demo.rs` | `cargo run -p fret-cookbook --example embedded_viewport_basics` | Cookbook is the canonical entry; keep the larger demo as a maintainer-grade reference. |
| external_texture_import | Keep | native cookbook: `apps/fret-cookbook/examples/external_texture_import_basics.rs`; reference demos: `apps/fret-examples/src/external_texture_imports_demo.rs` + web `apps/fret-examples/src/external_texture_imports_web_demo.rs` | native: `cargo run -p fret-cookbook --example external_texture_import_basics`; web: `fretboard dev web --demo external_texture_imports_web_demo` | Cookbook is the canonical entry; keep the larger demos as maintainer-grade references (and web coverage). |
| liquid_glass | Maint | `apps/fret-examples/src/liquid_glass_demo.rs`, `apps/fret-demo/src/bin/liquid_glass_demo.rs` | `fretboard dev native --bin liquid_glass_demo` | Renderer lab; likely stays native-first initially. |
| custom_effect_v1/v2/v3 | Maint | `apps/fret-examples/src/custom_effect_*`, `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/*` | native/web demos exist (see `apps/fret-demo-web/src/wasm.rs`) | Keep as “Labs”; gate by capabilities + budgets. |

## Web demo selection (current)

Web “demo IDs” live in:

- `apps/fretboard/src/demos.rs` (list of web demos)
- `apps/fret-demo-web/src/wasm.rs` (selection + wiring)

This duplication is a known drift risk; the v1 plan is to consolidate into a single source of truth
(see `catalog-source-of-truth.md`).
