# Editor Ecosystem Fearless Refactor v1 - Design

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Dear ImGui: `repo-ref/imgui`
- egui: `repo-ref/egui`
- Zed / GPUI: `repo-ref/zed`, `repo-ref/gpui-component`
- shadcn/ui: `repo-ref/ui`
- Base UI: `repo-ref/base-ui`
- Radix primitives: `repo-ref/primitives`

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

Status: Proposed (orchestration workstream; ADRs remain the source of truth)
Last updated: 2026-03-09

This workstream is the coordination layer for Fret's editor ecosystem refactor.

It does not replace the existing notes immediately. Instead, it ties together the currently
separate threads that already exist in-tree:

- editor controls and gaps:
  - `docs/workstreams/ui-editor-v1.md`
  - `docs/workstreams/ui-editor-v1-todo.md`
  - `docs/workstreams/ui-editor-imgui-alignment-v1.md`
  - `docs/workstreams/ui-editor-egui-imgui-gap-v1.md`
- immediate authoring boundary:
  - `docs/workstreams/imui-authoring-facade-v2.md`
- workspace shell and docking follow-ups:
  - `docs/workstreams/workspace-crate-boundaries-v1.md`
  - `docs/workstreams/workspace-shell-tabstrip-fearless-refactor-v1/DESIGN.md`
  - `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`
- theme/token layering:
  - `docs/workstreams/theme-token-alignment-v1/design.md`

The missing piece is a single plan that answers:

1. which crate should own which editor-grade surface,
2. how `imui` fits without becoming a second runtime or second widget library,
3. how editor/workspace theming should work without coupling `fret-ui-editor` to shadcn/material,
4. which app-layer prototypes should move into ecosystem crates and which should stay app-specific.

---

## 0) Why this needs a fearless refactor

Fret already has the right raw ingredients for editor-grade UI:

- mechanism/runtime contracts in `crates/*`,
- policy-heavy ecosystems in `ecosystem/*`,
- an immediate-style authoring frontend (`fret-imui`),
- an editor control crate (`fret-ui-editor`),
- a workspace shell crate (`fret-workspace`),
- docking policy separated from core ops (`fret-docking`),
- and an app-layer incubator (`apps/fret-editor`).

What is still missing is a stable ownership model across those pieces.

Today the repo risks drifting in four ways:

1. `fret-imui` could accidentally grow from "authoring frontend" into a parallel widget library.
2. `fret-ui-editor` could become too design-system-specific if shadcn/material styling decisions leak
   into its core surface.
3. `fret-workspace` and `fret-docking` could duplicate tabstrip/shell chrome responsibilities.
4. `apps/fret-editor` could keep accumulating reusable editor building blocks without a clear
   extraction rule.

This workstream exists to stop that drift before the editor ecosystem becomes expensive to change.

---

## 1) Goals

### G1 - Lock crate ownership for editor-grade UI

Define a durable ownership split for:

- authoring frontends,
- editor primitives/controls/composites,
- workspace shell chrome,
- docking interaction policy,
- app-specific editor experiments and protocols.

### G2 - Build an editor library from outcomes, not API imitation

Use Dear ImGui and egui as outcome references for:

- dense but readable widgets,
- predictable edit sessions,
- stable identity rules,
- unified widget visuals/spacing,
- editor-grade starter-set completeness.

Do not attempt to copy their APIs or runtime model.

### G3 - Keep `fret-ui` mechanism-only

The refactor must continue to honor ADR 0066:

- no editor policy in `crates/fret-ui`,
- no design-system defaults in `crates/fret-ui`,
- no "just add a runtime knob" escape hatches for ecosystem styling drift.

### G4 - Make theming and skinning adapter-based

Define how editor/workspace surfaces are skinned by shadcn, Material, or app-specific themes without:

- adding reverse dependencies from editor crates into recipe crates,
- forcing editor tokens into typed core runtime keys too early,
- or creating a second token vocabulary just for `imui`.

### G5 - Leave behind a boring migration path

The end state must be teachable and maintainable:

- one source of truth per widget,
- one obvious place for shell chrome,
- one token namespace story,
- and a clear extraction rubric for app-layer prototypes.

---

## 2) Non-goals (v1)

- Turning `fret-imui` into a second runtime or a second canonical component library.
- Freezing a long-lived public API for every editor widget before in-tree adoption exists.
- Forcing a dedicated `fret-ui-editor-shadcn` crate immediately; the important part is the boundary,
  not the exact adapter crate name.
- Migrating every `apps/fret-editor` module out of the app layer in one batch.
- Defining a full editor product architecture for assets/projects/scene graphs; this workstream is
  about reusable UI/library boundaries first.

---

## 3) References (sources of truth)

### ADRs

- Mechanism vs policy boundary: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Unified builder / authoring surface: `docs/adr/0160-unified-authoring-builder-surface-v1.md`
- Ecosystem crate taxonomy: `docs/adr/0154-ecosystem-crate-taxonomy-glue-and-ui-kit-split-v1.md`
- Inspector/property protocol (app-layer example): `docs/adr/0048-inspector-property-protocol-and-editor-registry.md`
- Docking layering and split sizing:
  - `docs/adr/0075-docking-layering-b-route-and-retained-bridge.md`
  - `docs/adr/0077-resizable-panel-groups-and-docking-split-sizing.md`
- Code editor ecosystem split: `docs/adr/0185-code-editor-ecosystem-v1.md`
- Theme token tiers / missing token behavior: `docs/adr/0270-theme-token-contract-tiers-and-missing-token-policy-v1.md`
- This workstream's boundary ADR:
  - `docs/adr/0316-editor-ecosystem-token-namespaces-and-skinning-boundary-v1.md`

### Existing workstreams

- `docs/workstreams/ui-editor-v1.md`
- `docs/workstreams/imui-authoring-facade-v2.md`
- `docs/workstreams/theme-token-alignment-v1/design.md`
- `docs/workstreams/workspace-crate-boundaries-v1.md`

### This workstream's audit notes

- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/OWNERSHIP_AUDIT.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/PARITY_MATRIX.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/IMGUI_LIKE_PRESET.md`

---

## 4) Decision snapshot (v1)

These decisions are the execution baseline for this workstream:

- `fret-imui` remains a policy-light authoring frontend.
- `fret-ui-editor` is the single source of truth for reusable editor primitives, controls, and
  composites.
- `fret-ui-editor::imui` is a thin adapter layer over declarative implementations, not a second
  widget implementation tree.
- `fret-workspace` owns editor shell chrome (frame, panes, top/status bars, command scope,
  non-docking shell composition).
- `fret-docking` owns dock-graph-aware interaction policy and docking-specific tab/drop chrome.
- `apps/fret-editor` is an incubation zone for app-specific protocols and experiments; moving
  code into ecosystem crates requires an explicit extraction audit.
- Editor/workspace theming remains ecosystem string-token based (`editor.*`, `workspace.*`) and
  is skinned by adapters/preset seeding, not by reverse dependencies.

---

## 5) Ownership model (non-negotiable)

### 5.1 `crates/fret-ui`

Owns:

- layout/focus/capture/overlay/text/accessibility mechanisms,
- stable theme resolution behavior,
- diagnostics-visible identity and event routing primitives.

Does not own:

- editor hand feel,
- workspace shell chrome defaults,
- tabstrip style defaults,
- design-system mappings,
- editor token families as typed core keys in v1.

### 5.2 `ecosystem/fret-imui`

Owns:

- immediate-style authoring ergonomics,
- response objects and keyed identity helpers,
- thin mounting/building helpers that compile into declarative element output.

Does not own:

- canonical editor widget logic,
- editor-specific token namespaces,
- design-system defaults.

### 5.3 `ecosystem/fret-ui-editor`

Owns:

- editor primitives (`EditSession`, dense chrome, widget visuals),
- editor controls (`DragValue`, `Slider`, `EnumSelect`, `TextField`, `ColorEdit`, `VecNEdit`, ...),
- editor composites (`PropertyRow`, `PropertyGrid`, `InspectorPanel`, ...),
- editor token families (`editor.*`),
- a thin optional `imui` facade that delegates to the same widgets.

Does not own:

- dock-graph-specific interactions,
- workspace shell frame composition,
- app-specific project/asset/scene protocols,
- shadcn/material dependencies.

### 5.4 `ecosystem/fret-workspace`

Owns:

- shell composition for editor applications,
- workspace frame/top bar/status bar/pane composition,
- shell-level command scope and focus coordination,
- workspace token families (`workspace.*`).

Does not own:

- generic editor controls,
- dock graph behavior,
- design-system-specific recipe choices.

### 5.5 `ecosystem/fret-docking`

Owns:

- docking interaction policy,
- tab/drop/split behaviors tied to dock graphs,
- docking-specific chrome and tokens.

Important boundary:

- if a tabstrip surface is dock-graph-aware, it belongs to `fret-docking`;
- if it is generic editor shell chrome, it belongs to `fret-workspace`;
- if the look should match, that is a skinning/preset concern, not a crate-merging excuse.

### 5.6 `apps/fret-editor`

Owns:

- app-specific or not-yet-proven protocols,
- editor-product experiments,
- concrete scene/project/property editing flows that still need evidence before extraction.

Extraction rule:

- move code into ecosystem crates only when it is no longer tied to a specific app data model and
  has at least one additional in-tree consumer or a strong planned reuse path.

---

## 6) What to learn from ImGui/egui, and how to translate it into Fret

This refactor should borrow outcomes, not copy structure.

### 6.1 Dear ImGui teaches us

- editor widgets need a tight, predictable "hand feel",
- edit sessions are first-class (start/live/commit/cancel),
- explicit identity matters,
- dense UIs need coherent spacing and input affordances.

Fret translation:

- centralize edit-session behavior in `fret-ui-editor` primitives,
- prefer explicit `id_source` and stable keyed identity,
- keep scrubbing/typing/commit semantics shared across controls,
- validate behavior in proof demos and scripted gates.

### 6.2 egui teaches us

- widget visuals and spacing must resolve from one place,
- response objects must feel coherent across the widget set,
- a mature editor library is more about consistency than widget count.

Fret translation:

- `EditorWidgetVisuals` and density/chrome resolution belong in `fret-ui-editor`,
- `fret-imui` should expose ergonomic responses, but the underlying widget behavior must still come
  from the single declarative implementation,
- the starter set should be closed before scaling into long-tail widgets.

### 6.3 shadcn/Base UI/Radix teach us

- policy and recipes should sit above headless mechanisms,
- different skins can share the same structural/interaction primitives.

Fret translation:

- editor/workspace crates stay design-system agnostic,
- shadcn/material/custom-app skins seed or alias tokens and recipes from outside,
- no reverse dependencies from editor crates into recipe crates.

### 6.4 Zed / GPUI teach us

- editor apps need a strong shell architecture,
- authoring ergonomics and runtime boundaries must stay aligned,
- app shells and editor surfaces should scale without collapsing all concerns into one crate.

Fret translation:

- keep `fret-workspace`, `fret-docking`, `fret-ui-editor`, and app-specific editor protocols as
  distinct layers,
- let `imui` remain an authoring frontend rather than an architecture bucket.

---

## 7) Theme/token and skinning model

### 7.1 Namespace ownership

v1 namespace direction:

- `editor.*` -> owned by `fret-ui-editor`
- `workspace.*` -> owned by `fret-workspace`
- docking-specific tab/drop/split tokens -> owned by `fret-docking`
- generic semantic palette / baseline theme mechanisms -> owned by `crates/fret-ui`

Suggested families:

- `editor.density.*`
- `editor.numeric.*`
- `editor.property.*`
- `editor.field_status.*`
- `editor.axis.*`
- `workspace.frame.*`
- `workspace.top_bar.*`
- `workspace.status_bar.*`
- `workspace.pane.*`
- `workspace.tabstrip.*`

This does not require every family to exist immediately.
It does require that new token families have an owning crate and do not drift into `fret-ui`.

### 7.2 Skinning is one-way

Allowed direction:

- `fret-ui-shadcn` / future adapter crates
  -> depend on `fret-ui-editor` and/or `fret-workspace`
  -> seed or alias `editor.*` / `workspace.*`
  -> optionally provide recipes/presets

Forbidden direction:

- `fret-ui-editor` or `fret-workspace`
  -> depending on `fret-ui-shadcn`
  -> depending on `fret-ui-material3`
  -> depending on app crates for tokens

### 7.3 Missing-token behavior follows ADR 0270

New editor/workspace namespaces must:

- degrade safely by default,
- be diagnosable when missing,
- avoid panicking in default runtime mode.

This keeps ecosystem growth compatible with contract-tiered token resolution.

### 7.4 `imui` does not get a separate token language

The immediate-mode facade must consume the same token namespaces as declarative widgets.

Rejected direction:

- `imui.editor.*`
- facade-only style knobs that bypass the declarative implementation

Accepted direction:

- same `editor.*` / `workspace.*` families,
- same widget visuals,
- same fallbacks,
- different authoring syntax only.

---

## 8) Refactor tracks

### Track A - Ownership and extraction audit

Produce an explicit inventory for:

- `apps/fret-editor` modules,
- `fret-ui-editor` starter set,
- `fret-workspace` shell pieces,
- `fret-docking` overlaps.

The output should classify each surface as:

- stay app-specific,
- move to ecosystem now,
- or keep incubating until a second consumer exists.

### Track B - Authoring convergence

Close the gap between:

- declarative editor widgets,
- `fret-ui-editor::imui`,
- proof demos using `fret-imui`.

Definition of done for this track:

- no duplicate widget implementations,
- `fret-ui-editor::imui` is no longer a placeholder-only seam,
- response/test-id/id-source conventions are documented and exercised.

### Track C - Editor starter set and shell baseline

Converge on the smallest credible reusable baseline:

- editor control starter set in `fret-ui-editor`,
- workspace shell starter set in `fret-workspace`,
- docking-specific behaviors staying in `fret-docking`.

### Track D - Theme/token and skin adapters

Lock token ownership, seed rules, and adapter responsibilities for:

- shadcn-aligned skins,
- future material-style skins,
- app-specific presets.

### Track E - Proof harnesses, gates, and migration evidence

Treat proof surfaces as first-class architecture evidence:

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- workspace shell demos
- `apps/fret-editor` integration points

Each major move should leave behind:

- a runnable proof surface,
- at least one focused gate,
- evidence anchors back to the owning crate.

---

## 9) Migration and cleanup rules

This workstream is not complete until it leaves the repo simpler than before.

Cleanup rules:

- do not keep both app-local and ecosystem versions of the same reusable widget indefinitely,
- do not add editor policy to `crates/fret-ui` to "save time",
- do not let `fret-imui` grow adapter-only widgets that have no declarative owner,
- do not let workspace shell style choices force crate coupling.

Recommended cleanup sequence:

1. lock docs + boundary ADR,
2. classify ownership and extraction candidates,
3. land the thin `fret-ui-editor::imui` path,
4. move or delete duplicated app-local surfaces,
5. add/update proof harnesses and scripted gates,
6. keep docs pointing to one boring recommended path.

---

## 10) Expected v1 outcomes

If this workstream succeeds, the repo should end up with:

- a clear answer to "where does this editor UI code belong?",
- a reusable editor component layer that does not depend on a specific design system,
- a workspace shell layer that does not duplicate docking policy,
- a thin `imui` editor facade instead of a second widget tree,
- and a token/skinning story that can support shadcn, Material, and custom app presets without
  reopening `fret-ui` contracts.
