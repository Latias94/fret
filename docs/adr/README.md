# ADR Index (Module-Oriented)

This index groups ADRs by subsystem so that implementation work can quickly locate the relevant contracts.

Notes:

- ADR filenames keep their numeric IDs stable.
- Cross-references in `docs/architecture.md` should remain valid even as the code evolves.
- Implementation audit tracking: `docs/adr/IMPLEMENTATION_ALIGNMENT.md`.

See `docs/adr/0027-framework-scope-and-responsibilities.md` for the framework vs editor-app scope boundary.

Non-normative code references:

- Zed/GPUI is pinned under `repo-ref/zed/` and is used as an implementation reference for runtime
  substrate shapes (window/input dispatch, scheduling, overlays, theme, and settings).

## How To Use ADRs

- `Status: Accepted`: stable contract; implementations should conform.
- `Status: Proposed`: under active design; treat as a decision gate before scaling surface area (currently rare; prefer landing decisions as `Accepted` once locked).
- `Status: Deferred`: intentionally out of framework scope for now.

## Decision Backlog (What Still Needs Locking)

This section tracks "hard-to-change" decisions that are not fully locked yet, even if adjacent ADRs
exist. The goal is to turn each item into an `Accepted` contract (either by extending an existing
ADR or adding a new ADR) before scaling feature surface area.

### P0 (Lock before scaling the UI kit)

- **Renderer v3: postprocessing substrate + effect semantics**
  - Proposed: `docs/adr/0118-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`,
    `docs/adr/0119-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
  - Decide: public effect ops shape, ordering/clip/transform rules, and integration with the renderer plan.

- **Paint/Brush primitives (gradients, procedural patterns)**
  - Proposed: `docs/adr/1172-paint-primitives-brushes-and-gradients-v1.md`
  - Decide: minimal paint vocabulary (solid/gradients), coordinate + color space rules, and how this
    layers into controlled `MaterialId` extensibility (ADR 0125).

- **Imported render targets + external texture imports (staged, capability-gated)**
  - Proposed: `docs/adr/1173-imported-render-targets-and-external-texture-imports-v1.md`
  - Decide: end-to-end “contract path” validation, per-frame keepalive/lifetime rules, and how
    capability-gated zero-copy imports layer in without leaking backend handles (ADR 0124 / ADR 0121).

- **Controlled materials registry (Tier B procedural paints)**
  - Proposed: `docs/adr/1174-controlled-materials-registry-and-procedural-paints-v1.md`
  - Decide: registry surface (`MaterialId`), fixed param payload, budgets/telemetry, and deterministic fallbacks.

- **Compositing groups and blend modes (isolated saveLayer blends)**
  - Proposed: `docs/adr/1180-compositing-groups-and-blend-modes-v1.md`
  - Decide: minimal blend vocabulary for UI-native looks and deterministic degradation under budgets.

- **Sampled materials (v2; fixed binding shapes)**
  - Proposed: `docs/adr/1181-sampled-materials-and-fixed-binding-shapes-v2.md`
  - Decide: controlled “params + one image” expansion without opening arbitrary shader/resource graphs.

- **Pointer motion snapshots (velocity + paint-time reads)**
  - Proposed: `docs/adr/1182-pointer-motion-snapshots-and-move-coalescing-v1.md`
  - Decide: a non-reactive pointer snapshot seam that stays consistent under transforms and coalesced move delivery.

- **Procedural determinism (explicit seeds + explicit time)**
  - Proposed: `docs/adr/1183-procedural-material-determinism-seeds-and-time-inputs-v1.md`
  - Decide: no hidden time in Tier B materials; stable seeds; reduced-motion-friendly inputs.

- **Creative authoring surface (ecosystem recipes)**
  - Proposed: `docs/adr/1184-ecosystem-visual-recipes-and-creative-authoring-surface-v1.md`
  - Decide: `fret-ui-kit` as the canonical recipe/catalog layer that resolves to mechanism with capability/budget fallbacks.

- **Pointer coordinate spaces (window-local vs element-local, transform-aware)**
  - Proposed: `docs/adr/1177-pointer-coordinate-spaces-and-element-local-mapping-v1.md`
  - Decide: the canonical coordinate spaces exposed to widgets, capture semantics, and a minimal
    mechanism helper surface to avoid ad-hoc `position - bounds.origin` math across the ecosystem.

- **Mask layers (alpha masks beyond rect/rrect clipping)**
  - Proposed: `docs/adr/1178-mask-layers-and-alpha-masks-v1.md`
  - Decide: v1 mask stack shape, gradient-only portable sources, hit-testing semantics (mask is
    paint-only), and deterministic degradation under budgets.

- **Frame clock and reduced-motion gates (animation time base)**
  - Proposed: `docs/adr/1179-frame-clock-and-reduced-motion-gates-v1.md`
  - Decide: monotonic per-frame clock exposure without poisoning view-cache keys, and the canonical
    reduced-motion response pattern for ecosystem components.

- **User-facing effect recipes and tier selection (Tier A vs Tier B)**
  - Proposed: `docs/adr/0149-effect-recipes-and-tier-selection-v1.md`
  - Decide: recommended user story for postprocessing, and the stable recipe authoring pattern for `fret-ui-kit`.

- **Renderer budgets (intermediates + streaming uploads)**
  - Proposed: `docs/adr/0120-renderer-intermediate-budgets-and-effect-degradation-v1.md`,
    `docs/adr/0123-streaming-upload-budgets-and-backpressure-v1.md`
  - Decide: accounting scopes, deterministic degradation rules, and observability requirements.

- **Streaming surfaces + capture**
  - Proposed: `docs/adr/0121-streaming-images-and-video-surfaces.md`,
    `docs/adr/0122-offscreen-rendering-frame-capture-and-readback.md`
  - Decide: minimal pixel-format/metadata vocabulary, coalescing keys, and capability-gated fast paths.

- **Font discovery + user font loading + stable IDs**
  - Update: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`, `docs/adr/0014-settings-and-configuration-files.md`
  - Update: `docs/adr/0162-font-stack-bootstrap-and-textfontstackkey-v1.md`
  - Decide: persistence format (store family + features + fallbacks, never numeric `FontId`), and invalidation/revision semantics when font DB changes.
  - Implement: `crates/fret-render-wgpu/src/text.rs`, platform font enumeration hooks (future: `crates/fret-platform`).

- **Docking keep-alive + early submission + programmatic close without flicker**
  - Update: `docs/adr/0013-docking-ops-and-persistence.md`, `docs/adr/0011-overlays-and-multi-root.md`
  - Decide: what it means for a dock host to be "kept alive" when collapsed/hidden, and the required ordering constraints for building docking targets.
  - Decide: the `DockOp` pattern that avoids one-frame "holes" when closing tabs programmatically (ImGui `SetTabItemClosed`-class issue).
  - Implement: `ecosystem/fret-docking/src/dock/space.rs`, `ecosystem/fret-docking/src/dock/manager.rs`, app integration points applying `DockOp` + invalidation.

- **Docking drag vs overlays vs viewport capture: arbitration matrix**
  - Update: `docs/adr/0072-docking-interaction-arbitration-matrix.md` (Accepted)
  - Decide: drag start/stop precedence, which overlays close/freeze during dock drags, and how modal barriers intentionally block docking/tool input.
  - Implement: `ecosystem/fret-docking/src/dock/space.rs`, `ecosystem/fret-ui-kit/src/overlay.rs`, `crates/fret-ui/src/tree/mod.rs` (capture + layering).

- **Multi-window degradation policy (single-window platforms)**
  - Update: `docs/adr/0084-multi-window-degradation-policy.md` (Accepted)
  - Implement: `crates/fret-core` import fallback + `ecosystem/fret-docking` tear-off degradation (demo harness still recommended).

### P1 (Lock soon; otherwise behavior will drift)

- **Docking split sizing + resizable primitive contract**
  - Update: `docs/adr/0077-resizable-panel-groups-and-docking-split-sizing.md` (Accepted)
  - Decide: runtime-owned resize mechanics, docking integration shape, and whether to eventually persist pixel `preferred_px` hints vs fractions-only.
  - Implement: docking host rendering in `ecosystem/fret-docking`, runtime substrate in `crates/fret-ui`.

- **Effect vocabulary extensions (color matrix + alpha threshold)**
  - Proposed: `docs/adr/1175-effect-steps-color-matrix-and-alpha-threshold-v1.md`
  - Decide: minimal postprocessing steps needed for SVG-filter-class recipes without going full material graphs.

- **Text input semantics for multiline + IME composition ranges**
  - Update: `docs/adr/0071-text-input-multiline-composition-contract.md` (Accepted)
  - Update: `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`, `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
  - Decide: exact selection/composition range semantics (byte vs grapheme), and caret-rect reporting requirements for IME candidate placement.

- **Active descendant semantics for composite widgets (command palette / listbox / combobox)**
  - Update: `docs/adr/0073-active-descendant-and-composite-widget-semantics.md` (Accepted)
  - Decide: minimal schema extension to support cmdk-style navigation without moving focus away from text input.
  - Implement: semantics production in `crates/fret-ui/src/tree/mod.rs`, AccessKit mapping in `crates/fret-a11y-accesskit/src/lib.rs`, backend glue in `crates/fret-runner-winit/src/accessibility.rs`.

- **Accessibility conformance baseline (Narrator/VoiceOver/AT-SPI)**
  - Update: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
  - Decide: minimum roles/actions/fields required for text fields (value/selection/composition), menus, tabs, and viewports.
  - Implement: semantics production in `crates/fret-ui/src/tree/mod.rs`, AccessKit mapping in `crates/fret-a11y-accesskit/src/lib.rs`, backend glue in `crates/fret-runner-winit/src/accessibility.rs`.

- **Cross-root focus traversal and focus scopes**
  - Update: `docs/adr/0068-focus-traversal-and-focus-scopes.md`, `docs/adr/0020-focus-and-command-routing.md`
  - Decide: traversal order across overlay roots, modal trap semantics, and focus restore rules for nested overlays + docking.

## Task Jump Table (Fast Navigation)

Use this as the “what should I read first?” map when implementing a subsystem.

- **Declarative/composable authoring (GPUI-style)**: `docs/adr/0028-declarative-elements-and-element-state.md`, `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`, `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- **Fluent authoring ergonomics (unified builder surface)**: `docs/adr/0175-unified-authoring-builder-surface-v1.md`
- **Declarative layout semantics (Flex + sizing)**: `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`, `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md`, `docs/adr/0042-virtualization-and-large-lists.md`
- **Tailwind layout vocabulary (margin/position/grid/aspect-ratio)**: `docs/adr/0062-tailwind-layout-primitives-margin-position-grid-aspect-ratio.md`
- **Container queries (panel-width responsiveness)**: `docs/adr/1170-container-queries-and-frame-lagged-layout-queries-v1.md`
- **Environment queries (viewport/device capabilities)**: `docs/adr/1171-environment-queries-and-viewport-snapshots-v1.md`
- **Environment queries (preference extensions)**: `docs/adr/1185-environment-queries-preference-extensions-v1.md`
- **Rounded clipping / overflow-hidden**: `docs/adr/0063-rounded-clipping-and-soft-clip-masks.md`
- **Overflow conventions (surfaces, focus rings, portals)**: `docs/adr/0088-overflow-and-clipping-conventions.md`
- **Docking + multi-window tear-off**: `docs/adr/0013-docking-ops-and-persistence.md`, `docs/adr/0011-overlays-and-multi-root.md`, `docs/adr/0017-multi-window-display-and-dpi.md`, `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- **Headless drag-and-drop toolbox (sensors/collision/modifiers)**: `docs/adr/0172-headless-dnd-v1-contract-surface.md`, `docs/adr/0164-headless-dnd-toolbox-and-ui-integration.md`, `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- **Pointer identity and multi-pointer dispatch**: `docs/adr/0165-pointer-identity-and-multi-pointer-capture.md`, `docs/adr/0017-multi-window-display-and-dpi.md`, `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- **Multi-pointer drag sessions (host + routing keys)**: `docs/adr/0166-multi-pointer-drag-sessions-and-routing-keys.md`, `docs/adr/0165-pointer-identity-and-multi-pointer-capture.md`, `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- **Docking interaction arbitration (overlays/tools)**: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- **Dismissable non-modal overlays (outside press)**: `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- **Text input / IME**: `docs/adr/0012-keyboard-ime-and-text-input.md`, `docs/adr/0029-text-pipeline-and-atlas-strategy.md`, `docs/adr/0020-focus-and-command-routing.md`
- **Color emoji / polychrome glyph pipeline**: `docs/adr/0167-polychrome-glyphs-and-emoji-pipeline-v1.md`, `docs/adr/0029-text-pipeline-and-atlas-strategy.md`, `docs/adr/0157-text-system-v2-parley-attributed-spans-and-quality-baseline.md`
- **Multiline text input + IME composition**: `docs/adr/0071-text-input-multiline-composition-contract.md`
- **Typography (weight/line-height/tracking)**: `docs/adr/0058-typography-v1-textstyle-weight-lineheight-tracking.md`
- **Text editing commands + selection model**: `docs/adr/0044-text-editing-state-and-commands.md`
- **Text geometry queries (caret/hit-test, multiline affinity)**: `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`, `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
- **Editor-grade code editor workstream**: `docs/adr/0200-code-editor-ecosystem-v1.md`, `docs/adr/0194-text-navigation-and-word-boundaries-v1.md`, `docs/adr/0195-web-ime-and-text-input-bridge-v1.md`, `docs/adr/0203-code-editor-display-fragments-and-displaymap-composition-v1.md`
- **Shortcut arbitration / AltGr / multi-stroke bindings**: `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`, `docs/adr/0021-keymap-file-format.md`, `docs/adr/0020-focus-and-command-routing.md`, `docs/adr/1157-input-dispatch-phases-prevent-default-and-action-availability-v2.md`
- **Observability + diagnostics bundles + scripted UI interaction tests**: `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`, `docs/adr/0174-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
- **Renderer (ordering, batching, shapes/SDF)**: `docs/adr/0009-renderer-ordering-and-batching.md`, `docs/adr/0030-shape-rendering-and-sdf-semantics.md`, `docs/adr/0002-display-list.md`
- **Renderer clipping (soft/rounded)**: `docs/adr/0063-rounded-clipping-and-soft-clip-masks.md`
- **Icons (semantic keys + SVG-first packaging)**: `docs/adr/0065-icon-system-and-asset-packaging.md`
- **Editor interaction affordances (selection/docking highlights)**: `docs/adr/0030-shape-rendering-and-sdf-semantics.md`, `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`, `docs/adr/0011-overlays-and-multi-root.md`
- **Engine viewports (embedded 3D)**: `docs/adr/0010-wgpu-context-ownership.md`, `docs/adr/0015-frame-lifecycle-and-submission-order.md`, `docs/adr/0025-viewport-input-forwarding.md`, `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- **Viewport tools and overlays**: `docs/adr/0168-viewport-tooling-host-helpers-and-arbitration-v1.md`, `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`
- **Ecosystem crates (glue vs kit vs primitives)**: `docs/adr/0169-ecosystem-crate-taxonomy-glue-and-ui-kit-split-v1.md`
- **Inspector / property editing (example editor layer)**: `docs/adr/0048-inspector-property-protocol-and-editor-registry.md`
- **Theme tokens and theme config (P0 styling)**: `docs/adr/0032-style-tokens-and-theme-resolution.md`, `docs/adr/0050-theme-config-schema-and-baseline-tokens.md`
- **State-driven style resolution (interactive states)**: `docs/adr/1158-state-driven-style-resolution-v1.md`
- **Elevation and shadows (no-blur baseline)**: `docs/adr/0060-shadows-and-elevation.md`
- **Focus rings (outline) and focus-visible**: `docs/adr/0061-focus-rings-and-focus-visible.md`
- **Component sizing/density (Tailwind-like scales)**: `docs/adr/0056-component-size-and-density-system.md`
- **Editor-scale performance**: `docs/adr/0042-virtualization-and-large-lists.md`, `docs/adr/0070-virtualization-contract.md`, `docs/adr/0034-timers-animation-and-redraw-scheduling.md`, `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`, `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`, `docs/adr/1152-cache-roots-and-cached-subtree-semantics-v1.md`
- **Model observation / reactive invalidation (GPUI-style)**: `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`, `docs/adr/0031-app-owned-models-and-leasing-updates.md`, `docs/adr/0005-retained-ui-tree.md`
- **Editor-scale lists contract (keys + data source)**: `docs/adr/0047-virtual-list-data-source-and-stable-item-keys.md`

Note: P0 explicitly defers “dashed borders” as a general `SceneOp::Quad` feature. Implement dashed selection rectangles
and docking drop-zone highlights as component-level overlay primitives that expand into multiple short `SceneOp::Quad`
segments (see ADR 0030 and ADR 0039 for the rationale and semantics).

## Code Entry Points (Where To Start Reading Code)

These anchors are intentionally few; use `rg` to drill down from them.

- App/effects/models: `crates/fret-app/src/app.rs`
- Desktop runner (winit + wgpu): `crates/fret-launch/src/runner/mod.rs`
- UI runtime substrate (UiTree + declarative bridge): `crates/fret-ui/src/tree/mod.rs` and `crates/fret-ui/src/declarative/`
- Docking UI (`DockSpace`, policy-heavy): `ecosystem/fret-docking/src/dock/space.rs`
- Core contracts (IDs, dock graph, scene ops): `crates/fret-core/src/lib.rs`
- Renderer (quads/SDF/text hooks): `crates/fret-render-wgpu/src/renderer/mod.rs`
- Demo (end-to-end wiring): `apps/fret-examples/src/components_gallery.rs`, `apps/fret-examples/src/docking_demo.rs`

## Current Focus (“Decide Early”)

These ADRs are intentionally prioritized because they tend to cause large rewrites if decided late:

- `docs/adr/0028-declarative-elements-and-element-state.md` (authoring/runtime model)
- `docs/adr/0029-text-pipeline-and-atlas-strategy.md` (text implementation strategy)
- `docs/adr/0058-typography-v1-textstyle-weight-lineheight-tracking.md` (typography expressiveness)
- `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md` (text geometry query boundary)
- `docs/adr/0046-multiline-text-layout-and-geometry-queries.md` (multiline geometry semantics)
- `docs/adr/0030-shape-rendering-and-sdf-semantics.md` (shape semantics over SDF)
- `docs/adr/0031-app-owned-models-and-leasing-updates.md` (GPUI-style ownership + updates)
- `docs/adr/0032-style-tokens-and-theme-resolution.md` (typed styling + theming)
- `docs/adr/0033-semantics-tree-and-accessibility-bridge.md` (A11y-ready semantics tree)
- `docs/adr/0034-timers-animation-and-redraw-scheduling.md` (event-driven scheduling + continuous mode)
- `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md` (hybrid layout, editor-friendly)
- `docs/adr/0057-declarative-layout-style-and-flex-semantics.md` (Tailwind-friendly sizing + Flex semantics)
- `docs/adr/0115-available-space-and-non-reentrant-measurement.md` (constraint-correct measurement, avoid layout re-entry)
- `docs/adr/0116-window-scoped-layout-engine-and-viewport-roots.md` (multi-viewport end-state for declarative flow layout)
- `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md` (profiling and inspector hooks)
- `docs/adr/0055-frame-recording-and-subtree-replay-caching.md` (recording/replay caching for editor-scale UI)
- `docs/adr/1152-cache-roots-and-cached-subtree-semantics-v1.md` (cache roots / cached subtree semantics for GPUI-style composition)
- `docs/adr/0037-workspace-boundaries-and-components-repository.md` (workspace boundaries + external components repo)
- `docs/adr/0038-engine-render-hook-and-submission-coordinator.md` (engine integration without queue ownership)
- `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md` (composable authoring layer)
- `docs/adr/0040-color-management-and-compositing-contracts.md` (linear compositor + viewport encoding metadata)
- `docs/adr/0102-semantic-theme-keys-and-extensible-token-registry.md` (semantic-first theming, shadcn alignment)
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md` (cross-window internal drag + clipboard boundary)
- `docs/adr/0042-virtualization-and-large-lists.md` (virtualization contract for editor-scale surfaces)
- `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md` (shortcut arbitration + AltGr + pending bindings)

## Organization Policy

- Keep ADRs in a single directory (`docs/adr/`) so links remain stable.
- Use this index to group ADRs by subsystem and by scope, instead of moving files around.

## App Runtime (`fret-app`)

- `docs/adr/0001-app-effects.md`
- `docs/adr/0014-settings-and-configuration-files.md`
- `docs/adr/0016-plugin-and-panel-boundaries.md`
- `docs/adr/0020-focus-and-command-routing.md`
- `docs/adr/0021-keymap-file-format.md`
- `docs/adr/0022-when-expressions.md`
- `docs/adr/0023-command-metadata-menus-and-palette.md`
- `docs/adr/0136-undo-redo-infrastructure-boundary.md`
- `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- `docs/adr/0086-model-handle-lifecycle-and-weak-models.md`
- `docs/adr/0087-models-are-main-thread-only-and-not-send.md`
- `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`
- `docs/adr/0044-text-editing-state-and-commands.md`

## Core Contracts (`fret-core`)

- `docs/adr/0002-display-list.md`
- `docs/adr/0004-resource-handles.md`
- `docs/adr/0006-text-system.md`
- `docs/adr/0058-typography-v1-textstyle-weight-lineheight-tracking.md`
- `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`
- `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
- `docs/adr/0012-keyboard-ime-and-text-input.md`
- `docs/adr/0071-text-input-multiline-composition-contract.md`
- `docs/adr/0018-key-codes-and-shortcuts.md`
- `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`
- `docs/adr/0019-scene-state-stack-and-layers.md`
- `docs/adr/0078-scene-transform-and-clip-composition.md`
- `docs/adr/0079-scene-layers-marker-only-v1.md`
- `docs/adr/0080-vector-path-contract.md`
- `docs/adr/0082-draworder-is-non-semantic.md`

## UI Runtime (`fret-ui`)

- `docs/adr/0005-retained-ui-tree.md`
- `docs/adr/0011-overlays-and-multi-root.md`
- `docs/adr/0013-docking-ops-and-persistence.md`
- `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- `docs/adr/0020-focus-and-command-routing.md`
- `docs/adr/0028-declarative-elements-and-element-state.md`
- `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0137-canvas-widgets-and-interactive-surfaces.md`
- `docs/adr/0201-kurbo-geometry-backend-for-canvas-hit-testing.md`
- `docs/adr/0150-node-graph-canvas-middleware.md`
- `docs/adr/0139-viewport-gizmos-engine-pass-and-ui-overlay-boundary.md`
- `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- `docs/adr/0104-layout-driven-anchored-overlays.md`
- `docs/adr/0032-style-tokens-and-theme-resolution.md`
- `docs/adr/0050-theme-config-schema-and-baseline-tokens.md`
- `docs/adr/0102-semantic-theme-keys-and-extensible-token-registry.md`
- `docs/adr/0056-component-size-and-density-system.md`
- `docs/adr/0060-shadows-and-elevation.md`
- `docs/adr/0061-focus-rings-and-focus-visible.md`
- `docs/adr/0068-focus-traversal-and-focus-scopes.md`
- `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- `docs/adr/0085-virtualized-accessibility-and-collection-semantics.md`
- `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md`
- `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
- `docs/adr/0062-tailwind-layout-primitives-margin-position-grid-aspect-ratio.md`
- `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`
- `docs/adr/0042-virtualization-and-large-lists.md`
- `docs/adr/0070-virtualization-contract.md`
- `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`
- `docs/adr/0047-virtual-list-data-source-and-stable-item-keys.md`
- `docs/adr/0100-markdown-rendering-streaming-and-injection.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/0044-text-editing-state-and-commands.md`
- `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`
- `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
- `docs/adr/0084-multi-window-degradation-policy.md`

## Renderer (`fret-render`)

- `docs/adr/0009-renderer-ordering-and-batching.md`
- `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
- `docs/adr/0089-renderer-architecture-v2-scene-compiler.md`
- `docs/adr/0118-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
- `docs/adr/0119-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
- `docs/adr/0120-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- `docs/adr/0121-streaming-images-and-video-surfaces.md`
- `docs/adr/0122-offscreen-rendering-frame-capture-and-readback.md`
- `docs/adr/0123-streaming-upload-budgets-and-backpressure-v1.md`
- `docs/adr/0124-renderer-capabilities-and-optional-zero-copy-imports.md`
- `docs/adr/0125-renderer-extensibility-materials-effects-and-sandboxing-v1.md`
- `docs/adr/0126-streaming-image-update-effects-and-metadata-v1.md`
- `docs/adr/0127-frame-capture-options-and-determinism-v1.md`
- `docs/adr/0096-renderer-perf-snapshot-and-stress-harness.md`

## Platform (`fret-platform`)

- `docs/adr/0003-platform-boundary.md`
- `docs/adr/0091-platform-backends-native-web.md`
- `docs/adr/0094-window-close-and-web-runner-destroy.md`
- `docs/adr/0154-window-styles-and-utility-windows.md`
- `docs/adr/0017-multi-window-display-and-dpi.md`
- `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`
- `docs/adr/0053-external-drag-payload-portability.md`
- `docs/adr/0054-platform-capabilities-and-portability-matrix.md`

## Engine Integration / Viewports

- `docs/adr/0007-viewport-surfaces.md`
- `docs/adr/0010-wgpu-context-ownership.md`
- `docs/adr/0015-frame-lifecycle-and-submission-order.md`
- `docs/adr/0025-viewport-input-forwarding.md`
- `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- `docs/adr/0040-color-management-and-compositing-contracts.md`
- `docs/adr/0168-viewport-tooling-host-helpers-and-arbitration-v1.md`
- `docs/adr/0116-window-scoped-layout-engine-and-viewport-roots.md`

## Example Editor App Notes (Out of Scope for Fret Framework)

- `docs/adr/0024-undo-redo-and-edit-transactions.md`
- `docs/adr/0026-asset-database-and-import-pipeline.md`
- `docs/adr/0048-inspector-property-protocol-and-editor-registry.md`
- `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`

## Cross-Cutting

- `docs/adr/0008-threading-logging-errors.md`
- `docs/adr/0199-execution-and-concurrency-surface-v1.md`
- `docs/adr/0027-framework-scope-and-responsibilities.md`
- `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
- `docs/adr/0097-plot-widgets-and-crate-placement.md`
- `docs/adr/0098-plot3d-rendering-strategy.md`
- `docs/adr/0099-plot-architecture-and-performance.md`
- `docs/adr/1128-delinea-headless-chart-engine.md`
- `docs/adr/1129-delinea-transform-pipeline-and-datazoom-semantics.md`
- `docs/adr/1130-delinea-axis-scales-and-coordinate-mapping.md`
- `docs/adr/1131-delinea-marks-identity-and-renderer-contract.md`
- `docs/adr/1132-delinea-large-data-and-progressive-rendering.md`
- `docs/adr/1133-delinea-interaction-and-hit-testing-contract.md`
- `docs/adr/1134-delinea-multi-axis-and-layout-contract.md`
- `docs/adr/1135-delinea-axis-interaction-locks-and-shortcuts.md`
- `docs/adr/1136-delinea-datazoom-y-and-2d-semantics.md`
- `docs/adr/1137-delinea-row-selection-and-filtering-contract.md`
- `docs/adr/1138-delinea-datazoom-component-composition-and-span-policy.md`
- `docs/archive/delinea-adr-bootstrap/README.md`
- `docs/adr/0107-dev-hotpatch-subsecond-and-hot-reload-safety.md`
- `docs/adr/0108-ecosystem-bootstrap-ui-assets-and-dev-tools.md`
- `docs/adr/0111-user-facing-crate-surfaces-and-golden-path.md`
- `docs/adr/0112-golden-path-ui-app-driver-and-pipelines.md`
- `docs/adr/0113-ecosystem-integration-contracts.md`
- `docs/adr/0114-ui-assets-facade-and-golden-path-wiring.md`
- `docs/adr/0093-crate-structure-core-backends-apps.md`
- `docs/adr/0090-radix-aligned-headless-primitives-in-fret-components-ui.md`
- `docs/adr/0101-headless-table-engine.md`
- `docs/adr/0135-node-graph-editor-and-typed-connections.md`
- `docs/adr/0058-typography-v1-textstyle-weight-lineheight-tracking.md`
- `docs/adr/0059-text-overflow-ellipsis-and-truncation.md`
- `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`
- `docs/adr/0037-workspace-boundaries-and-components-repository.md`
- `docs/adr/0040-color-management-and-compositing-contracts.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- `docs/adr/0164-headless-dnd-toolbox-and-ui-integration.md`
- `docs/adr/0165-pointer-identity-and-multi-pointer-capture.md`
- `docs/adr/0166-multi-pointer-drag-sessions-and-routing-keys.md`
- `docs/adr/0167-polychrome-glyphs-and-emoji-pipeline-v1.md`
- `docs/adr/0056-component-size-and-density-system.md`
- `docs/adr/0052-ui-host-runtime-boundary.md`
- `docs/adr/0053-external-drag-payload-portability.md`
- `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
