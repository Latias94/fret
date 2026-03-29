# imui editor-grade surface closure v1 - design

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Dear ImGui: `repo-ref/imgui`
- egui: `repo-ref/egui`
- Zed / GPUI: `repo-ref/zed`, `repo-ref/gpui-component`

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

Status: Proposed active workstream

Last updated: 2026-03-29

## Purpose

This workstream is the immediate follow-on to
`docs/workstreams/imui-stack-fearless-refactor-v1/`.

The stack-reset workstream already closed the structural cleanup:

- minimal `fret-imui`,
- canonical `fret-ui-kit::imui` naming,
- thin `fret-ui-editor::imui` adapters for the promoted editor control starter set,
- and explicit contract gates for `UiWriter` / `Response`.

What remains is no longer "surface coherence".
What remains is **editor-grade closure**:

- the set of immediate-mode helpers and adapters still needed to make inspector/outliner/tool-panel
  authoring feel competitive with Dear ImGui and egui,
- without reopening compatibility baggage or turning `fret-imui` into a giant widget crate.

## Current assessment

Relative to Dear ImGui and egui, the current Fret `imui` surface is already strong on:

- floating windows and in-window overlays,
- popup/menu behavior,
- ID/keyed identity hygiene,
- response-query semantics,
- basic form controls,
- and editor scalar/vector controls.

The highest-value remaining gaps are mostly **editor skeleton surfaces**, not more primitive control
count.

Today the main missing or underpowered areas are:

- no thin immediate adapters for editor composites such as `PropertyGroup`, `PropertyGrid`,
  `PropertyGridVirtualized`, and `InspectorPanel`,
- no first-class generic immediate tree / collapsing-header surface,
- no first-class immediate tooltip helper even though the hover/query substrate already exists,
- no explicit immediate drag-and-drop payload surface for editor/outliner workflows,
- and no documented owner split for which "shell-like" immediate surfaces belong in `imui` versus
  docking/workspace-owned crates.

So the answer is:

- the direction remains correct,
- the primitive starter set is already respectable,
- but the editor-grade surface is still incomplete where real tool UIs are actually composed.

## Why this follow-on should exist

After the fearless reset, the main risk is no longer a messy API.

The new risk is subtler:

- primitive controls are good enough that the stack *looks* complete,
- but actual editor screens still require too much declarative boilerplate around inspectors,
  collapsible groups, outliners, tooltips, and drag targets,
- which makes `imui` feel less productive than the upstream references in real editor workflows.

That should be fixed with a focused closure pass, not by reopening the old compatibility-shrink lane.

## Goals

### G1 - Close editor skeleton gaps before adding more primitive count

This workstream prioritizes:

- inspector/property composites,
- tree/collapsing surfaces,
- tooltip helpers,
- and drag/drop authoring seams

ahead of adding more one-off leaf widgets.

### G2 - Keep ownership explicit by layer

The new surfaces must land in the right crate:

- `fret-imui`: minimal frontend only,
- `fret-ui-kit::imui`: generic immediate helpers and policy wrappers,
- `fret-ui-editor::imui`: thin adapters for editor-specific declarative controls/composites,
- docking/workspace crates: shell policy such as docking tab bars, tear-off rules, and workspace tab
  choreography.

### G3 - Preserve single source of truth

If a new `imui` surface represents an existing declarative editor composite, the `imui` side must
stay a thin adapter.

If an adapter cannot stay thin, fix the declarative owner first instead of growing a second local
implementation.

### G4 - Accept fearless refactors and avoid compatibility residue

This workstream explicitly allows breaking migrations.

If a first cut lands on the wrong layer or teaches the wrong noun, delete or replace it instead of
adding aliases.

### G5 - Improve hand-feel without cloning ImGui/egui grammar

The goal is outcome parity for editor workflows, not source-level API mimicry.

We should reuse the lessons from Dear ImGui and egui while preserving Fret's layering and retained
runtime model.

## Non-goals

- Reopening the completed `imui-stack-fearless-refactor-v1` compatibility-shrink work.
- Adding a style stack (`PushStyleVar`, `PushStyleColor`, etc.) to `fret-imui`.
- Recreating "last item" implicit context as a default authoring rule.
- Moving docking/workspace shell policy into `fret-imui` or `fret-ui-kit::imui`.
- Adding a second implementation path for editor composites.
- Treating "more helper count" as the success metric.

## Non-negotiable boundaries

| Layer | Owns | Must not own |
| --- | --- | --- |
| `ecosystem/fret-imui` | minimal frontend, `UiWriter`-based composition, keyed identity helpers | editor composites, shell policy, compatibility helper bloat |
| `ecosystem/fret-ui-kit::imui` | generic immediate helpers such as layout, popup/menu, tooltip, tree/collapsing, generic drag/drop bridge if portable | editor-specific composites, docking/workspace shell policy |
| `ecosystem/fret-ui-editor::imui` | thin adapters for editor declarative controls and composites | adapter-local widget logic, duplicate state machines |
| `ecosystem/fret-docking` / workspace crates | docking tab bars, tear-off behavior, shell/workspace arbitration | generic immediate helper APIs pretending to be portable across non-docking apps |

## Decision snapshot

### 1) The next missing surfaces are composite-first

The next `imui` closure batch should start with:

- `PropertyGroup`
- `PropertyGrid`
- `PropertyGridVirtualized`
- `InspectorPanel`

These are the real editor authoring skeletons that currently force call sites back into lower-level
declarative composition.

### 2) Tree/collapsing belongs in `fret-ui-kit::imui`, not `fret-imui`

Tree nodes and collapsing headers are generic immediate authoring helpers, not editor-only controls,
so they should live in the richer generic facade layer.

Outliner-specific composition built from those helpers belongs in editor/workspace layers, not in
the generic frontend crate.

### 3) Tooltip should become first-class

The hover/query substrate already knows about tooltip-style delays and disabled-item hover rules.

What is missing is a first-class immediate helper that packages that behavior into a teachable
surface instead of forcing every caller to hand-wire it.

### 4) Drag/drop needs an explicit contract, not ad hoc action glue

An immediate drag/drop surface should only land if it can map cleanly onto the existing runtime drag
contracts.

If the portable boundary is not clean enough yet, we should write a defer note and keep the lane
explicitly open rather than shipping a stringly stopgap.

Status note (2026-03-29):

- M3 landed a clean first slice in `fret-ui-kit::imui`:
  `drag_source(...)` / `drop_target::<T>(...)`,
- the helper is response-driven and model-backed,
- and it deliberately avoids widening the object-safe runtime action-host seam just to create typed
  `DragSession` payloads.

### 5) Shell surfaces must remain explicitly partitioned

This workstream should evaluate shell-like immediate gaps, but it must not absorb docking/workspace
ownership by accident.

Rule:

- generic inspector/outliner authoring helpers: yes,
- docking tab bar / workspace shell policy: no, unless a smaller owner split is written down first.

## Target architecture

### `ecosystem/fret-ui-editor::imui`

Expected promoted adapter family:

- `property_group(...)`
- `property_grid(...)`
- `property_grid_virtualized(...)`
- `inspector_panel(...)`

Possible later thin adapters if evidence warrants them:

- `gradient_editor(...)`
- other editor composites that remain common on proof/demo surfaces

All of these should follow the same rule as the existing control adapters:

- accept the declarative owner type,
- forward through `UiWriter`,
- no adapter-local state or policy.

### `ecosystem/fret-ui-kit::imui`

Expected new generic helper family:

- `tooltip(...)` / `tooltip_with_options(...)`
- `collapsing_header(...)` or equivalent
- `tree_node(...)` or equivalent
- `drag_source(...)` / `drop_target::<T>(...)`

These helpers may contain policy glue, but they must remain generic and portable enough to justify
living outside editor-specific crates.

Stable tree/outliner authoring guidance:

- tree identity is an explicit `id` argument, not the visible label,
- nested items should use semantic, path-like ids (`scene/root/camera`, `assets/materials/pbr`)
  rather than ImGui-style `"##"` suffix tricks,
- hierarchy depth is explicit (`TreeNodeOptions::level`) rather than inferred from an implicit
  push/pop stack,
- and selection/open state should stay app-owned or model-owned instead of being smuggled through
  label text.

### Proof surfaces

The closure work should prove itself on:

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-demo/src/bin/imui_editor_proof_demo.rs`

If a proposed helper cannot make those proof surfaces materially simpler or clearer, it is probably
not the right next `imui` surface.

## Success criteria

This workstream is successful when:

- editor proof/demo surfaces can express inspector/outliner patterns without dropping back to noisy
  declarative wrapper glue,
- the next missing `imui` surfaces are obviously the right crate owners,
- no new compatibility aliases are introduced,
- and the resulting authoring story feels closer to Dear ImGui / egui in editor workflows without
  compromising Fret's layering.
