# Real application probes

Use this file to keep framework-consumer audits grounded in real product needs instead of toy demos.

Rule:

- `todo` is acceptable as a warm-up or baseline.
- It is not enough to close a framework-consumer audit by itself.
- For any meaningful audit, pick at least one real application probe below.

## 1) Editor notes workbench

Use when:

- you want a realistic editor-style shell without jumping straight to a huge product
- you need side rails, inspector content, text editing, and theme/chrome cohesion

Why it is high-signal:

- exposes shell composition, inspector ergonomics, text-field behavior, and token rhythm
- close to the “editor-grade app” positioning of Fret

Good capability coverage:

- workspace shell
- inspector/property composition
- state ownership across multiple panes
- focus and keyboard flow
- visual/chrome cohesion

Repo anchors:

- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/editor_notes_device_shell_demo.rs`
- `apps/fret-examples/tests/editor_notes_device_shell_surface.rs`

## 2) Workspace shell / IDE-lite

Use when:

- you need to test the product shape Fret claims as a first-class target
- tab strips, file tree, pane layout, dirty-close policy, commands, and docking all matter

Why it is high-signal:

- catches the gap between “component library works” and “editor shell actually feels coherent”
- quickly exposes layering mistakes and public-surface drift

Good capability coverage:

- docking and pane layout
- command routing
- workspace chrome
- multi-panel composition
- diagnostics ownership for editor-grade surfaces

Repo anchors:

- `docs/workspace-shell.md`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`

## 3) Markdown / knowledge viewer

Use when:

- you want a content-heavy real app rather than a pure control showcase
- async resource loading, rich text, links, anchors, and preview behavior matter

Why it is high-signal:

- combines document rendering, async work, asset loading, scrolling, and interaction semantics
- useful for detecting “works in simple forms, breaks in document apps” problems

Good capability coverage:

- async/query surfaces
- markdown rendering
- code blocks and content layout
- scroll/anchor navigation
- remote image and asset handling

Repo anchors:

- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-cookbook/examples/markdown_and_code_basics.rs`
- `docs/adr/0099-markdown-rendering-streaming-and-injection.md`

## 4) Data-heavy admin surface

Use when:

- you need to know whether large tables, pagination, sorting, and selection stay ergonomic
- app authors are likely to build CRUD/admin/internal tools with Fret

Why it is high-signal:

- reveals state shape costs, virtualization/selection friction, and headless table interop gaps
- often exposes whether “simple demo ergonomics” survive realistic data volume

Good capability coverage:

- large list/table state
- pagination/sorting/selection
- headless engine integration
- command and keyboard affordances

Repo anchors:

- `apps/fret-examples/src/datatable_demo.rs`
- `apps/fret-cookbook/examples/data_table_basics.rs`
- `docs/adr/0100-headless-table-engine.md`
- `docs/workstreams/headless-table-tanstack-parity/headless-table-tanstack-parity.md`

## 5) Asset browser / preview surface

Use when:

- the product needs images, SVGs, asset lifecycle, reloads, previews, or cache behavior
- you want to test whether “golden path” asset stories remain pleasant in app code

Why it is high-signal:

- asset setup often looks fine in docs but becomes awkward in real app composition
- catches cache/setup/budget/config drift early

Good capability coverage:

- asset registration and budgets
- preview components
- async/resource lifecycle
- app setup and startup wiring

Repo anchors:

- `apps/fret-examples/src/assets_demo.rs`
- `apps/fret-cookbook/examples/icons_and_assets_basics.rs`
- `apps/fret-cookbook/examples/app_owned_bundle_assets_basics.rs`
- `docs/adr/0112-ui-assets-facade-and-golden-path-wiring.md`

## 6) Node graph / canvas editor

Use when:

- your audit must cover one of Fret's high-ceiling ecosystem bets
- pointer interaction, overlays, searchers, commands, and canvas-space math all matter

Why it is high-signal:

- this is where “editor-grade” claims are hardest to fake
- exposes interaction policy, diagnostics quality, perf, and ecosystem layering all at once

Good capability coverage:

- canvas interaction
- searcher overlays
- keyboard routing
- selection and drag behavior
- high-ceiling ecosystem composition

Repo anchors:

- `apps/fret-examples/src/node_graph_demo.rs`
- `docs/node-graph-roadmap.md`
- `docs/adr/0126-node-graph-editor-and-typed-connections.md`

## Selection guidance

Pick probes by product relevance, not by novelty:

- If Fret is being judged as an editor-grade framework, start with `Editor notes workbench` or `Workspace shell / IDE-lite`.
- If the team needs broad app coverage, pair one editor-style probe with one content/data probe.
- If the audit is about ecosystem claims, add `Node graph / canvas editor` or `Asset browser / preview surface`.

Recommended minimum set for a serious audit:

1. one shell-oriented probe
2. one content/data-oriented probe
3. optional one high-ceiling ecosystem probe if that claim matters to the product
