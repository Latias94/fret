# GPUI Ergonomics and Boundary Audit (2026-07)

This audit compares Fret against the pinned Zed/GPUI reference under `repo-ref/zed` from the
perspective of a general-purpose Rust GUI framework. It is an audit note, not an ADR.

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

## Immediate Documentation Fixes

- Keep the default onboarding ladder as `hello` -> `simple-todo` -> `todo`.
- Route realistic app probes explicitly as probes, not first-contact examples.
- Add a short link from `docs/ui-ergonomics-and-interop.md` to this audit.
- Add a real app probe table to `docs/examples/README.md`.

## Gates For Future Code Refactors

- `cargo fmt`
- `python3 tools/check_surface_policy.py`
- `python3 tools/gate_examples_source_tree_policy.py`
- `cargo nextest run -p fret-ui`
- `cargo nextest run -p fret-app`
- `cargo nextest run -p fret-ui-shadcn`
- `python3 tools/check_layering.py`
- For app-facing frame or workspace changes, run or update:
  `cargo run -p fretboard-dev -- diag suite ui-gallery-workspace-shell --launch -- cargo run -p fret-ui-gallery --release`.
