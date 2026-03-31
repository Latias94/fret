# imui compatibility retained surface reduction v1 - design

Status: Closed closeout record

Last updated: 2026-03-31

Previous closeout:
`docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`

Baseline audit:
`docs/workstreams/imui-compat-retained-surface-v1/BASELINE_AUDIT_2026-03-31.md`

Closeout audit:
`docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`

Status note (2026-03-31): this lane is complete. References below to `fret-node::imui` and
`fret-plot::imui` should now be read as historical/deleted surfaces. The surviving node-graph
compatibility proof is `apps/fret-examples/src/imui_node_graph_demo.rs`, and the surviving
declarative compatibility seam is
`fret_node::ui::declarative::node_graph_surface_compat_retained(...)`.

## Purpose

`imui-stack-fearless-refactor-v2` closed the generic vocabulary, editor adapter, and teaching
surface cleanup lane.

What remains open is narrower:

- compatibility-only retained surfaces that are still visible on first-party `imui` proof paths,
- public ecosystem `imui` modules that still host retained subtrees/canvases,
- and the question of which of those surfaces should survive as explicit compatibility facades
  versus which should be renamed, relocated, or deleted.

This lane exists to close that narrower question without reopening the whole retained-bridge exit
plan or the whole `imui` stack reset.

## Current assessment

The stack direction remains correct after the v2 closeout:

- generic immediate vocabulary belongs in `ecosystem/fret-ui-kit::imui`
- editor-owned immediate nouns belong in `ecosystem/fret-ui-editor::imui`
- runtime mechanisms remain in `crates/fret-ui`

The remaining drift is now specifically about retained compatibility surfaces:

- `apps/fret-examples/src/imui_node_graph_demo.rs` is intentionally compatibility-only and already
  says so
- `ecosystem/fret-node/src/imui.rs` still exposes raw retained subtree hosting as a public
  immediate authoring surface
- `ecosystem/fret-node/src/ui/declarative/compat_retained.rs` already provides a better-labeled
  declarative compatibility surface
- `ecosystem/fret-plot/src/imui.rs` exposes retained-canvas hosting from a public immediate module,
  but the compatibility-only status was not yet locked as explicitly as the node-graph path

## Why this needs a new lane

The old `retained-bridge-exit-v1` plan is too broad for the next decision.

That plan covers allowlists and crate-wide bridge removal across docking, node graph, chart, plot,
and plot3d.
This lane is intentionally smaller:

- only public/proof `imui` and adjacent first-party teaching surfaces,
- only compatibility-retained authoring surfaces,
- only the keep / relabel / delete decisions that affect what the repo teaches today.

Without this narrower lane, the repo keeps two bad outcomes alive:

- public `imui` compatibility facades can remain under-labeled and drift toward seeming normal,
- or delete-ready cleanup stalls because the compatibility surfaces were never explicitly classified.

## Goals

### G1 - Make compatibility-retained `imui` surfaces explicit and auditable

Every surviving retained-backed public `imui` surface must read as:

- compatibility-only,
- delete-planned,
- and non-default for new downstream authoring.

### G2 - Keep first-party proof surfaces honest

First-party `imui` examples may keep one explicit compatibility proof surface where needed, but they
must not silently normalize retained-bridge authoring as a default immediate-mode path.

### G3 - Prefer declarative compatibility surfaces over raw retained-hosting helpers when both exist

If a crate already has a declarative compatibility surface that hides retained internals better than
a raw `UiWriter` retained-subtree helper, the docs and first-party examples should teach the
declarative compatibility surface first.

### G4 - Leave runtime ownership unchanged

This lane does not widen `crates/fret-ui`.
If a retained compatibility surface is awkward, that does not justify adding new runtime policy or
new compatibility knobs to the mechanism layer.

### G5 - Leave behind source-policy gates

The lane should end with cheap source-policy gates that prevent:

- first-party `imui` examples from silently reintroducing retained-bridge authoring,
- and retained-backed ecosystem `imui` modules from drifting back into unlabeled default surfaces.

## Non-goals

- Reopening the generic `imui` vocabulary closure question.
- Reopening the editor adapter closure question already closed in v2.
- Executing the full retained-bridge exit plan for docking, charts, plots, and node graph.
- Changing `crates/fret-ui` runtime contracts in this lane.
- Preserving ambiguous compatibility surfaces without classification.

## Non-negotiable boundaries

| Layer | Owns | Must not own |
| --- | --- | --- |
| `crates/fret-ui` | retained-bridge mechanism while it still exists | public compatibility authoring policy, demo teaching posture |
| `ecosystem/fret-node` / `ecosystem/fret-plot` | explicit compatibility facades for retained-backed integrations when still needed | ambiguous default authoring guidance |
| `apps/fret-examples` | one explicit proof surface when a retained-backed path still needs visible evidence | silent normalization of retained-bridge authoring as the default `imui` path |
| workstream docs | keep / relabel / delete decisions plus evidence | implicit drift based on old inertia |

## Decision stance

### 1) Compatibility surfaces are allowed only when they are explicit

Retained-backed `imui` helpers may survive for a time, but only if they are unmistakably described
as compatibility-only and delete-planned.

### 2) One proof surface per retained compatibility family is enough

This lane prefers one explicit proof surface rather than multiple examples all implicitly teaching
the same retained escape hatch.

### 3) A better-labeled declarative compatibility surface should outrank a raw retained helper

For node graph specifically, `node_graph_surface_compat_retained(...)` is a clearer long-term
teaching surface than raw public `retained_subtree(...)` helpers.

### 4) `fret-plot::imui` needs an explicit classification, not silence

If the plot retained canvas path must survive for now, it should survive as an explicit
compatibility-only public surface.
If that classification is not acceptable, the surface should be renamed or moved instead of
remaining ambiguous.

## Current target closure set

### A. Entry-point reset

- point the top-level docs at this lane as the active `imui` follow-on
- keep v2 as the closed closeout record

### B. First-party proof gate

- keep `imui_node_graph_demo` as the only explicit retained-bridge `imui` example unless new
  evidence appears
- prevent other first-party `imui` demos from silently gaining retained-bridge hosting

### C. Ecosystem compatibility surface labeling

- make `ecosystem/fret-node/src/imui.rs` explicitly compatibility-only
- make `ecosystem/fret-plot/src/imui.rs` explicitly compatibility-only
- add source-policy tests so this labeling does not drift

### D. Delete-ready follow-up decisions

- decide whether `fret_node::imui::retained_subtree*` survives as-is, is renamed under a narrower
  compatibility name, or is deleted after first-party migration
- decide whether `fret_plot::imui::line_plot_canvas*` survives as an explicit compatibility facade
  or should be renamed / relocated

## Primary evidence anchors

- `apps/fret-examples/src/imui_node_graph_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `ecosystem/fret-node/src/imui.rs`
- `ecosystem/fret-node/src/ui/declarative/compat_retained.rs`
- `ecosystem/fret-node/README.md`
- `ecosystem/fret-plot/src/imui.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/design.md`

## Definition of done

This lane is done when:

- docs entrypoints no longer point to v2 as if it were still active,
- only explicit compatibility proof surfaces teach retained-backed `imui` authoring,
- retained-backed public `imui` modules are clearly labeled and source-gated as compatibility-only,
- and the remaining survival/delete decisions are written down in a final audit rather than left as
  ambient inertia.
