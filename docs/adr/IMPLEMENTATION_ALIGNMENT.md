# ADR Implementation Alignment Matrix

This document tracks whether each ADR is implemented and whether the current implementation aligns with the ADR contract.
It is **non-normative**: the ADR itself remains the source of truth; this file is a practical audit checklist.

## Legend

- **Aligned**: core mechanism exists; behavior matches ADR intent; evidence paths are listed.
- **Aligned (with known gaps)**: largely aligned, with gaps explicitly captured (usually in the ADR itself).
- **Partially aligned**: some mechanisms exist, but important outcomes are missing or incomplete (notes specify gaps).
- **Not implemented**: ADR exists, but the corresponding framework surface is not implemented yet.
- **Not audited**: not reviewed recently; do not assume alignment.
- **N/A (superseded)**: ADR is superseded; do not implement new work against it.

## Summary

- Last updated: 2026-01-09
- ADR count (numbered): 139

- Aligned: 37
- Aligned (with known gaps): 5
- N/A (superseded): 1
- Not audited: 84
- Not implemented: 3
- Partially aligned: 9

## Matrix

| ADR | ADR Status | Implementation Alignment | Notes |
| --- | --- | --- | --- |
| [`0001-app-effects.md`](0001-app-effects.md) | Accepted | Aligned | Effects queue + redraw coalescing: `crates/fret-app/src/app.rs` (`push_effect`, `flush_effects`); bounded fixed-point draining: `crates/fret-launch/src/runner/desktop/mod.rs` and `crates/fret-launch/src/runner/web.rs`. |
| [`0002-display-list.md`](0002-display-list.md) | Accepted | Aligned | `SceneOp` contract: `crates/fret-core/src/scene.rs`; renderer preserves op iteration order: `crates/fret-render/src/renderer/render_scene/encode/mod.rs`. |
| [`0003-platform-boundary.md`](0003-platform-boundary.md) | Superseded | N/A (superseded) | Superseded ADR; do not implement new work against this contract. |
| [`0004-resource-handles.md`](0004-resource-handles.md) | Accepted | Aligned | Stable IDs: `crates/fret-core/src/ids.rs`; effect-driven registration: `crates/fret-runtime/src/effect.rs` (`ImageRegisterRgba8`, `ImageUnregister`); runner/renderer handles at flush point: `crates/fret-launch/src/runner/desktop/mod.rs`. |
| [`0005-retained-ui-tree.md`](0005-retained-ui-tree.md) | Accepted | Aligned | Retained tree + bounds/hit-test routing: `crates/fret-ui/src/tree/mod.rs` and `crates/fret-ui/src/tree/dispatch.rs` (hit-test, capture, focus fallback). |
| [`0006-text-system.md`](0006-text-system.md) | Accepted | Aligned | Text boundary trait: `crates/fret-core/src/text.rs` (`TextService`); renderer implementation: `crates/fret-render/src/text.rs` + `crates/fret-render/src/renderer/services.rs`; UI emits `SceneOp::Text`: `crates/fret-ui/src/declarative/host_widget/paint.rs`. |
| [`0007-viewport-surfaces.md`](0007-viewport-surfaces.md) | Accepted | Aligned | `RenderTargetId` + `SceneOp::ViewportSurface`: `crates/fret-core/src/ids.rs`, `crates/fret-core/src/scene.rs`; registry: `crates/fret-render/src/targets.rs`. |
| [`0008-threading-logging-errors.md`](0008-threading-logging-errors.md) | Accepted | Aligned | Main-thread runner loop + `pollster` init: `crates/fret-launch/src/runner/desktop/app_handler.rs`; structured logging via `tracing` across crates (e.g. `crates/fret-launch/src/runner/desktop/mod.rs`). |
| [`0009-renderer-ordering-and-batching.md`](0009-renderer-ordering-and-batching.md) | Accepted | Aligned | Renderer encodes ops in order: `crates/fret-render/src/renderer/render_scene/encode/mod.rs`; adjacency batching without reordering via `ordered_draws` + quad batch flush: `crates/fret-render/src/renderer/render_scene/encode/state.rs`. |
| [`0010-wgpu-context-ownership.md`](0010-wgpu-context-ownership.md) | Accepted | Aligned | Host-provided `WgpuContext`: `crates/fret-render/src/lib.rs`; runner supports provided vs created context: `crates/fret-launch/src/runner/common.rs`; surfaces created from `context.instance`: `crates/fret-render/src/lib.rs` (`create_surface`). |
| [`0011-overlays-and-multi-root.md`](0011-overlays-and-multi-root.md) | Accepted | Aligned | Overlay roots + modal blocking: `crates/fret-ui/src/tree/layers.rs` (`push_overlay_root_ex`, `blocks_underlay_input`, `visible_layers_in_paint_order`); hit-test across roots: `crates/fret-ui/src/tree/hit_test.rs`; tests: `crates/fret-ui/src/tree/tests/hit_test.rs`. |
| [`0012-keyboard-ime-and-text-input.md`](0012-keyboard-ime-and-text-input.md) | Accepted | Partially aligned | Shortcut deferral for text input in `crates/fret-ui/src/tree/dispatch.rs`; desktop IME effects in `crates/fret-launch/src/runner/desktop/mod.rs`; web runner lacks IME effect handling (`crates/fret-launch/src/runner/web.rs`). |
| [`0013-docking-ops-and-persistence.md`](0013-docking-ops-and-persistence.md) | Accepted | Aligned | Dock ops + persistence: `crates/fret-core/src/dock_op.rs`, `crates/fret-core/src/dock_layout.rs`, `crates/fret-core/src/panels.rs`; runner applies via `Effect::Dock`: `crates/fret-launch/src/runner/desktop/mod.rs`; demo persistence: `apps/fret-examples/src/docking_arbitration_demo.rs`. |
| [`0014-settings-and-configuration-files.md`](0014-settings-and-configuration-files.md) | Accepted | Partially aligned | Strongly typed settings file: `crates/fret-app/src/settings.rs`; bootstrap loads `.fret/settings.json`: `ecosystem/fret-bootstrap/src/lib.rs`; user-level OS config dirs + scope layering not yet wired. |
| [`0015-frame-lifecycle-and-submission-order.md`](0015-frame-lifecycle-and-submission-order.md) | Accepted | Aligned | Runner render pipeline + submission ordering: `crates/fret-launch/src/runner/desktop/app_handler.rs` (`RedrawRequested` records engine + UI and submits engine cmd buffers before UI cmd); `TickId` vs `FrameId`: same file (`about_to_wait` vs frame increment). |
| [`0016-plugin-and-panel-boundaries.md`](0016-plugin-and-panel-boundaries.md) | Accepted | Partially aligned | Stable panel identity exists (`crates/fret-core/src/panels.rs`, `crates/fret-core/src/dock.rs`); ecosystem panel registry service exists (`ecosystem/fret-ui-docking/src/dock/panel_registry.rs`); plugin lifecycle/command/settings registration remains app-owned and not yet standardized. |
| [`0017-multi-window-display-and-dpi.md`](0017-multi-window-display-and-dpi.md) | Accepted | Aligned | Logical px window events: `crates/fret-core/src/input.rs` (`WindowScaleFactorChanged`, `WindowMoved`, `WindowResized`); metrics service: `crates/fret-core/src/window.rs`; runners apply: `crates/fret-launch/src/runner/desktop/mod.rs` and `crates/fret-launch/src/runner/web.rs` (`apply_window_metrics_event`). |
| [`0018-key-codes-and-shortcuts.md`](0018-key-codes-and-shortcuts.md) | Accepted | Aligned | Physical key codes as `KeyCode` (aligned with `keyboard-types::Code`): `crates/fret-core/src/input.rs`; winit mapping uses physical keys: `crates/fret-runner-winit/src/lib.rs` (`map_physical_key`). |
| [`0019-scene-state-stack-and-layers.md`](0019-scene-state-stack-and-layers.md) | Accepted | Aligned | Scene state ops + debug validation: `crates/fret-core/src/scene.rs` (`PushTransform/Opacity/Layer/Clip` + `validate`); renderer encodes stack into draw state: `crates/fret-render/src/renderer/render_scene/encode/ops.rs`. |
| [`0020-focus-and-command-routing.md`](0020-focus-and-command-routing.md) | Accepted | Aligned | Shortcut→command as effects: `crates/fret-ui/src/tree/shortcuts.rs` (`Effect::Command`, repeatable gating via `CommandMeta`); widget-scope dispatch: `crates/fret-ui/src/tree/commands.rs`; runner drains and calls driver handlers: `crates/fret-launch/src/runner/desktop/mod.rs`. |
| [`0021-keymap-file-format.md`](0021-keymap-file-format.md) | Accepted | Partially aligned | Keymap parsing + platform/when support: `crates/fret-runtime/src/keymap.rs` (v1 + sequences); disk IO helper: `crates/fret-app/src/keymap.rs`; end-to-end layered loading (user/project) not standardized yet. |
| [`0022-when-expressions.md`](0022-when-expressions.md) | Accepted | Aligned | Parser/validator/evaluator: `crates/fret-runtime/src/when_expr.rs` (unknown identifiers evaluate false via `eval_ident_bool`). |
| [`0023-command-metadata-menus-and-palette.md`](0023-command-metadata-menus-and-palette.md) | Accepted | Partially aligned | Command metadata + scopes: `crates/fret-runtime/src/commands.rs` (`CommandMeta`, `CommandScope`), stored on app: `crates/fret-app/src/app.rs`; menu model: `crates/fret-runtime/src/menu.rs`; command palette widget exists (`ecosystem/fret-ui-shadcn/src/command.rs`) but registry→palette wiring remains app-owned. |
| [`0024-undo-redo-and-edit-transactions.md`](0024-undo-redo-and-edit-transactions.md) | Deferred | Not audited |  |
| [`0025-viewport-input-forwarding.md`](0025-viewport-input-forwarding.md) | Accepted | Aligned | Core event + mapping: `crates/fret-core/src/input.rs` (`ViewportInputEvent`), `crates/fret-core/src/viewport.rs` (`ViewportMapping`); UI emits `Effect::ViewportInput`: `ecosystem/fret-ui-docking/src/dock/space.rs`; runner drains into driver hook: `crates/fret-launch/src/runner/desktop/mod.rs`. |
| [`0026-asset-database-and-import-pipeline.md`](0026-asset-database-and-import-pipeline.md) | Deferred | Not audited |  |
| [`0027-framework-scope-and-responsibilities.md`](0027-framework-scope-and-responsibilities.md) | Accepted | Aligned | Layering matches: `fret-core` stays platform/render-agnostic while `wgpu` lives in `crates/fret-render`; policy-heavy surfaces live under `ecosystem/` (see `AGENTS.md`, `docs/dependency-policy.md`). |
| [`0028-declarative-elements-and-element-state.md`](0028-declarative-elements-and-element-state.md) | Accepted | Aligned | Declarative mount contract: `crates/fret-ui/src/declarative/mount.rs` (`render_root`); stable IDs: `crates/fret-ui/src/elements/id.rs` + hashing (`crates/fret-ui/src/elements/hash.rs`); cross-frame state: `crates/fret-ui/src/elements/access.rs` (`with_element_state`). |
| [`0029-text-pipeline-and-atlas-strategy.md`](0029-text-pipeline-and-atlas-strategy.md) | Accepted | Not audited |  |
| [`0030-shape-rendering-and-sdf-semantics.md`](0030-shape-rendering-and-sdf-semantics.md) | Accepted | Not audited |  |
| [`0031-app-owned-models-and-leasing-updates.md`](0031-app-owned-models-and-leasing-updates.md) | Accepted | Not audited |  |
| [`0032-style-tokens-and-theme-resolution.md`](0032-style-tokens-and-theme-resolution.md) | Accepted | Aligned | Token registry + resolution + linearized colors: `crates/fret-ui/src/theme.rs`, `crates/fret-ui/src/theme_registry.rs` (includes sRGB/HSL/OKLCH parsing to linear + revision). |
| [`0033-semantics-tree-and-accessibility-bridge.md`](0033-semantics-tree-and-accessibility-bridge.md) | Accepted | Aligned | Portable schema: `crates/fret-core/src/semantics.rs`; snapshot build w/ overlay barrier root: `crates/fret-ui/src/tree/mod.rs` (`refresh_semantics_snapshot`); AccessKit mapping + action decoding: `crates/fret-a11y-accesskit/src/lib.rs`; winit adapter: `crates/fret-runner-winit/src/accessibility_accesskit_winit.rs`. |
| [`0034-timers-animation-and-redraw-scheduling.md`](0034-timers-animation-and-redraw-scheduling.md) | Accepted | Aligned | Scheduling effects: `crates/fret-runtime/src/effect.rs` (`RequestAnimationFrame`, `SetTimer`, `CancelTimer`); RAII continuous frames: `crates/fret-ui/src/elements/runtime.rs`; runner timers + bounded draining: desktop `crates/fret-launch/src/runner/desktop/mod.rs`, web `crates/fret-launch/src/runner/web.rs` (`MAX_EFFECT_DRAIN_TURNS=8`). |
| [`0035-layout-constraints-and-optional-taffy-integration.md`](0035-layout-constraints-and-optional-taffy-integration.md) | Accepted | Aligned | Hybrid explicit bounds + internal Taffy: `crates/fret-ui/src/declarative/host_widget.rs` (Flex/Grid caches + measurement hook), `crates/fret-ui/src/declarative/taffy_layout.rs`. |
| [`0036-observability-tracing-and-ui-inspector-hooks.md`](0036-observability-tracing-and-ui-inspector-hooks.md) | Accepted | Partially aligned | Structured tracing is pervasive (e.g. `crates/fret-launch/src/runner/desktop/mod.rs`); UI inspector toggles + per-frame debug stats exist: `crates/fret-ui/src/tree/mod.rs` (`inspection_active`, `UiDebugFrameStats`); standardized renderer metrics surface is not yet implemented (beyond ad-hoc snapshots like `crates/fret-render/src/renderer/types.rs`). |
| [`0037-workspace-boundaries-and-components-repository.md`](0037-workspace-boundaries-and-components-repository.md) | Accepted | Not audited |  |
| [`0038-engine-render-hook-and-submission-coordinator.md`](0038-engine-render-hook-and-submission-coordinator.md) | Accepted | Aligned | Engine frame recording + target updates: `crates/fret-launch/src/runner/common.rs` (`EngineFrameUpdate`, `RenderTargetUpdate`); runner applies updates before UI submits: desktop `crates/fret-launch/src/runner/desktop/app_handler.rs`, web `crates/fret-launch/src/runner/web.rs`. |
| [`0039-component-authoring-model-render-renderonce-and-intoelement.md`](0039-component-authoring-model-render-renderonce-and-intoelement.md) | Accepted | Aligned (with known gaps) | Authoring traits exist in `crates/fret-ui/src/element.rs` (`IntoElement`, `RenderOnce`); derive-macro ergonomics (`fret-macros`) not implemented yet. |
| [`0040-color-management-and-compositing-contracts.md`](0040-color-management-and-compositing-contracts.md) | Accepted | Aligned (with known gaps) | Linear shading + conditional output encoding: `crates/fret-render/src/renderer/shaders.rs` (`encode_output_premul`, `output_is_srgb`); swapchain prefers sRGB: `crates/fret-render/src/surface.rs`; viewport target encoding via view format + `RenderTargetColorSpace`: `crates/fret-render/src/targets.rs`. Gap: render target alpha semantics (opaque vs premul) not modeled yet. |
| [`0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`](0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md) | Accepted | Aligned (with known gaps) | App-scoped internal drag session: `crates/fret-runtime/src/drag.rs` + stored on app (`crates/fret-app/src/app.rs`); internal drag routing override: `crates/fret-ui/src/drag_route.rs` + dispatch rules in `crates/fret-ui/src/tree/dispatch.rs`; external file drag/drop events + tokenized reads: `crates/fret-core/src/input.rs` (`ExternalDrag*`, `ExternalDropData`), effects in `crates/fret-runtime/src/effect.rs` (`ExternalDropReadAll*`, `ExternalDropRelease`), runner wiring in `crates/fret-launch/src/runner/desktop/app_handler.rs` + `crates/fret-platform-web/src/wasm.rs`. Gap: external drag initiation (`Effect::StartExternalDrag`) is still deferred; `DragSessionId`/phase/pointer id are not modeled yet. |
| [`0042-virtualization-and-large-lists.md`](0042-virtualization-and-large-lists.md) | Accepted | Aligned | VirtualList container + scroll model: `crates/fret-ui/src/element.rs` (`VirtualListProps`, `VirtualListState`), implementation in `crates/fret-ui/src/declarative/host_widget/layout.rs`; tests: `crates/fret-ui/src/declarative/tests/virtual_list.rs`; demos: `apps/fret-examples/src/virtual_list_*`. |
| [`0043-shortcut-arbitration-pending-bindings-and-altgr.md`](0043-shortcut-arbitration-pending-bindings-and-altgr.md) | Accepted | Not audited |  |
| [`0044-text-editing-state-and-commands.md`](0044-text-editing-state-and-commands.md) | Accepted | Aligned | UTF-8 byte offsets clamped to char boundaries: `crates/fret-ui/src/text_edit.rs` (`utf8::*`); core `text.*` command vocabulary: `crates/fret-ui/src/text_edit.rs`, `crates/fret-ui/src/text_surface.rs`, widgets `crates/fret-ui/src/text_input/widget.rs` + `crates/fret-ui/src/text_area/widget.rs`; IME cursor area effect: `Effect::ImeSetCursorArea` emitted by text widgets. |
| [`0045-text-geometry-queries-hit-testing-and-caret-metrics.md`](0045-text-geometry-queries-hit-testing-and-caret-metrics.md) | Accepted | Aligned | Geometry query hooks in `crates/fret-core/src/text.rs` (`TextService::{caret_x,hit_test_x,caret_rect,hit_test_point}`); renderer implementation: `crates/fret-render/src/text.rs` + `crates/fret-render/src/renderer/services.rs`; widgets use caret rect/hit testing: `crates/fret-ui/src/text_input/widget.rs`, `crates/fret-ui/src/text_area/mod.rs`. |
| [`0046-multiline-text-layout-and-geometry-queries.md`](0046-multiline-text-layout-and-geometry-queries.md) | Accepted | Aligned | Affinity + hit-test result types: `crates/fret-core/src/text.rs` (`CaretAffinity`, `HitTestResult`); multiline caret/hit-test implementation: `crates/fret-render/src/text.rs` (`caret_rect`, `hit_test_point`); multiline widget uses affinity for selection/IME: `crates/fret-ui/src/text_area/mod.rs`. |
| [`0047-virtual-list-data-source-and-stable-item-keys.md`](0047-virtual-list-data-source-and-stable-item-keys.md) | Accepted | Not audited |  |
| [`0048-inspector-property-protocol-and-editor-registry.md`](0048-inspector-property-protocol-and-editor-registry.md) | Accepted | Not audited |  |
| [`0049-viewport-tools-input-capture-and-overlays.md`](0049-viewport-tools-input-capture-and-overlays.md) | Deferred | Not audited |  |
| [`0050-theme-config-schema-and-baseline-tokens.md`](0050-theme-config-schema-and-baseline-tokens.md) | Accepted | Aligned | Baseline typed tokens + default theme + semantic alias keys (shadcn bridge): `crates/fret-ui/src/theme.rs` (includes tests). |
| [`0051-model-observation-and-ui-invalidation-propagation.md`](0051-model-observation-and-ui-invalidation-propagation.md) | Accepted | Aligned | Changed model/global propagation: `crates/fret-app/src/app.rs` (`take_changed_models`) + `crates/fret-ui/src/tree/mod.rs` (observation registries and invalidation); tests: `crates/fret-ui/src/tree/tests/models.rs`. |
| [`0052-ui-host-runtime-boundary.md`](0052-ui-host-runtime-boundary.md) | Accepted | Aligned | Host boundary trait surface: `crates/fret-runtime/src/ui_host.rs`; default host impl: `crates/fret-app/src/ui_host.rs`; integrated bridge crate: `crates/fret-ui-app`. |
| [`0053-external-drag-payload-portability.md`](0053-external-drag-payload-portability.md) | Accepted | Not audited |  |
| [`0054-platform-capabilities-and-portability-matrix.md`](0054-platform-capabilities-and-portability-matrix.md) | Accepted | Not audited |  |
| [`0055-frame-recording-and-subtree-replay-caching.md`](0055-frame-recording-and-subtree-replay-caching.md) | Accepted | Not audited |  |
| [`0056-component-size-and-density-system.md`](0056-component-size-and-density-system.md) | Accepted | Partially aligned | Prototype exists in `ecosystem/fret-ui-kit/src/sizing.rs`; “density modes” follow-up not implemented yet (see ADR). |
| [`0057-declarative-layout-style-and-flex-semantics.md`](0057-declarative-layout-style-and-flex-semantics.md) | Accepted | Not audited |  |
| [`0058-typography-v1-textstyle-weight-lineheight-tracking.md`](0058-typography-v1-textstyle-weight-lineheight-tracking.md) | Accepted | Not audited |  |
| [`0059-text-overflow-ellipsis-and-truncation.md`](0059-text-overflow-ellipsis-and-truncation.md) | Unknown | Not audited |  |
| [`0060-shadows-and-elevation.md`](0060-shadows-and-elevation.md) | Unknown | Not audited |  |
| [`0061-focus-rings-and-focus-visible.md`](0061-focus-rings-and-focus-visible.md) | Unknown | Aligned | Focus-visible heuristic in `crates/fret-ui/src/focus_visible.rs`; updated during dispatch in `crates/fret-ui/src/tree/dispatch.rs`. |
| [`0062-tailwind-layout-primitives-margin-position-grid-aspect-ratio.md`](0062-tailwind-layout-primitives-margin-position-grid-aspect-ratio.md) | Unknown | Aligned | Implemented in `LayoutStyle` + Taffy mapping: `crates/fret-ui/src/element.rs`, `crates/fret-ui/src/declarative/taffy_layout.rs`, `crates/fret-ui/src/declarative/host_widget/layout/grid.rs`. |
| [`0063-rounded-clipping-and-soft-clip-masks.md`](0063-rounded-clipping-and-soft-clip-masks.md) | Accepted | Not audited |  |
| [`0064-overlay-placement-contract.md`](0064-overlay-placement-contract.md) | Unknown | Not audited |  |
| [`0065-icon-system-and-asset-packaging.md`](0065-icon-system-and-asset-packaging.md) | Accepted | Not audited |  |
| [`0066-fret-ui-runtime-contract-surface.md`](0066-fret-ui-runtime-contract-surface.md) | Accepted | Not audited |  |
| [`0067-overlay-policy-architecture-dismissal-focus-portal.md`](0067-overlay-policy-architecture-dismissal-focus-portal.md) | Accepted | Not audited |  |
| [`0068-focus-traversal-and-focus-scopes.md`](0068-focus-traversal-and-focus-scopes.md) | Accepted | Aligned (with known gaps) | `focus.next/focus.previous` in `crates/fret-ui/src/tree/commands.rs`; tests in `crates/fret-ui/src/tree/tests/focus_scope.rs` and `crates/fret-ui/src/tree/tests/scroll_into_view.rs`. |
| [`0069-outside-press-and-dismissable-non-modal-overlays.md`](0069-outside-press-and-dismissable-non-modal-overlays.md) | Accepted | Aligned | Outside-press observer pass in `crates/fret-ui/src/tree/mod.rs`; observer dispatch invariants in `crates/fret-ui/src/tree/dispatch.rs`; tests in `crates/fret-ui/src/tree/tests/outside_press.rs` + `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`. |
| [`0070-virtualization-contract.md`](0070-virtualization-contract.md) | Unknown | Not audited |  |
| [`0071-text-input-multiline-composition-contract.md`](0071-text-input-multiline-composition-contract.md) | Accepted | Not audited |  |
| [`0072-docking-interaction-arbitration-matrix.md`](0072-docking-interaction-arbitration-matrix.md) | Accepted | Not audited |  |
| [`0073-active-descendant-and-composite-widget-semantics.md`](0073-active-descendant-and-composite-widget-semantics.md) | Accepted | Not audited |  |
| [`0074-component-owned-interaction-policy-and-runtime-action-hooks.md`](0074-component-owned-interaction-policy-and-runtime-action-hooks.md) | Accepted | Not audited |  |
| [`0075-docking-layering-b-route-and-retained-bridge.md`](0075-docking-layering-b-route-and-retained-bridge.md) | Accepted | Not audited |  |
| [`0076-declarative-layout-performance-hardening.md`](0076-declarative-layout-performance-hardening.md) | Accepted | Not audited |  |
| [`0077-resizable-panel-groups-and-docking-split-sizing.md`](0077-resizable-panel-groups-and-docking-split-sizing.md) | Accepted | Not audited |  |
| [`0078-scene-transform-and-clip-composition.md`](0078-scene-transform-and-clip-composition.md) | Accepted | Not audited |  |
| [`0079-scene-layers-marker-only-v1.md`](0079-scene-layers-marker-only-v1.md) | Accepted | Not audited |  |
| [`0080-vector-path-contract.md`](0080-vector-path-contract.md) | Accepted | Not audited |  |
| [`0082-draworder-is-non-semantic.md`](0082-draworder-is-non-semantic.md) | Accepted | Not audited |  |
| [`0083-render-transform-hit-testing.md`](0083-render-transform-hit-testing.md) | Accepted | Not audited |  |
| [`0084-multi-window-degradation-policy.md`](0084-multi-window-degradation-policy.md) | Accepted | Not audited |  |
| [`0085-virtualized-accessibility-and-collection-semantics.md`](0085-virtualized-accessibility-and-collection-semantics.md) | Accepted | Not audited |  |
| [`0086-model-handle-lifecycle-and-weak-models.md`](0086-model-handle-lifecycle-and-weak-models.md) | Accepted | Not audited |  |
| [`0087-models-are-main-thread-only-and-not-send.md`](0087-models-are-main-thread-only-and-not-send.md) | Accepted | Not audited |  |
| [`0088-overflow-and-clipping-conventions.md`](0088-overflow-and-clipping-conventions.md) | Accepted | Not audited |  |
| [`0089-renderer-architecture-v2-scene-compiler.md`](0089-renderer-architecture-v2-scene-compiler.md) | Accepted | Not audited |  |
| [`0090-radix-aligned-headless-primitives-in-fret-components-ui.md`](0090-radix-aligned-headless-primitives-in-fret-components-ui.md) | Proposed | Not audited |  |
| [`0091-platform-backends-native-web.md`](0091-platform-backends-native-web.md) | Accepted | Not audited |  |
| [`0092-keyboard-types-physical-keycodes.md`](0092-keyboard-types-physical-keycodes.md) | Accepted | Not audited |  |
| [`0093-crate-structure-core-backends-apps.md`](0093-crate-structure-core-backends-apps.md) | Accepted | Not audited |  |
| [`0094-window-close-and-web-runner-destroy.md`](0094-window-close-and-web-runner-destroy.md) | Unknown | Not audited |  |
| [`0095-menu-open-modality-and-entry-focus.md`](0095-menu-open-modality-and-entry-focus.md) | Proposed | Not audited |  |
| [`0096-renderer-perf-snapshot-and-stress-harness.md`](0096-renderer-perf-snapshot-and-stress-harness.md) | Proposed | Not audited |  |
| [`0097-plot-widgets-and-crate-placement.md`](0097-plot-widgets-and-crate-placement.md) | Accepted | Not audited |  |
| [`0098-plot3d-rendering-strategy.md`](0098-plot3d-rendering-strategy.md) | Proposed | Not audited |  |
| [`0099-plot-architecture-and-performance.md`](0099-plot-architecture-and-performance.md) | Proposed | Not audited |  |
| [`0100-markdown-rendering-streaming-and-injection.md`](0100-markdown-rendering-streaming-and-injection.md) | Accepted | Not audited |  |
| [`0100-pointer-click-count-and-double-click.md`](0100-pointer-click-count-and-double-click.md) | Proposed | Not audited |  |
| [`0101-headless-table-engine.md`](0101-headless-table-engine.md) | Accepted | Not audited |  |
| [`0102-semantic-theme-keys-and-extensible-token-registry.md`](0102-semantic-theme-keys-and-extensible-token-registry.md) | Accepted | Not audited |  |
| [`0103-text-decorations-and-markdown-theme-tokens.md`](0103-text-decorations-and-markdown-theme-tokens.md) | Proposed | Not audited |  |
| [`0104-layout-driven-anchored-overlays.md`](0104-layout-driven-anchored-overlays.md) | Accepted | Aligned (with known gaps) | `Anchored` primitive exists: `crates/fret-ui/src/element.rs` (`AnchoredProps`), layout-driven placement + render-transform mapping: `crates/fret-ui/src/declarative/host_widget/layout.rs` (`ElementInstance::Anchored`) + `crates/fret-ui/src/declarative/host_widget.rs` (`render_transform`); tests: `crates/fret-ui/src/declarative/tests/anchored.rs`. Gap: ecosystem overlays still mostly compute placement from last-frame bounds (`ecosystem/fret-ui-kit/src/overlay.rs`, `ecosystem/fret-ui-kit/src/primitives/popper.rs`) instead of using intrinsic sizing under `Anchored`. |
| [`0106-node-graph-editor-and-typed-connections.md`](0106-node-graph-editor-and-typed-connections.md) | Proposed | Not audited |  |
| [`0106-plot-overlays-and-annotations.md`](0106-plot-overlays-and-annotations.md) | Accepted | Not audited |  |
| [`0107-dev-hotpatch-subsecond-and-hot-reload-safety.md`](0107-dev-hotpatch-subsecond-and-hot-reload-safety.md) | Accepted | Not audited |  |
| [`0108-ecosystem-bootstrap-ui-assets-and-dev-tools.md`](0108-ecosystem-bootstrap-ui-assets-and-dev-tools.md) | Accepted | Not audited |  |
| [`0109-readonly-text-selection-and-clipboard.md`](0109-readonly-text-selection-and-clipboard.md) | Proposed | Not audited |  |
| [`0109-rich-text-runs-and-text-quality-v2.md`](0109-rich-text-runs-and-text-quality-v2.md) | Proposed | Not audited |  |
| [`0110-rich-content-selection-and-clipboard.md`](0110-rich-content-selection-and-clipboard.md) | Proposed | Not audited |  |
| [`0111-user-facing-crate-surfaces-and-golden-path.md`](0111-user-facing-crate-surfaces-and-golden-path.md) | Accepted | Not audited |  |
| [`0112-golden-path-ui-app-driver-and-pipelines.md`](0112-golden-path-ui-app-driver-and-pipelines.md) | Accepted | Not audited |  |
| [`0113-ecosystem-integration-contracts.md`](0113-ecosystem-integration-contracts.md) | Accepted | Not audited |  |
| [`0114-ui-assets-facade-and-golden-path-wiring.md`](0114-ui-assets-facade-and-golden-path-wiring.md) | Accepted | Not audited |  |
| [`0115-available-space-and-non-reentrant-measurement.md`](0115-available-space-and-non-reentrant-measurement.md) | Proposed | Partially aligned | `AvailableSpace` + `LayoutConstraints`: `crates/fret-ui/src/layout_constraints.rs`; non-reentrant `measure_in` with recursion guard: `crates/fret-ui/src/tree/layout.rs` (`measure_stack`); Flex/Grid measure callbacks use `measure_in` (no layout re-entry): `crates/fret-ui/src/declarative/host_widget.rs`, `crates/fret-ui/src/declarative/host_widget/layout/flex.rs`, `grid.rs`; `Fill` only resolves to a definite size when the axis is definite during intrinsic measurement: `crates/fret-ui/src/declarative/host_widget/measure.rs` (`clamp_to_constraints_in_measure`); conformance tests: `crates/fret-ui/src/declarative/tests/layout.rs` (`fill_only_resolves_under_definite_available_space_in_measurement`) + `crates/fret-ui/src/tree/tests/measure_in.rs` (`measure_in_reentrancy_panics_in_debug_builds`); engine v2 also passes `AvailableSpace` through `compute_root_with_measure`: `crates/fret-ui/src/layout_engine.rs`. Gaps: tests still do not cover percent sizing and `MaxContent` behavior; release-mode re-entrancy diagnostics are not rate-limited yet. |
| [`0116-window-scoped-layout-engine-and-viewport-roots.md`](0116-window-scoped-layout-engine-and-viewport-roots.md) | Proposed | Partially aligned | Prototype window-owned engine + viewport roots exist behind `layout-engine-v2`: engine core `crates/fret-ui/src/layout_engine.rs` + flow builder `crates/fret-ui/src/layout_engine/flow.rs`; viewport root registration + independent solves: `crates/fret-ui/src/tree/layout.rs` (`register_viewport_root`, `precompute_viewport_root_flow_island`) + tests `crates/fret-ui/src/declarative/tests/layout.rs` (viewport roots do not couple fill). Gaps vs ADR: not yet a full window-wide “request/build then compute/apply” graph; current usage is still container-driven islands that temporarily take/put the engine (e.g. `crates/fret-ui/src/declarative/host_widget/layout/flex.rs`, `grid.rs`). |
| [`0117-trigger-composition-and-no-slot-aschild.md`](0117-trigger-composition-and-no-slot-aschild.md) | Proposed | Aligned | No general Slot/`asChild` mechanism is implemented; triggers are modeled as typed elements (typically `Pressable`) with children-based visuals across shadcn surfaces (e.g. `ecosystem/fret-ui-shadcn/src/popover.rs`, `tooltip.rs`, `select.rs`). A11y/relations are stamped via narrow helper functions (e.g. `ecosystem/fret-ui-kit/src/primitives/trigger_a11y.rs` + `apply_*_trigger_a11y` call sites). Layout engine wrapper skipping is explicit and restricted (not prop merging): `crates/fret-ui/src/layout_engine/flow.rs` (`passthrough_wrapper_child`). |
| [`0118-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`](0118-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md) | Proposed | Not audited |  |
| [`0119-effect-layers-and-backdrop-filters-scene-semantics-v1.md`](0119-effect-layers-and-backdrop-filters-scene-semantics-v1.md) | Proposed | Not audited |  |
| [`0120-renderer-intermediate-budgets-and-effect-degradation-v1.md`](0120-renderer-intermediate-budgets-and-effect-degradation-v1.md) | Proposed | Not audited |  |
| [`0121-streaming-images-and-video-surfaces.md`](0121-streaming-images-and-video-surfaces.md) | Proposed | Not audited |  |
| [`0122-offscreen-rendering-frame-capture-and-readback.md`](0122-offscreen-rendering-frame-capture-and-readback.md) | Proposed | Not audited |  |
| [`0123-streaming-upload-budgets-and-backpressure-v1.md`](0123-streaming-upload-budgets-and-backpressure-v1.md) | Proposed | Not audited |  |
| [`0124-renderer-capabilities-and-optional-zero-copy-imports.md`](0124-renderer-capabilities-and-optional-zero-copy-imports.md) | Proposed | Not audited |  |
| [`0125-renderer-extensibility-materials-effects-and-sandboxing-v1.md`](0125-renderer-extensibility-materials-effects-and-sandboxing-v1.md) | Proposed | Not audited |  |
| [`0126-streaming-image-update-effects-and-metadata-v1.md`](0126-streaming-image-update-effects-and-metadata-v1.md) | Proposed | Not audited |  |
| [`0127-frame-capture-options-and-determinism-v1.md`](0127-frame-capture-options-and-determinism-v1.md) | Proposed | Not audited |  |
| [`0128-delinea-headless-chart-engine.md`](0128-delinea-headless-chart-engine.md) | Accepted | Not audited |  |
| [`0129-delinea-transform-pipeline-and-datazoom-semantics.md`](0129-delinea-transform-pipeline-and-datazoom-semantics.md) | Proposed | Not audited |  |
| [`0130-delinea-axis-scales-and-coordinate-mapping.md`](0130-delinea-axis-scales-and-coordinate-mapping.md) | Proposed | Not audited |  |
| [`0131-delinea-marks-identity-and-renderer-contract.md`](0131-delinea-marks-identity-and-renderer-contract.md) | Proposed | Not audited |  |
| [`0132-delinea-large-data-and-progressive-rendering.md`](0132-delinea-large-data-and-progressive-rendering.md) | Proposed | Not audited |  |
| [`0133-delinea-interaction-and-hit-testing-contract.md`](0133-delinea-interaction-and-hit-testing-contract.md) | Proposed | Not audited |  |
| [`0134-delinea-multi-axis-and-layout-contract.md`](0134-delinea-multi-axis-and-layout-contract.md) | Proposed | Not audited |  |
| [`0135-window-styles-and-utility-windows.md`](0135-window-styles-and-utility-windows.md) | Proposed | Not implemented | ADR only; no portable window-style/utility-window surface wired yet. |
| [`0136-undo-redo-infrastructure-boundary.md`](0136-undo-redo-infrastructure-boundary.md) | Proposed | Not implemented | ADR only; no shared undo/redo substrate committed yet. |
| [`0137-canvas-widgets-and-interactive-surfaces.md`](0137-canvas-widgets-and-interactive-surfaces.md) | Proposed | Not implemented | ADR only; `fret-canvas` ecosystem crate not created yet. |
| [`0138-tooltip-scroll-dismissal.md`](0138-tooltip-scroll-dismissal.md) | Accepted | Aligned | Scroll dismissal: `UiTree::set_layer_scroll_dismiss_elements` (`crates/fret-ui/src/tree/layers.rs`) + dispatch in `crates/fret-ui/src/tree/dispatch.rs`; wiring in `ecosystem/fret-ui-kit/src/window_overlays/render.rs` and `ecosystem/fret-ui-shadcn/src/tooltip.rs` (plus tests). |
