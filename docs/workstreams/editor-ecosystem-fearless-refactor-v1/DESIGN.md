# Editor ecosystem fearless refactor v1 - design

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

Status: Active orchestration workstream

Last updated: 2026-03-27

## Purpose

This workstream is the coordination layer for Fret's editor ecosystem.
Its job is to make the editor-facing crate stack boring to reason about:

- one obvious place for reusable editor widgets,
- one obvious place for shell chrome,
- one obvious place for dock-aware policy,
- one story for skinning and presets,
- and one extraction rubric for app-level editor protocols.

This document is intentionally shorter and more execution-oriented than the earlier draft.
Detailed audits and reference notes still exist in the same directory, but they are supporting
material rather than the primary execution surface.

## Why a reset was needed

The repo already has most of the right pieces:

- mechanisms in `crates/*`,
- policy-heavy ecosystems in `ecosystem/*`,
- a small immediate-style frontend in `fret-imui`,
- reusable editor widgets in `fret-ui-editor`,
- shell chrome in `fret-workspace`,
- docking policy in `fret-docking`,
- and real proof surfaces in demos/apps.

The remaining risk is no longer "do we have the right crates?".
The risk is that editor-facing work drifts because documentation is split across too many
workstreams and because "what should be built next" is not sharp enough.

## Goals

### G1 - Freeze boundaries before scaling the ecosystem further

Lock the ownership split across:

- authoring frontends,
- editor widgets and composites,
- shell chrome,
- docking policy,
- app-local protocols and services.

### G2 - Close the editor starter kit from behavior first

Use ImGui and egui as outcome references for:

- unified interaction-state visuals,
- dense but safe controls,
- predictable edit sessions,
- stable identity and response semantics,
- a credible editor starter set.

Do not copy their APIs or runtime model.

### G3 - Define a reusable editor component system

Fret needs more than a bag of controls.
It needs a component system with:

- a default visual baseline,
- a common state model,
- consistent density rules,
- and clear guidance on how shell/design-system skins should compose over it.

### G3a - Finish the baseline before scaling the component surface

The current execution mistake would be:

- adding more promoted controls while the default editor baseline is still visually weak,
- while typing/focus/invalid states are still inconsistent,
- and while proof surfaces do not reliably expose those states for review.

For this workstream, foundation-first means:

- baseline visuals and token hierarchy are reviewed first,
- property/inspector layout grammar is explicit,
- proof surfaces are composed so screenshots are actually reviewable,
- and only then does component expansion resume.

### G4 - Keep `crates/fret-ui` mechanism-only

Continue honoring ADR 0066:

- no editor policy in `crates/fret-ui`,
- no shell recipe defaults in `crates/fret-ui`,
- no design-system compensation knobs in runtime contracts.

### G5 - Leave behind a boring migration path

The end state should be teachable:

- declarative first,
- optional `imui` facade second,
- adapters seed tokens one-way,
- app-local protocol extraction is explicit rather than accidental.

## Non-goals (v1)

- Turning `fret-imui` into a second widget library or second runtime.
- Freezing every editor widget API before real in-tree adoption exists.
- Forcing a dedicated `fret-ui-editor-*` adapter crate immediately.
- Moving every `apps/fret-editor` module into ecosystem crates in one batch.
- Treating "pixel-perfect ImGui look" as the core goal.

## Decision snapshot (execution baseline)

- `fret-imui` remains a small, policy-light authoring frontend.
- `fret-ui-kit::imui` is allowed to host richer immediate-mode facade helpers so `fret-imui`
  remains minimal.
- `fret-ui-editor` is the single source of truth for reusable editor primitives, controls, and
  composites.
- `fret-ui-editor::imui` remains a thin adapter over the same underlying widget implementations.
- `fret-workspace` owns editor shell chrome and shell-level command/focus coordination.
- `fret-docking` owns dock-graph-aware interactions and chrome.
- Inspector/property protocols are not the same thing as editor widgets and should not be stuffed
  into `fret-ui-editor`.
- Editor/workspace theming remains ecosystem-owned string-token territory
  (`editor.*`, `workspace.*`) per ADR 0316.

## Ownership model (non-negotiable)

| Layer | Owns | Does not own |
| --- | --- | --- |
| `crates/fret-ui` | layout, focus/capture, overlays, text/input mechanisms, semantics, damage/invalidation correctness, theme fallback behavior | editor hand feel, shell defaults, design-system mappings, editor token policy |
| `ecosystem/fret-imui` | immediate-style syntax, keyed authoring helpers, lightweight response ergonomics | canonical editor widget logic, facade-only style vocabulary |
| `ecosystem/fret-ui-kit::imui` | richer immediate facade helpers and policy bridges that should not live in `fret-imui` | core reusable editor widget ownership |
| `ecosystem/fret-ui-editor` | editor primitives, controls, composites, editor-owned token families, thin `imui` adapters | dock-graph-aware policy, shell composition, app/product protocols, design-system dependencies |
| `ecosystem/fret-workspace` | frame, top bar, status bar, pane chrome, shell tabstrip, shell command/focus coordination | generic editor controls, docking behavior, design-system recipes |
| `ecosystem/fret-docking` | dock-aware tabs, insert markers, split previews, drop overlays, docking interactions | generic shell chrome or generic editor controls |
| app layer / future protocol crates | inspector/property protocol, edit services, project/asset/document services until extraction is justified | reusable editor widget chrome by default |

## Product-line strategy

Fret should treat the editor ecosystem as a stack, not as one giant crate:

1. authoring syntax,
2. reusable widget primitives and controls,
3. reusable composites,
4. shell chrome,
5. specialized editor surfaces,
6. protocols and app services.

This is important because the reference inspirations do not map 1:1:

- ImGui/egui teach editor interaction and starter-set completeness.
- shadcn/Base UI/Radix teach composition and adapter discipline.
- Zed/GPUI teach product-level shell quality and editor-app framing.

The correct synthesis is not "pick one upstream and copy it".
It is "assign each lesson to the right Fret layer".

## Component-system strategy

The next stage of this workstream is not mainly about adding more crates.
It is about making `fret-ui-editor` and adjacent shell crates behave like one coherent component
system.

The design baseline is captured in:

- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_COMPONENT_SYSTEM.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_INTERACTION_CONTRACT.md`

Current recommended stance:

- default reusable editor visuals should target a neutral engineering baseline,
- `imgui_like_dense` remains an optional preset proving density and hand-feel,
- shell and docking should align visually through adapter seeding rather than ownership collapse,
- new components should be admitted only when they satisfy interaction, layout, and gate criteria.

## Current execution pivot (March 2026)

The workstream is now explicitly foundation-first.

The next tranche of work should prioritize:

1. fixing the default editor baseline,
2. converging shared widget-state visuals,
3. making inspector/property layout rules explicit,
4. improving proof-surface composition so screenshots are actually useful,
5. and locking those outcomes with diagnostics + screenshot gates.

This is a deliberate sequencing choice.
The repo already has enough component surface area to expose baseline problems.
Pushing more promoted components before those problems are fixed will increase drift rather than
reduce it.

### Current foundation status (2026-03-11)

Recent baseline work clarified the next bottleneck:

- editor preset replay now survives host-owned shadcn theme reapply / window-metrics-driven theme
  sync,
- `imui_editor_proof_demo` now supports a review-only composition via
  `FRET_IMUI_EDITOR_PROOF_LAYOUT=editor_review`,
- the default screenshot proof uses that mode so overview / typing / validation captures focus on
  the editor inspector instead of mixing in parity and docking surfaces,
- shared `InspectorLayoutMetrics` now drive property row / grid / group / panel spacing and slot
  widths, so the inspector grammar is no longer spread across per-composite magic numbers,
- `PropertyRow` now encodes one reusable lane model
  (`label -> value -> reset slot -> status/actions slot`), while special rows can explicitly
  collapse trailing slots when needed,
- the latest `r15` screenshot proof now resets the search field before capture, and repeated runs
  in the same session reach `passed` while still emitting reviewable overview / typing / validation
  artifacts,
- and the remaining proof/gate gap is now narrow and explicit:
  make `fretboard-dev diag run --session-auto --launch` exit promptly after a successful run and reduce
  the repeated `global access while leased` warnings during typed-mode interactions.

### Foundation-first rule

Do not treat "new reusable component count" as the success metric for the next phase.

For promoted editor work, baseline infrastructure now comes first:

- `EditorWidgetVisuals` convergence,
- editor-owned token hierarchy and default-preset cleanup,
- editor preset replay after host-owned theme resets,
- layout grammar for rows/groups/status/reset slots,
- typed-edit / focus / invalid state clarity,
- proof/gate quality for overview, typing, and error states.

Only after those are in better shape should the workstream resume broad component promotion.

### Readiness checklist before new promoted components

Before this workstream advertises another reusable editor component as promoted, the following
should be true for the existing baseline:

1. the default editor preset is visually legible and reviewable,
2. focus, typing, active, and invalid states are distinguishable without reading source code,
3. proof surfaces expose the relevant states in screenshots/bundles without ad hoc manual setup,
4. starter-set controls follow one shared layout/state grammar,
5. and the interaction + screenshot gates are keeping that baseline from drifting.

## Relationship to adjacent workstreams

This workstream does not replace every editor-related document.
It coordinates them.

Use:

- `docs/workstreams/ui-editor-v1/ui-editor-v1.md`
  for the canonical `fret-ui-editor` surface and starter-set direction
- `docs/workstreams/standalone/ui-editor-egui-imgui-gap-v1.md`
  for detailed editor capability gaps vs egui/ImGui
- `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
  for the current immediate-mode stack reset and ownership baseline
- `docs/workstreams/code-editor-ecosystem-v1/code-editor-ecosystem-v1.md`
  for the code/text editor surface

Use this workstream for:

- cross-crate ownership,
- component-system direction,
- shell/docking/editor alignment,
- extraction strategy,
- and the recommended execution order.

## v1 exit criteria

This workstream has done its job when:

1. the crate ownership story is boring and documented in one place,
2. the editor component system has a clear default baseline and landing checklist,
3. the editor starter set has a promoted proof surface, focused gates, and a stable interaction contract,
4. shell vs docking alignment is explicit and no longer causes ownership confusion,
5. the next extraction move for app-local inspector/property work is either scheduled or
   explicitly deferred with reasons.

## Sources of truth and references

### ADRs

- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0154-ecosystem-crate-taxonomy-glue-and-ui-kit-split-v1.md`
- `docs/adr/0160-unified-authoring-builder-surface-v1.md`
- `docs/adr/0185-code-editor-ecosystem-v1.md`
- `docs/adr/0270-theme-token-contract-tiers-and-missing-token-policy-v1.md`
- `docs/adr/0316-editor-ecosystem-token-namespaces-and-skinning-boundary-v1.md`

### Workstreams

- `docs/workstreams/ui-editor-v1/ui-editor-v1.md`
- `docs/workstreams/standalone/ui-editor-egui-imgui-gap-v1.md`
- `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/code-editor-ecosystem-v1/code-editor-ecosystem-v1.md`

### Primary notes in this directory

- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_COMPONENT_SYSTEM.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_INTERACTION_CONTRACT.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/WORKSPACE_SHELL_STARTER_SET.md`

### Supporting notes in this directory

- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/OWNERSHIP_AUDIT.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/TOKEN_INVENTORY.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/IMGUI_LIKE_PRESET.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/PARITY_MATRIX.md`
