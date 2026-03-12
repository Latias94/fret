# ADR 0316: Editor Ecosystem Token Namespaces and Skinning Boundary v1

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Dear ImGui: `repo-ref/imgui`
- egui: `repo-ref/egui`
- shadcn/ui: `repo-ref/ui`
- Base UI: `repo-ref/base-ui`
- Radix primitives: `repo-ref/primitives`

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

Status: Accepted

## Context

Fret targets editor-grade UI and already has a layered ecosystem structure:

- `crates/fret-ui` is the mechanism/runtime contract layer (ADR 0066).
- `ecosystem/fret-ui-editor` is the emerging reusable editor control surface.
- `ecosystem/fret-workspace` is the editor shell/chrome layer.
- `ecosystem/fret-docking` owns dock-graph-aware interaction policy and chrome.
- `ecosystem/fret-imui` is an authoring frontend, not a second runtime.

At the same time, the repo is actively growing multiple design-system layers and token strategies:

- shadcn-aligned recipes and presets,
- future Material-style adapters,
- app-specific themes and presets,
- cross-ecosystem token migration work (ADR 0270, theme-token workstreams).

Without a boundary decision, editor ecosystem growth can drift in two bad directions:

1. editor/workspace styling decisions leak into `crates/fret-ui`,
2. or `fret-ui-editor` / `fret-workspace` start depending directly on shadcn/material recipe crates.

Both directions make the ecosystem harder to evolve and harder to reuse outside one skin.

This ADR locks the v1 namespace and skinning boundary before the editor ecosystem scales further.

## Goals

1. Keep `crates/fret-ui` mechanism-only for editor/workspace theming concerns.
2. Define ownership of editor/workspace token namespaces.
3. Define how design-system skins adapt editor/workspace surfaces without reverse dependencies.
4. Keep missing-token behavior compatible with ADR 0270.
5. Keep `imui` and declarative authoring on the same token vocabulary.

## Non-goals

- Freezing the final public API of every editor or workspace component.
- Defining a full design-system palette or theme file schema for all apps.
- Renaming existing docking token families if they already have a stable consumer path.
- Deciding the exact adapter crate name for every future design system.

## Decision

### 1) Editor/workspace token namespaces are ecosystem-owned, not mechanism-owned typed core keys

In v1:

- `editor.*` is owned by `ecosystem/fret-ui-editor`.
- `workspace.*` is owned by `ecosystem/fret-workspace`.
- docking-specific tokens remain owned by `ecosystem/fret-docking`.

`crates/fret-ui` continues to own:

- token resolution mechanics,
- typed baseline theme keys,
- missing-token behavior,
- diagnostics-visible token lookup behavior.

`crates/fret-ui` does not gain typed core keys for editor/workspace-specific styling in v1.

Rationale:

- editor/workspace styling is policy-heavy and still evolving,
- freezing those keys as mechanism-owned typed runtime keys is premature,
- string-token namespaces are sufficient when paired with ADR 0270 fallback behavior.

### 2) Skinning is adapter-based and one-way

Allowed dependency direction:

- design-system recipe/preset crates
  -> depend on `fret-ui-editor` and/or `fret-workspace`
  -> seed or alias `editor.*` / `workspace.*`
  -> optionally provide higher-level recipes

Forbidden dependency direction:

- `fret-ui-editor`
  -> `fret-ui-shadcn`
  -> `fret-ui-material3`
  -> app-specific theme crates

- `fret-workspace`
  -> `fret-ui-shadcn`
  -> `fret-ui-material3`
  -> app-specific theme crates

The exact adapter landing site is intentionally flexible in v1:

- it may live in `fret-ui-shadcn`,
- in a dedicated editor/workspace adapter crate,
- or in another ecosystem preset module.

What is normative is the direction: skin crates depend on editor/workspace crates, not the reverse.

### 3) Namespace design should stay outcome-oriented and small

Recommended direction:

- `editor.*` for control density, visuals, edit-session-adjacent metrics, property surfaces, axis colors, and similar editor-control concerns.
- `workspace.*` for shell frame, top bar, status bar, pane chrome, and shell-level tabstrip chrome that is not dock-graph-specific.
- docking-owned tokens for drop overlays, split previews, drag insertion markers, and other dock-graph-aware chrome.

This ADR does not require a complete token table today.
It does require that new families have a clear owner and do not silently drift into the wrong crate.

Examples of acceptable families:

- `editor.density.*`
- `editor.numeric.*`
- `editor.property.*`
- `workspace.frame.*`
- `workspace.status_bar.*`
- `workspace.tabstrip.*` (only for non-dock-aware shell tabstrip chrome)

Examples of token families that should stay docking-owned:

- docking drop overlay colors,
- split preview colors,
- tab insert markers,
- dock-graph-aware drag affordance visuals.

### 4) Missing editor/workspace tokens must degrade safely per ADR 0270

Editor/workspace token lookups are string-token lookups and therefore follow ADR 0270:

- no panics by default,
- warn-once diagnostics,
- stable fallback behavior,
- optional strict mode via `FRET_STRICT_RUNTIME=1`.

This applies equally to:

- declarative editor/workspace components,
- `imui` facades,
- and design-system adapters reading or seeding these namespaces.

### 5) `imui` and declarative widgets share the same token vocabulary

`fret-imui` and `fret-ui-editor::imui` must not introduce a separate editor styling vocabulary.

Rejected:

- `imui.editor.*`
- facade-only editor tokens
- separate `imui` defaults that bypass declarative widget visuals

Accepted:

- `imui` as an authoring syntax,
- the same `editor.*` / `workspace.*` namespaces,
- the same single source-of-truth widget implementations underneath.

### 6) App-layer prototypes may experiment, but promotion requires namespace ownership discipline

`apps/fret-editor` and other app shells may temporarily use local tokens while exploring new surfaces.

However, promoting an app-local token family into reusable ecosystem crates requires:

1. a clear owning crate,
2. a demonstrated reusable surface,
3. and compatibility with the one-way skinning rule above.

This prevents app-specific product vocabulary from silently becoming a framework-level token contract.

## Consequences

### Positive

- `fret-ui-editor` and `fret-workspace` stay reusable across design systems.
- `crates/fret-ui` stays free of editor/workspace policy drift.
- shadcn/material/custom-app skins can converge on the same editor/workspace crates.
- `imui` does not fork the styling story.

### Costs / risks

- More namespace discipline is required when adding new editor/workspace surfaces.
- Some existing styling decisions may need to move out of app crates or recipe crates to line up with the new ownership rule.
- Workspace tabstrip and docking chrome boundaries will still need careful review to avoid overlap.

## Alternatives considered

### A) Put editor/workspace token families into typed core runtime keys now

Rejected for v1:

- the surface is still evolving quickly,
- typed core keys would freeze policy too early,
- and the ecosystem already has a viable string-token + fallback strategy via ADR 0270.

### B) Let `fret-ui-editor` depend directly on `fret-ui-shadcn`

Rejected:

- this would make the editor crate design-system-specific,
- reduce reuse for non-shadcn apps,
- and make future Material/custom skins harder to support cleanly.

### C) Let each authoring frontend define its own editor token vocabulary

Rejected:

- it would create styling drift,
- duplicate maintenance,
- and break the "one widget implementation, multiple authoring syntaxes" direction.

## Implementation guidance (non-normative)

Primary workstream:

- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/TODO.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/MILESTONES.md`

Evidence anchors for current crate roles:

- `ecosystem/fret-imui/src/lib.rs`
- `ecosystem/fret-ui-editor/src/lib.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
- `ecosystem/fret-workspace/src/lib.rs`
- `apps/fret-editor/src/lib.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `tools/diag-scripts/ui-editor/imui/imui-editor-proof-authoring-parity-shared-models.json`

Related contracts and workstreams:

- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0154-ecosystem-crate-taxonomy-glue-and-ui-kit-split-v1.md`
- `docs/adr/0270-theme-token-contract-tiers-and-missing-token-policy-v1.md`
- `docs/workstreams/theme-token-alignment-v1/design.md`
- `docs/workstreams/ui-editor-v1/ui-editor-v1.md`
