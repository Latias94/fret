# ImUi Dear ImGui Gap Closure v1 - TODO

Status: Active
Last updated: 2026-05-30

## Worktree Convergence - 2026-05-26

- [x] Stop feature development in both dirty worktrees and record the convergence plan.
- [x] Checkpoint the main worktree before merging:
      `d078e25122 refactor(imui): checkpoint gap closure convergence slices`.
- [x] Checkpoint the `imui-imgui-editor-grade-refactor` worktree before merging:
      `05727e284b refactor(imui): checkpoint editor-grade convergence worktree`.
- [x] Merge `imui-imgui-editor-grade-refactor` back into `main`, resolving overlapping IMUI
      changes by topic instead of treating either side as globally authoritative.
- [x] Preserve the editor-grade facade/container/listbox organization, the main image-item owner
      split, and the union source gate coverage.
- [x] Run focused convergence gates and record the result in `EVIDENCE_AND_GATES.md`.
- [x] Continue follow-up IMUI development only from `F:/SourceCodes/Rust/fret` on `main`.

## Current Gap Refresh - 2026-05-27

- [x] Refresh the canonical workbench teaching docs so cookbook/examples no longer claim
      `imui_editor_workbench_demo` delegates to the workspace shell owner after the route was moved
      to direct editor-notes workflow mounting.
      Result: `docs/examples/README.md`, `apps/fret-cookbook/README.md`, and
      `apps/fret-cookbook/EXAMPLES.md` now describe the workbench as the canonical editor route
      that mounts editor-notes directly, while `imui_editor_proof_demo` remains the supporting
      dense panel / explicit stable identity proof. `tools/gate_imui_facade_teaching_source.py`
      now freezes that wording.
- [x] Refresh `P0_CURRENT_SOURCE_AUDIT_2026-05-06.md` and the P2 TODO status so the active gap map
      no longer treats `imui_editor_proof_demo` as the canonical editor-panel route after the
      workbench lane closed.
      Result: the active P0 read now names `cargo run -p fret-demo --bin
      imui_editor_workbench_demo` as the canonical product-facing editor workbench route, keeps
      `imui_editor_proof_demo`, `workspace_shell_demo`, and docking demos as supporting proof
      surfaces, and points current workbench verification at
      `imui_editor_workbench_golden_path_surface`.
- [x] Refresh `P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md` so the active gap map no longer treats
      ListBox, plot adapter, or style/theme preset picker as open candidate-only gaps after their
      closed proof lanes landed.
      Result: the catalog now records ListBox as a kit-owned container proof, plot as an opt-in
      `fret-plot/imui` adapter, and style/theme editing as editor-owned preset tooling exposed by
      the canonical workbench.

## Owner Split Follow-Ups - 2026-05-26

- [x] Split remaining IMUI facade root scope/basic/disclosure trait default method declarations out
      of `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` without changing the public
      `UiWriterImUiFacadeExt` trait, caller import behavior, push-id/disabled-scope, text,
      separator, debug-draw, collapsing-header, or tree-node method names and behavior.
      Result: `facade_writer.rs` keeps the single public trait hub plus surface macro expansion
      only. `scope_surface.rs`, `basic_surface.rs`, and `disclosure_surface.rs` own the remaining
      trait default declarations/forwarding, while `scope_methods.rs`, `basic_items.rs`, and
      `disclosure_controls` remain the behavior owners.
- [x] Split IMUI facade basic surface macro owner into text, debug-draw, and separator child
      owners without changing public trait method names, default option forwarding, response
      returns, macro expansion order, or concrete `basic_items` behavior ownership.
      Result: `basic_surface.rs` is now a module/re-export hub. `basic_surface/text.rs` owns
      text, wrapped text, and bullet text forwarding; `basic_surface/debug_draw.rs` owns
      debug-draw forwarding; and `basic_surface/separators.rs` owns separator and separator-text
      forwarding.
- [x] Split IMUI facade disclosure surface macro owner into collapsing-header and tree-node child
      owners without changing public trait method names, stable identity/depth docs, response
      returns, macro expansion order, or concrete `disclosure_controls` behavior ownership.
      Result: `disclosure_surface.rs` is now a module/re-export hub.
      `disclosure_surface/collapsing_header.rs` owns collapsing-header forwarding, while
      `disclosure_surface/tree_node.rs` owns tree-node forwarding and explicit depth guidance.
- [x] Split IMUI facade support hub into constants, geometry, runtime, state, and ui-writer child
      owners without changing public IMUI key names, frame preparation, point helpers, model
      change tracking, or `UiWriterUiKitExt` re-export paths.
      Result: `facade_support.rs` is now a module/re-export hub.
      `facade_support/constants.rs` owns IMUI key/timing constants, `geometry.rs` owns point and
      device-pixel helpers, `runtime.rs` owns frame preparation, `state.rs` owns model change
      tracking, and `ui_writer.rs` owns the bridge trait implementation.
- [x] Split IMUI facade scope method behavior owner into push-id and disabled-scope child owners
      without changing public facade method names, keyed identity scoping, disabled alpha/gating,
      runtime frame preparation, or `scope_surface` forwarding.
      Result: `facade_writer/scope_methods.rs` is now a module/re-export hub.
      `scope_methods/push_id.rs` owns keyed child facade execution and result propagation, while
      `scope_methods/disabled_scope.rs` owns disabled-scope wrapping, alpha, pointer blocking, and
      focus traversal gating.
- [x] Split IMUI facade container/layout trait default method declarations out of
      `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` without changing the public
      `UiWriterImUiFacadeExt` trait, caller import behavior, item-flow, same-line,
      dummy/spacing/indent, layout group, menu/tab bar, ListBox, grid, table, virtual-list, scroll,
      child-region method names, porting-sugar spacing docs, or container behavior.
      Result: `facade_writer.rs` keeps the single public trait hub and expands
      `facade_writer/container_surface.rs`. `container_surface.rs` owns container/layout trait
      default declarations/forwarding, while existing `container_methods/*` owners still carry the
      concrete layout/container behavior.
- [x] Split IMUI facade container surface macro owner into layout, menu/tab, collection, and
      region child owners without changing the public `UiWriterImUiFacadeExt` trait, caller import
      behavior, item-flow, menu/tab bar, ListBox, grid, table, virtual-list, scroll, child-region,
      or concrete `container_methods/*` behavior ownership.
      Result: `facade_writer/container_surface.rs` is now a module/re-export hub.
      `container_surface/layout.rs` owns layout/spacing forwarding,
      `container_surface/menu_tabs.rs` owns menu-bar and tab-bar forwarding,
      `container_surface/collections.rs` owns ListBox/grid/table/virtual-list forwarding, and
      `container_surface/regions.rs` owns scroll and child-region forwarding.
- [x] Split IMUI facade container collection surface macro owner into ListBox, grid, Table, and
      VirtualList child owners without changing public trait method names, collection forwarding,
      response returns, macro expansion order, or concrete `container_methods/*` behavior
      ownership.
      Result: `container_surface/collections.rs` is now a module/re-export hub.
      `collections/list_box.rs`, `collections/grid.rs`, `collections/table.rs`, and
      `collections/virtual_list.rs` own the corresponding trait forwarding groups.
- [x] Split IMUI facade container layout surface macro owner into flow and group child owners
      without changing public trait method names, item-flow/same-line/dummy/spacing/indent
      forwarding, horizontal/vertical forwarding, or concrete `container_methods/*` behavior
      ownership.
      Result: `container_surface/layout.rs` is now a module/re-export hub. `layout/flow.rs` owns
      item-flow, same-line, dummy, spacing, and indent forwarding, while `layout/groups.rs` owns
      horizontal and vertical group forwarding.
- [x] Split IMUI facade container layout method behavior owner into linear, grid/scroll, and
      child-region child owners without changing public facade methods, build-focus forwarding,
      horizontal/vertical/grid/scroll element routing, child-region response behavior, or
      `container_methods` re-export paths.
      Result: `facade_writer/container_methods/layout.rs` is now a module/re-export hub.
      `layout/linear.rs` owns horizontal/vertical forwarding, `layout/grid_scroll.rs` owns grid
      and scroll forwarding, and `layout/child_region.rs` owns child-region forwarding and response
      return.
- [x] Split IMUI facade container collection method behavior owner into ListBox, Table, and
      VirtualList child owners without changing public facade methods, build-focus forwarding,
      collection element routing, response returns, or `container_methods` re-export paths.
      Result: `facade_writer/container_methods/collections.rs` is now a module/re-export hub.
      `collections/list_box.rs` owns ListBox option normalization and element forwarding,
      `collections/table.rs` owns Table forwarding and response return, and
      `collections/virtual_list.rs` owns VirtualList forwarding and response return.
- [x] Split IMUI facade container flow method behavior owner into sequence, spacer, and indent
      child owners without changing public facade methods, build-focus forwarding, porting-sugar
      layout routing, or `container_methods` re-export paths.
      Result: `facade_writer/container_methods/flow.rs` is now a module/re-export hub.
      `flow/sequences.rs` owns item-flow and same-line forwarding, `flow/spacers.rs` owns dummy and
      spacing forwarding, and `flow/indent.rs` owns indent forwarding.
- [x] Split IMUI facade container menu/tab method behavior owner into menu-bar and tab-bar child
      owners without changing public facade methods, build-focus forwarding, menu/tab element
      routing, tab response return, or `container_methods` re-export paths.
      Result: `facade_writer/container_methods/menu_tabs.rs` is now a module/re-export hub.
      `menu_tabs/menu.rs` owns menu-bar forwarding, and `menu_tabs/tabs.rs` owns tab-bar forwarding
      and response return.
- [x] Route the IMUI facade inherent ListBox wrapper through the collection method owner instead of
      direct `list_box_controls` element construction without changing public inherent method names,
      build-focus forwarding, or ListBox behavior.
      Result: `facade_writer/container_wrappers/collections.rs` now delegates
      `list_box_with_options(...)` to `container_methods::list_box_with_options(...)`, matching
      the Table and VirtualList wrapper pattern.
- [x] Split IMUI facade container collection inherent wrapper owner into ListBox, Table, and
      VirtualList child owners without changing public inherent method names, build-focus
      forwarding, response returns, or `container_methods/*` delegation.
      Result: `container_wrappers/collections.rs` is now a module hub. `collections/list_box.rs`
      owns ListBox label/options wrappers, `collections/table.rs` owns Table wrappers, and
      `collections/virtual_list.rs` owns VirtualList wrappers.
- [x] Split IMUI facade floating/popup/tooltip/drag/window trait default method declarations out of
      `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` without changing the public
      `UiWriterImUiFacadeExt` trait, caller import behavior, floating area/window, popup, tooltip,
      drag/drop method names, popup/window docs, or floating/popup/drag behavior.
      Result: `facade_writer.rs` keeps the single public trait hub and expands
      `facade_writer/floating_surface.rs` at the original floating, tooltip/drag, and window
      positions. `floating_surface.rs` owns floating layer/area, popup open/drop/begin, tooltip,
      drag/drop, and in-window floating-window trait default declarations/forwarding. Existing
      `floating_popup/*` owners still delegate to concrete floating, popup, tooltip, drag/drop, and
      window behavior modules.
- [x] Split IMUI facade floating surface macro owner into popup/floating-area, tooltip/drag, and
      window child owners without changing the public `UiWriterImUiFacadeExt` trait, facade macro
      expansion points, popup/window docs, or concrete `floating_popup/*` behavior ownership.
      Result: `facade_writer/floating_surface.rs` is now a module/re-export hub.
      `floating_surface/popup.rs` owns floating layer/area plus popup open/drop/begin forwarding,
      `floating_surface/tooltip_drag.rs` owns tooltip and drag/drop forwarding, and
      `floating_surface/window.rs` owns in-window floating-window forwarding.
- [x] Split IMUI facade floating tooltip/drag surface macro owner into tooltip and drag/drop child
      owners without changing public trait method names, tooltip forwarding, drag/drop forwarding,
      drag/drop docs, or concrete `floating_popup/*` behavior ownership.
      Result: `floating_surface/tooltip_drag.rs` is now a module/re-export hub.
      `tooltip_drag/tooltip.rs` owns tooltip text/custom-content forwarding, while
      `tooltip_drag/drag_drop.rs` owns typed drag source/drop target forwarding and docs.
- [x] Split IMUI facade floating popup surface macro owner into floating-area/layer, popup state,
      and begin-popup child owners without changing the public `UiWriterImUiFacadeExt` trait,
      caller import behavior, floating layer/area/drag-surface, popup open/drop/close, popup
      menu/modal method names, or concrete `floating_popup/*` behavior ownership.
      Result: `floating_surface/popup.rs` is now a module/re-export hub.
      `floating_surface/popup/area.rs` owns floating layer/area/drag-surface forwarding,
      `floating_surface/popup/state.rs` owns popup open-model/drop/open/close forwarding, and
      `floating_surface/popup/begin.rs` owns popup menu/modal begin forwarding.
- [x] Split IMUI facade menu/selection trait default method declarations out of
      `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` without changing the public
      `UiWriterImUiFacadeExt` trait, caller import behavior, menu item, begin menu/submenu,
      selectable, multi-selectable, combo, context-menu method names, focusable recording wrappers,
      popup/menu behavior, or selectable/combo behavior.
      Result: `facade_writer.rs` keeps the single public trait hub and expands
      `facade_writer/menu_selection_surface.rs` inside it. `menu_selection_surface.rs` owns menu
      item, begin menu/submenu, selectable, multi-selectable, combo, and context-menu trait default
      declarations/forwarding. Existing `menu_items.rs`, `selection_combo.rs`, and
      `floating_popup/*` owners still carry inherent wrappers and underlying behavior.
- [x] Split IMUI facade menu/selection surface macro owner into menu-item, menu-family,
      selection/combo, and context-popup child owners without changing the public
      `UiWriterImUiFacadeExt` trait, caller import behavior, menu item, begin menu/submenu,
      selectable, multi-selectable, combo, context-menu method names, or underlying behavior
      owners.
      Result: `facade_writer/menu_selection_surface.rs` is now a module/re-export hub.
      `menu_selection_surface/menu_items.rs` owns menu item forwarding,
      `menu_selection_surface/menu_family.rs` owns begin menu/submenu forwarding,
      `menu_selection_surface/selection_combo.rs` owns selectable/multi-selectable/combo
      forwarding, and `menu_selection_surface/context_popup.rs` owns context-menu popup
      forwarding.
- [x] Split IMUI facade menu/selection selection-combo surface macro owner into selection and
      combo child owners without changing public trait method names, default option forwarding,
      response returns, macro expansion order, focusable recording wrappers, or concrete
      selectable/combo behavior ownership.
      Result: `menu_selection_surface/selection_combo.rs` is now a module/re-export hub.
      `selection_combo/selectables.rs` owns selectable and multi-selectable forwarding, while
      `selection_combo/combo.rs` owns combo forwarding.
- [x] Split IMUI facade selectable/combo inherent wrapper behavior owner into selectable and combo
      child owners without changing public inherent method names, focusable recording, disabled
      checks, selectable/multi-selectable delegation, combo delegation, or `fret-imui` thinness.
      Result: `facade_writer/selection_combo.rs` is now a module hub.
      `selection_combo/selectables.rs` owns selectable and multi-selectable focusable-recording
      wrappers, while `selection_combo/combo.rs` owns direct combo focusable-recording wrappers.
- [x] Split IMUI facade model/control trait default method declarations out of
      `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` without changing the public
      `UiWriterImUiFacadeExt` trait, caller import behavior, checkbox/radio/switch,
      slider/combo model, input text, picker/history text, textarea method names, focusable
      recording wrappers, or model-control behavior.
      Result: `facade_writer.rs` keeps the single public trait hub and expands
      `facade_writer/model_surface.rs` inside it. `model_surface.rs` owns checkbox/radio/switch,
      slider/combo model, input text model, input text picker/history model, and textarea model
      trait default declarations/forwarding. Existing `boolean_wrappers.rs`, `value_models.rs`,
      and `text_models.rs` inherent wrappers still own focusable recording.
- [x] Split IMUI facade model surface macro owner into boolean, value/combo, and text child owners
      without changing the public `UiWriterImUiFacadeExt` trait, caller import behavior,
      checkbox/radio/switch, slider/combo model, input text, picker/history text, textarea method
      names, focusable recording wrappers, or model-control behavior.
      Result: `facade_writer/model_surface.rs` is now a module/re-export hub.
      `model_surface/boolean.rs` owns checkbox/radio/switch model forwarding,
      `model_surface/value_combo.rs` owns slider and combo-model forwarding, and
      `model_surface/text.rs` owns input text, input text picker/history, and textarea forwarding.
- [x] Split IMUI facade text model surface macro owner into input, picker/history, and textarea
      child owners without changing public trait method names, default option forwarding,
      response returns, macro expansion order, focusable recording wrappers, or concrete
      text-control behavior ownership.
      Result: `model_surface/text.rs` is now a module/re-export hub. `text/input.rs` owns
      input-text model forwarding, `text/picker.rs` owns completion/history picker forwarding, and
      `text/textarea.rs` owns textarea forwarding.
- [x] Split IMUI facade boolean-control inherent wrapper behavior owner into checkbox, radio, and
      switch child owners without changing public inherent method names, disabled checks,
      focusable recording, trait delegation paths, or `fret-imui` thinness.
      Result: `facade_writer/boolean_wrappers.rs` is now a module hub.
      `boolean_wrappers/checkbox.rs` owns checkbox model wrappers,
      `boolean_wrappers/radio.rs` owns radio wrappers, and `boolean_wrappers/switch.rs` owns switch
      model wrappers.
- [x] Split IMUI facade value/combo-model inherent wrapper behavior owner into slider and
      combo-model child owners without changing public inherent method names, disabled checks,
      focusable recording, trait delegation paths, or `fret-imui` thinness.
      Result: `facade_writer/value_models.rs` is now a module hub. `value_models/slider.rs` owns
      slider model wrappers, and `value_models/combo_model.rs` owns combo-model wrappers.
- [x] Split IMUI facade text-model inherent wrapper behavior owner into input, picker, and textarea
      child owners without changing public inherent method names, disabled/focusable checks, picker
      focusable calculation, trait delegation paths, or `fret-imui` thinness.
      Result: `facade_writer/text_models.rs` is now a module hub. `text_models/input.rs` owns
      single-line input wrappers, `text_models/picker.rs` owns completion/history picker wrappers,
      and `text_models/textarea.rs` owns textarea wrappers.
- [x] Split IMUI facade button/image/action trait default method declarations out of
      `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` without changing the public
      `UiWriterImUiFacadeExt` trait, caller import behavior, button/image/action method names,
      button-command presentation forwarding, focusable recording wrappers, image-button option
      normalization, or button/image smoke behavior.
      Result: `facade_writer.rs` keeps the single public trait hub and expands
      `facade_writer/button_surface.rs` inside it. `button_surface.rs` owns button,
      small/arrow/invisible button, image item/button, action button, payload action button, and
      button-command trait default declarations/forwarding. Existing `button_actions.rs` /
      `button_actions/*` inherent wrappers still own focusable recording, while `image_items.rs`
      still owns image-button option normalization.
- [x] Split IMUI facade button surface macro owner into plain-button, image-button, and
      action-button child owners without changing the public `UiWriterImUiFacadeExt` trait, caller
      import behavior, button/image/action method names, command presentation forwarding,
      focusable recording wrappers, or image-button option normalization.
      Result: `facade_writer/button_surface.rs` is now a module/re-export hub.
      `button_surface/plain.rs` owns plain/small/arrow/invisible button forwarding,
      `button_surface/images.rs` owns image item/button forwarding, and
      `button_surface/actions.rs` owns action button, payload action button, and button-command
      forwarding.
- [x] Split IMUI facade button action surface macro owner into action, payload-action, and
      command-button child owners without changing public trait method names, default option
      forwarding, payload bounds, command presentation forwarding, macro expansion order,
      focusable recording wrappers, or concrete button/action behavior ownership.
      Result: `button_surface/actions.rs` is now a module/re-export hub.
      `actions/action.rs` owns action button forwarding, `actions/payload.rs` owns payload action
      button forwarding, and `actions/command.rs` owns command-button forwarding.
- [x] Split IMUI facade button/action inherent wrapper behavior owner into button-family and
      command-button child owners without changing public inherent method names, focusable
      recording, command metadata lookup, action/payload action wrappers, or `fret-imui` thinness.
      Result: `facade_writer/button_actions.rs` keeps only module wiring and the private
      button-command helper re-export. `button_actions/buttons.rs` owns plain/small/arrow/invisible
      button wrappers, `button_actions/commands.rs` owns command-button wrappers, and existing
      `action_methods.rs` / `button_command.rs` now import directly from the facade parent instead
      of relying on root hub imports.
- [x] Split IMUI shared pressable item pointer hook bodies out of
      `ecosystem/fret-ui-kit/src/imui/item_behavior/install.rs` without changing shared button,
      checkbox/radio, selectable, combo, image-item, debug-draw pressable, context-menu,
      pointer-click, double-click, drag, long-press, lifecycle, or response population behavior.
      Result: `item_behavior/install.rs` keeps hook clearing, model capture, and behavior assembly;
      `item_behavior/install/pointer_down.rs` owns lifecycle activation and drag start
      preparation; `pointer_move.rs` owns drag-threshold move handling; `pointer_up.rs` owns
      lifecycle deactivation, drag finish, context-menu transients, pointer-click modifier capture,
      and double-click transients.
- [x] Split IMUI table response header/resize accessors out of
      `ecosystem/fret-ui-kit/src/imui/response/widgets/table.rs` without changing public response
      names, field privacy, header lookup behavior, resize drag accessors, width clamping, table
      smoke behavior, or table-column visibility behavior.
      Result: `response/widgets/table.rs` owns `TableResponse` aggregation and header lookup,
      `response/widgets/table/header.rs` owns `TableHeaderResponse`, and
      `response/widgets/table/resize.rs` owns `TableColumnResizeResponse`.
- [x] Split IMUI debug-draw root draw-list state and facade entry glue out of
      `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs` without changing public debug-draw
      list/options/response names, command recording, channel merging, summary projection,
      element mounting, or debug-draw smoke behavior.
      Result: `debug_draw_controls/draw_list.rs` owns `ImUiDebugDrawList` and channel-split state,
      while `debug_draw_controls/facade.rs` owns `debug_draw_with_options(...)` list capture,
      summaries, command boxing, keyed element mounting, and response assembly.
- [x] Split IMUI popup-overlay root state helpers and context-menu wrapper out of
      `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs` into private owners without changing popup
      open/close/drop/open-at behavior, context-menu anchor fallback, menu/modal entrypoints,
      popup-hover behavior, or popup/menu facade behavior.
      Result: `popup_overlay/state.rs` owns popup open model lookup, drop/open/open-at/close
      mutations, keep-alive generation writes, anchor writes, and redraw requests.
      `popup_overlay/context_menu.rs` owns context-menu trigger inspection, 1px fallback anchor
      construction, and delegation to the menu owner.
- [x] Split IMUI tab-bar item builder methods out of
      `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs` into a private owner without
      changing public `ImUiTabBar` method names, label identity parsing, panel test-id fallback,
      focused-child capture, tab smoke behavior, or fret-imui tab behavior.
      Result: `tab_family_controls/item_methods.rs` owns `tab_item`,
      `tab_item_with_options`, `begin_tab_item`, and `begin_tab_item_with_options`; the root file
      keeps `ImUiTabBar` storage and `tab_bar_element` assembly.
- [x] Split IMUI floating-area option/context records into options and opaque context private
      owners without changing public option/context names, area defaults, accessor-first context
      shape, root re-exports, floating-area smoke behavior, or fret-imui floating behavior.
      Result: `floating_options/area.rs` is now a public re-export hub.
      `floating_options/area/options.rs` owns `FloatingAreaOptions`, while
      `floating_options/area/context.rs` owns `FloatingAreaContext` storage and accessors.
- [x] Split IMUI floating-window option records into behavior, resize, and root window option
      private owners without changing public option names, defaults, builder methods, root
      re-exports, floating-window smoke behavior, or fret-imui floating behavior.
      Result: `floating_options/window.rs` is now a public re-export hub.
      `floating_options/window/behavior.rs` owns `FloatingWindowOptions`,
      `floating_options/window/resize.rs` owns `FloatingWindowResizeOptions`, and
      `floating_options/window/options.rs` owns `WindowOptions` and its builder methods.
- [x] Split editor-owned IMUI style/theme preset picker option records into a private owner
      without changing public control names, option fields/defaults, controls re-exports, IMUI
      adapter callsite, listbox semantics, selected-state behavior, or reversible preset replay.
      Result: `controls/editor_theme_preset_picker.rs` keeps picker rendering and behavior, while
      `controls/editor_theme_preset_picker/options.rs` owns `EditorThemePresetPickerOptions`.
- [x] Split IMUI leaf control option records into selection, tab-item, and slider private owners
      without changing public option type names, fields, defaults, root re-exports, selectable
      smoke behavior, tab/menu behavior, or slider model behavior.
      Result: `options/controls/selection.rs`, `options/controls/tab.rs`, and
      `options/controls/value.rs` are now public re-export hubs. `selection/options.rs` owns
      `SelectableOptions`, `tab/options.rs` owns `TabItemOptions`, and `value/slider.rs` owns
      `SliderOptions`.
- [x] Split IMUI input-text-picker option records into filter and options private owners without
      changing public option type names, filter matching, default popup/input/options, root
      re-exports, picker smoke behavior, or fret-imui picker behavior.
      Result: `options/controls/text/picker.rs` is now a public re-export hub.
      `picker/filter.rs` owns `InputTextPickerFilter`, and `picker/options.rs` owns
      `InputTextPickerOptions`.
- [x] Split IMUI textarea option records into submit-key and options private owners without
      changing public option type names, multiline submit/cancel defaults, stable-line-box default,
      root re-exports, textarea smoke behavior, or fret-imui textarea model behavior.
      Result: `options/controls/text/textarea.rs` is now a public re-export hub.
      `textarea/submit_key.rs` owns `TextAreaSubmitKey`, and `textarea/options.rs` owns
      `TextAreaOptions`.
- [x] Split IMUI input-text option records into mode and options private owners without changing
      public option type names, text-field semantics default, command-policy defaults, filter
      fields, root re-exports, input-text option smoke behavior, or fret-imui text-model behavior.
      Result: `options/controls/text/input.rs` is now a public re-export hub. `input/mode.rs` owns
      `InputTextMode`, and `input/options.rs` owns `InputTextOptions`.
- [x] Split IMUI popup option records into popup-menu and popup-modal private owners without
      changing public option type names, popup placement defaults, menu size/modal/auto-focus
      defaults, modal size/outside-press defaults, root re-exports, popup smoke behavior, or
      popup-hover behavior.
      Result: `options/menus/popup.rs` is now a public re-export hub. `popup/menu.rs` owns
      `PopupMenuOptions`, and `popup/modal.rs` owns `PopupModalOptions`.
- [x] Split IMUI misc option records into drag-source, drop-target, separator-text, and bullet-text
      private owners without changing public option type names, default enabled/cross-window
      behavior, `test_id` fields, root re-exports, drag/drop smoke behavior, separator/bullet smoke
      behavior, or fret-imui composition/drag behavior.
      Result: `options/misc.rs` is now a public re-export hub. `misc/drag_source.rs` owns
      `DragSourceOptions`, `misc/drop_target.rs` owns `DropTargetOptions`,
      `misc/separator_text.rs` owns `SeparatorTextOptions`, and `misc/bullet_text.rs` owns
      `BulletTextOptions`.
- [x] Split IMUI spacer flow option records into dummy, spacing, and indent private owners without
      changing public option type names, default optional size, indent token default, `test_id`
      fields, flow re-exports, container smoke behavior, or porting-sugar behavior.
      Result: `options/containers/flow/spacer.rs` is now a private re-export hub.
      `flow/spacer/dummy.rs` owns `DummyOptions`, `flow/spacer/spacing.rs` owns `SpacingOptions`,
      and `flow/spacer/indent.rs` owns `IndentOptions`.
- [x] Split IMUI linear flow option records into horizontal and vertical private owners without
      changing public option type names, default gaps, default center/stretch item alignment,
      `test_id` fields, flow re-exports, container smoke behavior, or porting-sugar behavior.
      Result: `options/containers/flow/linear.rs` is now a private re-export hub.
      `flow/linear/horizontal.rs` owns `HorizontalOptions`, and `flow/linear/vertical.rs` owns
      `VerticalOptions`.
- [x] Split IMUI inline flow option records into item-flow and same-line private owners without
      changing public option type names, default gaps, default alignment/stretch behavior,
      `test_id` fields, flow re-exports, container smoke behavior, or porting-sugar behavior.
      Result: `options/containers/flow/inline.rs` is now a private re-export hub.
      `flow/inline/item_flow.rs` owns `ItemFlowOptions`, and `flow/inline/same_line.rs` owns
      `SameLineOptions`.
- [x] Split IMUI child-region option records into chrome, body options, and resize private owners
      without changing public option type names, default chrome, scroll/layout fields, resize
      defaults, resize builder methods, root re-exports, child-region smoke behavior, or
      composition behavior.
      Result: `options/containers/child_region.rs` is now a public re-export hub.
      `child_region/chrome.rs` owns `ChildRegionChrome`, `child_region/options.rs` owns
      `ChildRegionOptions`, and `child_region/resize.rs` owns `ChildRegionResizeXOptions` and
      `ChildRegionResizeYOptions`.
- [x] Split IMUI table/row/cell option records into private owners without changing public option
      type names, default values, `TableOptions` debug formatting, horizontal-scroll defaults,
      row/cell background seams, root re-exports, table smoke behavior, or table composition
      behavior.
      Result: `options/collections/table.rs` is now a public re-export hub. `table/root.rs` owns
      `TableOptions`, `table/row.rs` owns `TableRowOptions`, and `table/cell.rs` owns
      `TableCellOptions`.
- [x] Split IMUI menu option records into menu-bar, begin-menu/submenu, and menu-item private
      owners without changing public option type names, default values, submenu popup placement,
      shortcut fields, root re-exports, menu smoke behavior, or menu/tab interaction behavior.
      Result: `options/menus/menu.rs` is now a public re-export hub. `menu/bar.rs` owns
      `MenuBarOptions`, `menu/begin.rs` owns `BeginMenuOptions` and `BeginSubmenuOptions`, and
      `menu/item.rs` owns `MenuItemOptions`.
- [x] Split IMUI combo control option types into direct-combo and model-combo private owners
      without changing public option type names, default values, placeholder text, popup defaults,
      shortcut fields, facade imports, combo smoke behavior, or combo-model behavior.
      Result: `options/controls/combo.rs` is now a public re-export hub.
      `combo/direct.rs` owns `ComboOptions`; `combo/model.rs` owns `ComboModelOptions`.
- [x] Split IMUI boolean control option types into checkbox, radio, and switch private owners
      without changing public option type names, default values, shortcut fields, facade imports,
      button smoke coverage, or boolean control behavior.
      Result: `options/controls/boolean.rs` is now a public re-export hub.
      `boolean/checkbox.rs`, `boolean/radio.rs`, and `boolean/switch.rs` own the three option
      structs and their default values.
- [x] Split IMUI disclosure control option types into collapsing-header and tree-node private
      owners without changing public option type names, default values, shortcut fields, hierarchy
      metadata defaults, facade imports, disclosure smoke behavior, or disclosure-control tests.
      Result: `options/controls/disclosure.rs` is now a public re-export hub.
      `disclosure/collapsing_header.rs` owns `CollapsingHeaderOptions`, and
      `disclosure/tree_node.rs` owns `TreeNodeOptions`.
- [x] Split IMUI button/image control option types into button and image-item private owners
      without changing public option type names, default values, image-item builder methods,
      shortcut fields, facade imports, button smoke behavior, or image-item smoke behavior.
      Result: `options/controls/button_image.rs` is now a public re-export hub.
      `button_image/button.rs` owns `ButtonArrowDirection`, `ButtonVariant`, and `ButtonOptions`;
      `button_image/image.rs` owns `ImageItemVariant` and `ImageItemOptions`.
- [x] Split IMUI debug-draw rect path construction into plain-rect and rounded-rect private owners
      without changing clockwise rect command ordering, rounded-rect effective rounding clamp,
      per-corner sampling, fallback square points, path-builder call sites, path tests, or public
      debug-draw behavior.
      Result: `debug_draw_controls/paths/rects.rs` is now a private re-export hub.
      `rects/plain.rs` owns plain closed rect path commands and `rects/rounded.rs` owns
      rounded-rect point generation plus corner arc sampling.
- [x] Split IMUI debug-draw linear path construction into polyline, polygon fill, and primitive
      private owners without changing open/closed stroke point requirements, polyline command
      ordering, convex/concave fill forwarding, triangle/quad closure, paint-shape call sites, path
      tests, or public debug-draw behavior.
      Result: `debug_draw_controls/paths/linear.rs` is now a private re-export hub.
      `linear/polyline.rs` owns stroke point requirements and polyline commands,
      `linear/fills.rs` owns convex/concave fill forwarding, and `linear/primitives.rs` owns
      triangle/quad path construction.
- [x] Split IMUI debug-draw round path construction into circle, ngon, and ellipse private owners
      without changing circle cubic approximation, ngon validation/point generation, ellipse
      default segment fallback/rotation, paint-shape call sites, path tests, or public debug-draw
      behavior.
      Result: `debug_draw_controls/paths/round.rs` is now a private re-export hub.
      `round/circle.rs` owns circle cubic path construction, `round/ngon.rs` owns regular polygon
      path construction, and `round/ellipse.rs` owns ellipse path validation and rotation sampling.
- [x] Split IMUI debug-draw path sampling helpers into segment, arc, and Bezier private owners
      without changing default segment fallback, arc/elliptical arc point generation, Bezier point
      interpolation, path-builder command output, or public debug-draw behavior.
      Result: `debug_draw_controls/paths/sampling.rs` is now a private re-export hub.
      `sampling/segments.rs` owns default segment fallback, `sampling/arcs.rs` owns circular and
      elliptical arc point sampling, and `sampling/beziers.rs` owns quadratic/cubic Bezier point
      interpolation.
- [x] Split IMUI debug-draw geometry helpers into finite, rectangle, and triangle private owners
      without changing finite checks, rect emptiness/finite checks, rect quad point generation,
      effective rounding clamp rules, triangle degeneracy checks, indexed triangle lookup,
      sequential index generation, or public debug-draw behavior.
      Result: `debug_draw_controls/geometry.rs` is now a private re-export hub.
      `geometry/finite.rs` owns point/UV/vertex finite checks; `geometry/rects.rs` owns rect
      checks and rounding clamp rules; `geometry/triangles.rs` owns triangle degeneracy/
      drawability, indexed triangle lookup, and sequential index generation.
- [x] Split IMUI facade menu-item inherent wrappers into a private owner without changing plain
      menu-item wrappers, checkbox/radio wrappers, action menu item wrappers, focusable recording,
      begin-menu/submenu wrappers, command presentation forwarding, or public facade method names.
      Result: `facade_writer/menu_items/item_methods.rs` owns plain, checkbox/radio, and action
      menu item inherent wrappers. `facade_writer/menu_items.rs` keeps begin-menu/submenu inherent
      wrappers and command menu item wiring.
- [x] Split IMUI menu dispatch checked/action entry variants into private owners without changing
      plain menu-item routing, pressable-hook entry routing, checkbox/radio semantics, action
      dispatch forwarding, label identity handling, mount routing, or public facade menu item
      behavior.
      Result: `menu_controls/routing/dispatch/entries/checked.rs` owns checkbox/radio entry
      wrappers and checked-state semantics; `entries/action.rs` owns action entry forwarding.
      `dispatch/entries.rs` keeps plain entry routing, shared implementation forwarding,
      pressable-hook entry routing, and private re-exports.
- [x] Split IMUI facade button action inherent wrappers into a private owner without changing plain
      button wrappers, command button forwarding, action dispatch, payload action dispatch,
      focusable recording, response projection, or public facade method names.
      Result: `facade_writer/button_actions/action_methods.rs` owns `action_button`,
      `action_button_with_options`, `action_payload_button`, and
      `action_payload_button_with_options` inherent wrappers. `facade_writer/button_actions.rs`
      keeps ordinary button wrappers and command button wiring.
- [x] Split IMUI text-picker core input-root phase into a private owner without changing model
      reads, candidate filtering, keyboard snapshot preparation, popup snapshot reads, expanded
      semantics, input-root request construction, keyboard-handler installation, popup open
      policy, pick response merge, or public text-picker facade behavior.
      Result: `text_picker_controls/core/input_root.rs` owns prepared input-root request
      construction, root mounting, response extraction, and popup item test-id base forwarding.
      `text_picker_controls/core.rs` keeps model/candidate/keyboard/open-policy/popup/response
      orchestration.
- [x] Split IMUI facade-core disabled-scope behavior into a private owner without changing
      `ImUiFacade` storage, keyed id helpers, `UiWriter` implementation, disabled-depth handling,
      pointer event swallowing, opacity dimming, focus traversal gating, scoped runtime
      preparation, or public facade behavior.
      Result: `facade_writer/facade_core/disabled_scope.rs` owns `ImUiFacade::disabled_scope`
      behavior. `facade_writer/facade_core.rs` keeps the facade storage shape, focus recording,
      keyed id helpers, and `UiWriter` implementation.
- [x] Split IMUI table builder row and cell methods into private owners without changing
      `ImUiTable` / `ImUiTableRow` public methods, row key scopes, row/cell test-id derivation,
      child IMUI mounting, cell packing, text-cell rendering, or table facade behavior.
      Result: `table_controls/builder/row_methods.rs` owns `row` / `row_with_options` row
      collection and keyed row scopes; `table_controls/builder/cell_methods.rs` owns `cell` /
      `cell_with_options` / `cell_text` / `cell_text_with_options` child mounting and cell
      packing. `table_controls/builder.rs` keeps built row/cell data shapes and `build_table_rows`.
- [x] Split IMUI child-region resize stack assembly into a private owner without changing
      resizable child-region detection, scroll/content construction, resize handle test-id
      defaults, X/Y resize response writes, stack layout refinement, stack root test-id routing, or
      public child-region facade/response behavior.
      Result: `child_region/resize_stack.rs` owns resize handle test-id derivation, X/Y handle
      creation, stack layout/style projection, children ordering, and resizable root test-id
      stamping. `child_region.rs` keeps option normalization, scroll owner dispatch, response
      aggregation, and the non-resizable vs resizable branch.
- [x] Split IMUI child-region scroll/content/chrome assembly into a private owner without changing
      IMUI child mounting, scroll axis/options forwarding, framed/bare chrome, content/viewport/
      root test-id routing, resize layout override, resize handle assembly, stack root test-id
      routing, or public child-region facade/response behavior.
      Result: `child_region/scroll.rs` owns scroll-area builder construction, content mounting,
      framed chrome, handle forwarding, viewport test-id routing, and non-resizable root test-id
      stamping. `child_region.rs` keeps resize option detection, resize handle assembly, stack
      layout/test-id routing, and response aggregation.
- [x] Split IMUI popup-modal overlay identity and request submission into a private owner without
      changing overlay id naming, modal root naming, trigger forwarding, open-model forwarding,
      instant modal presence, layer children, dismiss request forwarding, initial focus handoff, or
      public popup modal facade behavior.
      Result: `popup_overlay/modal/request.rs` owns modal overlay id/root-name construction and
      `OverlayRequest::modal` submission. `popup_overlay/modal.rs` keeps open-state gating,
      layout/dismiss/layer owner dispatch, and the final request input assembly.
- [x] Split IMUI popup-modal open and keep-alive state handling into a private owner without
      changing popup store model identity, closed-modal render gating, keep-alive generation
      writeback, modal root naming, dismiss policy, layer assembly, focus initialization, overlay
      request assembly, or public popup modal facade behavior.
      Result: `popup_overlay/modal/state.rs` owns modal open-model lookup, is-open reads, and
      keep-alive generation writeback. `popup_overlay/modal.rs` keeps dismiss policy creation,
      overlay identity/root naming, layout owner dispatch, layer owner dispatch, overlay request
      assembly, and final focus target selection.
- [x] Split IMUI popup-modal layer and panel assembly into a private owner without changing popup
      store open state reads, keep-alive generation, modal root naming, backdrop barrier behavior,
      centered panel layout, facade child mounting, focus initialization, overlay request assembly,
      or public popup modal facade behavior.
      Result: `popup_overlay/modal/layer.rs` owns modal layer/root mounting, barrier construction,
      panel semantics mounting, facade child rendering, focus-state construction, and panel focus
      handoff. `popup_overlay/modal.rs` keeps open-state gating, keep-alive writeback, dismiss
      policy creation, overlay request assembly, and final focus target selection.
- [x] Split IMUI popup-modal dismiss request policy into a private owner without changing popup
      store open state reads, keep-alive generation, modal root naming, backdrop/panel assembly,
      Escape close behavior, outside-press close option, dismiss prevention, focus initialization,
      or public popup modal facade behavior.
      Result: `popup_overlay/modal/dismiss.rs` owns modal `OnDismissRequest` policy for Escape,
      optional outside press, and default prevention. `popup_overlay/modal.rs` keeps open-state
      gating, keep-alive writeback, layer/panel assembly, overlay request assembly, and focus
      initialization.
- [x] Split IMUI button pressable props and a11y assembly into a private owner without changing
      enabled/focusable projection, variant layout application, button a11y metadata, chrome
      assembly, activation/keyboard/response dispatch, or public button facade behavior.
      Result: `button_controls/behavior/props.rs` owns `PressableProps` construction, focusable
      gating, variant layout, and a11y metadata. `button_controls/behavior.rs` keeps chrome owner
      dispatch, behavior owner dispatch, response projection dispatch, and visual resolution.
- [x] Split IMUI button pressable response projection into a private owner without changing
      clicked transient consumption, shared pressable item response population, lifecycle/hover/
      drag response projection, visual assembly, or public button facade behavior.
      Result: `button_controls/behavior/response.rs` owns button clicked transient consumption and
      shared `PressableItemResponseInput` projection. `button_controls/behavior.rs` keeps
      pressable props/chrome assembly, activation/keyboard owner dispatch, and visual assembly.
- [x] Split IMUI button pressable activation behavior into a private owner without changing
      pressable props/chrome assembly, keyboard lifecycle marking, clicked transient recording,
      action dispatch, response population, or public button facade behavior.
      Result: `button_controls/behavior/activation.rs` owns pressable activate-hook installation,
      keyboard activation lifecycle marking, clicked transient recording, action dispatch, and
      notify. `button_controls/behavior.rs` keeps pressable props/chrome assembly, keyboard owner
      dispatch, response population, and visual assembly.
- [x] Split IMUI button pressable keyboard behavior into a private owner without changing
      pressable props/chrome assembly, action dispatch, shortcut repeat policy, keyboard lifecycle
      marking, context-menu key handling, response population, or public button facade behavior.
      Result: `button_controls/behavior/keyboard.rs` owns focused button activate-shortcut handling
      and keyboard context-menu requests. `button_controls/behavior.rs` keeps pressable props,
      action activation, response population, and visual assembly.
- [x] Split IMUI debug-draw round path-command paint dispatch into stroked and filled private
      owners without changing public draw-list commands, round path paint routing, stroke/fill
      painter calls, command fallthrough, or debug-draw smoke behavior.
      Result: `paint_shapes/path_commands/round.rs` now only dispatches to `round/stroked.rs` and
      `round/filled.rs`; those owners handle stroked circle/ngon/ellipse commands and filled
      circle/ngon/ellipse commands respectively.
- [x] Split IMUI debug-draw linear path-command paint dispatch into stroked and filled private
      owners without changing public draw-list commands, path paint routing, stroke/fill painter
      calls, command fallthrough, or debug-draw smoke behavior.
      Result: `paint_shapes/path_commands/linear.rs` now only dispatches to
      `linear/stroked.rs` and `linear/filled.rs`; those owners handle stroked line/poly/rect/quad/
      triangle commands and filled polygon/quad/triangle commands respectively.
- [x] Split IMUI debug-draw geometry summary projection into linear, mesh, round, and Bezier
      private owners without changing public command summaries, point/vertex/index/triangle counts,
      clip-state projection, media/text summary projection, or debug-draw smoke behavior.
      Result: `commands/summary_projection/geometry.rs` now only dispatches to family owners;
      `geometry/linear.rs`, `geometry/mesh.rs`, `geometry/round.rs`, and `geometry/beziers.rs`
      own the concrete geometry summary counts.
- [x] Split IMUI debug-draw stroked linear path painters into line/polyline and rect/quad/triangle
      private owners without changing public draw-list commands, path command generation, shared
      stroke style dispatch, canvas path dispatch, or debug-draw smoke behavior.
      Result: `paint_shapes/paths/stroked/linear/line_poly.rs` owns line and polyline stroke
      painting, and `paint_shapes/paths/stroked/linear/rect_quad_triangle.rs` owns rect, quad, and
      triangle stroke painting. `stroked/linear.rs` keeps private re-exports.
- [x] Split IMUI debug-draw list summary command-kind classification into a private owner without
      changing public summary accessors, aggregate counts, clip-stack depth accounting, command
      summary shape, or debug-draw smoke behavior.
      Result: `debug_draw_controls/summaries/list/classification.rs` owns command-kind to list
      summary class mapping. `summaries/list.rs` keeps aggregate counters, public accessors,
      final clip-depth writeback, and include-time counter updates.
- [x] Split IMUI text-picker popup item rendering and pick commit into a private owner without
      changing popup lifecycle, keyboard handler installation, candidate filtering, selectable row
      presentation, active-descendant writeback, model update, popup close, or public picker
      response behavior.
      Result: `text_picker_controls/popup/item.rs` owns selectable candidate rows, item test-id
      derivation, active element writeback, model update, popup close, and click pick result.
      `popup.rs` keeps popup lifetime, keyboard handler installation, and aggregate pick result
      merging.
- [x] Split IMUI table header resize grip visual into a private owner without changing pointer
      region hit width, resize drag lifecycle hooks, cursor behavior, drag response edge merging,
      resize test-id attachment, or table column resize public behavior.
      Result: `table_controls/header/resize/visual.rs` owns resize grip color, disabled alpha, and
      visual dimensions. `header/resize.rs` keeps pointer-region drag setup, response writeback,
      and test-id attachment.
- [x] Split IMUI debug-draw filled path painters into polygon-fill and round-fill private owners
      without changing public draw-list commands, path command generation, shared fill style,
      canvas path dispatch, summaries, or debug-draw smoke behavior.
      Result: `paint_shapes/paths/filled/polygons.rs` owns convex/concave/quad/triangle fills and
      `paint_shapes/paths/filled/round.rs` owns circle/ngon/ellipse fills. `filled.rs` keeps the
      shared fill style plus private re-exports.
- [x] Split IMUI disclosure header indicator, padding, and border metrics into a private owner
      without changing header row composition, palette resolution, indicator glyph text role, tree
      row label text role, or tree-node/collapsing-header public behavior.
      Result: `disclosure_controls/visual/header/metrics.rs` owns indicator glyph selection,
      tree indentation padding, and header border edges. `visual/header.rs` keeps palette lookup,
      row element composition, glyph/text rendering, and spacer layout.
- [x] Split IMUI menu-item command presentation, shortcut defaulting, and enabled gating into a
      private owner without changing public menu item wrapper methods, focusable-recording
      behavior, command metadata lookup, shortcut propagation, or action menu item dispatch.
      Result: `facade_writer/menu_items/command.rs` owns command presentation lookup, enabled
      gating, and shortcut fallback. `menu_items.rs` keeps public menu item wrappers and the
      private helper re-export.
- [x] Split IMUI tooltip runtime layout/placement calculation into a private owner without
      changing trigger-id validation, event/open model setup, pointer-move open gate installation,
      interaction update, open model synchronization, overlay request submission, or public tooltip
      facade behavior.
      Result: `tooltip_overlay/runtime/layout.rs` owns anchor bounds, measured/estimated panel
      sizing, and floating bounds calculation. `tooltip_overlay/runtime.rs` keeps trigger gates,
      interaction updates, open state writeback, and overlay request submission.
- [x] Split IMUI button-command presentation and enabled gating into a private owner without
      changing public button wrapper methods, focusable-recording behavior, command metadata
      lookup, or action button dispatch.
      Result: `facade_writer/button_actions/button_command.rs` owns command presentation lookup and
      enabled gating. `button_actions.rs` keeps the public button wrappers and private helper
      re-export.
- [x] Split IMUI pressable drag state machine into a private interaction-runtime owner without
      changing drag kind derivation, theme threshold reads, pointer-down active item marking,
      long-press cancellation/arming, thresholded move transitions, drag started/stopped transient
      events, pointer-up cleanup, or public response drag state.
      Result: `interaction_runtime/drag/pressable.rs` owns pressable pointer down/move/up drag
      state transitions. `interaction_runtime/drag.rs` keeps drag kind/threshold helpers and
      private sub-owner re-exports.
- [x] Split IMUI drag-source payload lifecycle hooks into a private owner without changing
      drag-source trigger-id gating, enabled/cross-window pointer-down policy, active payload
      tracking, hovered-target preservation, drop delivery writeback, or public drag/drop response
      behavior.
      Result: `drag_drop/source/hooks/payload_lifecycle.rs` owns pointer-move active payload
      tracking and pointer-up delivery insertion. `drag_drop/source/hooks.rs` keeps enabled gating,
      cross-window drag upgrade policy, and the private payload-lifecycle delegation.
- [x] Split IMUI table-column visibility menu-item toggle behavior into a private owner without
      changing header context-menu trigger selection, menu item group composition, test-id suffix
      generation, shared visibility state updates, changed/edited response flags, or public
      visibility helper behavior.
      Result: `table_column_visibility/menu/item.rs` owns single checkbox item rendering and
      visibility model mutation. `menu.rs` keeps header context-menu orchestration, item group
      composition, identity/test-id filtering, and the private item-owner re-export.
- [x] Split IMUI menubar active-trigger reconciliation into a private owner without changing
      open-menu synchronization, active-trigger installation, close-after-render reconciliation,
      popup close restoration, or begin-menu public behavior.
      Result: `menu_family_controls/menu_state/open_policy/active_trigger/reconcile.rs` owns
      close/reconcile state cleanup. `active_trigger.rs` keeps active-trigger sync/activation and
      re-exports the reconcile owner.
- [x] Split IMUI begin-menu capture read helpers into a private owner without changing
      row/popup/was-open model identity, open-menu model reads, render-state writeback, or menubar
      open-policy behavior.
      Result: `menu_family_controls/menu_state/capture/read.rs` owns bool/open-menu model reads.
      `capture.rs` keeps `BeginMenuState`, `MenuRenderState`, model capture, render-state
      writeback, and read facade methods.
- [x] Split IMUI table builder row/cell test-id derivation into a private owner without changing
      public `ImUiTable` / `ImUiTableRow` methods, row option explicit test-id override behavior,
      default row/cell test-id strings, child `ImUiFacade` mounting, or table render behavior.
      Result: `table_controls/builder/test_ids.rs` owns row/cell test-id derivation. `builder.rs`
      keeps row/cell collection, keyed row scopes, child mounting, and public table-builder
      methods.
- [x] Split IMUI selectable popup-menu keyboard navigation into a private owner without changing
      selectable shortcut activation, popup close-on-shortcut, context-menu key handling, inherited
      popup menu item registration, Arrow/Home/End focus movement, or public selectable/menu-item
      behavior.
      Result: `selectable_controls/keyboard/popup_nav.rs` owns inherited popup menu nav item
      registration and focus movement. `keyboard.rs` keeps shortcut and context-menu key handling.
- [x] Split IMUI menu-family trigger menubar behavior into a private owner without changing
      active trigger install/population, click/shortcut activation, menubar trigger-row registry
      sync, patient-click timer wiring, ArrowDown/ArrowUp open behavior, or public menu facade
      behavior.
      Result: `menu_family_controls/trigger/behavior/menubar.rs` owns menubar trigger-row
      registration, sync, toggle-on-activate, and vertical-arrow open support. `behavior.rs` keeps
      base active-trigger behavior and response population.
- [x] Split IMUI disclosure trigger response projection into a private owner without changing
      disclosure pointer/shortcut behavior, context-menu and double-click transient signaling,
      hover-state projection, active-item hover blocking, or public disclosure facade behavior.
      Result: `disclosure_controls/trigger/behavior/response.rs` owns trigger response
      population. `behavior.rs` keeps pressable/key/pointer hook installation and delegates the
      response projection to the new owner.
- [x] Split IMUI slider pointer value-update logic into a private owner without changing pointer
      down/move/up capture, active-item mutation, lifecycle activation/deactivation, changed
      response emission, or slider pointer behavior.
      Result: `slider_controls/interaction/pointer/value_update.rs` owns pointer-to-value
      projection, clamp/snap, and value write detection. `pointer.rs` keeps pointer hook
      installation, active-item updates, capture, and lifecycle edit emission.
- [x] Split IMUI combo trigger visual props/a11y/chrome assembly into a private owner without
      changing ComboBox semantics, trigger activation behavior, open/close toggling, shortcut
      handling, preview/label rendering, or public combo facade behavior.
      Result: `combo_controls/trigger/visual.rs` owns the trigger props, chrome, children, and
      a11y label helper. `combo_controls/trigger.rs` keeps the behavior installation and visual
      owner dispatch.
- [x] Split IMUI popup-menu overlay request assembly into a private request owner without changing
      overlay id/root naming, popup open model forwarding, trigger fallback, auto-focus targets,
      focus-outside submenu preservation, menubar close-auto-focus suppression, submenu pointer
      move handler installation, modal flag forwarding, or public popup/menu facade behavior.
      Result: `popup_overlay/menu/request.rs` owns dismiss/close-auto-focus handlers and
      `dismissible_menu_request_with_modal_and_dismiss_handler(...)` assembly. `menu.rs` keeps
      overlay id creation, policy lookup, panel build orchestration, and request owner dispatch.
- [x] Split IMUI text-picker keyboard preparation into a private core sub-owner without changing
      keyboard model identity, enabled/empty/exact-match reconciliation, pending keyboard pick
      projection, active descendant element projection, input-root forwarding, popup forwarding,
      or public text-picker facade behavior.
      Result: `text_picker_controls/core/keyboard_state.rs` owns keyboard model creation and
      snapshot reconciliation. `text_picker_controls/core.rs` keeps model/candidate/input/open/
      popup/response orchestration.
- [x] Split IMUI child-region resize pointer-handle behavior into a private owner without changing
      X/Y resize response setup, min/max forwarding, handle layout/axis constants, pointer-region
      drag start/move/up behavior, cursor selection, drag response population, started/stopped edge
      synthesis, or handle test-id stamping.
      Result: `child_region/resize/handle.rs` owns the shared pointer-region resize handle and drag
      response projection. `resize.rs` keeps X/Y entry points and response option/min/max wiring.
- [x] Split IMUI submenu state mutation into private clear/select owners without changing submenu
      open-value updates, trigger updates, geometry clearing, pending-open cleanup, pointer-grace
      timer cleanup, close/focus/open timer cleanup, focus retry reset, or submenu selection writes.
      Result: `menu_family_controls/submenu_state.rs` is now a private module/re-export index.
      `submenu_state/clear.rs` owns clear/reset behavior, and `submenu_state/select.rs` owns
      selection writes.
- [x] Split IMUI menu-item keyboard behavior into private popup-menu and menubar owners without
      changing popup item nav registration, Arrow/Home/End focus movement, shortcut activation,
      popup-close-on-key activation, lifecycle instant marking, menubar horizontal-arrow
      suppression, or menubar primitive wiring.
      Result: `menu_controls/keyboard.rs` is now a private module/re-export index.
      `keyboard/popup.rs` owns popup menu keyboard behavior, and `keyboard/menubar.rs` owns menubar
      horizontal switching support.
- [x] Split IMUI interaction-runtime model helpers into private element, window, scope, and state
      owners without changing context-menu anchor model creation, long-press signal storage,
      pointer-click modifier storage, lifecycle session storage, active-item per-window storage,
      float-window collapsed storage, disabled-scope depth reads, or public interaction runtime
      surfaces.
      Result: `interaction_runtime/models.rs` is now a private module/re-export index.
      `models/element.rs`, `models/window.rs`, `models/scope.rs`, and `models/state.rs` own the
      respective helper families and state shapes.
- [x] Split IMUI input-text props and assistive-semantics assembly into a private owner without
      changing model reads, response lifecycle population, select-all-on-focus effect dispatch,
      input filters/custom filter ordering, password mode, accessibility metadata, placeholder/
      command forwarding, compact IMUI chrome/style, or public input-text surfaces.
      Result: `text_controls/input/props.rs` owns `InputTextAssistiveSemantics`,
      `TextInputProps` construction, insert filters, password-mode projection, a11y metadata, and
      input chrome/style. `input.rs` keeps model/lifecycle/effect/element orchestration.
- [x] Split IMUI text-control policy command installation into private input and textarea owners
      without changing completion/history/undo/redo key handling, repeat gating, IME/Alt/Meta
      guards, textarea Enter/CtrlEnter/Escape capture policy, repeat-consume semantics, or public
      text-control surfaces.
      Result: `text_controls/policy_commands.rs` is now a private module/re-export index.
      `policy_commands/input.rs` owns input key-down command dispatch, and
      `policy_commands/textarea.rs` owns textarea submit/cancel capture dispatch.
- [x] Split IMUI table-column visibility state methods into private override, snapshot-IO, and
      column-application owners without changing public state constructors/accessors, empty-id
      filtering, last-entry-wins behavior, snapshot roundtrips, table-column visibility application,
      or opaque state storage.
      Result: `table_column_visibility/state.rs` now owns only the state/override storage shape and
      public snapshot re-export. `state/overrides.rs` owns runtime override mutation/query,
      `state/snapshot_io.rs` owns snapshot conversion/restoration, and `state/columns.rs` owns
      `TableColumn` application.
- [x] Split IMUI child-region resize responses into private X/Y response owners without changing
      public `ChildRegionResizeXResponse` / `ChildRegionResizeYResponse` re-exports, enabled/min/
      max accessors, drag edge accessors, drag delta/total projection, clamp-from-start helpers, or
      opaque response fields.
      Result: `response/widgets/child_region/resize.rs` is now a private module/re-export index.
      `resize/x.rs` owns width-axis response projection and tests, while `resize/y.rs` owns
      height-axis response projection and tests.
- [x] Split IMUI input-text filter options into private built-in and custom-filter owners without
      changing `InputTextFilters` constructors, public filter flags, character filtering,
      uppercase/no-blank behavior, `InputTextCustomFilter` closure storage, debug output, or public
      text option exports.
      Result: `options/controls/text/filters.rs` is now a private module/re-export index.
      `filters/builtin.rs` owns `InputTextFilters` plus decimal/scientific/hex/uppercase/no-blank
      character filtering, and `filters/custom.rs` owns `InputTextCustomFilter`.
- [x] Split IMUI debug-draw draw-list image authoring into private mesh, raster, and rounded-image
      owners without changing `ImUiDebugDrawList` image method names, default option forwarding,
      command payload variants, vertex/index collection, image-region/quad recording, rounded
      command recording, summaries, paint dispatch, or public debug-draw APIs.
      Result: `debug_draw_controls/draw_list/images.rs` is now a private module index.
      `images/mesh.rs` owns triangle-mesh command recording, `images/raster.rs` owns image,
      image-region, and image-quad recording, and `images/rounded.rs` owns rounded image/region
      command recording.
- [x] Split IMUI debug-draw paint helpers into private media, mesh, and rounded-corner owners
      without changing opacity sanitization, UV validation, rounded-corner projection, triangle
      mesh filtering, image triangle mesh filtering, raster image scene ops, region scene ops, or
      debug-draw public APIs.
      Result: `debug_draw_controls/paint_helpers.rs` is now a private module/re-export index.
      `paint_helpers/media.rs` owns opacity/UV validation plus raster image scene ops,
      `paint_helpers/meshes.rs` owns vertex-color and image triangle mesh scene ops, and
      `paint_helpers/rounded.rs` owns rounded-corner visibility/projection.
- [x] Split IMUI debug-draw path-builder shape methods into private rect, bezier, and arc owners
      without changing `ImUiDebugDrawPath` method names, path point storage, invalid input
      handling, default segment fallback, rounded-rect sampling, Bezier sampling, arc sampling, or
      public debug-draw APIs.
      Result: `debug_draw_controls/path_builder/shape_methods.rs` is now a private module index.
      `shape_methods/rects.rs` owns rect and rounded-rect point appending,
      `shape_methods/beziers.rs` owns quadratic/cubic Bezier sampling, and
      `shape_methods/arcs.rs` owns circular, fast 12-step, and elliptical arc sampling.
- [x] Split IMUI shared hover-delay state/store/model lookup out of
      `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/shared_delay.rs` into a private
      state owner without changing window-scoped shared-delay model allocation, short/normal delay
      flags, hover-enter timer scheduling, hover-leave clear timer scheduling, clear-timer
      cancellation, or shared delay reads.
      Result: `interaction_runtime/hover/shared_delay/state.rs` owns
      `ImUiSharedHoverDelayState`, `ImUiSharedHoverDelayStore`, `model_for_window`, and
      `delay_flags`. `hover/shared_delay.rs` keeps hover/timer event policy.
- [x] Split IMUI hover query delay read/projection out of
      `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover.rs` into a private read owner
      without changing hover hook installation, stationary/short/normal delay timers, shared-delay
      timers, long-press timer handling, active-item blocking, transient consumption, or
      `HoverQueryDelayRead` values.
      Result: `interaction_runtime/hover/read.rs` owns local hover-delay state, transient
      consumption, shared-delay flag reads, and `HoverQueryDelayRead` projection.
      `interaction_runtime/hover.rs` keeps active-item blocking, hover-change hook installation,
      timer dispatch, shared-delay delegation, and long-press delegation.
- [x] Split IMUI porting layout sugar into private scoped-layout and spacer owners without
      changing `items`, `same_line`, `dummy`, `spacing`, `indent`, layout-token defaults,
      explicit dummy sizing, indent composition, test-id stamping, or public-in-IMUI APIs.
      Result: `layout_sugar/scoped.rs` owns item-flow, same-line, and indent container
      composition. `layout_sugar/spacers.rs` owns dummy and spacing spacer construction.
      `layout_sugar.rs` is now a private module/re-export index.
- [x] Split IMUI input-text picker keyboard handler out of
      `ecosystem/fret-ui-kit/src/imui/text_picker_controls/keyboard.rs` into a private handler
      owner without changing keyboard navigation enablement, repeat/IME/modifier gating, arrow
      highlight movement, Enter pick handling, popup close, model writes, or picker response
      projection.
      Result: `text_picker_controls/keyboard/handler.rs` owns key-down capture plus Arrow/Enter
      model writes. `text_picker_controls/keyboard.rs` keeps keyboard pick/state/snapshot storage
      and reconciliation.
- [x] Split IMUI menu item routing dispatch into private entry and core owners without changing
      menu item method names, checkbox/radio/action semantics roles, action forwarding,
      label-identity scoping, pressable hook injection, mount routing, response population, or
      public-in-IMUI APIs.
      Result: `menu_controls/routing/dispatch/entries.rs` owns public-in-IMUI menu-item entry
      wrappers and semantics/action selection.
      `menu_controls/routing/dispatch/core.rs` owns no-op pressable hook and identity-to-mount
      dispatch. `menu_controls/routing/dispatch.rs` is now a private module/re-export index.
- [x] Split IMUI slider math helpers out of
      `ecosystem/fret-ui-kit/src/imui/facade_support.rs` into a private slider-math owner without
      changing slider range normalization, step fallback, clamp/snap behavior, pointer-to-value
      mapping, slider a11y value projection, slider pointer/keyboard interaction, or public-in-IMUI
      APIs.
      Result: `facade_support/slider_math.rs` owns `slider_step_or_default`,
      `slider_normalize_range`, `slider_clamp_and_snap`, and `slider_value_from_pointer`.
      `facade_support.rs` keeps writer bridge support, transient keys, runtime frame prep,
      device-pixel snapping, point arithmetic, and model-change detection.
- [x] Split IMUI drag source hook installation out of
      `ecosystem/fret-ui-kit/src/imui/drag_drop/source.rs` into a private hooks owner without
      changing drag source trigger-id validation, payload boxing, store pruning, drag kind
      selection, cross-window drag promotion, active payload publication, pointer-up delivery, or
      `DragSourceResponse` population.
      Result: `drag_drop/source/hooks.rs` owns enabled/cross-window policy, pointer-down
      cross-window promotion, pointer-move active payload publication, and pointer-up delivery
      insertion. `drag_drop/source.rs` keeps trigger validation, payload boxing, store model
      lifecycle/pruning, drag-kind selection, hook owner dispatch, and source response projection.
- [x] Split IMUI interaction lifecycle response projection out of
      `ecosystem/fret-ui-kit/src/imui/interaction_runtime/lifecycle.rs` into a private response
      owner without changing lifecycle activation/deactivation/edit mutation, transient
      consumption, active-state frame diffing, `ResponseExt` lifecycle signal setters/mergers, or
      public-in-IMUI APIs.
      Result: `interaction_runtime/lifecycle/response.rs` owns transient-to-response population,
      active-state lifecycle frame diffing, edited-state stamping, and activated/deactivated merge
      application. `interaction_runtime/lifecycle.rs` keeps pointer-down/up lifecycle mutation,
      instant edit mutation, lifecycle edit mutation, and private re-exports for callers.
- [x] Split IMUI tooltip overlay request assembly out of
      `ecosystem/fret-ui-kit/src/imui/tooltip_overlay/runtime.rs` into a private request owner
      without changing trigger-id validation, pointer-move open gating, hover/focus interaction
      updates, panel layout, dismiss behavior, hoverable-content pointer tracking, overlay request
      semantics, or public-in-IMUI APIs.
      Result: `tooltip_overlay/request.rs` owns panel child construction, tooltip overlay request
      creation, trigger binding, dismiss close-request signaling, optional hoverable-content pointer
      tracker installation, and request submission. `tooltip_overlay/runtime.rs` keeps trigger-id validation,
      event/open models, pointer-move open gate installation, hover/focus update gates,
      interaction updates, panel-size/anchor projection, and open-state synchronization.
- [x] Split IMUI floating area shell layout and hit-test gate selection out of
      `ecosystem/fret-ui-kit/src/imui/floating_surface/area.rs` into a private layout owner without
      changing floating-area registration, drag position reconciliation, no-input behavior,
      hit-test passthrough behavior, area test IDs, response population, or public-in-IMUI APIs.
      Result: `floating_surface/area/layout.rs` owns absolute area layout props,
      `interactivity_gate_props` selection for `no_inputs`, `hit_test_gate_props` selection for
      hit-test passthrough, and the container fallback. `area.rs` keeps layer child registration,
      drag snapshot/state reconciliation, child mounting, final test-id stamping, and
      `FloatingAreaResponse` construction.
- [x] Split IMUI floating layer z-order state and snapshot projection out of
      `ecosystem/fret-ui-kit/src/imui/floating_surface/layer.rs` into a private owner without
      changing floating layer child registration, bring-to-front activation, missing-window
      pruning, rank sorting, hit-test order, absolute layer layout, or floating layer public-in-IMUI
      APIs.
      Result: `floating_surface/layer/z_order.rs` owns `FloatWindowLayerZOrder`, z-order
      membership, bring-to-front reordering, missing-window pruning, and rank snapshot projection.
      `layer.rs` keeps layer marker state, child registration, activation dispatch, layer child
      mounting, and rank sort application.
- [x] Split IMUI floating layer absolute shell layout out of
      `ecosystem/fret-ui-kit/src/imui/floating_surface/layer.rs` into a private owner without
      changing floating layer child registration, bring-to-front activation, z-order sorting,
      hit-test order, absolute fill layout, visible overflow, or floating layer public-in-IMUI
      APIs.
      Result: `floating_surface/layer/layout.rs` owns the absolute fill visible-overflow layer
      container and id stamping. `layer.rs` keeps layer marker state, child registration,
      activation dispatch, layer child mounting, and z-order snapshot reconciliation before
      delegating rank sort and layout.
- [x] Split IMUI floating layer z-order rank sort application out of
      `ecosystem/fret-ui-kit/src/imui/floating_surface/layer.rs` into a private owner without
      changing child registration, bring-to-front activation, missing-window pruning, unknown-rank
      fallback, original-order tie-breaks, hit-test order, or absolute layer layout.
      Result: `floating_surface/layer/sort.rs` owns z-order rank lookup, unknown-rank fallback, and
      original-index stable tie-break sorting. `layer.rs` keeps marker state, child registration,
      activation dispatch, child mounting, and z-order snapshot reconciliation before handing the
      sorted windows to the layout owner.
- [x] Split shared IMUI pressable item hook installation out of
      `ecosystem/fret-ui-kit/src/imui/item_behavior.rs` into a private install owner without
      changing shared button, checkbox/radio, selectable, combo, image-item, debug-draw pressable,
      context-menu, pointer-click, double-click, drag, long-press, lifecycle, or response
      population behavior.
      Result: `item_behavior/install.rs` owns pressable pointer hook clearing,
      active-item/long-press/lifecycle model capture, and assembly. Later pointer-hook sub-owner
      splits move down/move/up transient bodies into `item_behavior/install/*`. The root
      `item_behavior.rs` keeps shared data shapes plus install/response re-exports.
- [x] Split IMUI facade floating/popup thin forwarding out of
      `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup.rs` into private floating,
      popup, tooltip, drag/drop, and window owner modules without changing trait method names,
      facade forwarding, popup open/close behavior, tooltip forwarding, drag/drop forwarding, or
      floating-window routing.
      Result: `floating_popup.rs` is now a private module/re-export index; `floating.rs` owns
      floating layer/area forwarding, `popup.rs` owns popup open/close/menu/modal/context
      forwarding, `tooltip.rs` owns tooltip forwarding, `drag_drop_facade.rs` owns drag/drop
      forwarding, and `window.rs` owns floating-window forwarding.
- [x] Split IMUI facade floating-popup popup behavior forwarding into popup state and begin-popup
      child owners without changing public facade method names, popup open-model/drop/open/close
      forwarding, popup menu/modal/context-menu begin forwarding, or `floating_popup.rs`
      re-export paths.
      Result: `facade_writer/floating_popup/popup.rs` is now a module/re-export hub.
      `popup/state.rs` owns open-model, drop, open, anchor-open, and close forwarding, while
      `popup/begin.rs` owns menu, modal, and context-menu begin forwarding.
- [x] Split IMUI button action payload and command dispatch out of
      `ecosystem/fret-ui-kit/src/imui/button_controls/behavior.rs` into a private action owner
      without changing button pressable behavior, shortcut activation, command gating, action
      payload forwarding, context-menu signaling, visual resolution, or `ResponseExt` population.
      Result: `button_controls/behavior/action.rs` owns `ButtonAction`, payload storage, command
      dispatch source recording, pending payload recording, and final action dispatch, while
      `behavior.rs` keeps pressable props, shortcut/context-menu handlers, enabled gating,
      lifecycle marking, response population, and visual resolution.
- [x] Split IMUI button visual a11y and variant layout/glyph policy out of
      `ecosystem/fret-ui-kit/src/imui/button_controls/visual.rs` into private owner modules
      without changing public button APIs, button a11y labels, arrow a11y labels, arrow glyphs,
      variant sizing, or chrome/content assembly.
      Result: `button_controls/visual/a11y.rs` owns button `PressableA11y` construction and
      label fallback policy, `button_controls/visual/variant.rs` owns variant sizing plus arrow
      glyph selection, and `visual.rs` keeps `ButtonVisual`, `ButtonVisualContent`, chrome
      resolution, and visible/invisible content assembly.
- [x] Split IMUI tab item list semantics and selected-panel rendering out of
      `ecosystem/fret-ui-kit/src/imui/tab_family_controls/items.rs` into private list and panel
      owners without changing selected-model normalization, trigger response collection, focus
      fallback behavior, tab-list semantics, tab-panel semantics, or public tab-bar APIs.
      Result: `tab_family_controls/items/list.rs` owns tab-list semantics, trigger rendering,
      selected/first-focusable trigger tracking, and `TabTriggerResponse` collection.
      `items/panel.rs` owns selected tab-panel semantics and panel child mounting. `items.rs`
      keeps `BuiltTabItem`, selected-model normalization, build-focus propagation, final column
      assembly, and `TabBarResponse` construction.
- [x] Split IMUI input-text picker core orchestration out of
      `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs` into a private core owner without
      changing completion/history wrapper calls, candidate filtering, keyboard navigation, input
      root semantics, popup open policy, popup pick handling, or `InputTextPickerResponse`.
      Result: `text_picker_controls/core.rs` owns model reads, candidate visibility, keyboard
      snapshot reconciliation, input root mounting, open-policy application, popup rendering, and
      pick response merging. `text_picker_controls.rs` is now a private module index and re-export
      hub for core and entry wrappers.
- [x] Split IMUI table header cell layout/resize wrapping out of
      `ecosystem/fret-ui-kit/src/imui/table_controls/header.rs` into a private cell owner without
      changing sortable/plain header behavior, resize handle wiring, header test IDs, table layout,
      or `TableHeaderResponse` collection.
      Result: `table_controls/header/cell.rs` owns header cell layout, resize-handle attachment,
      resize test-id suffixing, and header content flex wrapping. `header.rs` keeps sortable/plain
      header trigger orchestration and `BuiltHeaderCell` response assembly.
- [x] Split IMUI debug-draw path construction by shape family out of
      `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths.rs` into private linear, round,
      and bezier owner modules without changing path helper names, validation behavior, sampling
      helpers, rect path construction, paint dispatch, or debug-draw tests.
      Result: `paths.rs` is now a private path-family re-export hub; `paths/linear.rs` now indexes
      polyline/fill/primitive subowners; `paths/round.rs` now indexes circle/ngon/ellipse
      subowners; `paths/beziers.rs` owns quadratic and cubic bezier path construction.
      2026-05-28 follow-up: `paths/linear.rs` is now itself a private re-export hub; polyline,
      fill, and primitive construction live in
      `paths/linear/{polyline,fills,primitives}.rs`.
      2026-05-28 follow-up: `paths/round.rs` is now itself a private re-export hub; circle, ngon,
      and ellipse construction live in `paths/round/{circle,ngon,ellipse}.rs`.
- [x] Split IMUI debug-draw command payload variants out of
      `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands.rs` into a private payload
      owner without changing command variant names, draw-list recording paths, summary projection,
      paint dispatch, public debug-draw summaries, or facade APIs.
      Result: `debug_draw_controls/commands/types.rs` owns private `DebugDrawCommand` payload
      variants. `commands.rs` keeps command module wiring, summary projection installation, and the
      parent-visible `DebugDrawCommand` re-export.
- [x] Split IMUI debug-draw media paint routing out of
      `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint/media.rs` into private raster,
      rounded, and SVG owner modules without changing image/region/quad/SVG paint behavior, opacity
      filtering, UV validation, rounded clip balancing, or media command no-op routing.
      Result: `paint/media.rs` keeps `paint_debug_draw_media_command(...)` routing,
      `paint/media/raster.rs` owns image/region/quad paint, `paint/media/rounded.rs` owns rounded
      image/region paint and clip push/pop balancing, and `paint/media/svg.rs` owns SVG image/mask
      icon paint.
- [x] Split IMUI debug-draw pressable element behavior out of
      `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/element.rs` into a private owner module
      without changing noninteractive canvas output, pressable canvas wrapping, keyboard activation
      lifecycle marking, pointer-click reporting, click response population, cache policy, clipping,
      or paint routing.
      Result: `debug_draw_controls/element/behavior.rs` owns pressable behavior installation,
      keyboard activation lifecycle marking, clicked transient reads, and `ResponseExt` population.
      `element.rs` keeps canvas composition, fill-layout policy for interactive canvases, cache
      policy, clipping, test-id routing, and debug-draw command painting.
- [x] Split IMUI table row-group pinned-cell splitting, row flex layout, and horizontal-scroll
      wrapper helpers out of `ecosystem/fret-ui-kit/src/imui/table_controls/row_groups.rs` into
      private owner modules without changing table row/header layout, pinned-column ordering,
      horizontal-scroll wrapping, or table public APIs.
      Result: `row_groups.rs` keeps `wrap_pinned_table_row_groups(...)` orchestration,
      `row_groups/split.rs` owns pinned-cell grouping, `row_groups/layout.rs` owns horizontal row
      flex wrappers, and `row_groups/scroll.rs` owns center horizontal scroll wrapping.
- [x] Split IMUI debug-draw draw-list linear shape authoring out of
      `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list_shapes/linear.rs` into private
      line/poly and rect/quad/triangle owner modules without changing draw-list authoring method
      names, command payloads, or summary/paint behavior.
      Result: `draw_list_shapes/linear/line_poly.rs` owns line/polyline/filled-polygon command
      recording, `linear/rect_quad_triangle.rs` owns rect/quad/triangle command recording, and
      `linear.rs` is now a private module index.
- [x] Split IMUI table-column construction, identity/accessor, visibility, sort, resize, and pin
      methods out of `ecosystem/fret-ui-kit/src/imui/options/collections/table_column.rs` into
      private owner modules without changing public `TableColumn` type names, method names, chained
      builder behavior, stable-id inference, or table composition behavior.
      Result: `table_column.rs` keeps the `TableColumn` storage shape and primitive re-exports;
      `table_column/construction.rs`, `identity.rs`, `visibility.rs`, `sorting.rs`, `resize.rs`,
      and `pinning.rs` now own the corresponding impl method families.
- [x] Split IMUI drag/drop store state, lifecycle, source-response query, and target-payload query
      out of `ecosystem/fret-ui-kit/src/imui/drag_drop/store.rs` into private owner modules without
      changing `drag_source_with_options(...)`, `drop_target_with_options(...)`, payload lifetime,
      stale-session pruning, or delivered-payload expiry behavior.
      Result: `drag_drop/store/state.rs` owns the shared model and payload records,
      `store/lifecycle.rs` owns global model creation plus pruning, `store/source_response.rs` owns
      source response projection, `store/target_payloads.rs` owns active/delivered typed payload
      lookup, and `store.rs` is now a private re-export index.
- [x] Split IMUI debug-draw command summary geometry and clip-state projection out of
      `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection.rs` into
      private owner modules without changing `DebugDrawCommandSummary` values, channel assignment,
      clip stack depth, media summaries, or command-list summary behavior.
      Result: `commands/summary_projection/geometry.rs` owns point/vertex/index/triangle-count
      summaries for geometric commands, `commands/summary_projection/clip_state.rs` owns
      push/pop/current clip rect and depth updates, and the root summary projection file keeps the
      public-in-debug-draw entry point plus media/text/clip command routing.
- [x] Split IMUI debug-draw path-command dispatch by shape family out of
      `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands.rs` into
      private linear, round, and bezier owners without changing path command routing, canvas keys,
      draw order, or path paint helpers.
      Result: `path_commands/linear.rs` owns line/polyline/polygon/rect/quad/triangle dispatch,
      `path_commands/round.rs` owns circle/ngon/ellipse dispatch, and
      `path_commands/beziers.rs` owns quadratic/cubic bezier dispatch. The `path_commands.rs`
      root is now a thin family router.
- [x] Split IMUI debug-draw path-shape paint dispatch out of
      `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes.rs` into a private owner
      module without changing command order, canvas keys, stroke/fill geometry, mesh painting,
      text painting, or media no-op routing.
      Result: `debug_draw_controls/paint_shapes/path_commands.rs` owns line/polyline/polygon,
      rect-outline, quad, triangle, circle, ngon, ellipse, and bezier command dispatch into the
      path paint owners. `paint_shapes.rs` keeps draw-order/key setup plus non-path command routing
      for filled rects, meshes, text, and ignored media/clip commands.
- [x] Split IMUI menu-item entry routing and label identity scoping out of
      `ecosystem/fret-ui-kit/src/imui/menu_controls.rs` into a private owner module without
      changing public menu-item, checkbox, radio, action, or submenu pressable-hook call paths.
      Result: `menu_controls/routing.rs` owns public-in-IMUI menu item dispatch, semantic role
      selection, `##/###` label identity parsing, item-id scoping, response assembly, and final
      element insertion. The root file is now a thin module/re-export index beside the existing
      element, interaction, keyboard, visual, and tests owners.
- [x] Split IMUI menu-item label identity scoping out of
      `ecosystem/fret-ui-kit/src/imui/menu_controls/routing.rs` into a private owner module
      without changing public menu item dispatch, `##/###` visible-label semantics, stable
      item-id scoping, response assembly, or final element insertion.
      Result: `menu_controls/routing/identity.rs` owns `parse_label_identity(...)`, visible label
      extraction, and `menu-item-label` `push_id` scoping. `routing.rs` keeps menu item entry
      dispatch, semantic role selection, response assembly, and element insertion.
- [x] Split IMUI menu-item final mounting and response assembly out of
      `ecosystem/fret-ui-kit/src/imui/menu_controls/routing.rs` into a private owner module
      without changing public menu item dispatch, semantic role/action selection, `##/###`
      identity scoping, pressable-hook forwarding, final element insertion, or `ResponseExt`
      assembly.
      Result: `menu_controls/routing/mount.rs` owns menu-item element mounting,
      `ResponseExt::default()` initialization, final `ui.add(...)`, and response return.
      `routing.rs` keeps public dispatch, checkbox/radio/action role selection, noop-hook routing,
      and label identity scoping.
- [x] Split IMUI menu-item routing dispatch out of
      `ecosystem/fret-ui-kit/src/imui/menu_controls/routing.rs` into a private dispatch owner
      without changing public menu item entry points, checkbox/radio/action role selection,
      noop-hook routing, `##/###` label identity scoping, pressable-hook forwarding, response
      assembly, or final element mounting.
      Result: `menu_controls/routing/dispatch.rs` owns public menu-item entry wrappers,
      checkbox/radio/action role selection, noop-hook routing, and identity-to-mount dispatch.
      `routing.rs` is now a private dispatch/identity/mount module index and re-export hub.
- [x] Split IMUI container child building, linear layout, scroll, and grid element composition out
      of `ecosystem/fret-ui-kit/src/imui/containers.rs` into private owner modules without
      changing horizontal/vertical/grid/scroll facade helpers, option forwarding, test-id
      placement, viewport test-id placement, or child focus propagation.
      Result: `containers/children.rs` owns child `ImUiFacade` mounting,
      `containers/linear.rs` owns horizontal/vertical flex containers, `containers/scroll.rs` owns
      scroll-area construction, and `containers/grid.rs` owns grid row batching/keyed rows. The
      root `containers.rs` is now a thin module/re-export index plus tests.
- [x] Split shared IMUI active-trigger keyboard, pointer, and response behavior out of
      `ecosystem/fret-ui-kit/src/imui/active_trigger_behavior.rs` into private owner modules
      without changing active-item lifecycle, right-click context-menu signaling, keyboard
      context-menu requests, or shared `ResponseExt` population.
      Result: `active_trigger_behavior/keyboard.rs` owns ContextMenu/Shift+F10 request handling,
      `active_trigger_behavior/pointer.rs` owns primary active-item pointer lifecycle and
      secondary-click anchor signaling, `active_trigger_behavior/response.rs` owns context-menu
      response fields plus shared pressable response population, and the root file keeps handler
      clearing, model lookup, options/input structs, and owner dispatch.
- [x] Split IMUI button visual/layout/accessibility ownership out of
      `ecosystem/fret-ui-kit/src/imui/button_controls.rs` into a private owner module without
      changing the public IMUI surface.
      Result: `ecosystem/fret-ui-kit/src/imui/button_controls/visual.rs` now owns button variant
      layout, a11y label construction, arrow labels/glyphs, and chrome/content assembly, while
      `button_controls.rs` keeps pressable behavior, shortcut handling, action dispatch, and
      response population.
- [x] Split IMUI button pressable/action behavior out of
      `ecosystem/fret-ui-kit/src/imui/button_controls.rs` into a private owner module without
      changing the public button/small-button/arrow/invisible/action facade surface.
      Result: `button_controls/behavior.rs` owns button action payload storage, command gating,
      pressable props, shortcut/context-menu handling, action dispatch metadata/payload forwarding,
      and button `ResponseExt` population. The root file now keeps public entry routing plus
      label-identity scoping, while `visual.rs` remains the layout/a11y/chrome owner.
- [x] Split IMUI shared control chrome text/pill helpers out of
      `ecosystem/fret-ui-kit/src/imui/control_chrome.rs` into a private owner module without
      changing compact button/control label text roles, caption text color routing, pill badge
      chrome, or existing `control_chrome::*` call paths.
      Result: `control_chrome/text.rs` owns `control_text`, `fill_text`, `caption_text`, and
      `pill`. At this slice, the root `control_chrome.rs` kept style constants, control palette,
      button/field chrome, row/stack layout props, and test module wiring; the later chrome owner
      split below moved palette/button/field chrome out too.
- [x] Split IMUI shared control chrome row/stack layout helpers out of
      `ecosystem/fret-ui-kit/src/imui/control_chrome.rs` into a private owner module without
      changing row direction, fill-width behavior, gap tokens, justification, or alignment for
      existing `control_chrome::*_props` call paths.
      Result: `control_chrome/layout.rs` owns `fill_row_props`, `centered_row_props`, and
      `fill_stack_props`. At this slice, the root `control_chrome.rs` kept style constants,
      control palette, button/field chrome, text helper re-exports, and test module wiring; the
      later chrome owner split below moved palette/button/field chrome out too.
- [x] Split IMUI shared control palette/theme chrome out of
      `ecosystem/fret-ui-kit/src/imui/control_chrome.rs` into a private owner module without
      changing `control_chrome::button_chrome`, `field_chrome`, `ImUiControlPalette`, dense
      button/field chrome defaults, row/stack layout helpers, or text/pill helpers.
      Result: `control_chrome/chrome.rs` owns `ImUiControlPalette`, button theme color resolution,
      field theme color resolution, and compact button/field container chrome. The root
      `control_chrome.rs` now keeps style constants, owner module wiring, and private re-exports.
- [x] Split IMUI shared control chrome palette, button, and field theme resolution out of
      `ecosystem/fret-ui-kit/src/imui/control_chrome/chrome.rs` into narrower private owners
      without changing `control_chrome::button_chrome`, `field_chrome`, `ImUiControlPalette`,
      dense button/field chrome defaults, theme token fallback order, or caller paths.
      Result: `control_chrome/chrome/palette.rs` owns `ImUiControlPalette`,
      `chrome/button.rs` owns button theme resolution and chrome props, `chrome/field.rs` owns
      field theme resolution and chrome props, and `chrome.rs` is now a private module/re-export
      index.
- [x] Split IMUI input-text picker candidate visibility and keyboard state reconciliation out of
      `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs` into private owner modules without
      changing the public IMUI surface.
      Result: `text_picker_controls/candidates.rs` owns filter/max-item/exact-match/open-empty
      visibility decisions, and `text_picker_controls/keyboard.rs` owns active-source cleanup plus
      pending keyboard pick extraction. The root file keeps input/popup composition and response
      merging.
- [x] Split IMUI input-text picker popup item rendering and pick commit out of
      `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs` into a private owner module without
      changing completion/history picker behavior.
      Result: `text_picker_controls/popup.rs` owns popup mounting, optional popup-scoped keyboard
      handler installation, candidate selectable rows, active-element synchronization, click commit,
      popup close, and picked-result reporting. The root file keeps input composition, assistive
      semantics, open/close policy, candidate/keyboard snapshots, and final response merge.
- [x] Split IMUI input-text picker input-root composition out of
      `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs` into a private owner module without
      changing completion/history picker behavior, ComboBox semantics, test-id derivation, active
      descendant wiring, root fill sizing, input-focused keyboard navigation, popup open/close
      policy, or picked response merging.
      Result: `text_picker_controls/input.rs` owns picker input option/test-id preparation,
      assistive semantics, root container construction, text input mounting, and input-focused
      keyboard handler installation. The root file keeps candidate visibility, popup-open state,
      popup lifecycle policy, popup rendering delegation, and final `InputTextPickerResponse`
      merge.
- [x] Split IMUI input-text picker popup open policy out of
      `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs` into a private owner module without
      changing popup open/panel-id reads, active-descendant wiring, open-on-focus behavior,
      empty/exact-match close behavior, keyboard navigation, popup rendering, or picked response
      merging.
      Result: `text_picker_controls/open_policy.rs` owns popup snapshot reads, expanded-state
      calculation, empty/exact-match close policy, and open-on-focus anchoring. The root file keeps
      candidate resolution, input/popup orchestration, keyboard reconciliation, and response
      assembly.
- [x] Split IMUI input-text picker picked-response merge out of
      `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs` into a private owner module without
      changing model re-read timing, element-id change detection, changed/edited/
      deactivated-after-edit merge semantics, popup rendering, keyboard navigation, or final
      response shape.
      Result: `text_picker_controls/response.rs` owns selected-value re-read,
      `model_value_changed_for(...)`, and picked-candidate `ResponseExt` merge writes. The root
      file keeps candidate resolution, input/popup orchestration, keyboard reconciliation, and final
      response assembly.
- [x] Split IMUI input-text picker completion/history entry wrappers out of
      `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs` into a private entry owner without
      changing public facade call paths, completion picker behavior, history picker filter/open
      normalization, core picker orchestration, or final response shape.
      Result: `text_picker_controls/entry.rs` owns completion/history wrapper functions and
      history option normalization. The root file keeps core picker orchestration and re-exports
      the entry helpers.
- [x] Split IMUI textarea element assembly out of
      `ecosystem/fret-ui-kit/src/imui/text_controls.rs` into a private owner module without changing
      the textarea facade, response semantics, select-all-on-focus behavior, submit/cancel command
      policy, chrome, or text style selection.
      Result: `text_controls/textarea.rs` owns textarea props assembly, lifecycle/response
      population, select-all command emission, submit/cancel policy installation, and text-area
      chrome/text-style selection. The root file keeps input-text assembly plus shared helper
      routing for text models.
- [x] Split IMUI input-text element assembly out of
      `ecosystem/fret-ui-kit/src/imui/text_controls.rs` into a private input owner without changing
      input-text facade calls, assistive semantics wiring for text pickers, response lifecycle
      population, select-all-on-focus behavior, insert filters, submit/cancel command policy,
      compact chrome, or text style selection.
      Result: `text_controls/input.rs` owns input-text model element assembly, assistive semantics,
      response lifecycle population, select-all command emission, input filters, policy-command
      installation, and compact input chrome/style selection. `text_controls.rs` is now a private
      focus/input/policy/style/textarea module index and re-export hub.
- [x] Split IMUI disclosure spec construction out of
      `ecosystem/fret-ui-kit/src/imui/disclosure_controls.rs` into a private owner module without
      changing the public collapsing-header/tree-node surface.
      Result: `disclosure_controls/spec.rs` owns `DisclosureKind`, `DisclosureSpec`, option-to-spec
      normalization, and leaf/children classification. The root file keeps pressable behavior,
      model/toggle wiring, content mounting, and response population.
- [x] Split IMUI disclosure trigger behavior and response population out of
      `ecosystem/fret-ui-kit/src/imui/disclosure_controls.rs` into a private owner module without
      changing the public collapsing-header/tree-node surface.
      Result: `disclosure_controls/trigger.rs` owns pressable shell construction and delegates
      trigger behavior/response population to its private behavior owner. The root file keeps label
      identity, spec/open-model wiring, content mounting, and aggregate `DisclosureResponse`
      open/toggled state.
- [x] Split IMUI disclosure trigger pressable behavior and response population out of
      `ecosystem/fret-ui-kit/src/imui/disclosure_controls/trigger.rs` into a private owner module
      without changing activation toggles, shortcut gating, context-menu keyboard requests,
      right-click anchor capture, double-click signaling, hover-delay reads, enabled sanitization,
      or trigger `ResponseExt` population.
      Result: `disclosure_controls/trigger/behavior.rs` owns pressable callback installation,
      activate shortcut/context-menu key handling, pointer down/up hooks, hover-delay reads,
      context-menu anchor reporting, enabled sanitization, and trigger `ResponseExt` population.
      `trigger.rs` keeps pressable props, a11y, header visual mounting, collapsible trigger
      controls, and test-id application.
- [x] Split IMUI disclosure content/root layout out of
      `ecosystem/fret-ui-kit/src/imui/disclosure_controls.rs` into a private layout owner without
      changing label identity parsing, open-model reads, trigger mounting, content body building,
      content/root test IDs, open/toggled response population, or public disclosure facade calls.
      Result: `disclosure_controls/layout.rs` owns content container composition, body `ImUiFacade`
      construction, root column layout, and content/root test-id application. The root file keeps
      label identity parsing, open-model reads, trigger mounting, and aggregate `DisclosureResponse`
      writes.
- [x] Split IMUI disclosure header-row visual construction out of
      `ecosystem/fret-ui-kit/src/imui/disclosure_controls/visual.rs` into a private owner module
      without changing collapsing-header/tree-node a11y, palette policy, indicator glyphs, label
      text roles, row chrome, indentation, or trigger behavior.
      Result: `disclosure_controls/visual/header.rs` owns header row container/flex assembly,
      indicator glyph mounting, label text mounting, row padding, border, and radius props.
      `visual.rs` keeps disclosure a11y, content padding, and palette resolution.
- [x] Split IMUI combo trigger behavior and visual chrome out of
      `ecosystem/fret-ui-kit/src/imui/combo_controls.rs` into a private owner module without
      changing the public combo/combo-model facade surface.
      Result: `combo_controls/trigger.rs` owns ComboBox pressable props, a11y label derivation,
      shortcut activation, context-menu request handling, trigger `ResponseExt` population, and
      open/menu badge visual assembly. The root file keeps label identity, popup open/close model
      wiring, popup mounting, and aggregate `ComboResponse` open/toggled state.
- [x] Split IMUI combo trigger behavior out of
      `ecosystem/fret-ui-kit/src/imui/combo_controls/trigger.rs` into a private owner module
      without changing trigger props, ComboBox semantics, a11y label derivation, shortcut
      activation, context-menu request handling, pressable response population, or popup behavior.
      Result: `combo_controls/trigger/behavior.rs` owns activate handling, activate-shortcut
      handling, context-menu shortcut handling, transient events, and `ResponseExt` population.
      `trigger.rs` keeps pressable props, ComboBox a11y, chrome/pill visual assembly, and a11y
      label derivation.
- [x] Split IMUI drag source/target response records out of
      `ecosystem/fret-ui-kit/src/imui/response/drag.rs` into private owner modules without
      changing public re-export paths, accessor-first response shape, drag/drop smoke behavior,
      or `ResponseExt` drag accessors.
      Result: `response/drag/source.rs` owns `DragSourceResponse` storage, inactive/new
      constructors, and source read accessors; `response/drag/target.rs` owns
      `DropTargetResponse<T>` storage, empty construction, preview/delivered payload and position
      accessors, source id reads, and session reads. The root `drag.rs` keeps generic
      `DragResponse` edge/motion state and re-exports.
- [x] Split IMUI floating option/context types out of
      `ecosystem/fret-ui-kit/src/imui/floating_options.rs` into private owner modules without
      changing public re-export paths, option field names/defaults, builder methods, floating
      behavior, or `FloatingAreaContext` accessor-first shape.
      Result: `floating_options/window.rs` owns `FloatingWindowResizeOptions`,
      `FloatingWindowOptions`, `WindowOptions`, defaults, and builder methods;
      `floating_options/area.rs` owns `FloatingAreaOptions`, `FloatingAreaContext`, area defaults,
      and context accessors. The root `floating_options.rs` is now a thin re-export index.
- [x] Split IMUI boolean-control visual chrome out of
      `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs` and its switch owner without changing
      the public checkbox/radio/switch surface.
      Result: `boolean_controls/visual.rs` owns checkbox badges, radio indicators, switch state
      badges, and shared boolean label text. The root checkbox/radio file and `switch.rs` keep
      pressable behavior, shortcut handling, model updates, and response population.
- [x] Split IMUI checkbox/radio boolean-control behavior out of
      `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs` into private owner modules without
      changing the public checkbox/radio/switch facade surface.
      Result: `boolean_controls/checkbox.rs` owns checkbox label identity, model toggling,
      shortcuts, context-menu requests, and response population; `boolean_controls/radio.rs` owns
      radio label identity, shortcut/context-menu handling, click response, and response
      population. The root `boolean_controls.rs` file is now a thin module/re-export index beside
      the existing switch and visual owners.
- [x] Split IMUI checkbox pressable behavior out of
      `ecosystem/fret-ui-kit/src/imui/boolean_controls/checkbox.rs` into a private owner module
      without changing label identity, checkbox a11y, model toggling, shortcut gating,
      context-menu keyboard requests, changed response population, field chrome, or visual row
      layout.
      Result: `boolean_controls/checkbox/behavior.rs` owns pressable behavior installation,
      activate handler model toggling, shortcut model toggling, context-menu key handling,
      transient changed reads, and `ResponseExt` population. `checkbox.rs` keeps label identity,
      `CheckboxOptions` a11y wiring, field chrome, checkbox indicator mounting, boolean label
      mounting, and fill-row visual assembly.
- [x] Split IMUI radio pressable behavior out of
      `ecosystem/fret-ui-kit/src/imui/boolean_controls/radio.rs` into a private owner module
      without changing label identity, radio a11y, shortcut gating, context-menu keyboard
      requests, click response population, field chrome, or visual row layout.
      Result: `boolean_controls/radio/behavior.rs` owns pressable behavior installation,
      activate handler click signaling, shortcut click signaling, context-menu key handling,
      transient clicked reads, and `ResponseExt` population. `radio.rs` keeps label identity,
      `RadioOptions` a11y wiring, field chrome, radio indicator mounting, boolean label mounting,
      and fill-row visual assembly.
- [x] Split IMUI switch pressable behavior out of
      `ecosystem/fret-ui-kit/src/imui/boolean_controls/switch.rs` into a private owner module
      without changing label identity, switch a11y, model toggling, shortcut gating, active-trigger
      lifecycle semantics, changed/clicked response population, field chrome, or visual row
      layout.
      Result: `boolean_controls/switch/behavior.rs` owns active-trigger behavior installation,
      activate handler model toggling, shortcut model toggling, lifecycle edit marking, transient
      changed/clicked reads, and `ResponseExt` population. `switch.rs` keeps label identity,
      `SwitchOptions` a11y wiring, field chrome, switch state badge mounting, boolean label
      mounting, and fill-row visual assembly.
- [x] Split IMUI interaction-runtime hover internals out of
      `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover.rs` into private owner modules
      without changing hovered-query, shared-delay, active-item block, or long-press behavior.
      Result: `hover/shared_delay.rs` owns window-scoped hover delay state/timers,
      `hover/timers.rs` owns deterministic per-element hover timer tokens, and
      `hover/long_press.rs` owns long-press timer emission. The root hover runtime file keeps the
      exported query helpers and local response-state assembly.
- [x] Split IMUI interaction-runtime drag internals out of
      `ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag.rs` into private owner modules
      without changing pressable drag, pointer-region drag/resize, active-item, or long-press
      behavior.
      Result: `drag/active_item.rs` owns active-item set/clear helpers, `drag/long_press_timer.rs`
      owns long-press arm/cancel, `drag/pointer_region.rs` owns pointer-region drag lifecycle, and
      `drag/response.rs` owns `DragResponse` population. The root drag runtime file keeps
      drag-kind/threshold helpers and the pressable drag state machine.
- [x] Split IMUI slider a11y/interaction/visual ownership out of
      `ecosystem/fret-ui-kit/src/imui/slider_controls.rs` into private owner modules without
      changing the public slider facade surface.
      Result: `slider_controls/a11y.rs` owns semantics value/range/step decoration,
      `slider_controls/interaction.rs` owns pointer/key model editing and lifecycle signals, and
      `slider_controls/visual.rs` owns track/fill/value badge assembly. The root slider file keeps
      label identity, option normalization, response population, and final element assembly.
- [x] Split IMUI slider pointer and keyboard interaction out of
      `ecosystem/fret-ui-kit/src/imui/slider_controls/interaction.rs` into private owner modules
      without changing pointer capture, active-item state, pointer model editing, keyboard
      step/page/home/end semantics, lifecycle edit signals, or changed response behavior.
      Result: `slider_controls/interaction/pointer.rs` owns pointer down/move/up capture,
      active-item set/clear, pointer value projection, pointer model mutation, and pointer
      lifecycle edit signals. `slider_controls/interaction/keyboard.rs` owns enabled keyboard
      gating, arrow/page/home/end value edits, snapping, and keyboard lifecycle edit signals.
      `interaction.rs` now keeps handler clearing, active/lifecycle model lookup, shared range
      input, and owner dispatch.
- [x] Split IMUI facade container-method dispatch out of
      `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods.rs` into private owner modules
      without changing facade method names.
      Result: `container_methods/flow.rs` owns item-flow / same-line / dummy / spacing / indent
      sugar, `container_methods/layout.rs` owns layout container / scroll / child-region dispatch,
      `container_methods/collections.rs` owns list-box / table / virtual-list dispatch, and
      `container_methods/menu_tabs.rs` owns menu-bar / tab-bar dispatch. The root container-methods
      file is now a thin re-export index.
- [x] Split IMUI facade container wrapper methods out of
      `ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers.rs` into private owner
      modules without changing `ImUiFacade` method names or behavior.
      Result: `container_wrappers/flow.rs` owns item-flow / same-line / dummy / spacing / indent
      wrappers, `container_wrappers/layout.rs` owns horizontal / vertical / grid / scroll /
      child-region wrappers, `container_wrappers/collections.rs` owns list-box / table /
      virtual-list wrappers, and `container_wrappers/menu_tabs.rs` owns menu-bar / tab-bar
      wrappers. The root container-wrappers file is now a thin module index.
- [x] Split IMUI widget response types out of
      `ecosystem/fret-ui-kit/src/imui/response/widgets.rs` into private owner modules without
      changing response type names or accessors.
      Result: `response/widgets/open.rs` owns disclosure/combo responses,
      `response/widgets/text_picker.rs` owns input text picker responses,
      `response/widgets/tabs.rs` owns tab responses, `response/widgets/table.rs` owns table
      response aggregation, `response/widgets/table/header.rs` owns table header responses,
      `response/widgets/table/resize.rs` owns table resize responses, and
      `response/widgets/virtual_list.rs` owns virtual-list responses. The root
      `widgets.rs` file is now a thin module/re-export index beside the existing child-region owner.
- [x] Split IMUI text-control option types out of
      `ecosystem/fret-ui-kit/src/imui/options/controls/text.rs` into private owner modules without
      changing option type names, fields, defaults, or re-export paths.
      Result: `text/filters.rs` owns named/custom input filters, `text/input.rs` is the current
      input-text option re-export hub, `input/mode.rs` owns `InputTextMode`, `input/options.rs`
      owns `InputTextOptions`, `text/picker.rs` is the current picker option re-export hub,
      `picker/filter.rs` owns picker filter matching, `picker/options.rs` owns picker default
      popup/options, and `text/textarea.rs` is the current textarea option re-export hub,
      `textarea/submit_key.rs` owns textarea submit-key policy, and `textarea/options.rs` owns
      textarea defaults. The root `text.rs` file is now a thin module/re-export index.
- [x] Split IMUI collection option types out of
      `ecosystem/fret-ui-kit/src/imui/options/collections.rs` into private owner modules without
      changing table, table-column, or virtual-list option type names and defaults.
      Result: `collections/table_column.rs` owns table column identity/visibility/sort/resize/pin
      helpers, `collections/table.rs` is the current table option re-export hub, and
      `collections/virtual_list.rs` owns virtual-list defaults. The root `collections.rs` file is
      now a thin module/re-export index.
- [x] Split IMUI table-column primitive option types out of
      `ecosystem/fret-ui-kit/src/imui/options/collections/table_column.rs` into a private owner
      module without changing public re-export paths, width constructors, resize defaults, sort
      direction, pinning helpers, identity inference, or table composition behavior.
      Result: `options/collections/table_column/primitives.rs` owns `TableColumnWidth`,
      `TableColumnResizeOptions`, `TableSortDirection`, `TableColumnPin`, width constructors, and
      default resize limits. `table_column.rs` keeps the `TableColumn` builder, identity
      inference, accessor seams, and visibility/sort/resize/pin policy methods.
- [x] Split IMUI table-column visibility snapshot data shapes out of
      `ecosystem/fret-ui-kit/src/imui/table_column_visibility/state.rs` into a private owner
      module without changing public re-export paths, serde payload shape, empty-id filtering,
      duplicate restore policy, or runtime visibility application.
      Result: `table_column_visibility/state/snapshot.rs` owns
      `TableColumnVisibilitySnapshot`, `TableColumnVisibilityEntry`, serde derives, public data
      fields, and snapshot/entry accessors. `state.rs` keeps runtime override storage, mutation
      helpers, snapshot restore/apply orchestration, and `TableColumn` visibility policy
      application.
- [x] Split IMUI table-column visibility menu identity helpers out of
      `ecosystem/fret-ui-kit/src/imui/table_column_visibility/menu.rs` into a private owner module
      without changing stable column-id filtering, visible-label filtering, generated test-id
      suffixes, menu item state updates, or table header context-menu behavior.
      Result: `table_column_visibility/menu/identity.rs` owns stable menu column id extraction,
      visible menu label parsing, and slug-like test-id suffix generation. `menu.rs` keeps header
      context-menu composition, menu item/group rendering, model updates, and response population.
- [x] Split IMUI table-column visibility response structs/accessors out of
      `ecosystem/fret-ui-kit/src/imui/table_column_visibility.rs` into a private response owner
      without changing public response type names, accessors, changed/clicked semantics, opaque
      fields, menu item construction, or header context-menu response aggregation.
      Result: `table_column_visibility/response.rs` owns
      `TableColumnVisibilityMenuResponse`, `TableColumnVisibilityHeaderContextMenuResponse`, and
      `TableColumnVisibilityMenuItemResponse`. The root file keeps options, state re-exports,
      public helper forwarding, and test wiring.
- [x] Split IMUI container/layout option types out of
      `ecosystem/fret-ui-kit/src/imui/options/containers.rs` into private owner modules without
      changing option type names, fields, defaults, or re-export paths.
      Result: `containers/flow.rs` owns item-flow/same-line/spacing/indent/grid options and the
      IMUI layout-token defaults, `containers/scroll.rs` owns scroll options,
      `containers/list_box.rs` owns list-box options, and `containers/child_region.rs` is the
      current child-region option re-export hub. The root `containers.rs` file is now a thin
      module/re-export index. The 2026-05-27 flow-option owner split below further divides the
      flow owner.
- [x] Split IMUI flow option defaults and records out of
      `ecosystem/fret-ui-kit/src/imui/options/containers/flow.rs` into private owner modules
      without changing option type names, fields, defaults, token keys, or public re-export paths.
      Result: `containers/flow.rs` is now a private module/re-export index,
      `flow/spacing.rs` owns the IMUI layout-token defaults, `flow/inline.rs` is the current
      inline option re-export hub, `flow/inline/item_flow.rs` owns item-flow options,
      `flow/inline/same_line.rs` owns same-line options, `flow/linear.rs` is the current linear
      option re-export hub, `flow/linear/horizontal.rs` owns horizontal options,
      `flow/linear/vertical.rs` owns vertical options, `flow/spacer.rs` is the current spacer
      option re-export hub, `flow/spacer/dummy.rs` owns dummy options, `flow/spacer/spacing.rs`
      owns spacing options, `flow/spacer/indent.rs` owns indent options, and `flow/grid.rs` owns
      grid options.
- [x] Split IMUI floating-surface drag-kind and state ownership out of
      `ecosystem/fret-ui-kit/src/imui/floating_surface.rs` into private owner modules without
      changing floating-area, floating-window, drag, resize, activation, or collapse behavior.
      Result: `floating_surface/kinds.rs` owns drag/resize kind ids, resize-handle tags, and
      transient activation/collapse keys; `floating_surface/state.rs` owns floating-area and
      floating-window state records. The root `floating_surface.rs` now keeps area composition,
      pointer-region wiring, layer wiring, and private re-exports.
- [x] Split IMUI floating-area composition out of
      `ecosystem/fret-ui-kit/src/imui/floating_surface.rs` into a private owner module without
      changing floating-area position state, drag snapshot application, layer registration,
      no-input/pass-through gates, test ids, or `FloatingAreaResponse` semantics.
      Result: `floating_surface/area.rs` owns area registration, facade content mounting,
      absolute area layout, interaction gates, and response assembly. The root
      `floating_surface.rs` keeps drag-surface pointer-region behavior, layer/kind/state
      re-exports, and module wiring.
- [x] Split IMUI floating-area drag/state reconciliation out of
      `ecosystem/fret-ui-kit/src/imui/floating_surface/area.rs` into a private owner module without
      changing drag snapshot matching, device-pixel snapping, test-id overrides, child window
      resize feedback, or `FloatingAreaResponse` dragging/position semantics.
      Result: `floating_surface/area/drag_state.rs` owns drag snapshot discovery,
      drag-position reconciliation, scale-factor snapping, test-id state updates, and final
      placement readback. `floating_surface/area.rs` now only orchestrates layer registration,
      context creation, IMUI child mounting, layout shell creation, and response assembly.
- [x] Split IMUI floating-area drag-surface behavior out of
      `ecosystem/fret-ui-kit/src/imui/floating_surface.rs` into a private owner module without
      changing drag setup delegation, focusable key stub installation, double-click hooks,
      activation signals, drag threshold handling, or IMUI child mounting.
      Result: `floating_surface/drag_surface.rs` owns `floating_area_drag_surface_element(...)`,
      pointer-region wiring, double-click dispatch, activation event recording, pointer drag
      move/up handling, setup callback invocation, and IMUI child mounting. The root
      `floating_surface.rs` is now a module index/re-export hub for area, drag-surface, kinds,
      layer, and state owners.
- [x] Split IMUI floating-window resize state/snapshot ownership out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_resize.rs` into a private owner module
      without changing resize handles, left/right/top/bottom/corner resize behavior, collapse
      reset, device-pixel snapping, or resize test-id generation.
      Result: `floating_window_resize/state.rs` owns active resize snapshot lookup, drag delta
      application, min/max size clamping, origin updates for left/top handles, collapse reset,
      device-pixel snapping, and resize state/test-id output. The root file is now a thin
      `handles`/`state` index plus the shared handle test-id record.
- [x] Split IMUI floating-window active resize snapshot lookup out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_resize/state.rs` into a private owner module
      without changing resize handle enumeration, runtime drag matching, or downstream resize
      calculation.
      Result: `floating_window_resize/snapshot.rs` owns active resize drag discovery and snapshot
      capture. `state.rs` now focuses on applying resize deltas, clamping size, updating origin,
      resetting collapsed/non-drag state, snapping to device pixels, and producing resize output.
- [x] Split IMUI floating-window resize drag-delta application out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_resize/state.rs` into a private owner module
      without changing active resize snapshots, left/top origin updates, min/max size clamping,
      corner-handle behavior, collapsed reset policy, device-pixel snapping, or resize test ids.
      Result: `floating_window_resize/state/drag_apply.rs` owns drag delta calculation, min/max
      clamping, left/top origin reconciliation, all eight resize-handle branches, and
      `last_resize_position` advancement. `state.rs` keeps lifecycle state lookup, reset/snap
      policy, resize output assembly, and handle test-id packaging.
- [x] Split IMUI floating-window resize initial state and output DTO out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_resize/state.rs` into private child owners
      without changing resize test-id strings, initial size defaults, collapsed reset policy,
      device-pixel snapping, drag application, or handle test-id packaging.
      Result: `floating_window_resize/state/initial.rs` owns initial `FloatWindowState` and stable
      resize/title/close test-id construction, `state/output.rs` owns
      `FloatingWindowResizeStateOutput`, and `state.rs` keeps `cx.state_for(...)`, snapshot/collapse
      orchestration, pixel snapping, drag application, and output assembly.
- [x] Split IMUI floating-window resize handle layout and pointer behavior out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_resize/handles.rs` into private owner modules
      without changing resize handle placement, cursors, drag lifecycle, activation handoff, or
      pointer capture/release behavior.
      Result: `floating_window_resize/handles/layout.rs` owns handle geometry,
      `floating_window_resize/handles/pointer.rs` owns pointer-region wiring, pointer capture,
      runtime drag begin/update/cancel, cursor updates, and activation handoff. `handles.rs` now
      only stacks the body/blocker with the eight resize handles.
- [x] Split IMUI floating-window resize cursor mapping out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_resize/handles/layout.rs` into a private
      owner module without changing handle placement, cursor icons, pointer capture/release,
      activation handoff, or resize drag lifecycle.
      Result: `floating_window_resize/handles/cursor.rs` owns handle-to-cursor mapping for all
      eight handles, `floating_window_resize/handles/layout.rs` owns layout geometry only, and
      `floating_window_resize/handles/pointer.rs` composes both before wiring pointer-region
      behavior.
- [x] Split IMUI selectable keyboard ownership out of
      `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs` into a private owner module without
      changing selectable activation, popup close, menu navigation, or context-menu behavior.
      Result: `selectable_controls/keyboard.rs` now owns shortcut handling, popup close-on-activate
      behavior, context-menu requests, and popup menu arrow-key navigation. The root file keeps
      label identity, pressable assembly, and row rendering.
- [x] Split IMUI selectable pressable behavior out of
      `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs` into a private owner module without
      changing label identity, selectable a11y, pointer click reporting, keyboard activation
      lifecycle marking, popup close-on-activate behavior, shortcut/context-menu/nav delegation,
      response population, or row visual composition.
      Result: `selectable_controls/behavior.rs` owns pressable behavior installation,
      activate-handler popup close/click signaling, keyboard owner delegation, transient clicked
      reads, and `ResponseExt` population. `selectable_controls.rs` keeps label identity,
      `SelectableOptions` a11y wiring, selected/highlighted state reads, and row visual assembly.
- [x] Split IMUI child-region resize handle/drag ownership out of
      `ecosystem/fret-ui-kit/src/imui/child_region.rs` into a private owner module without
      changing the public child-region facade or response surface.
      Result: `child_region/resize.rs` now owns resize axis layout, resize handle constants,
      pointer-region drag wiring, enabled/min/max response writes, and drag edge reconciliation.
      The root file keeps scroll-area/content composition, framed chrome, root test-id routing, and
      stack assembly.
- [x] Split IMUI child-region resize axis geometry out of
      `ecosystem/fret-ui-kit/src/imui/child_region/resize.rs` into a private owner module without
      changing handle keys, cursor selection, absolute handle layout, pointer-region drag wiring,
      response writes, or resize option smoke behavior.
      Result: `child_region/resize/axis.rs` owns X/Y handle width/height constants, axis keys,
      resize cursors, and absolute handle layout. `resize.rs` keeps handle entry points,
      response writes, pointer-region drag lifecycle wiring, and drag edge merging.
- [x] Split IMUI child-region resize response records out of
      `ecosystem/fret-ui-kit/src/imui/response/widgets/child_region.rs` into a private owner module
      without changing public re-export paths, aggregate `ChildRegionResponse` accessors, drag
      accessors, min/max accessors, or width/height clamping helpers.
      Result: `response/widgets/child_region/resize.rs` owns `ChildRegionResizeXResponse`,
      `ChildRegionResizeYResponse`, drag/min/max accessors, width/height clamping helpers, and
      clamping tests. `child_region.rs` keeps aggregate response storage/accessors plus resize
      response re-exports.
- [x] Split IMUI tooltip overlay pointer-open and panel composition ownership out of
      `ecosystem/fret-ui-kit/src/imui/tooltip_overlay.rs` into private owner modules without
      changing the public tooltip facade or hover/dismissal behavior.
      Result: `tooltip_overlay/trigger.rs` now owns pointer-move open gating and pointer-transit
      checks, `tooltip_overlay/panel.rs` owns concrete panel placement, chrome, semantics, and
      content column assembly, `tooltip_overlay/runtime.rs` owns tooltip lifecycle, interaction
      bounds, update, dismissal, and request orchestration, and the root file is now a thin module
      index.
- [x] Split IMUI menu/popup/tab/tooltip option types out of
      `ecosystem/fret-ui-kit/src/imui/options/menus.rs` into private owner modules without
      changing option type names, fields, defaults, or re-export paths.
      Result: `menus/popup.rs` is the current popup option re-export hub, `popup/menu.rs` owns
      popup-menu options, `popup/modal.rs` owns popup-modal options, `menus/menu.rs` owns menu bar,
      begin-menu/submenu, and menu-item options plus shortcut seams, `menus/tab.rs` owns tab-bar
      selection/gap/test-id options, and `menus/tooltip.rs` owns tooltip placement, timing,
      hoverable-content, and diagnostics options. The root `menus.rs` file is now a thin
      module/re-export index.
- [x] Split IMUI begin-menu state/open-policy ownership out of
      `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu.rs` into a private owner module
      without changing menubar trigger activation, popup open/close, active-trigger synchronization,
      disabled-popup cleanup, or `DisclosureResponse` open/toggled semantics.
      Result: `menu_family_controls/menu_state.rs` owns begin-menu state capture, row/popup/was-open
      models, menubar open-menu synchronization, active trigger state writes, open-request
      resolution, disabled-popup cleanup, and render-state recording. `menu.rs` now keeps public
      flow orchestration plus trigger and popup mounting.
- [x] Split IMUI begin-menu state capture/read helpers out of
      `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state.rs` into a private owner
      module without changing row/popup/was-open model identity, render-state recording, or menubar
      open-policy behavior.
      Result: `menu_family_controls/menu_state/capture.rs` owns `BeginMenuState`,
      `MenuRenderState`, model capture, row/open-menu reads, and render-state recording.
      `menu_state.rs` now focuses on menubar open-menu mutation, active-trigger synchronization,
      open-request resolution, and disabled-popup cleanup.
- [x] Split IMUI begin-menu open-policy mutations out of
      `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state.rs` into a private owner
      module without changing menubar open-menu synchronization, active-trigger writes,
      trigger-click toggling, open-request resolution, or disabled-popup cleanup.
      Result: `menu_family_controls/menu_state/open_policy.rs` owns the begin-menu open-policy
      state machine. `menu_state.rs` is now a thin capture/open-policy module index.
- [x] Split IMUI begin-menu active-trigger open-policy out of
      `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/open_policy.rs` into a
      private owner module without changing menubar active-trigger synchronization, group-active
      writes, post-trigger reconciliation, trigger-click toggling, open-request resolution, or
      disabled-popup cleanup.
      Result: `menu_family_controls/menu_state/open_policy/active_trigger.rs` owns active-trigger
      synchronization, post-trigger reconciliation, and `MenubarActiveTrigger` writes.
      `open_policy.rs` now keeps trigger-click toggling, open-request resolution,
      disabled-popup cleanup, and the private owner re-export.
- [x] Split IMUI begin-submenu trigger wiring and open-policy reconciliation out of
      `ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu.rs` into private owner modules
      without changing submenu trigger geometry hints, hover/shortcut behavior, sibling switching,
      popup open/close semantics, or `DisclosureResponse` open/toggled reporting.
      Result: `menu_family_controls/submenu/trigger.rs` owns submenu menu-item trigger assembly,
      submenu flag/expanded semantics, shortcut option forwarding, and `sub_trigger::wire(...)`
      geometry hints. `menu_family_controls/submenu/open_policy.rs` owns clicked-trigger
      submenu-state reconciliation, stale-open cleanup, and popup open/close anchoring. The root
      `submenu.rs` keeps the public flow, state reads, popup mounting, and response assembly.
- [x] Split IMUI begin-menu trigger behavior out of
      `ecosystem/fret-ui-kit/src/imui/menu_family_controls/trigger.rs` into a private owner module
      without changing menu trigger a11y, label identity, activate shortcut gating, keyboard
      lifecycle marking, menubar registry synchronization, arrow-open behavior, click response
      population, or trigger visual chrome.
      Result: `menu_family_controls/trigger/behavior.rs` owns active-trigger behavior
      installation, keyboard activation lifecycle marking, activate shortcut handling, menubar row
      registry/sync wiring, arrow-down/up open behavior, transient click reads, and trigger
      `ResponseExt` population. `trigger.rs` keeps label identity, `PressableA11y`, pressable shell
      construction, and `visual::menu_trigger_visual(...)` mounting.
- [x] Split IMUI table header row assembly out of
      `ecosystem/fret-ui-kit/src/imui/table_controls/render.rs` into a private owner module
      without changing header visibility, sortable/plain header cells, resize response metadata,
      pinned/horizontal-scroll wrapping, test ids, or aggregate `TableResponse` headers.
      Result: `table_controls/header_row.rs` owns the keyed header row, visible-header-cell
      assembly, sortable/plain wrapper selection, resize response initialization, header response
      collection, and header row wrapping. `render.rs` keeps palette, visible-column, scroll, and
      header-presence decisions plus body rows, root chrome, semantics, and final response
      assembly.
- [x] Split IMUI table body-row preparation out of
      `ecosystem/fret-ui-kit/src/imui/table_controls/render.rs` into a private owner module
      without changing hidden-column filtering, fallback empty cells, default/explicit test-id
      precedence, striped/background selection, pinned/horizontal-scroll wrapping, or aggregate
      `TableResponse` headers.
      Result: `table_controls/render/body_rows.rs` owns keyed body row assembly, cell iteration,
      hidden-column filtering, fallback empty-cell insertion, body cell wrapping, and body row
      wrapping. `render.rs` keeps palette, visible-column, scroll/header decisions, root chrome,
      semantics, and final `TableResponse` assembly.
- [x] Split IMUI table builder row/cell collection out of
      `ecosystem/fret-ui-kit/src/imui/table_controls.rs` into a private builder owner without
      changing `table(...)`/`table_with_options(...)` facade calls, public `ImUiTable` /
      `ImUiTableRow` method names, row/cell test-id derivation, child `ImUiFacade` mounting,
      `cell_text(...)` text routing, or final table rendering.
      Result: `table_controls/builder.rs` owns `ImUiTable` / `ImUiTableRow`, built row/cell
      records, row/cell test-id derivation, child `ImUiFacade` mounting, and `cell_text(...)`
      table-cell text routing. `table_controls.rs` keeps module wiring, public table builder
      re-exports, `table_element(...)`, and final render dispatch.
- [x] Split IMUI table header label/sort helpers out of
      `ecosystem/fret-ui-kit/src/imui/table_controls/header.rs` into a private owner module without
      changing visible-label parsing, sortable/plain header wrapping, sort glyph text role,
      sortable a11y labels, resize handle placement, or header response aggregation.
      Result: `table_controls/header/labels.rs` owns visible header label parsing,
      sort-indicator text, sortable a11y labels, header content boxes, and header label text.
      `header.rs` keeps sortable/plain header-cell assembly and resize-handle wrapping.
- [x] Split IMUI table header trigger behavior out of
      `ecosystem/fret-ui-kit/src/imui/table_controls/header/trigger.rs` into a private owner module
      without changing sortable/plain trigger props, primary activation policy, keyboard lifecycle
      marking, context-menu request propagation, plain-header click suppression, response
      population, or sortable header visual layout.
      Result: `table_controls/header/trigger/behavior.rs` owns active-trigger behavior
      installation, sortable keyboard activation lifecycle marking, clicked transient draining for
      plain headers, and `ResponseExt` population. `trigger.rs` keeps header trigger props,
      a11y/key-activation policy, keyed surface assembly, and sortable header visual construction.
- [x] Split IMUI table row-group mechanics out of
      `ecosystem/fret-ui-kit/src/imui/table_controls/body.rs` into a private owner module without
      changing row semantics, cell wrapping, pinned left/right grouping, horizontal center-scroll
      wrapping, column gaps, or table response behavior.
      Result: `table_controls/row_groups.rs` owns pinned-cell splitting, left/center/right row
      groups, horizontal center scroll, and the shared horizontal flex primitive. `body.rs` keeps
      `PreparedTableCell`, `TablePalette`, row semantics/background selection, and cell wrapping.
- [x] Split IMUI pressable item response population out of
      `ecosystem/fret-ui-kit/src/imui/item_behavior.rs` into a private owner module without
      changing shared button/checkbox/radio/selectable/combo/image/debug-draw pressable behavior,
      context-menu signals, pointer-click modifiers, drag response merging, hover query hooks, or
      `ResponseExt` population semantics.
      Result: `item_behavior/response.rs` owns transient signal reads, context anchor/modifier
      reads, drag response merging, hover query hook installation, and final pressable response
      population. `item_behavior.rs` keeps pressable hook installation, active-item/long-press/
      lifecycle/context-menu models, pointer-up transient emission, and the existing re-exported
      call surface.
- [x] Split IMUI tab-family item collection, selected-model normalization, and panel/list assembly
      out of `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs` into a private owner module
      without changing the public tab-bar builder or response surface.
      Result: `tab_family_controls/items.rs` owns `BuiltTabItem`, tab-list semantics, trigger
      response aggregation, focus fallback, and selected panel assembly. The root
      `tab_family_controls.rs` keeps the public `ImUiTabBar` builder and `tab_bar_element(...)`
      entrypoint, while `trigger.rs` keeps per-trigger activation and shortcut behavior.
- [x] Split IMUI tab-family selected-model normalization out of
      `ecosystem/fret-ui-kit/src/imui/tab_family_controls/items.rs` into a private owner module
      without changing selected fallback order, disabled-tab filtering, selected-model correction,
      trigger response aggregation, focus fallback, tab-list semantics, panel mounting, or
      `TabBarResponse` assembly.
      Result: `tab_family_controls/items/selection.rs` owns selected model reads, current-tab
      validity checks, default-selected fallback, first-enabled fallback, and model correction
      writes. `tab_family_controls/items.rs` keeps `BuiltTabItem`, trigger response aggregation,
      focus fallback, tab-list/panel assembly, and final `TabBarResponse` construction.
- [x] Split IMUI tab trigger active-trigger behavior out of
      `ecosystem/fret-ui-kit/src/imui/tab_family_controls/trigger.rs` into a private behavior owner
      without changing tab a11y, selected-model writes, activate-shortcut handling, clicked
      response population, or tab visual construction.
      Result: `tab_family_controls/trigger/behavior.rs` owns active-trigger behavior installation,
      keyboard lifecycle marking, selected-model writes, activate-shortcut handling, clicked
      transient reads, and `ResponseExt` population. `trigger.rs` keeps tab trigger props,
      collection a11y, keyed trigger assembly, and visual mounting.
- [x] Split IMUI popup-menu policy state and panel composition out of
      `ecosystem/fret-ui-kit/src/imui/popup_overlay/menu.rs` into private owner modules without
      changing popup/menu/submenu behavior or facade entry points.
      Result: `popup_overlay/menu/policy.rs` owns `ImUiMenuNavState`,
      `ImUiPopupMenuPolicyState`, and root submenu-policy synchronization;
      `popup_overlay/menu/panel.rs` owns popup panel placement, menu semantics, nav-state
      installation, content mounting, and focus targets. The root `popup_overlay/menu.rs` keeps
      overlay id/root-name setup, menubar policy lookup, dismiss/auto-focus handlers, and overlay
      request dispatch.
- [x] Split IMUI popup-menu panel layout/chrome and content mounting out of
      `ecosystem/fret-ui-kit/src/imui/popup_overlay/menu/panel.rs` into private owner modules
      without changing popup panel placement, menu semantics, nav-state installation, provider
      nesting, IMUI child mounting, or focus target extraction.
      Result: `popup_overlay/menu/panel/layout.rs` owns popper placement, menu semantics layout,
      panel palette/chrome, and panel column props; `popup_overlay/menu/panel/content.rs` owns
      popup/menubar policy provider nesting and IMUI child mounting. The root `panel.rs` keeps
      open/anchor lifecycle reads, keepalive updates, nav-state installation, panel id storage, and
      `PopupMenuBuilt` assembly.
- [x] Split IMUI popup modal layout/chrome construction out of
      `ecosystem/fret-ui-kit/src/imui/popup_overlay/modal.rs` into a private owner module without
      changing modal open/keepalive policy, Escape/outside-press dismissal, barrier behavior,
      focus handoff, centered panel placement, test ids, or overlay request semantics.
      Result: `popup_overlay/modal/layout.rs` owns modal palette, centered panel geometry,
      layer stack props, backdrop props, dialog semantics layout, and panel chrome props.
      `modal.rs` keeps popup store reads, keepalive generation, dismiss policy, focus tracking,
      facade content mounting, and `OverlayRequest::modal` assembly.
- [x] Split IMUI menu-item interaction behavior out of
      `ecosystem/fret-ui-kit/src/imui/menu_controls/element.rs` into a private owner module without
      changing menu item, checkbox/radio menu item, command menu item, submenu, or menubar behavior.
      Result: `menu_controls/interaction.rs` owns enabled/action gating, pressable props,
      activation/shortcut handlers, popup menu roving focus, menubar horizontal-arrow switching,
      command dispatch source metadata, and menu-item `ResponseExt` population. The element file
      keeps row panel/indicator/shortcut/label visual assembly and the custom `pressable_hook`
      insertion point.
- [x] Split IMUI menu-item keyboard/navigation behavior out of
      `ecosystem/fret-ui-kit/src/imui/menu_controls/interaction.rs` into a private owner module
      without changing popup menu roving focus, shortcut, or menubar horizontal-arrow behavior.
      Result: `menu_controls/keyboard.rs` owns item-local activate shortcut handling, popup menu
      roving focus, menubar close-auto-focus suppression, and horizontal-arrow menu switching.
      `interaction.rs` now keeps enabled/action gating, pressable props, activation dispatch, and
      response population.
- [x] Split IMUI menu-item active-trigger/activation/response behavior out of
      `ecosystem/fret-ui-kit/src/imui/menu_controls/interaction.rs` into a nested private owner
      without changing enabled/action gating, pressable props, popup close-on-activate, command
      dispatch metadata, keyboard owner wiring, or menu-item `ResponseExt` population.
      Result: `menu_controls/interaction/behavior.rs` owns active-trigger installation,
      activation handler wiring, clicked transient draining, command dispatch source metadata, and
      active-trigger response population. `interaction.rs` keeps menu-item interaction structs,
      enabled/action gating, pressable prop construction, and thin forwarding call sites for
      element/keyboard users.
- [x] Split IMUI multi-select state storage and normalization out of
      `ecosystem/fret-ui-kit/src/imui/multi_select.rs` into a private owner module without
      widening the public collection helper surface.
      Result: `multi_select/state.rs` owns `ImUiMultiSelectState` storage and read-only accessors,
      while `multi_select/state/selection.rs` owns selected-order normalization, anchor repair,
      crate-local mutation helpers, and `is_selected(...)`. The root `multi_select.rs` keeps model
      hook, selectable response wiring, click-modifier policy, and response changed reporting.
- [x] Split IMUI virtual-list runtime projection and row mechanics out of
      `ecosystem/fret-ui-kit/src/imui/virtual_list_controls.rs` into private owner modules without
      changing the facade virtual-list API or row clipping semantics.
      Result: `virtual_list_controls/runtime.rs` owns runtime option projection and list layout,
      `virtual_list_controls/row.rs` owns row packing, test-id suffixing, row-height resolution,
      striped row chrome, and fixed-height clipping. The root file keeps keyed list assembly,
      render-range tracking, focus child mounting, and list-level semantics.

## P0 - Source Baseline

- [x] Create the dedicated `imui-imgui-gap-closure-v1` workstream.
- [x] Refresh the current-source audit from Fret source and `repo-ref/imgui`.
- [x] Mark `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md` as partially superseded for
      current gap reads.
- [x] Wire the new lane into `docs/workstreams/README.md`, `docs/roadmap.md`, and
      `docs/todo-tracker.md`.
- [x] Run the P0 doc/source gates listed in `EVIDENCE_AND_GATES.md`.
      Result: `json shape`, `workstream catalog`, `imui facade teaching source`,
      `imui workstream source`, `git diff --check`, and `cargo check -p fret-examples-imui`
      all pass.

## P1 - Fearless Cleanup / Deletion Candidates

- [x] Audit public teaching imports for stale direct `fret_imui::` or `fret_ui_kit::imui::`
      default-path examples.
      First slice landed: `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs` now routes
      `TableSortDirection` through the app-facing `fret::imui::kit` facade, and
      `tools/gate_imui_facade_teaching_source.py` forbids the stale direct kit import from
      returning there.
      Second slice landed: `apps/fret-examples/src/workspace_shell_demo.rs` now routes pane-proof
      IMUI option types through `fret::imui::kit`, and both IMUI source gates forbid direct
      `fret_ui_kit::imui` imports from returning to that default pane-first proof.
      Third slice landed: `apps/fret-examples/src/imui_editor_proof_demo.rs` and its
      `collection.rs` module now route golden-proof IMUI option/state types through `fret::imui`,
      while recipe-layer imports stay explicit.
      2026-05-16 cleanup: `imui_interaction_showcase_demo` now describes its current root
      `fret::imui` teaching surface instead of the historical direct `fret_imui` control-flow
      wording, and the facade teaching source gate rejects that stale wording from returning.
- [x] Identify duplicate helper aliases that can be deleted behind a source-policy gate.
      Audit result: historical duplicate aliases are already deleted from active source,
      `imui::adapters` is contract-only, and `*_with_options(...)` helpers are canonical explicit
      option entry points rather than compatibility aliases. P1 closes with no further delete.
- [x] Check whether `fret-ui-editor::imui` remains a pure adapter over declarative editor controls.
      Audit result: current `ecosystem/fret-ui-editor/src/imui.rs` remains a thin adapter around
      declarative editor controls/composites via `into_element(...)`; no code refactor is needed in
      that crate for this P1 pass.
- [x] Check large `fret-ui-kit::imui` implementation files for owner splits that can be performed
      without public API changes.
      Audit result: `debug_draw_controls.rs` was split in the dedicated
      `imui-debug-draw-owner-split-v1` follow-on and that lane is now closed. Future additive debug
      draw capabilities still need separate follow-ons.
      2026-05-13 rebase: `imui-kit-owner-split-v1`,
      `imui-facade-disclosure-owner-split-v1`, and
      `imui-facade-text-model-owner-split-v1` are also closed. The subsequent
      `imui-facade-boolean-wrapper-owner-split-v1` and
      `imui-facade-value-model-owner-split-v1` lanes are closed as well. The later
      `imui-facade-container-wrapper-owner-split-v1` lane is closed too.
      2026-05-14 rebase: `imui-facade-floating-popup-owner-split-v1` lane is closed too. These
      lanes moved focused facade wrappers, trait default implementation bodies, and shared
      pressable response assembly into private owners without public API or runtime contract
      widening.

## P2 - User-Usable Golden Path

- [x] Pick the smallest runnable proof that should teach a complete editor panel.
      Current result: `apps/fret-examples/src/imui_editor_workbench_demo.rs` is the canonical
      product-facing editor workbench route and mounts the editor-notes workflow directly.
      `imui_editor_proof_demo` remains supporting collection-first proof evidence rather than the
      route a new user should open first.
- [x] Verify the proof includes state, command/action dispatch, editor controls, menu/popup, and
      diagnostic-friendly `test_id`s.
      Current result: `imui_editor_workbench_golden_path_surface` proves the canonical route owns
      the app shell, mounts a real editor workflow, surfaces the editor-owned style/theme preset
      picker, is exported by `fret-examples`, and is advertised by `fret-demo`.
- [x] Promote missing cookbook/docs references only after the proof runs and source gates pass.
      Current result: `apps/fret-cookbook/README.md`, `apps/fret-cookbook/EXAMPLES.md`,
      `docs/examples/README.md`, diagnostics first-open docs, and Demo/Metrics/Debug discovery now
      point to `cargo run -p fret-demo --bin imui_editor_workbench_demo`, while the older proof
      demos stay discoverable as supporting evidence.

## P3 - Dear ImGui-Class Follow-On Candidates

The catalog order below is a readiness/review order, not an implementation queue. Current execution
priority stays: product/golden workflow coherence, runner/backend multi-window hand-feel,
diagnostics/DevTools discoverability, then proof-led helper/API widening. See
`P3_EXECUTION_PRIORITY_REVIEW_2026-05-06.md`.

Readiness order for the next locally testable review slices:

1. Public facade/API catalog: keep the app-facing `fret::imui` lane explicit and freeze owner
   rules before adding more public helpers.
   Current readiness audit: `P3_PUBLIC_SURFACE_CATALOG_2026-05-06.md`. Keep `fret-imui`
   policy-light, put generic policy-heavy helpers in `kit`, editor controls in `editor`, docking
   helpers in `docking`, and continue teaching apps through `fret::imui`.
   2026-05-14 cleanup: `FloatingAreaContext` now exposes accessors instead of public identity /
   drag-kind fields, so callers can read the facade-created context without constructing invalid
   contexts.
   2026-05-14 follow-up: floating responses now follow the same accessor-first shape; tests use
   `resp.id()` rather than reaching through `resp.area.id`.
   2026-05-14 response follow-up: disclosure and combo responses now keep trigger/open/toggle
   fields crate-local, remove external default construction, and expose read-only `response()`,
   `open()`, and `toggled()` methods for callers.
   2026-05-14 text-picker follow-up: `InputTextPickerResponse` now keeps input/open/pick storage
   crate-local, removes external default construction, and exposes read-only `response()`, `open()`,
   `picked()`, and `picked_index()` accessors.
   2026-05-14 tab follow-up: tab-bar aggregate and tab-trigger responses now follow the same
   accessor-first shape; tab response storage stays crate-local and `TabBarResponse` no longer
   exposes external default construction.
   2026-05-14 virtual-list follow-up: `VirtualListResponse` now keeps its scroll handle and
   rendered-range storage crate-local while preserving read-only `handle()` and
   `rendered_range()` accessors.
   2026-05-14 table follow-up: table aggregate/header/resize responses now expose header metadata
   and resize bounds through accessors instead of public fields; response storage stays crate-local.
   2026-05-14 drag follow-up: `DragResponse` now keeps drag edge/delta storage crate-local while
   preserving read-only drag accessors and higher-level `ResponseExt` helpers.
   2026-05-14 drag/drop follow-up: source and target drag/drop responses now keep their storage and
   construction paths crate-local; public code reads helper-returned responses through accessors.
   2026-05-14 response drag-state follow-up: `ResponseExt` now keeps its aggregate drag response
   crate-local too. Public callers read drag state through `drag()`, `drag_started()`,
   `dragging()`, `drag_stopped()`, `drag_delta()`, and `drag_total()`, while internal assemblers use
   crate-local mutators.
   2026-05-14 press/context follow-up: `ResponseExt` press and context-menu derived signals now
   keep storage private. Public callers stay on `secondary_clicked()`, `double_clicked()`,
   `long_pressed()`, `press_holding()`, `context_menu_requested()`, `context_menu_anchor()`,
   `pointer_clicked()`, and `pointer_click_modifiers()`, while runtime assemblers use crate-local
   setters.
   2026-05-14 lifecycle follow-up: `ResponseExt` lifecycle edge storage is private as well. Public
   callers stay on `activated()`, `deactivated()`, `edited()`, and `deactivated_after_edit()`;
   runtime lifecycle, combo, text-picker, and disabled paths use crate-local set/merge/clear helpers.
   2026-05-14 hover/nav follow-up: raw hover, hover-delay, active-item block, and nav-highlight
   storage is private. Public callers and tests use accessor methods; runtime pressable/disclosure
   assemblers use crate-local setters, while disabled sanitization only clears nav highlight.
   2026-05-14 enabled follow-up: `ResponseExt.enabled` storage is private too. Public/demo/test
   callers use `enabled()`, while disabled sanitization and text controls use crate-local
   `set_enabled(...)`. `ResponseExt.id` is now sealed as a routing-identity surface too: public
   callers use `id()`, while response assemblers use crate-local `set_id(...)`. `ResponseExt.core`
   is now accessor-only as well: public code can still round-trip the shared
   `fret_authoring::Response` through `core()` / `from_core(...)`, while runtime assembly writes
   through crate-local core setters. The adapter signal record is now read-only too: adapter seam
   inputs keep builder-friendly public options, but emitted `AdapterSignalRecord` /
   `AdapterSignalMetadata` values expose identity, response, and metadata through accessors.
   2026-05-26 hover query owner split: `ImUiHoveredFlags` now lives in
   `response/hover/flags.rs`, while `hovered_like_imgui(...)` / `is_hovered(...)` query policy
   lives in `response/hover/query.rs`. The root `response/hover.rs` stays focused on
   `ResponseExt` storage, mutators, accessors, and drag convenience helpers.
   2026-05-26 lifecycle owner split: `ResponseExt` lifecycle signal mutators, merge helpers,
   clearing, and read-only accessors now live in `response/hover/lifecycle.rs`. The root
   `response/hover.rs` still owns the lifecycle storage fields but no longer owns lifecycle
   behavior bodies.
   2026-05-26 press/context owner split: `ResponseExt` secondary-click, double-click, long-press,
   hold, context-menu, pointer-click, and pointer-modifier behavior now lives in
   `response/hover/press_context.rs`. The root `response/hover.rs` keeps storage only for those
   signals.
   2026-05-26 hover-state owner split: `ResponseExt` raw pointer hover, popup-barrier hover,
   hover-delay, active-item block, and nav-highlight mutators/accessors now live in
   `response/hover/hover_state.rs`. The root `response/hover.rs` keeps the hover state storage
   fields only.
   2026-05-26 core-state owner split: `ResponseExt` core `fret_authoring::Response`, id, enabled,
   clicked, changed, rect, hover, press, and focus mutators/accessors now live in
   `response/hover/core_state.rs`. The root `response/hover.rs` keeps core/id/enabled storage only.
   2026-05-26 menu-family menu owner split: `begin_menu_with_options(...)` now lives in
   `menu_family_controls/menu.rs`. The root `menu_family_controls.rs` keeps menubar policy state,
   menu-bar element construction, module wiring, and tests only.
   2026-05-26 debug-draw response owner split: `DebugDrawResponse` now lives in
   `debug_draw_controls/response.rs`, and the opaque-output source gate follows the new owner. The
   root `debug_draw_controls.rs` re-exports the public surface.
   2026-05-26 debug-draw options owner split: public debug draw options/style/vertex types now
   live in `debug_draw_controls/options.rs`. The root `debug_draw_controls.rs` re-exports them and
   later owner splits moved draw-list state to `debug_draw_controls/draw_list.rs` and facade entry
   glue to `debug_draw_controls/facade.rs`.
   2026-05-27 debug-draw options sub-owner split: `debug_draw_controls/options.rs` is now a thin
   re-export index; `options/root.rs` owns root/interaction canvas options, `options/stroke.rs`
   owns stroke style and path-style conversion, `options/round_corners.rs` owns rounded-corner
   flags, `options/media.rs` owns image/svg option bags, and `options/vertex.rs` owns mesh
   vertices.
   2026-05-27 debug-draw path-builder shape-method owner split:
   `debug_draw_controls/path_builder/shape_methods.rs` now owns rect, Bezier, arc, fast-arc, and
   elliptical-arc authoring methods. `path_builder.rs` keeps point basics, stroke/fill recording,
   and path state accessors.
   2026-05-27 debug-draw paint media owner split: `debug_draw_controls/paint/media.rs` now owns
   image, image-region, image-quad, rounded-image, rounded-image-region, SVG image, and SVG
   mask-icon painting. Root `paint.rs` keeps clip-stack balancing and media/shape command dispatch.
   2026-05-14 editor drag-value follow-up: `DragValueCoreResponse` now keeps drag/hover/press/focus
   storage private and no longer exposes external default construction. `DragValueCore` still owns
   response construction, while editor controls read visual state through `dragging()`, `hovered()`,
   `pressed()`, and `focused()`.
   2026-05-14 debug-draw follow-up: `DebugDrawResponse` now keeps response and summary storage
   private, removes external default construction, and exposes the underlying interaction response
   through `response()`.
   2026-05-14 debug-draw summary follow-up: `DebugDrawCommandSummary` and
   `DebugDrawListSummary` now keep diagnostic storage private as well. Public callers read command
   kind/channel/clip/count metrics through explicit accessors instead of copying fields.
   2026-05-27 debug-draw summary owner split: `debug_draw_controls/summaries.rs` is now a thin
   re-export index; `summaries/command.rs` owns `DebugDrawCommandKind` and
   `DebugDrawCommandSummary`, while `summaries/list.rs` owns `DebugDrawListSummary` aggregation.
   2026-05-27 debug-draw media summary projection owner split:
   `debug_draw_controls/commands/summary_projection/media.rs` now owns image/SVG/media command
   summary count and image-id assembly. `summary_projection.rs` keeps clip-stack tracking and
   non-media command projection.
   2026-05-14 source-gate follow-up: the IMUI workstream source gate now carries a reusable
   opaque-output-struct check for sealed response/context/summary records, so public output fields
   cannot return by simply changing field names.
   2026-05-14 editor axis-outcome follow-up: `VecEditAxisOutcome` and
   `TransformEditAxisOutcome` now keep section/axis/outcome storage private. Public proof code reads
   axis edit events through explicit accessors.
   2026-05-14 output-catalog gate follow-up: the IMUI workstream source gate now scans the
   `fret-imui`, `fret-ui-editor`, and `fret-ui-kit::imui` source roots for new public
   `*Response`/`*Outcome`/`*Summary`/`*Signal`/`*Record`/`*Context` structs and fails unless they
   are explicitly covered by the opaque-output-struct check.
   2026-05-14 editor color-event follow-up: `ColorEditPaletteSlotDrop`,
   `ColorEditEyedropperRequest`, and `ColorEditDragDropPayload` now follow the same accessor-first
   rule. The opaque-output catalog now covers `*Request`, `*Payload`, and `*Drop` records in the
   IMUI/editor scan roots.
   2026-05-14 state-catalog gate follow-up: the source gate now treats public `*State` structs as
   opaque IMUI structs too and registers `ImUiMultiSelectState`, so future shared state helpers
   cannot expose mutable storage by bypassing the one-off multi-select field checks.
2. Component surface catalog: keep the widget/component gap read source-backed before opening
   implementation follow-ons.
   Current readiness audit: `P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md`. Current coverage is broad
   enough for the active editor proof; list-box, plot, image-item, style-editor, advanced-table, and
   child-flag mirrors remain behavior-specific candidates, not a broad widget backlog.
   2026-05-16 follow-on result: `imui-image-item-proof-v1` closed after adding standalone
   response-bearing image item / image button authoring in `fret-ui-kit::imui`, using Fret
   `ImageId` and `ImageProps` without introducing Dear ImGui texture-ID runtime state or widening
   `fret-imui`.
   2026-05-16 selectable highlight result: `imui-selectable-highlight-policy-v1` closed after
   adding a Dear ImGui-style forced-highlight policy to `SelectableOptions` and splitting the
   input-text picker active-candidate visual from selected semantics.
   2026-05-16 floating posture refresh: `window(...)` source docs now describe the current
   in-window floating surface instead of deferring z-order/focus arbitration to a future work item.
   Current `fret-imui` floating tests already cover bring-to-front hit-test order,
   focus-on-click vs activation, no-inputs / pointer-pass-through policy, close, resize, and
   collapse; OS-window tear-out and multi-viewport parity stay in docking/runner lanes.
   2026-05-16 table gap wording refresh: the component catalog now distinguishes existing
   `TableOptions::striped` alternating row backgrounds from the explicit per-row/per-cell
   background override axis. Do not open a broad `imui-table-advanced-flags-v1` lane just because
   the old wording said "row background targets."
   2026-05-16 table background/text role follow-up: explicit per-row/per-cell background overrides
   now live in `TableRowOptions::background` / `TableCellOptions::background`, and
   `ImUiTableRow::cell_text(...)` routes through the shared `text_table_cell(...)` helper instead
   of bare paragraph text. At that point the still-open advanced-table axes were freeze panes,
   runtime visibility persistence, and old columns API shape; later follow-ups below narrow or close
   those axes.
   2026-05-16 table header text follow-up: sortable/plain table header labels now reuse
   `text_table_cell(...)` too, so header text follows the same single-line ellipsis role as body
   cells instead of falling back to default word wrapping.
   2026-05-17 table sort glyph follow-up: sortable header sort indicators now reuse
   `text_chrome_glyph(...)`, keeping table header labels on table-cell text while treating arrows
   as fixed chrome glyphs.
   2026-05-16 static table column visibility follow-up: `TableColumn::hidden()` and
   `TableColumn::with_visible(bool)` now provide a narrow author-declared visibility API. Hidden
   columns keep row cell submission in declared column order but skip header/body rendering and do
   not emit header responses. Runtime hideable-column policy is now covered by the
   `ImUiTableColumnVisibilityState` and header menu follow-ups below; persistence remains a
   separate follow-on.
   2026-05-17 runtime table column visibility follow-up: `ImUiTableColumnVisibilityState` now
   provides a narrow stable-id visibility override helper in `fret-ui-kit::imui`. It applies to
   `TableColumn` lists before render and reuses the existing hidden-column path; menu/popup policy
   is covered by the subsequent helpers below, while persistence, freeze panes, and old columns API
   shape were still separate follow-ons at that historical point. A `fret-imui` composition test
   proves the helper drives the existing table render path without moving the state policy into
   `fret-imui`.
   2026-05-17 table visibility menu-item follow-up: `table_column_visibility_menu_item(...)` now
   provides the checkbox menu-item bridge between `TableColumn` and
   `ImUiTableColumnVisibilityState`. It reuses existing menu item policy; automatic header
   context-menu popup wiring is now covered by the helper below, while persistence, freeze panes,
   and old columns API shape were still separate follow-ons at that historical point.
   2026-05-17 table visibility menu-items group follow-up:
   `table_column_visibility_menu_items(...)` now composes the repeated checkbox menu section for
   stable-id, human-labeled table columns and returns accessor-first per-column responses. It still
   leaves popup presentation to the helper below and keeps persistence, freeze panes, and old
   columns API shape outside `fret-imui`.
   2026-05-17 table header context-menu request follow-up: sortable and plain table header
   responses now report context-menu requests through a shared header trigger surface. Sortable
   headers keep their primary activation/click lifecycle; plain headers expose right-click,
   ContextMenu-key, and Shift+F10 requests without reporting left-click activation. That trigger
   signal is now consumed by the visibility menu helper below; persistence, freeze panes, and old
   columns API shape remain separate follow-ons.
   2026-05-17 table header visibility menu wiring follow-up:
   `table_column_visibility_header_context_menu(...)` now composes table header context-menu
   requests from both sortable and plain headers, popup placement, and
   `table_column_visibility_menu_items(...)` into a narrow accessor-first helper in
   `fret-ui-kit::imui`. `TableColumnVisibilityHeaderContextMenuOptions` exposes popup/menu policy
   knobs. Callers still own applying the visibility model to their column list, and `fret-imui`
   stays policy-light.
   2026-05-17 table visibility snapshot follow-up:
   `TableColumnVisibilitySnapshot` / `TableColumnVisibilityEntry` now provide the persistence seam
   for `ImUiTableColumnVisibilityState`. The snapshot stores only stable column ids plus visible
   flags, round-trips through serde, ignores empty ids on restore, and uses last-entry-wins for
   duplicate ids. Apps/editors still own storage, schema placement, and when to apply the restored
   state. Later entries below close the freeze-pane seam and old API-shape cleanup.
   2026-05-17 table column pinning follow-up: `TableColumn::pinned_left()` and
   `TableColumn::pinned_right()` now provide the narrow freeze-pane seam for IMUI tables. The
   render path splits visible cells into left/center/right groups and keeps left/right frozen
   outside the shared center horizontal scroll handle. This stays in `fret-ui-kit::imui`, reuses
   `fret-ui` scroll mechanics, and does not add a table-state runtime to `fret-imui`. The old
   columns API shape stayed as the separate cleanup closed below.
   2026-05-18 table column API-shape first pass: `TableColumn` now has accessor-first read methods
   for header, stable id, width, visibility, sortability, sort direction, resize options, and pin
   state. The IMUI table render path, visibility helper, `fret-imui` composition tests, and public
   smoke tests now read through those accessors instead of teaching direct field reads. This
   prepared the separate private-field cleanup below.
   2026-05-18 table column private-field cleanup: `TableColumn` fields are now private after the
   accessor-first audit found no in-repo struct-literal construction or direct public field reads.
   Public callers keep the builder/accessor surface, `fret-ui-kit::imui` keeps crate-local
   `header_arc(...)`, `id_arc(...)`, and `set_visible_for_policy(...)` seams for render/policy
   internals, and `tools/gate_imui_workstream_source.py` prevents the public field bag from
   returning. This closes the old columns API-shape table follow-on.
   2026-05-18 table gap current read: visibility snapshot/restore, freeze-pane pinning, and the old
   `TableColumn` public field-bag shape are no longer current table gaps. Broader storage/schema
   ownership and any full Dear ImGui-style mutable table runtime remain outside `fret-imui` until a
   separate proof needs them.
   2026-05-16 control readout text role follow-up: `text_control_readout(...)` now lives beside
   `text_table_cell(...)` in `fret-ui-kit::declarative::text`, and the UI Gallery code-editor
   toolbar readouts route through that shared role instead of carrying app-local text layout policy.
   Continue converging text into a small stable role set before adding more per-surface helpers.
   2026-05-16 button label text role follow-up: `text_button_label(...)` now owns compact
   single-line button-label text in `fret-ui-kit::declarative::text`, and IMUI control chrome uses
   it for button/pill labels instead of word-wrapping control text.
   2026-05-16 code block text role follow-up: `text_code_block(...)` now owns the shared
   monospace, horizontal-scroll-friendly code text role, and the UI Gallery docs scaffold no longer
   hand-rolls code-block `TextProps`.
   2026-05-16 paragraph text role follow-up: `text_paragraph(...)` and
   `text_paragraph_break_words(...)` now provide semantic aliases over the existing prose helpers,
   leaving `text_prose(...)` available for shadcn/Tailwind naming while giving Fret apps a stable
   paragraph role name.
   2026-05-17 compact paragraph text follow-up: `text_compact_paragraph(...)` now owns dense
   wrapping body copy for editor/IMUI panels, and both `bullet_text(...)` labels and
   `UiWriterImUiFacadeExt::text_wrapped(...)` route through it instead of local `TextProps`.
   2026-05-18 tooltip body text follow-up: `tooltip_text(...)` / `tooltip_text_with_options(...)`
   now route their default body copy through a crate-local `tooltip_body_text(...)` helper backed
   by `text_compact_paragraph(...)`, while rich-content `tooltip(...)` closures still let callers
   choose their own text role.
   2026-05-18 collection proof text-role follow-up: `imui_editor_proof_demo` collection titles,
   status/readout lines, asset metadata/path text, context-menu selection summaries, and drop
   status now route through proof-local helpers backed by shared section-chrome and control-readout
   roles. The inline rename explanatory copy explicitly uses `text_wrapped(...)`, keeping app-owned
   collection behavior intact while avoiding fixed-row bare text under resize.
   2026-05-16 trigger label reuse follow-up: IMUI tab triggers and menubar triggers now reuse
   `text_button_label(...)` because they are button-like trigger labels.
   2026-05-16 list row text role follow-up: `text_list_row_label(...)` now owns dense
   selectable/menu/tree row label text. IMUI menu items, selectables, and disclosure/tree rows use
   the shared role, giving them fill-width, min-width-zero, single-line ellipsis semantics without
   recasting them as button labels.
   2026-05-16 menu shortcut readout reuse follow-up: menu item shortcut labels now reuse
   `text_control_readout(...)` as muted compact auxiliary readouts instead of carrying local
   nowrap/clip `TextProps` policy.
   2026-05-20 menu item row-anchor follow-up: the root pressable now owns the visible row children
   instead of using an absolute overlay sibling, so the menu item's `test_id` and interaction box
   stay on the same row as the visible label/shortcut/glyph children.
   2026-05-17 menu indicator glyph follow-up: menu checkbox/radio indicators and submenu chevrons
   now reuse `text_chrome_glyph(...)`, so glyph-only menu chrome no longer falls back to bare
   `cx.text(...)` default wrapping semantics.
   2026-05-17 section chrome label text follow-up: `text_section_chrome_label(...)` now owns
   compact separator/section chrome labels in `fret-ui-kit::declarative::text`, and IMUI
   `separator_text` labels route through it instead of local default-wrapping `TextProps`.
   2026-05-17 chrome title text follow-up: `text_chrome_title(...)` now owns medium, fill-width
   floating window title-bar text, and floating window titles route through shared chrome text roles
   instead of local `TextProps`.
   2026-05-17 chrome glyph text follow-up: `text_chrome_glyph(...)` now owns compact fixed-slot
   chrome glyphs in `fret-ui-kit::declarative::text`, and disclosure/tree indicators route through
   it instead of keeping a local `TextProps` constructor.
   2026-05-17 floating close glyph follow-up: floating-window close button glyphs now reuse
   `text_chrome_glyph(...)` too, so fixed title-bar chrome no longer falls back to bare
   `cx.text(...)` default wrapping semantics.
   2026-05-18 text role source-gate hardening follow-up: `tools/gate_imui_workstream_source.py`
   rejects direct `TextProps` construction under `fret-ui-kit::imui`, including both
   `TextProps::new(...)` and struct-literal forms, so new compact IMUI text policy must route
   through the shared role vocabulary instead of gaining local text layout exceptions.
   2026-05-16 IMUI text item resize follow-up: `UiWriterImUiFacadeExt::text(...)` now follows Dear
   ImGui's default `Text()` posture by staying single-line, shrinkable, and ellipsis-truncated
   under narrow resize. `text_wrapped(...)` is the explicit opt-in path for explanatory copy that
   should wrap, and first-party proof prose now uses that API.
   2026-05-17 IMUI text item role cleanup: `UiWriterImUiFacadeExt::text(...)` now delegates to the
   shared `text_section_chrome_label(...)` role instead of hand-rolling `TextProps`, and the source
   gate no longer keeps an allowlist exception for IMUI default text.
   2026-05-16 control chrome fill-text follow-up: checkbox/radio/switch labels plus combo/slider
   captions now inherit the same compact single-line shrink/ellipsis posture through
   `control_chrome::fill_text(...)`, so fixed-height control chrome cannot grow rows by word
   wrapping under resize.
   2026-05-17 control label text follow-up: `text_control_label(...)` now owns compact control
   labels in `fret-ui-kit::declarative::text`, and `control_chrome::fill_text(...)` routes through
   it instead of owning local `TextProps` policy.
   2026-05-17 editor input value text follow-up: `fret-ui-editor` now shares
   `editor_input_value_text(...)` for drag-value and axis-drag-value scrub readouts. The helper
   keeps numeric value text fill-width, `min-width: 0`, shrinkable, single-line, and ellipsis
   clipped without moving editor-specific density/chrome policy into `fret-imui`.
   2026-05-19 editor input-group text ownership follow-up: `primitives/input_group.rs` no longer
   owns direct `TextProps` construction for text segments, value text, or axis markers. Those
   policies now live in `primitives/readout.rs` as editor text role helpers, and the source gate
   removed `input_group.rs` from the direct editor `TextProps` allowlist.
   2026-05-17 editor status badge text follow-up: `FieldStatusBadge` now routes its compact label
   through `editor_status_badge_text_props(...)`, keeping badge text in the editor readout
   primitive layer instead of hand-rolling `TextProps` inside the control.
   2026-05-17 editor inline error text follow-up: `ColorEdit` and its numeric popup now share
   `editor_inline_error_text_props(...)` for compact single-line destructive readouts, leaving
   wrapping validation prose to controls that explicitly need multi-line errors.
   2026-05-17 editor validation message text follow-up: `NumericInput` inline validation messages
   now route through `editor_validation_message_text_props(...)` instead of owning local
   `TextProps`. The role is the explicit wrapping error/prose path for editor controls, while the
   IMUI workstream source gate now allowlists direct editor `TextProps` construction only inside
   editor text primitive owners.
   2026-05-17 transform label text follow-up: `TransformEdit` now routes section badges, section
   headings, and inline link/uniform checkbox labels through shared editor readout helpers instead
   of owning local `TextProps` policy. The helpers keep these compact control labels single-line
   and shrinkable under resize without moving editor-specific density policy into `fret-imui`.
   2026-05-17 popup list text follow-up: enum-select and text-assist popup rows plus empty-state
   labels now share `editor_popup_list_row_text_props(...)` /
   `editor_popup_empty_text_props(...)`, replacing `TextProps::new(...)` default word wrapping in
   popup/editor assist surfaces. Color-edit copy menu rows and popup option captions now reuse the
   same popup-list family through start-aligned, centered-row, and fixed-caption variants.
   2026-05-19 popup list text ownership follow-up: the popup-list text props family moved from
   `primitives/popup_list.rs` into the shared editor text role owner in `primitives/readout.rs`.
   `popup_list.rs` now owns only popup-list state, dimensions, and palette policy, and the source
   gate forbids direct text layout/wrap policy from returning there.
   2026-05-17 color preview/tooltip text follow-up: color side-preview captions and tooltip
   numeric lines now use dedicated editor readout helpers instead of local `TextProps`, keeping
   them separate from popup-list rows while preserving single-line resize behavior.
   2026-05-17 property chrome text follow-up: property-group header labels, property-row reset
   glyphs, and inspector panel titles now share editor readout helpers instead of local/default
   text policy. Fixed inspector chrome text stays single-line, min-width-zero where it shrinks, and
   line-height constrained under resize without moving editor-specific policy into `fret-imui`.
   The inspector title path also has a narrow-header layout gate with toolbar siblings, so panel
   titles cannot drift back to default word wrapping.
   2026-05-17 property-row label text follow-up: `editor_property_row_label_text_props(...)` now
   owns fixed inspector label text, and eager/virtualized property-grid row contexts expose
   `label_text(...)` as the preferred first-party label path. `PropertyRow` label slots also clamp
   their own line boxes to the editor row height, so accidental bare/default label text cannot
   reintroduce row growth under resize.
   2026-05-17 proof teaching follow-up: `imui_editor_proof_demo` property-grid labels now use
   `row_cx.label_text(...)`, and the IMUI workstream source gate forbids representative proof
   labels from drifting back to bare `|cx| cx.text(...)` label slots.
   2026-05-19 editor proof main text follow-up: `imui_editor_proof_demo` main headline, mode hint,
   authoring parity explanatory copy, shared-state hint, and editor section label now route through
   proof-local IMUI helpers backed by shared section-chrome, readout, and compact paragraph roles
   instead of local `fret_ui_kit::ui::text(...).text_xs()` / `.font_semibold()` styling.
   2026-05-17 workspace shell proof text follow-up: `workspace_shell_demo` editor-rail buttons,
   property labels, and compact property values now teach the shared button-label, property-label,
   and control-readout text roles instead of bare `cx.text(...)`.
   2026-05-19 workspace shell paragraph text follow-up: the remaining editor-rail header copy now
   uses a local helper backed by shared `text_paragraph(...)` instead of local
   `fret_ui_kit::ui::text(...).text_sm().text_color(...)` styling.
   2026-05-17 editor notes proof text follow-up: `editor_notes_demo` inspector metadata labels,
   subtitle, and compact status values now teach the same property-label and control-readout roles
   instead of relying on bare `cx.text(...)` inside fixed property rows.
   2026-05-19 editor notes center/collection text follow-up: collection summary/readouts and
   center preview chrome/prose now route through local helpers backed by shared readout, section,
   and paragraph roles instead of local `ui::text(...).wrap(...)` styling.
   2026-05-19 editor notes device shell text follow-up: compact mobile header title/body copy now
   use device-shell local helpers backed by shared section-chrome and paragraph roles instead of
   local `ui::text(...).font_semibold()` / `.text_color(...).wrap(...)` styling.
   2026-05-17 workspace shell prompt text follow-up: the dirty-close prompt title/details now
   reuse the shared section-chrome and control-readout text roles instead of bare dialog text.
   2026-05-17 editor proof readout follow-up: `imui_editor_proof_demo` now routes selected
   empty-state labels, authoring shared-state readouts, and the declarative gradient section label
   through proof-local helpers backed by shared text roles instead of bare `cx.text(...)`.
   2026-05-17 drag-preview text follow-up: proof drag-preview cards now render title and subtitle
   as separate shared text-role elements instead of a newline-joined bare text blob.
   2026-05-17 gradient editor empty-state follow-up: `GradientEditor` now routes its `No stops`
   empty-state label through an editor readout primitive instead of bare/default text, keeping
   compact inspector empty states single-line and shrinkable under resize.
   2026-05-17 text role matrix follow-up: `P3_TEXT_ROLE_MATRIX_2026-05-17.md` now freezes the
   stable base role vocabulary for resize triage: control readout, button label, paragraph, code
   text, and table cell text. The matrix also classifies current derived roles, keeps wrapping text
   out of fixed-height control rows unless parents measure multi-line height, and explicitly
   rejects adding a public `TextRole` enum until two consumers need a data-driven role value.
   2026-05-17 shared text-role layout gate follow-up: the base shared text roles now have a real
   narrow-layout regression proving single-line roles stay one measured line while paragraph text
   measures multi-line height through `UiTree::layout_all(...)`.
   2026-05-17 property-row value overflow follow-up: `PropertyRow` value slots no longer force
   `Overflow::Clip`, so explicit wrapping validation/prose children such as `NumericInput` inline
   validation messages can grow with their measured multi-line height instead of painting past a
   clipped inspector row. Fixed label/reset/action chrome slots still clip themselves; this is a
   layout-container contract fix in `fret-ui-editor`, not a `fret-imui` runtime/API widening.
   2026-05-17 property-row wrapping layout follow-up: the value-slot fix now has a real layout
   gate, not only a structural overflow check. A narrow row with the editor validation-message role
   runs through `UiTree::layout_all(...)`, and public element-bounds queries prove the multi-line
   validation text is contained by the value slot and row bounds.
   2026-05-17 property-grid wrapping layout follow-up: `PropertyGrid` now has a composition-level gate
   with mixed single-line and wrapping rows. The test proves a multi-line validation
   value grows its row and pushes the following row down, so the fix is locked at the realistic
   inspector-grid composition layer instead of only the single-row container layer.
   2026-05-17 generic list text follow-up: `list_from_strings(...)`, the compatibility string-list
   helper over retained virtual lists, now routes leading glyphs, row labels, and trailing shortcuts
   through `text_chrome_glyph(...)`, `text_list_row_label(...)`, and
   `text_control_readout(...)` instead of bare `cx.text(...)`. The helper remains a generic
   declarative kit convenience, not an IMUI runtime/list-box widening.
   2026-05-17 generic tree text follow-up: the default retained tree row renderer now routes row
   labels through `text_list_row_label(...)` and toggle glyphs through `text_chrome_glyph(...)`
   instead of keeping a local truncate/text path. Custom tree row renderers still own their own
   content policy.
   2026-05-17 file tree text follow-up: `file_tree_view_retained_v0(...)`, the retained workspace
   file-tree surface, now routes fixed row icons through `text_chrome_glyph(...)` and file labels
   through `text_list_row_label(...)` instead of hand-rolling text truncation inside fixed rows.
   2026-05-17 retained table text follow-up: retained table headers plus grouped row label and
   aggregation text now route through `text_table_cell(...)`, keeping fixed table cells on the
   shared single-line ellipsis role outside the IMUI-specific table wrapper.
   2026-05-17 examples table proof text follow-up: `table_demo` and `table_stress_demo` now route
   fixed header/status readouts through `text_control_readout(...)` and fixed table header/body
   cells through `text_table_cell(...)`, so the runnable table proof surfaces no longer teach bare
   default wrapping text inside fixed table rows.
   2026-05-17 datatable proof text follow-up: `datatable_demo` now routes its compact selected/sort
   status line through `text_control_readout(...)` and its DataTable body cells through
   `text_table_cell(...)`, keeping the shadcn data-table proof aligned with the same fixed-row
   text semantics as the lower-level table demos.
   2026-05-17 virtual-list stress proof text follow-up: `virtual_list_stress_demo` now routes its
   compact scroll/state header through `text_control_readout(...)` and visible fixed-row labels
   through `text_list_row_label(...)`, so the runnable virtual-list stress surface no longer
   teaches bare default wrapping text inside fixed-height rows.
   2026-05-17 canvas datagrid stress proof text follow-up: `canvas_datagrid_stress_demo` now routes
   its compact canvas grid stats header through `text_control_readout(...)`, so the retained-canvas
   grid stress proof no longer teaches bare default wrapping text above the fixed grid slot.
   2026-05-17 date picker proof text follow-up: `date_picker_demo` now routes its compact
   open/selected/month status through `text_control_readout(...)`, switch captions through
   `text_control_label(...)`, and keyboard instructions through `text_paragraph(...)`, keeping the
   proof aligned with the fixed-chrome-vs-prose text role split.
   2026-05-17 form proof text follow-up: `form_demo` now routes its compact header
   submit/valid/dirty/status readout through `text_control_readout(...)`, keeping the runnable
   shadcn form proof from teaching bare default wrapping text inside fixed header chrome.
   2026-05-17 sonner proof text follow-up: `sonner_demo` now routes its fixed demo title through
   `text_section_chrome_label(...)` and promise/last-action status through
   `text_control_readout(...)`, keeping toast proof chrome single-line and shrinkable under
   resize.
   2026-05-17 echarts proof text follow-up: `echarts_demo` now routes its chart titles through
   `text_section_chrome_label(...)`, keeping chart scaffold labels on the shared single-line
   chrome role instead of bare default text.
   2026-05-17 components gallery table proof text follow-up: `components_gallery` now routes the
   retained table torture cell renderer through `text_table_cell(...)` and its explanatory table
   header through `text_paragraph(...)`. The same proof now routes its top chrome title, tree
   status, theme controls, color swatch labels, and control state readouts through the matching
   shared text roles. Overlay body copy now uses paragraph text, while the overlay last-action
   status uses `text_control_readout(...)`, keeping the runnable gallery proof from teaching bare
   default wrapping text in fixed cells, fixed control chrome, or overlay proof copy.
   2026-05-17 markdown proof chrome text follow-up: `markdown_demo` now routes its fixed demo
   title through `text_section_chrome_label(...)`, its explanatory preview copy through
   `text_paragraph(...)`, and its toolbar/readout state through `text_control_readout(...)`.
   2026-05-19 markdown image placeholder text follow-up: Markdown body rendering stays owned by
   the Markdown surface, but image placeholder copy now uses `text_paragraph_break_words(...)`
   with app-owned muted foreground instead of local `TextProps`, so long image URLs can break under
   narrow resize without keeping `markdown_demo` in the direct-text residual allowlist.
   2026-05-17 residual bare text capability follow-up: the remaining first-party
   `apps/fret-examples/src` bare `cx.text(...)` / `TextProps::new(...)` paths are now source-gated
   to explicit text/IME/rendering capability proofs only: `components_gallery` text smoke/font
   probes, `ime_smoke_demo` IME behavior instructions/status, and the explicit rendering-capability
   demos that still use direct `TextProps { ... }` constructors. New bare text in runnable proof
   apps must now either use a shared role or update the residual proof gate with a documented
   reason.
   2026-05-17 gallery retained-table torture text follow-up: the UI Gallery retained-table torture
   page now routes fixed cell text through `text_table_cell(...)` and table state/status readouts
   through `control_readout_text(...)`, leaving only explicitly multi-line/paragraph copy as bare
   prose.
   2026-05-17 gallery data-table torture text follow-up: the UI Gallery DataTable torture page now
   routes both retained and non-retained fixed cell renderers through `text_table_cell(...)`, and
   table sorting/filter/pinning status lines through `control_readout_text(...)`.
   2026-05-18 data-table snippet table-cell text follow-up: the app-facing copyable DataTable
   snippets now share a directory-local `table_cell_text(...)` helper backed by
   `text_table_cell(...)`. Default/basic/guide/RTL/reusable-component fixed cell text and fallback
   cells no longer teach bare `cx.text(...)`; amount columns keep their existing tabular numeric
   formatting until a dedicated numeric-cell role is designed.
   2026-05-19 table snippet table-cell text follow-up: ordinary copyable Table snippets now share
   directory-local `table_cell_text(...)` / `table_cell_text_emphasis(...)` helpers backed by shared
   kit roles. Demo/Usage/Footer/RTL/Actions and fixed body cells in Children no longer teach bare
   `ui::text(...)` inside table cells. The later children-API follow-up below routes the remaining
   rich header/caption sample text through table-cell and paragraph roles too.
   2026-05-19 table children custom text follow-up: the explicit `table_head_children(...)` /
   `table_caption_children(...)` sample now routes header child text through the shared
   table-cell helper and caption copy through `text_paragraph(...)`, closing the previous
   children-API bare text exception without changing shadcn table recipes.
   2026-05-19 checkbox table-cell text follow-up: the checkbox table snippet now keeps its
   action-first select-all surface while routing member/role fixed cells through a local helper
   backed by `text_table_cell(...)`, so checkbox-table examples no longer teach bare `ui::text(id)`
   / `ui::text(role)` inside fixed table rows.
   2026-05-19 typography table-cell text follow-up: standalone, demo, and RTL Typography table
   samples now share a typography-local `table_cell_text(...)` helper backed by
   `text_table_cell(...)`, keeping typography prose/rich-link examples unchanged while removing
   bare table-cell `ui::text(...)` from the fixed row samples.
   2026-05-18 AI AudioPlayer state-marker follow-up: copyable AudioPlayer snippets now expose
   state-only diagnostics anchors through zero-size `SpacerProps` children under generic semantics
   instead of empty `Text` nodes. This keeps test markers out of visible text layout semantics while
   preserving stable diagnostics `test_id`s.
   2026-05-18 AI visible text-role follow-up: Message and Terminal copyable snippets now route
   fixed titles through `text_section_chrome_label(...)`, explanatory prose through
   `text_paragraph(...)`, and compact Message action status through `text_control_readout(...)`.
   The Terminal empty-output marker also uses the same non-text spacer-marker pattern.
   2026-05-19 AI Terminal title text-role follow-up: the `fret-ui-ai` `TerminalTitle` component
   itself now routes its default label through `text_chrome_title(...)` instead of local
   `ui::raw_text(...).wrap(None).overflow(Clip)` policy, so actual terminal chrome titles inherit
   fill-width shrink/ellipsis behavior under resize.
   2026-05-19 AI EnvironmentVariables title text-role follow-up:
   `EnvironmentVariablesTitle` default/text paths now route through the shared medium
   `text_chrome_title(...)` role instead of local raw-text title policy. Custom title children keep
   the component-owned inherited title refinement because upstream accepts children.
   2026-05-19 AI EnvironmentVariables code-label follow-up: environment variable names and
   non-selectable masked/custom values now use the shared `text_code_label(...)` fixed identifier
   role with inherited foreground. Revealed values intentionally remain selectable text because
   selection is the capability surface. Empty custom-child/diagnostic paths now use the crate-local
   non-text placeholder instead of empty `Text` nodes.
   2026-05-19 AI PackageInfo code/paragraph follow-up: the real `PackageInfo` component family now
   uses `text_code_label_emphasis(...)` for primary package/target-version identifiers,
   `text_code_label(...)` for current-version/dependency identifiers,
   `text_section_chrome_label(...)` for the Dependencies heading, and
   `text_compact_paragraph_inherited(...)` for description copy that should keep component-owned
   description typography while sharing the wrapping/fill-width resize contract.
   2026-05-19 AI Agent text-role/accordion-boundary follow-up: the real `Agent` component family
   now routes header names through `text_chrome_title(...)`, section labels through
   `text_section_chrome_label(...)`, instruction body copy through `text_compact_paragraph(...)`,
   and tool trigger descriptions through `text_list_row_label(...)`. `AccordionTrigger` now keeps
   shared text-role typography/wrap/overflow metadata when a caller supplies a role child, while
   preserving its legacy wrapping defaults for bare trigger text.
   2026-05-18 AI visible text-role follow-up 2: Artifact, CodeBlock, and Sandbox copyable snippets
   now use the same role split for fixed demo titles, explanatory prose, and compact status text.
   CodeBlock's active-language diagnostics marker also moved from an invisible empty text node to a
   zero-size generic spacer marker.
   2026-05-18 AI Queue text-role follow-up: the Queue copyable snippet now routes its fixed demo
   title through `text_section_chrome_label(...)`, explanatory copy through `text_paragraph(...)`,
   and action-revision diagnostics through a generic zero-size spacer marker instead of empty or
   invisible text.
   2026-05-18 AI Checkpoint text-role follow-up: the Checkpoint copyable snippet now uses
   paragraph text for conversation/prose, control-readout text for restore status, button-label
   text for the checkpoint trigger, and chrome-glyph text for custom checkpoint icon glyphs.
   2026-05-18 AI simple chrome text-role follow-up: Agent, CodeBlock usage, Environment Variables,
   and OpenIn copyable snippets now route fixed demo titles through `text_section_chrome_label(...)`
   and explanatory body copy through `text_paragraph(...)`.
   2026-05-18 AI selector/branch marker follow-up: MessageBranch, MicSelector, and ModelSelector
   snippets now expose state-only diagnostics anchors through generic zero-size spacer markers
   instead of empty `Text`, while their fixed demo titles/body copy use shared text roles.
   2026-05-18 AI prompt/plan/commit-large text-role follow-up: CommitLarge now routes its
   opened-file diagnostics marker through a generic zero-size spacer anchor instead of empty
   `Text`, and its fixed title/body copy uses shared section-chrome/paragraph roles. Plan,
   PromptInputActionMenu, and PromptInputTooltip now put their outer demo titles/body copy on the
   same shared roles while leaving inner plan prose and shadcn/Button composition for a separate
   semantics pass.
   2026-05-19 AI PlanContent text-role follow-up: Plan's inner section headings now use
   section-chrome text, body copy uses paragraph text, bullet rows use list-row labels, and the
   custom Build button child uses button-label text. Plan streaming/open behavior and the
   `fret-ui-ai` Plan component surface remain unchanged.
   2026-05-18 AI large/status snippet text-role follow-up: StackTraceLarge and TestResultsLarge
   now use generic zero-size spacer markers for opened/activated diagnostics anchors, while
   StackTraceLarge, TestResultsLarge, Tool, and Suggestions route fixed outer title/body text
   through shared section-chrome/paragraph roles. Tool's fixed state-section labels also use
   section-chrome text; Suggestions custom-children content stays app-owned for a later custom
   content semantics pass.
   2026-05-18 AI queue-prompt/transcription text-role follow-up: QueuePromptInput now routes its
   sent-count diagnostics marker through a generic zero-size spacer marker, its custom Search
   button child through `text_button_label(...)`, and fixed outer title/body copy through shared
   section-chrome/paragraph roles. Transcription now uses generic zero-size spacer markers for
   time/active diagnostics anchors while keeping fixed title/body copy on shared roles.
   2026-05-18 AI WebPreview text-role follow-up: WebPreview state diagnostics now use generic
   zero-size spacer markers instead of empty `Text`, navigation glyphs use `text_chrome_glyph(...)`,
   and composable child fixed body/footer copy uses shared section-chrome/paragraph roles.
   2026-05-18 AI Chat text-role follow-up: Chat's prompt-nonempty diagnostics marker now uses a
   generic zero-size spacer, empty marker fallbacks use spacers instead of empty `Text`, fixed
   header instructions use paragraph roles, and exported markdown length uses control-readout text.
   Chat message body rendering remains app/content-owned for a separate semantics pass.
   2026-05-18 AI PromptInput provider/docs text-role follow-up: PromptInputProvider now routes its
   sent-count diagnostics marker through a generic zero-size spacer, its custom external-add label
   through `text_button_label(...)`, and fixed outer title/body copy through shared roles.
   PromptInput docs now routes the custom Search label through button-label text and fixed outer
   title/body copy through section-chrome/paragraph roles.
   2026-05-19 AI PromptInput cursor custom-text follow-up: Cursor-style PromptInput command rows,
   file/path labels, rules hover-card text, tabs footer copy, and custom trigger counts now use
   shared list-row/code/readout/button text roles instead of local `ui::text(...)` styling.
   2026-05-18 AI chrome/readout text-role follow-up: Reasoning, StackTrace, and VoiceSelector now
   route fixed outer title/body copy through shared section-chrome/paragraph roles; StackTrace and
   VoiceSelector compact status/diagnostics readouts use `text_control_readout(...)` instead of
   default wrapping text.
   2026-05-18 AI Confirmation content text-role follow-up: Confirmation request/body snippets now
   route prose through `text_paragraph(...)`, inline/code payloads through `text_code_wrap(...)`,
   approval/rejection result text through `text_control_readout(...)`, and the demo's fixed outer
   title/body copy through shared section-chrome/paragraph roles.
   2026-05-18 AI Task content text-role follow-up: Task item labels now route through
   `text_list_row_label(...)`, attached file names through `text_code_wrap(...)`, and the demo's
   fixed outer title/body copy through shared section-chrome/paragraph roles.
   2026-05-18 AI Conversation instrumentation text-role follow-up: Conversation length/export
   diagnostics now use generic semantics instead of text semantics, and the custom scroll-bottom
   label routes through `text_button_label(...)`.
   2026-05-18 AI usage snippet text-role follow-up: Attachments usage explanatory copy now uses
   paragraph text, and StackTrace usage title/body copy uses section-chrome/paragraph roles.
   2026-05-18 AI Message usage text-role follow-up: Message usage now routes user message text
   through paragraph text, the last-action marker through control-readout text, and fixed outer
   title/body copy through section-chrome/paragraph roles.
   2026-05-18 AI Canvas world spike text-role follow-up: the canvas spike now routes visible
   chrome, node copy, and debug/status readouts through shared section-chrome, paragraph, and
   control-readout roles instead of bare `cx.text(...)`.
   2026-05-18 AI Image demo text-role follow-up: the image demo now routes fixed explanatory copy
   through paragraph text and status/loading readouts through shared control-readout text instead of
   bare `cx.text(...)`.
   2026-05-18 AI PromptInput referenced sources text-role follow-up: fixed referenced-sources
   title/body copy now routes through section-chrome and paragraph roles instead of bare
   `cx.text(...)`.
   2026-05-19 AI Attachments inline hover-card text-role follow-up: app-owned hover-card
   attachment labels now use `text_list_row_label(...)`, and media-type readouts use
   `text_control_readout(...)` instead of default `ui::text(...)` builders. Attachment chip,
   remove affordance, and hover-card behavior remain owned by `fret-ui-ai`.
   2026-05-18 AI Artifact code display status-marker follow-up: the docs status marker now keeps
   the diagnostic `label_contains` contract on a generic semantics marker with a zero-size spacer
   instead of an invisible bare text element.
   2026-05-18 AI ChainOfThought composable text-role follow-up: composed header, step-label, and
   description child text now uses shared section-chrome and paragraph roles instead of bare
   `cx.text(...)`.
   2026-05-18 AI TestResults composable text-role follow-up: custom summary/progress/status/name
   and duration child text now uses shared readout/list-row roles instead of bare `cx.text(...)`.
   2026-05-18 AI Workflow snippet text-role follow-up: workflow fixed chrome, panel copy,
   node-content sample copy, footer labels, and click readouts now use shared text roles instead of
   bare `cx.text(...)`.
   2026-05-18 AI Suggestions/reasoning/transcript text-role follow-up: suggestions custom
   children, reasoning hook status, transcript torture header copy, and chat exported-status marker
   now use shared text roles or generic marker semantics instead of bare/default text surfaces.
   2026-05-19 AI Shimmer demo chrome text follow-up: Shimmer typography/duration/elements demo
   labels and inline non-shimmer text now use shared readout/section roles instead of local
   `ui::text(...)` styling, while `Shimmer::new(...)` remains the explicit shimmer text capability
   surface.
   2026-05-18 AI custom-children text-role follow-up: environment variables, package info,
   inline citations, persona, and sources custom-child snippets now route visible app-owned text
   through shared roles, including the new single-line `text_code_label(...)` identifier role.
   2026-05-17 gallery data-grid text follow-up: the UI Gallery DataGrid preview now routes
   virtualized grid cell text through `text_table_cell(...)` and the selected-row readout through
   `control_readout_text(...)`.
   2026-05-17 gallery data paragraph text follow-up: DataGrid/DataTable/Tree Torture explanatory
   header copy now routes through `paragraph_text(...)`, backed by shared `text_paragraph(...)`,
   instead of default `cx.text(...)`.
   2026-05-17 gallery inspector torture text follow-up: the UI Gallery Inspector Torture page now
   routes fixed virtual-row property labels through `text_list_row_label(...)` and fixed row values
   through `control_readout_text(...)`.
   2026-05-17 gallery virtual-list torture text follow-up: UI Gallery virtual-list harness row
   labels now use `text_list_row_label(...)`, row detail/editing readouts use
   `control_readout_text(...)`, and the UI Kit list torture custom row renderer also routes item
   labels through `text_list_row_label(...)`.
   2026-05-17 gallery harness header text follow-up: retained-table, hit-test, UI Kit list,
   virtual-list, and view-cache harness headers now route explanatory copy through
   `paragraph_text(...)` and mode/status lines through `control_readout_text(...)` instead of bare
   `cx.text(...)`.
   2026-05-17 gallery view-cache list text follow-up: the UI Gallery View Cache torture page now
   routes its cached inner virtual-list row labels through `text_list_row_label(...)`.
   2026-05-17 gallery view-cache control-label follow-up: fixed switch labels now route through
   `control_label_text(...)` instead of bare `cx.text(...)`.
   2026-05-17 gallery view-cache popover body follow-up: the cached Popover body copy now routes
   through `paragraph_text(...)` instead of bare `cx.text(...)`.
   2026-05-17 gallery tree torture status text follow-up: the UI Gallery Tree Torture dynamic
   target status now routes through `control_readout_text(...)` instead of carrying local
   muted/text-sm styling.
   2026-05-17 gallery overlay status text follow-up: overlay and menu last-action/status flags now
   route through `control_readout_text(...)` instead of bare `cx.text(...)`.
   2026-05-17 gallery overlay scroll-row text follow-up: dialog/sheet/portal scroll filler rows
   now route through `text_list_row_label(...)` instead of bare `cx.text(...)`.
   2026-05-17 gallery overlay body prose follow-up: HoverCard and Popover body copy now route
   through `paragraph_text(...)` instead of bare `cx.text(...)`.
   2026-05-17 gallery chrome torture control-label follow-up: the fixed text-input/textarea
   labels now route through `control_label_text(...)`, backed by shared `text_control_label(...)`,
   instead of bare `cx.text(...)`.
   2026-05-17 virtual row fallback follow-up: tree and file-tree virtualizer out-of-range fallback
   paths now use spacer placeholders instead of empty text nodes, removing another fixed-row
   `Text` escape hatch.
   2026-05-17 gallery disabled toaster placeholder follow-up: the UI Gallery disabled toaster
   driver path now uses a spacer placeholder instead of `cx.text("")`, so app-shell disabled
   placeholder plumbing no longer creates meaningless text nodes.
   2026-05-17 gallery app-sidebar collapsed placeholder follow-up: the copyable app-sidebar snippet
   now uses a spacer placeholder for collapsed project groups instead of teaching `cx.text("")` as
   a layout placeholder.
   2026-05-17 fret-ui-ai empty placeholder follow-up: hidden/missing-content AI element fallbacks
   now use a crate-local spacer helper instead of returning empty text nodes, without widening
   `fret-imui`.
   2026-05-17 gallery status-bar readout follow-up: UI Gallery status-bar metrics,
   inspector-state, and last-action text now route through
   `driver::text_roles::chrome_readout_text(...)`, backed by `text_control_readout(...)` instead
   of bare `cx.text(...)`, keeping fixed status chrome single-line and shrinkable under resize.
   2026-05-17 gallery driver chrome text follow-up: UI Gallery driver chrome now has a small
   `driver::text_roles` owner. Disabled tabs/sidebar/content placeholders use the shared
   control-readout role, and settings-sheet section labels use the shared section-chrome role
   instead of bare `cx.text(...)`.
   2026-05-17 gallery driver chrome label follow-up: the nav title now uses section-chrome text,
   and settings-sheet switch captions use control-label text through `driver::text_roles` instead
   of local `TextProps` policy.
   2026-05-17 gallery minimal-root text follow-up: the `BISECT_MINIMAL_ROOT` diagnostic root now
   routes its placeholder readout through `driver::text_roles::chrome_readout_text(...)` instead
   of bare `cx.text(...)`, so resize bisect surfaces do not teach default wrapping text.
   2026-05-17 gallery debug-HUD text follow-up: fixed-size debug HUD lines now route through
   `driver::text_roles::chrome_readout_text(...)` instead of local word-wrapping `TextProps`, so
   long metric/readout lines truncate rather than growing HUD row height under resize.
   2026-05-17 gallery shell content/nav text follow-up: the page header title/source and sidebar
   group headings now route through shared chrome/readout roles instead of local `TextProps`,
   keeping app-shell text policy centralized while staying outside `fret-imui`.
   2026-05-18 gallery sidebar snippet chrome text follow-up: the copyable Sidebar examples now
   route card body prose and missing-content fallbacks through paragraph roles, and debug/status
   lines through `text_control_readout(...)` instead of bare `cx.text(...)`.
   2026-05-18 gallery command snippet chrome text follow-up: copyable Command examples now route
   last-action/count/active-value status through `text_control_readout(...)`, short subsection
   headings through section-chrome text, and desktop-only/prose copy through paragraph text. The
   retained active-descendant command snippet remains an explicit text-input capability surface.
   2026-05-18 gallery Accordion trigger text follow-up: copyable Accordion examples now route
   trigger labels through `text_button_label(...)` instead of bare `cx.text(...)`, keeping those
   button-like trigger rows single-line/shrinkable under resize while leaving shadcn component
   internals and upstream parity tests untouched.
   2026-05-18 gallery ToggleGroup item text follow-up: copyable ToggleGroup examples now route
   ordinary item captions through `text_button_label(...)` instead of bare `cx.text(...)` /
   `ui::text(...)`, keeping toggle buttons single-line/shrinkable under resize. The custom
   weight-card snippet remains an explicitly styled visual sample rather than a default text-role
   migration target.
   2026-05-18 gallery Toggle item text follow-up: copyable Toggle examples now route ordinary
   toggle captions through `text_button_label(...)`, and the label-association pressed-state
   marker through `text_control_readout(...)`, so these fixed button-like snippets no longer teach
   wrapping text inside toggle chrome.
   2026-05-18 gallery Button children text follow-up: the copyable Button children snippet now
   routes its custom command button label through `text_button_label(...)` instead of
   `cx.text(...)`, keeping slotted button children on the same single-line/shrinkable role as other
   button-like chrome.
   2026-05-18 gallery Tabs custom text follow-up: copyable Tabs snippets now route custom
   icon+label trigger text through `text_button_label(...)` and usage-panel body copy through
   `text_paragraph(...)`, while leaving built-in `TabsItem` / `TabsTrigger` label handling inside
   shadcn recipes.
   2026-05-18 gallery Collapsible text follow-up: copyable Collapsible snippets now route trigger
   captions through button-label text, controlled-state status through control-readout text,
   body copy through paragraph text, repository identifiers through code-label text, and file-tree
   row labels through list-row text.
   2026-05-18 gallery AlertDialog custom text follow-up: rich-content body copy now uses
   paragraph text, rich-content action children use button-label text, and small/RTL custom title
   and description children use section-chrome/paragraph roles instead of local wrap/overflow
   policy. Rich attributed-title text remains an explicit text capability surface.
   2026-05-18 gallery HoverCard text follow-up: copyable HoverCard snippets now route app-owned
   title/chrome labels through section-chrome text, body copy through paragraph/break-words roles,
   date/status copy through control-readout text, and the usage trigger label through button-label
   text instead of raw/default text builders.
   2026-05-18 gallery Popover align text follow-up: the fixed align preview body labels now route
   through shared paragraph text instead of `cx.text(...)`, keeping the overlay content examples on
   the same resize-aware text-role vocabulary.
   2026-05-18 gallery Tooltip keyboard shortcut text follow-up: the custom keyboard-shortcut
   tooltip label now routes through shared control-readout text instead of bare `cx.text(...)`,
   while built-in `TooltipContent::text(...)` recipe paths stay recipe-owned.
   2026-05-19 gallery Kbd custom-copy text follow-up: copyable Kbd snippets now route key
   separators through shared chrome-glyph text and tooltip/inline helper copy through
   control-readout text instead of local `ui::text(...).text_sm()` policy. `Kbd` keycap text
   itself remains recipe-owned by `fret-ui-shadcn`.
   2026-05-19 gallery Separator menu text follow-up: the responsive separator menu helper now
   routes section titles through `text_section_chrome_label(...)` and descriptions through
   `text_control_readout(...)` instead of local `Theme`/`fixed_line_box_px` text policy, while
   keeping Separator itself a leaf primitive.
   2026-05-19 gallery Item slotted text follow-up: app-owned Item dropdown trigger copy now uses
   `text_button_label(...)`, the download header uses `text_section_chrome_label(...)`, and issue
   number side columns use `text_control_readout(...)`. Built-in `ItemTitle` /
   `ItemDescription` text stays recipe-owned.
   2026-05-19 gallery Spinner amount readout follow-up: Spinner item amount/status values in the
   demo and RTL snippets now use `text_control_readout(...)` instead of local `ui::text(...)` /
   `cx.text(...)` builders.
   2026-05-19 gallery AvatarStack direction label follow-up: Shadcn Extras avatar-stack LTR/RTL
   direction labels now use `text_section_chrome_label(...)` instead of local
   `ui::text(...).font_medium()` builders. Announcement title copy remained a separate candidate
   until the recipe-owner decision below.
   2026-05-19 gallery Kanban card title follow-up: Shadcn Extras Kanban app-owned card title slots
   now use `text_button_label(...)` instead of local `ui::text(item.name).font_medium().truncate()`
   builders. Announcement title copy remained a separate candidate because it is passed into a raw
   extras title component rather than rendered by the caller's card slot.
   2026-05-19 Shadcn Extras AnnouncementTitle follow-up: `AnnouncementTitle` stays a children-first
   composable title surface, matching the upstream Kibo source shape, but `fret-ui-shadcn` now owns
   the title row contract: `text-sm` medium inherited typography, shrinkable/min-width-zero layout,
   single-line ellipsis for nested text, and clipped title containers. The gallery snippet keeps
   `AnnouncementTitle::new([cx.text(...)])` intentionally, and source gates prevent both the
   component contract and the caller-side composable surface from drifting.
   2026-05-19 gallery Dialog scroll-row text follow-up: scrollable-content and sticky-footer
   filler rows now route through shared list-row label text instead of `ui::raw_text(format!(...))`,
   keeping scroll proof rows single-line/shrinkable under resize while dialog title/description
   recipe-owned text stays inside shadcn components.
   2026-05-19 gallery Drawer scroll/side text follow-up: drawer scroll filler rows now route
   through shared list-row label text, side body copy routes through paragraph text, and the
   historical `paragraph_block` helper name is gone so fixed scroll rows are not described as
   paragraph layout.
   2026-05-19 gallery Drawer goal/diagnostics text follow-up: demo and RTL goal readouts now use
   shared control-readout text, nested drawer guidance uses paragraph text, and the outside-press
   probe status/description now use shared paragraph/control-readout roles instead of `ui::text`.
   2026-05-19 gallery ScrollArea visible text follow-up: copyable ScrollArea snippets now route
   fixed tag/RTL rows through `text_list_row_label(...)`, section headings through
   `text_section_chrome_label(...)`, figure captions through `text_control_readout(...)`, and body
   prose through `text_paragraph(...)` instead of local `ui::text(...)` / raw typography policy.
   2026-05-19 gallery ContextMenu trigger text follow-up: copyable ContextMenu dashed trigger
   surfaces now route fine/coarse pointer copy through `text_control_readout(...)` instead of
   repeating local `ui::text(label).text_sm().text_color(...)` policy in every snippet.
   2026-05-19 gallery Pagination text follow-up: copyable Pagination page-number helpers now
   accept `cx` and route page labels through `text_button_label(...)` instead of no-context
   `ui::text(...).tabular_nums()` builders, because shared resize-safe roles are theme/context
   bound. RTL page numbers use the same helper, Fret-specific extras prose uses
   `text_paragraph(...)`, and built-in `PaginationPrevious` / `PaginationNext` visible text in
   `fret-ui-shadcn` now uses the shared button-label role. This keeps `fret-imui` thin while
   moving button-like pagination text policy into the component/kit layer.
   2026-05-19 gallery Carousel status/readout text follow-up: API/events/autoplay diagnostic
   status lines now route through `text_control_readout(...)` instead of local centered
   `TextProps` blocks with `TextWrap::Word`. The centering remains a layout concern in the
   snippet shell (`h_flex + justify_center`), while the resize contract stays on the shared
   control-readout role.
   2026-05-19 gallery NavigationMenu link-label text follow-up: custom icon/text link labels in
   the NavigationMenu docs/demo/RTL snippets now route through `text_button_label(...)` instead of
   bare `cx.text(label)`. Card title/body line-clamp text remains a separate candidate because it
   may need a list/card description role rather than a mechanical button-label migration.
   2026-05-19 compact paragraph line-clamp follow-up: `text_compact_paragraph_line_clamp(...)`
   now owns the dense paragraph-family `max-lines + ellipsis` contract for list/card descriptions,
   and NavigationMenu list-item titles/descriptions use shared button-label plus clamped paragraph
   roles instead of local `TextProps` line-clamp blocks. Featured NavigationMenu home-card brand
   copy remains explicit visual styling outside this slice.
   2026-05-17 gallery editor preview text follow-up: code-editor, Markdown, and Web IME preview
   headers now route explanatory copy through `paragraph_text(...)`, fixed status/debug values
   through `control_readout_text(...)`, and custom pointer-region labels through
   `button_label_text(...)`. This keeps resize-sensitive editor proof chrome on the shared
   role vocabulary without adding a public `TextRole` enum or moving policy into `fret-imui`.
   2026-05-18 code-view editor preview prose follow-up: the UI Gallery code-view torture header
   now routes its explanatory copy through `paragraph_text(...)` instead of bare `cx.text(...)`,
   keeping the scrollable code/text preview on the shared paragraph role vocabulary.
   2026-05-18 text editor/conformance header prose follow-up: the UI Gallery text
   editor/conformance headers for feature toggles, measure overlay, mixed-script fallback, outline
   stroke, selection perf, and BiDi/RTL now route explanatory copy through `paragraph_text(...)`;
   the BiDi sample-list heading uses `control_readout_text(...)`. Text capability probes remain on
   their explicit `TextProps` / `SelectableTextProps` / canvas text paths.
   2026-05-18 IMUI virtual-list fixed-row clip follow-up: fixed/known-height IMUI virtual-list
   rows now set `Overflow::Clip` on the row container, while measured rows keep visible overflow for
   runtime measurement. This closes the container side of the text-resize failure mode where a
   caller accidentally submits oversized/wrapping content into a fixed row; the fix stays in
   `fret-ui-kit::imui` and does not add a mutable list runtime to `fret-imui`.
   2026-05-18 retained tree fixed-row clip follow-up: `tree_view_retained(...)` now gives
   fixed/known-height retained tree rows an explicit fill-width, fixed-height, `Overflow::Clip`
   pressable row layout, while `Measured` retained tree rows stay auto-height/visible for runtime
   measurement. This closes the same resize-overflow class for generic editor trees without moving
   tree policy into `fret-imui`.
   2026-05-18 retained file-tree fixed-row clip follow-up:
   `file_tree_view_retained_v0(...)` now gives the retained Pressable row itself an explicit
   fill-width, fixed-height, `Overflow::Clip` layout, matching its known-height virtualizer
   contract instead of relying only on an inner content container to clip.
   2026-05-18 retained table fixed-row clip follow-up:
   `table_virtualized_retained_v0(...)` and grouped/eager table row owners now share
   `table_body_row_layout(...)`: fixed rows become fill-width fixed-height clip boundaries, while
   measured rows stay auto-height/visible for runtime measurement. This keeps table cell text and
   table row containers aligned with the same resize contract without moving table state/runtime
   policy into `fret-imui`.
   2026-05-17 code-editor IME gate button-label follow-up: code-editor MVP IME gate actions now
   route their custom pointer-region labels through `button_label_text(...)`, and both the gallery
   source test and IMUI workstream source gate forbid those fixed action labels from returning to
   bare `cx.text(...)`.
   2026-05-17 docking arbitration text-role follow-up: `docking_arbitration_demo` now routes
   Popover body copy through a local paragraph helper and state/debug status lines through a local
   readout helper backed by `text_control_readout(...)`. A source test and IMUI workstream source
   gate prevent the docking proof from returning to bare `cx.text(...)`.
   2026-05-17 docking/container-query panel text follow-up: `docking_demo` and
   `container_queries_docking_demo` now route fixed panel labels, readouts, and placeholders through
   local helpers backed by shared list-row, control-readout, and button-label text roles. Focused
   source tests plus the IMUI workstream source gate keep those resize-sensitive panel demos from
   returning to bare `cx.text(...)`.
   2026-05-19 imui node-graph compatibility title follow-up: `imui_node_graph_demo` keeps its
   retained-bridge compatibility-only posture, but the fixed proof title now routes through a local
   helper backed by `text_section_chrome_label(...)` instead of local
   `fret_ui_kit::ui::text(...).font_semibold()` styling. This does not promote the retained
   bridge as the default node-graph authoring path and does not widen `fret-imui`.
   2026-05-19 embedded viewport chrome text follow-up: `embedded_viewport_demo` now routes fixed
   ToggleGroup size labels through a button-label role and viewport status lines through a
   control-readout role instead of local `ui::text(...).text_sm()` builders. This closes another
   resize-sensitive app-owned chrome surface while leaving embedded viewport/input forwarding
   behavior unchanged.
   2026-05-19 window hit-test probe text follow-up: `window_hit_test_probe_demo` now routes its
   fixed 44px header title through section-chrome text, the logical-window diagnostic identifier
   through code-label text, and status through control-readout text. The multi-window hit-test
   repro path stays unchanged; only local fixed chrome/readout text policy was removed.
   2026-05-19 launcher utility window text follow-up: `launcher_utility_window_demo` now routes
   its frameless-window drag title through section-chrome text, the effective-style diagnostic line
   through code-label text, status through control-readout text, and the resize handle arrow
   through chrome-glyph text. The BeginDrag/BeginResize/SetVisible proof path stays unchanged; only
   local fixed chrome/readout/glyph text policy was removed.
   2026-05-19 launcher utility window materials text follow-up:
   `launcher_utility_window_materials_demo` now routes its fixed material-window title through
   section-chrome text, the effective material/style diagnostic through code-label text, and the
   status line through control-readout text. The background material request/diagnostics proof path
   stays unchanged; only local fixed chrome/readout text policy was removed.
   2026-05-19 API workbench lite text follow-up: `api_workbench_lite_demo` now routes its app
   title/sidebar labels through section-chrome text, first-contact copy through paragraph text,
   active base URL through code-label text, and history loading/error/empty states through
   control-readout text. The request/mutation/history proof path stays unchanged, and the old
   `shell_frame` theme-snapshot parameter was removed because it only existed to support local text
   color policy.
   2026-05-19 hello counter text follow-up: `hello_counter_demo` now routes its compact status
   line through control-readout text and its step help copy through paragraph text. The large count
   display remains an explicit visual readout until a dedicated large-display value role exists;
   the counter action/state path stays unchanged.
   2026-05-19 simple todo text follow-up: `simple_todo_demo` now routes its summary/footer
   readouts through control-readout text, empty-state copy through compact paragraph text, and row
   labels through list-row text with app-owned done/active foreground state. The demo no longer has
   `ui::text(...)` residuals.
   2026-05-19 todo demo text follow-up: `todo_demo` now routes title/status/progress/empty/filter
   labels through shared text roles and uses the new attributed list-row label role for completed
   rows that need strikethrough. The richer todo proof no longer has local `ui::text(...)` or
   `ui::rich_text(...)` residuals.
   2026-05-19 async playground text follow-up: `async_playground_demo` now routes fixed app chrome,
   catalog rows, policy/status readouts, query identifiers, and result body copy through shared
   chrome-title, section-chrome, list-row, control-readout, code-label, and compact-paragraph text
   roles. The obsolete `ThemeSnapshot` plumbing that only supported local muted text color policy
   was removed from the query panel helpers.
   2026-05-19 GenUI demo text follow-up: `genui_demo` now routes JSON/log/prompt panes through
   code-block text roles, toolbar/status/issue readouts through control-readout roles, and stream
   guidance through compact paragraph text. The old empty `ui::text("")` spacer and local
   `ui::text(...).text_sm()` policy were removed without moving GenUI runtime/rendering ownership
   into `fret-imui`.
   2026-05-19 extras marquee perf text follow-up: `extras_marquee_perf_demo` now routes its fixed
   perf-probe title through a section-chrome text role instead of local
   `ui::text(...).font_semibold()` styling. The marquee animation/perf path stays unchanged.
   2026-05-19 residual bare text gate tightening follow-up: `text_role_residual_surface` now counts
   `ui::text(...)` and `ui::rich_text(...)` in addition to `cx.text(...)`, `TextProps::new(...)`,
   and direct `TextProps { ... }` construction. The only remaining `ui::text(...)` entries in
   `fret-examples` are explicitly documented display/performance payloads:
   `hello_counter_demo`'s large numeric display and `hello_world_compare_demo`'s GPUI/Fret
   comparison title. Ordinary app/proof visible text must continue moving to shared roles instead
   of relying on those exceptions.
   2026-05-19 query detail text follow-up: the residual gate tightening exposed `ui::raw_text(...)`
   as another uncounted text-policy path. `query_demo` and `query_async_tokio_demo` now route their
   query status/error/duration/retry lines through `text_control_readout(...)` and their data values
   through `text_code_label(...)`, preserving app-owned error foreground via `inherit_foreground`.
   `imui_editor_proof_demo` also deleted its old direct `EditorCompactReadoutStyle::text_props(...)`
   readout helper and now uses the shared control-readout role.
   2026-05-19 custom effect overlay text follow-up: `custom_effect_v1_demo` and
   `custom_effect_v2_demo` no longer keep fixed overlay labels in the direct-text residual allowlist.
   Their pill labels now use `text_section_chrome_label(...)` with app-owned white foreground
   inheritance, leaving custom-effect runtime/ABI ownership unchanged.
   2026-05-19 custom effect web overlay text follow-up: `custom_effect_v2_web_demo` no longer keeps
   its unsupported-state readout, WebGPU badge label, or keyboard hint in the direct-text residual
   allowlist. The badge now uses section-chrome text, the status/hint use control-readout text, and
   absolute positioning stays in layout containers instead of local `TextProps`.
   2026-05-19 custom effect web template text follow-up:
   `custom_effect_v2_identity_web_demo`, `custom_effect_v2_lut_web_demo`, and
   `custom_effect_v2_glass_chrome_web_demo` no longer keep fixed overlay/control text in the
   direct-text residual allowlist. Starter/LUT badges use section-chrome text, status/hints use
   control-readout text, and the glass/chrome slider label/value row uses control-label/readout
   roles.
   2026-05-19 effect reference chrome text follow-up: `custom_effect_v3_demo`,
   `postprocess_theme_demo`, and `liquid_glass_demo` no longer keep fixed overlay/header/card
   titles in the direct-text residual allowlist. Effect/runtime proof ownership stays local, while
   fixed chrome/readout text now routes through `text_section_chrome_label(...)` and
   `text_control_readout(...)`.
   2026-05-19 shadcn Table role-preservation follow-up: `TableCell` and `TableHead` now treat
   recipe typography as a bare-text fallback. Caller-supplied shared text roles such as
   `text_table_cell(...)` keep their role-owned style/wrap/overflow contract, while plain
   `cx.text(...)` still receives table default typography and fixed-row clipping behavior.
   2026-05-19 shadcn DataTable role-preservation follow-up: `DataTable` body-cell default text
   styling now uses the same role-scope guard as Table. Bare cell text still receives the default
   table typography, but caller-supplied shared text roles are not rewritten by the virtualized
   data-table wrapper.
   2026-05-19 shadcn NavigationMenuLink role-preservation follow-up: link children now receive
   foreground through inherited foreground and only bare text receives the link default typography.
   Shared button-label roles keep their single-line ellipsis contract when used as custom link
   content.
   2026-05-19 shadcn ItemTitle role-preservation follow-up: ItemTitle still applies its strong
   title-slot fallback to bare and ordinary rich text children, but shared title/chrome text roles
   are treated as protected role scopes and keep their own typography/ellipsis contract.
   2026-05-19 shadcn CardTitle role-preservation follow-up: CardTitle now follows the same split
   for card titles. Bare and ordinary rich title children still receive the shadcn card-title
   typography and wrapping fallback, while explicit shared title/chrome roles keep their
   role-owned single-line ellipsis contract under card composition.
   2026-05-20 shadcn CardDescription children role-preservation follow-up:
   CardDescription now has a focused gate proving shared description/body text roles keep their
   role-owned wrap/overflow and inherited metadata when passed through the composable children
   lane.
   2026-05-20 shadcn Sheet/Popover description children role-preservation follow-up:
   SheetDescription and PopoverDescription now expose composable children lanes, and focused gates
   prove shared paragraph/body roles keep their role-owned wrap/layout and inherited metadata under
   overlay description composition.
   2026-05-20 shadcn existing description children role-preservation follow-up:
   AlertDescription, DialogDescription, AlertDialogDescription, and ItemDescription now have focused
   gates proving their existing children lanes preserve shared paragraph/body roles instead of only
   testing inherited typography or rich/selectable text.
   2026-05-19 shadcn AlertTitle role-preservation follow-up: AlertTitle now follows the same split
   for alert titles. Bare and ordinary rich title children still receive the alert title fallback,
   while explicit shared title/chrome roles keep their role-owned single-line ellipsis contract
   under alert composition.
   2026-05-19 shadcn AlertDialogTitle role-preservation follow-up: AlertDialogTitle now protects
   shared title/chrome roles under alert-dialog composition while preserving the shadcn dialog-title
   fallback for bare/rich children.
   2026-05-19 shadcn DialogTitle children-role follow-up: DialogTitle now exposes
   `new_children(...)` so dialog titles can carry shared title/chrome roles. Bare and ordinary rich
   title children still receive the shadcn dialog-title fallback, while explicit shared roles keep
   their role-owned single-line ellipsis contract.
   2026-05-19 shadcn SheetTitle children-role follow-up: SheetTitle now exposes
   `new_children(...)` with the same split. Bare/rich sheet-title children keep sheet defaults,
   while explicit shared title/chrome roles keep their role-owned single-line ellipsis contract.
   2026-05-19 shadcn PopoverTitle children-role follow-up: PopoverTitle now exposes
   `new_children(...)` for overlay/panel title composition. Bare/rich popover-title children keep
   popover defaults, while explicit shared title/chrome roles keep their role-owned single-line
   ellipsis contract.
   2026-05-19 shadcn FieldTitle children-role follow-up: FieldTitle now exposes
   `new_children(...)` for field/property panel title composition. Bare/rich field-title children
   keep field defaults and w-fit behavior, while explicit shared title/chrome roles keep their
   role-owned layout and ellipsis contract.
   2026-05-19 shadcn EmptyTitle children-role follow-up: EmptyTitle now exposes
   `new_children(...)` for empty-state title composition. Bare/rich empty-title children keep the
   shadcn empty-title defaults, while explicit shared title/chrome roles keep their role-owned
   single-line ellipsis contract.
   2026-05-20 shadcn SelectLabel menu-group text follow-up: `text_menu_group_label(...)` now owns
   the muted, single-line, fill-width group-label text role, and `SelectLabel` consumes that role
   instead of hand-rolling local `ui::text(...).text_size_px(...).nowrap()` policy in the overlay
   row renderer.
   2026-05-20 shadcn menu-family group-label follow-up: `DropdownMenuLabel`,
   `ContextMenuLabel`, and `MenubarLabel` now consume the same shared menu-group text role through
   local label element helpers. Menu item labels and icon/indicator color policy remain menu-owned;
   only the non-interactive group heading text policy moved out of local builders.
   2026-05-20 shadcn CommandGroup heading follow-up: `CommandGroup::heading(...)` now renders
   through a command-local helper backed by the same shared menu-group text role. This covers the
   command/listbox group-heading path used by combobox, native select, and data-table recipes
   without changing command item label/highlight rendering or widening `fret-imui`.
   2026-05-20 shared status-message/CommandEmpty follow-up: `text_status_message(...)` now owns
   the muted `text-sm` non-interactive empty/loading/status message role, and shadcn
   `CommandEmpty` / `CommandLoading` consume it instead of duplicating local text sizing/color
   builders.
   2026-05-20 shadcn DataTable toolbar text follow-up: DataTable toolbar faceted-trigger labels,
   faceted option labels, count chips, clear/reset actions, and selected-count readouts now route
   through shared button-label, list-row-label, and control-readout text roles instead of local
   `ui::text(...)` / `ui::raw_text(...)` builders.
   2026-05-20 inherited-feature readout follow-up: `TextStyleRefinement` now carries OpenType
   feature settings through passive-text measurement/cache, and DataTable pagination selected/page
   summaries consume shared tabular control-readout variants instead of local
   `ui::text(...).tabular_nums()` builders. This keeps numeric readouts inside the control-readout
   role family rather than adding a separate stable text role.
   2026-05-20 tabular readout resize-gate follow-up: the tabular control-readout variants are now
   explicitly covered by the shared narrow-layout single-line role gate and by the text role
   matrix, so the derived readout helpers cannot exist without resize evidence.
   2026-05-20 shadcn ButtonGroupText children-role follow-up: `ButtonGroupText::new_children(...)`
   now has a focused gate proving caller-supplied `text_button_label(...)` children keep their
   role-owned no-wrap, ellipsis, zero-min-width, and shrink contract under button-group chrome
   composition. The default `ButtonGroupText::new(...)` label path remains component-owned policy.
   2026-05-20 shadcn TabsTrigger children-role follow-up: trigger typography/foreground patching
   now treats `inherited_text_style` as a protected role scope. Bare trigger text keeps the shadcn
   fallback, while caller-supplied `text_button_label(...)` trigger children keep their role-owned
   single-line shrink/ellipsis contract under tabs chrome composition.
   2026-05-20 shadcn Toggle/ToggleGroup children-role follow-up: toggle and toggle-group item
   foreground patching now treats `inherited_text_style` as a protected role scope. Bare custom text
   keeps the component foreground fallback, while caller-supplied `text_button_label(...)` children
   keep their role-owned no-wrap, ellipsis, zero-min-width, and shrink contract.
   2026-05-20 shadcn Badge children-role follow-up: badge foreground patching now treats
   `inherited_text_style` as a protected role scope. Bare leading/trailing text keeps the badge
   foreground fallback, while caller-supplied `text_button_label(...)` children keep their role-owned
   no-wrap, ellipsis, zero-min-width, and shrink contract.
   2026-05-20 shadcn Button children-role follow-up: `Button::children(...)`,
   `Button::leading_children(...)`, and `Button::trailing_children(...)` now have focused gates
   proving caller-supplied `text_button_label(...)` roles keep their leaf no-wrap, ellipsis,
   zero-min-width, shrink, and inherited metadata contract under button chrome composition.
   2026-05-20 shadcn TooltipContent role-preservation follow-up: tooltip content defaults now treat
   `inherited_text_style` as a protected role scope and stamp tooltip foreground as inherited
   foreground on the content root. Bare tooltip text still receives tooltip `text-xs`/foreground
   defaults, while caller-supplied `text_control_readout(...)` children keep their role-owned
   no-wrap, ellipsis, zero-min-width, shrink, and inherited metadata contract.
   2026-05-20 shadcn BreadcrumbList role-preservation follow-up: list-level muted foreground now
   flows through inherited foreground instead of direct text leaf colors, and breadcrumb list text
   defaults skip shared text-role scopes. Bare loose breadcrumb text still receives breadcrumb
   typography, while caller-supplied `text_button_label(...)` children keep their role-owned
   no-wrap, ellipsis, zero-min-width, shrink, and inherited metadata contract.
   2026-05-20 shadcn AnnouncementTitle role-preservation follow-up: the extras title keeps its
   recipe-owned clipped title container and bare-text single-line ellipsis fallback, but title
   typography is now applied to bare text leaves instead of the title root. The recursive title
   contract treats `inherited_text_style` as a protected shared-role scope, so caller-supplied
   `text_button_label(...)` children keep their role-owned leaf style/color, no-wrap, ellipsis,
   zero-min-width, shrink, and inherited metadata contract under the raw extras title surface.
   2026-05-20 shadcn SidebarGroupLabel resize follow-up: the fixed-height sidebar group label no
   longer hand-rolls `ui::text(...).wrap(TextWrap::Word)` inside a 32px chrome row. The shared
   `text_menu_group_label(...)` role now carries `text-xs font-medium` plus fill/shrink/min-width-0
   single-line ellipsis semantics, and `SidebarGroupLabel` consumes it while overriding foreground
   from sidebar context. This directly targets the resize failure mode where chrome labels wrap
   into a second line and overrun the row bottom.
   2026-05-20 shadcn SidebarMenuBadge resize follow-up: upstream sidebar badges are fixed
   `h-5 min-w-5 text-xs font-medium tabular-nums` counter slots. Fret now keeps that in the
   control-readout role family via `text_control_readout_compact_tabular_emphasis(...)` instead of
   local `component.sidebar.menu_badge.text_px` / `line_height` text builders. `SidebarMenuBadge`
   consumes the derived readout role and keeps sidebar foreground ownership.
   2026-05-20 shadcn SidebarMenuButton/SubButton resize follow-up: upstream sidebar menu labels
   are fixed-row button-like triggers with `truncate` behavior (`text-sm` by default, `text-xs` for
   small rows). Fret now keeps the main and nested default labels in the button-label role family
   through `text_button_label_fill(...)` / `text_button_label_compact_fill(...)`, deleting the
   sidebar-local `menu_button_style(...)` / `menu_sub_button_style(...)` text builders and their
   local `text_size_px` / `line_height_px` policy. Sidebar still owns row chrome, collapse opacity,
   RTL ordering, foreground inheritance, and tooltip placement.
  2026-05-20 inherited-axis + shadcn Button default-label follow-up: `TextStyleRefinement` now
  carries variable font axes as subtree defaults, so role-based text paths can preserve
  `label_font_axis(...)` without falling back to leaf-local `TextStyle` builders. shadcn `Button`
  default labels now render through the shared button-label role and layer Button-owned font,
  feature, axis, weight, foreground, and test-id suffix behavior through inherited metadata. This
  removes the foundational fixed-height button path from local `ui::text(...).fixed_line_box_px(...)`
  policy while keeping custom child and inline slot role-preservation gates intact.
   2026-05-20 shadcn CalendarDayButton text-role follow-up: day numbers and optional supporting
   text now render through shared button-label/readout roles rather than Calendar-local
   `ui::label(...).line_height_px(...).nowrap()` builders. Single-date and range calendars share the
   same helper contract; Calendar remains responsible for fixed cell chrome, selected/today/range
   foreground, center alignment, and disabled opacity.
   2026-05-20 shadcn CalendarMultiple text-role follow-up: multiple-selection calendar day numbers
   now share `calendar_day_button_children(...)` with single/range day cells instead of carrying a
   local `ui::label(day_text).text_size_px(...).line_height_px(...).font_medium()` builder.
   CalendarMultiple remains responsible for multi-select updates and cell chrome; shared text roles
   own no-wrap, shrink, min-width-zero, ellipsis, and inherited text/foreground semantics.
   2026-05-20 shadcn CalendarHijri text-role follow-up: Hijri day numbers now share the Gregorian
   calendar day-cell helper instead of direct `TextProps::new(day_text)` fixed-line/clipped text.
   Hijri remains responsible for RTL order, Persian digits, Gregorian-date test ids, selection
   updates, and cell chrome; shared text roles own the fixed-cell resize semantics.
   2026-05-20 shadcn Kbd/ShortcutHint keycap text-role follow-up: fixed keycap/hint labels now
   route through `text_keycap_label(...)` instead of local
   `ui::label(...).fixed_line_box_px(...).line_box_in_bounds()` builders. Kbd and ShortcutHint keep
   shadcn `component.kbd.*` typography refinements, foreground, tooltip slot colors, icon children,
   and row layout; the shared role owns no-wrap, shrink, min-width-zero, and ellipsis.
   2026-05-20 shadcn menu item label follow-up: `text_list_row_label(...)` now includes the
   grow/basis-zero contract its name and tests already implied, and DropdownMenu, ContextMenu, and
   Menubar overlay item labels route through `menu_text::menu_item_label(...)`. The helper layers
   shadcn menu typography and state foreground through inherited metadata while the list-row role
   owns fixed-row no-wrap/shrink/min-width-0/ellipsis behavior. DropdownMenu's icon/currentColor
   slots are stamped on icon/custom/trailing subtrees instead of wrapping the label subtree, so menu
   label state colors are not overwritten by muted icon foreground.
   2026-05-20 shadcn NativeSelect text-role follow-up: NativeSelect trigger value text now routes
   through `text_control_label(...)`, and listbox option labels route through
   `text_list_row_label(...)`, instead of component-local `ui::text(...)` /
   `ui::label(...)` fixed-line builders. NativeSelect still owns trigger chrome, placeholder vs
   selected foreground, command-list selection, check icon visibility, popover placement, and
   RTL order; shared text roles own no-wrap, fill/grow/shrink, min-width-zero, ellipsis, and
   inherited typography/foreground semantics.
   2026-05-20 shadcn Combobox text-role follow-up: default Combobox trigger labels now route
   through `text_control_label(...)`, and non-search listbox option labels route through
   `text_list_row_label(...)`, instead of component-local `ui::label(...)` fixed-line builders.
   Combobox still owns trigger chrome, placeholder vs selected foreground, inline addons,
   clear/chevron buttons, popover/drawer behavior, search-enabled CommandPalette behavior, custom
   item content, and RTL order; shared text roles own default-label no-wrap, fill/grow/shrink,
   min-width-zero, ellipsis, and inherited typography/foreground semantics.
   2026-05-21 shadcn ComboboxChips text-role follow-up: empty-trigger placeholder text now routes
   through `text_control_label(...)`, and selected chip pill labels route through the new
   `text_chip_label(...)` shared role instead of component-local `ui::label(...).text_size_px(...)
   .truncate()` builders. ComboboxChips still owns trigger/chip chrome, remove button behavior,
   popover/search policy, selected-value lookup, wrapping chip row layout, and RTL order; shared
   roles own the resize-sensitive no-wrap/min-width-zero/ellipsis contracts, with chip labels kept
   non-growing so pill chrome does not expand like a row label.
   2026-05-21 shadcn Badge default-label text-role follow-up: default Badge labels now route
   through the compact `text_chip_label(...)` shared role instead of component-local
   `ui::text(...).text_size_px(...).fixed_line_box_px(...)` builders. Badge still owns variant
   chrome, foreground/currentColor scope, icon sizing, link underline, action/link semantics, and
   leading/trailing children fallback behavior; the shared role owns the default label's no-wrap,
   min-width-zero, shrink, ellipsis, and inherited text/foreground contract.
3. Design surface readiness: keep Dear ImGui-style density as an opt-in token/preset outcome, not a
   mutable runtime style stack.
   Current readiness audit: `P3_DESIGN_SURFACE_READINESS_2026-05-06.md`. `ImguiLikeDense` plus
   editor tokens cover the active proof; a style editor, preset selector, or visual gate should be a
   narrow follow-on with evidence.
   2026-05-14 cleanup: the unused `apply_editor_theme_patch_v1` compatibility wrapper was deleted;
   apps and tests now stay on explicit preset entry points.
4. Porting sugar readiness: `SameLine`/item-width/label-ID helpers only if two proof surfaces pay
   the same tax. Current proof surfaces already keep most of that tax local with
   `PropertyGrid`, `row_with`, `horizontal_with_options`, `child_region_with_options`, and
   explicit `id_source` / `test_id` wiring.
   Current readiness audit: `P3_PORTING_SUGAR_READINESS_2026-05-06.md`. Do not widen sugar until a
   second surface repeats the same pattern; do not copy Dear ImGui's string-label parser or
   stack/next-item width grammar into Fret by default.
   2026-05-14 cleanup: the unused `PropertyGridRow` wrapper was deleted so property-grid row
   authoring stays on `PropertyGridRowCx::row(...)` / `row_with(...)` plus raw `PropertyRow` for
   genuinely custom rows.
   2026-05-14 follow-up: eager and virtualized grid row contexts now keep `row_options`
   crate-local, so proof/app code cannot drift back to copying row policy fields by default.
   2026-05-14 inspector follow-up: `InspectorPanelCx` now hides `query_lower` and exposes query
   behavior through methods.
5. Diagnostics/devtools readiness: define a Fret equivalent of Demo/Metrics/Debug discoverability.
   Follow-on: `docs/workstreams/standalone/diag-devtools-gui-refresh-v1.md` and
   `docs/workstreams/diag-fearless-refactor-v2/README.md` keep the GUI
   productization / first-open workflow on the existing diagnostics-consumer lane.
   2026-05-16 first-open gate wording follow-up: the IMUI gap lane now distinguishes the
   cold-start `--discovery-only` entry from the fast `--discovery-only --reuse-built` drift check,
   so diagnostics discoverability failures are not conflated with large Rust build cost.
   2026-05-21 demo/metrics/debug discovery follow-up: `fretboard-dev list tool-apps` now surfaces
   the `demo-metrics-debug` route directly, and `list tool-apps --json` exposes the same route under
   `first_open_routes` with grouped demo, metrics, and debug commands, including `diag trace`.
   This keeps the Dear
   ImGui-style Demo/Metrics/Debug entry discoverable before a maintainer opens the DevTools GUI.
6. Collection helper readiness: keep app-owned until both proof surfaces require one helper.
   Current readiness audit: `P3_COLLECTION_HELPER_READINESS_2026-05-06.md`. The collection proof is
   editor-grade, but most behavior remains app-owned; the already-extracted shared pieces are
   `ImUiMultiSelectState`, sortable row recipes, and drag-preview recipes. `fret-node` is useful
   comparison evidence but not a second IMUI collection proof because it owns graph-specific
   node/edge/group semantics.
   2026-05-14 multi-select storage follow-up: `ImUiMultiSelectState` now keeps selection and anchor
   storage private behind explicit accessors/constructors. The collection proof remains app-owned,
   but it can no longer bypass the shared selection-storage API by mutating public fields.
   2026-05-14 ordered-selection follow-up: collection order normalization moved from the proof app
   into `ImUiMultiSelectState::from_ordered_selection(...)`, keeping the Dear ImGui-style storage
   helper in `fret-ui-kit::imui` while avoiding a monolithic `fret-imui` multi-select runtime.
   2026-05-14 request-vocabulary audit: `P3_COLLECTION_HELPER_READINESS_2026-05-06.md` now records
   that this storage extraction is not enough evidence to add `BeginMultiSelect`/`EndMultiSelect`
   or an `ImUiMultiSelectIO` runtime surface.
   2026-05-14 state-catalog gate follow-up: `ImUiMultiSelectState` is now covered by the reusable
   opaque-struct catalog because shared collection state is part of the public policy-layer
   contract, not a freely mutable data bag.
   2026-05-27 multi-select state owner split, refreshed 2026-05-30:
   `ImUiMultiSelectState` storage and accessors live in `multi_select/state.rs`; ordered-selection
   normalization, anchor repair, and crate-local selection mutation helpers live in
   `multi_select/state/selection.rs`; the root helper keeps click-modifier policy and response
   wiring.
7. Child-region depth: reopen only with a concrete `BeginChild()`-style behavior target.
   Current readiness audit: `P3_CHILD_REGION_READINESS_2026-05-06.md`. Fret already covers
   keyed scrollable child areas, chrome, scroll handles, nested shell panes, and app-owned
   collection behavior. The closed `imui-child-region-resize-y-v1` and
   `imui-child-region-resize-x-v1` follow-ons now cover axis-specific manual resize with
   app-owned size state; the unconstrained-axis child-region paths now have focused AutoResizeY and
   AutoResizeX composition gates, while visibility-return, nav-flattening, and always-auto-resize
   behavior remain behavior-specific candidates. Do not open a generic `BeginChild()` flag-mirror
   lane.
8. Multi-window parity: continue in `docking-multiwindow-imgui-parity`.
9. Performance alignment: keep Dear ImGui-class smoothness pressure in the dedicated perf lanes and
   product-chain perf gates, not in a broad widget/API backlog.
   Current review: `P4_PERFORMANCE_ALIGNMENT_REVIEW_2026-05-06.md`. The useful comparison axis is
   Zed-style attribution and reuse discipline plus egui-style integration/repaint clarity; do not
   treat egui's full-layout-every-frame model as an IMUI architecture target.
   2026-05-23 refresh: the Windows RTX4090 editor-paint closeout and the closed
   `editor-canvas-paint-replay-slice-v1` follow-on keep the current performance owner in
   editor paint / row-scene replay bookkeeping, not in `fret-imui`. r59 target-machine closeout
   passed without a checked-in baseline change, so Dear ImGui-class smoothness pressure remains a
   dedicated perf/editor owner-lane concern.

These slices should stay Windows/Web-verifiable first; Linux-specific validation is not a gate for
opening the slice.

## Closeout

- [x] Add a closeout audit once the first cleanup/refactor slice lands and gates pass.
      Result: `P1_CLOSEOUT_AUDIT_2026-05-06.md` closes P1 cleanup while leaving this lane active for
      P2/P3 sequencing.

## P5 - Fearless Refactor Execution

- [x] Split the floating-window resize/chrome logic out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs` into a dedicated internal helper
      without changing the public IMUI surface.
      Result: `ecosystem/fret-ui-kit/src/imui/floating_window_resize.rs` now owns the resize-handle
      interaction logic and `floating_window_on_area.rs` only orchestrates it.
- [x] Keep the floating-window public API and teaching surfaces stable while the internal owner
      split lands.
      Result: no public IMUI exports changed and the current floating smoke suite still passes.
- [x] Verify the floating smoke/tests and the current IMUI source gates after the split.
      Result: `cargo test -p fret-ui-kit --features imui --lib
      floating_window_close_glyph_uses_shared_chrome_glyph_text_role`, `cargo nextest run -p
      fret-imui floating --no-fail-fast`, `cargo check -p fret-ui-kit --features imui --lib`, `python
      tools/gate_imui_workstream_source.py`, `python tools/gate_imui_facade_teaching_source.py`,
      `python tools/check_workstream_catalog.py`, and `git diff --check` all pass.
- [x] Split the floating-window title-bar row/close-button composition out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs` into a dedicated internal helper
      without changing the public IMUI surface.
      Result: `ecosystem/fret-ui-kit/src/imui/floating_window_title_bar.rs` now owns the
      title-row / close-button orchestration, while `floating_window_on_area.rs` keeps shell,
      content, and resize orchestration.
- [x] Split the floating-window content/blocker orchestration out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_content.rs` into dedicated internal helpers
      without changing the public IMUI surface.
      Result: `floating_window_content.rs` now owns the content scroll/focus wrapper and
      `floating_window_blocker.rs` now owns the input-blocking overlay, while
      `floating_window_on_area.rs` keeps the shell that wires title, content, blocker, and resize
      stack together.
- [x] Split the floating-window resize-stack orchestration out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs` into the resize owner without
      changing the public IMUI surface.
      Result: `floating_window_resize.rs` now owns the body/blocker/resize-handle stack assembly,
      while `floating_window_on_area.rs` passes a clipped body, blocker, resize flags, and handle
      test ids into that owner.
- [x] Split the floating-window resize-state calculation out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs` into the resize owner without
      changing the public IMUI surface.
      Result: `floating_window_resize.rs` now owns the resize-state clamp/snap/update logic via
      `prepare_resize_state(...)`, while `floating_window_on_area.rs` only wires the resulting
      state into the shell, chrome, and stack assembly.
- [x] Split the floating-window resize drag snapshot read out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs` into the resize owner without
      changing the public IMUI surface.
      Result: `floating_window_resize.rs` now owns the handle enumeration, drag-kind lookup, drag
      snapshot shape, and collapsed-aware resizing signal, while `floating_window_on_area/state.rs`
      consumes `current_resize_snapshot(...)` / `prepare_resize_state(...)` as owner outputs for
      the on-area composition layer.
- [x] Split the floating-window on-area collapse/resize state preparation out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs` into a dedicated private owner
      without changing collapsed toggles, resize-state preparation, area position feedback,
      title/content/shell wiring, or `FloatingWindowChromeResponse` semantics.
      Result: `floating_window_on_area/state.rs` owns resizable-layout/resize-enabled derivation,
      collapse toggle/readback, scale-factor lookup, resize owner calls, area position feedback,
      and chrome response assembly. `floating_window_on_area.rs` now only wires the prepared state
      into title bar, content, shell, and facade output.
- [x] Split the floating-window shell/container composition out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs` into a dedicated internal helper
      without changing the public IMUI surface.
      Result: `floating_window_shell.rs` now owns the window frame, title-bar container, clipped
      body, blocker, and resize stack assembly, while `floating_window_on_area.rs` only wires the
      prepared owner outputs together.
- [x] Split the floating-window resize-handle layout mapping out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_resize.rs` into a private helper without
      changing the public IMUI surface.
      Result: `resize_handle_layout(...)` now owns the repeated cursor/inset/size mapping, and
      `resize_handle_element(...)` only wires that layout into the pointer-region behavior.
- [x] Split the floating-window resize drag application out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_resize.rs::prepare_resize_state(...)` into a
      private helper without changing the public IMUI surface.
      Result: `apply_resize_drag(...)` now owns the handle-driven size/position mutation, and
      `prepare_resize_state(...)` only handles snapshot selection, collapse checks, and pixel snap.
- [x] Split the floating-window shell `ContainerProps` / `ColumnProps` construction out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_shell.rs::floating_window_shell_element(...)`
      into private helpers without changing the public IMUI surface.
      Result: `window_frame_props(...)`, `shell_column_props(...)`,
      `title_bar_container_props(...)`, and `clipped_body_props(...)` now own shell frame/layout
      properties while the shell element only composes owner outputs.
- [x] Split the floating-window shell frame/title/body props helpers out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_shell.rs` into a private props owner without
      changing window frame sizing, title-bar clipping/padding/border radii, collapsed sizing,
      inner content clipping, blocker mounting, resize-stack composition, or public IMUI surface.
      Result: `floating_window_shell/props.rs` owns `window_frame_props(...)`,
      `shell_column_props(...)`, `title_bar_container_props(...)`, and `clipped_body_props(...)`.
      `floating_window_shell.rs` now keeps shell composition only.
- [x] Split the floating-window title-bar `RowProps` / drag-surface `PointerRegionProps` /
      close-button props construction out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_title_bar.rs::floating_window_title_bar_row(...)`
      into a dedicated internal helper module without changing the public IMUI surface.
      Result: `floating_window_title_bar_props.rs` now owns the title-row layout, drag-surface
      layout, and close-button accessibility/size props, while `floating_window_title_bar.rs`
      keeps title-bar behavior orchestration and text-role helpers.
- [x] Split the floating-window title-bar double-click / Escape / close-button behavior out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_title_bar.rs` into a private behavior owner
      without changing the public IMUI surface.
      Result: `floating_window_title_bar/behavior.rs` now owns double-click collapse event
      recording, title-bar Escape close key behavior, close-button activation wiring, and model
      update/notify calls. `floating_window_title_bar.rs` keeps row composition, title text-role
      selection, close-button prop selection, and close-glyph text construction.
- [x] Split the floating-window content scroll/container layout construction out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_content.rs::floating_window_content_element(...)`
      into private helpers without changing the public IMUI surface.
      Result: `floating_window_content_props.rs` now owns the content surface layout, scroll
      layout, and container props, while `floating_window_content.rs` keeps the pointer/focus
      orchestration and consumes the prepared owner outputs.
- [x] Split the floating-window content pointer/focus/activation behavior out of
      `ecosystem/fret-ui-kit/src/imui/floating_window_content.rs` into a private behavior owner
      without changing the public IMUI surface.
      Result: `floating_window_content/behavior.rs` now owns content-surface pointer-region
      wrapping, focusable key stub installation, background-click focus requests,
      activate-on-click event recording, and float-layer bring-to-front delegation.
      `floating_window_content.rs` keeps content scroll/container composition and IMUI child
      mounting.
- [x] Split IMUI table render/body/header ownership out of
      `ecosystem/fret-ui-kit/src/imui/table_controls.rs` into private owner modules without
      changing the public IMUI surface.
      Result: `ecosystem/fret-ui-kit/src/imui/table_controls/render.rs` now owns table assembly,
      test-id suffixing, palette resolution, and shared cell helpers; `table_controls/body.rs`
      owns prepared cells, pinned row grouping, horizontal scroll wrapping, and cell wrapping; and
      `table_controls/header.rs` plus `header/{trigger,resize}.rs` own sortable/plain header
      behavior and resize interaction. The root `table_controls.rs` keeps only authoring collection
      and row/cell facade wiring.
      2026-05-26 table render helper owner split: shared cell layout/packing helpers now live in
      `table_controls/cell.rs`, palette resolution lives in `table_controls/palette.rs`, and
      column test-id suffixing lives in `table_controls/test_ids.rs`. `render.rs` keeps table
      assembly only.
- [x] Add a narrow optional `fret-plot/imui` adapter over existing declarative plot panels without
      restoring retained plot code or adding plot dependencies to `fret-imui` /
      `fret-ui-kit::imui`.
      Result: `ecosystem/fret-plot/src/imui.rs` exposes thin `UiWriter` helpers for the declarative
      plot panel props under the opt-in `imui` feature, while the default `fret-plot` surface stays
      declarative and retained plot bridge code stays deleted.
- [x] Add a narrow Dear ImGui `BeginListBox`-style container proof without moving selection,
      filtering, active-descendant, command package, or collection policy into the container.
      Result: `ecosystem/fret-ui-kit/src/imui/list_box_controls.rs` now owns the semantic scroll
      host, `ListBoxOptions` exposes only layout/scroll/diagnostics semantics knobs, and the
      focused `fret-imui` composition test proves listbox semantics, scroll forwarding, stacked
      selectable rows, and no container-owned active-descendant policy.
- [x] Split the IMUI facade basic text/separator wrapper bodies out of
      `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` into a private owner module without changing
      the public IMUI facade trait surface.
      Result: `ecosystem/fret-ui-kit/src/imui/facade_writer/basic_items.rs` owns the default bodies
      for basic text, wrapped text, bullet text, plain separators, and separator text; the root
      facade trait remains the public method hub and only forwards those calls.
- [x] Split the IMUI facade image item/button default bodies out of
      `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` into a private owner module without changing
      the public IMUI facade trait surface.
      Result: `ecosystem/fret-ui-kit/src/imui/facade_writer/image_items.rs` owns the private
      `image_item_with_options` / `image_button_with_options` forwarding and the image-button
      default normalization, while `image_item_controls.rs` remains the interactive image widget
      policy owner.
- [x] Split IMUI image-item visual/props helpers out of
      `ecosystem/fret-ui-kit/src/imui/image_item_controls.rs` into a private owner module without
      changing image/image-button roles, focusability, pressable response population, item sizing,
      opacity sanitization, or UV filtering.
      Result: `image_item_controls/visual.rs` owns chrome selection, image props, size
      sanitization, opacity normalization, and UV validation. `image_item_controls.rs` keeps
      pressable interaction wiring and `ResponseExt` population.
- [x] Split IMUI image-item pressable behavior out of
      `ecosystem/fret-ui-kit/src/imui/image_item_controls.rs` into a private owner module without
      changing image/image-button roles, focusability, context-menu keyboard requests, activation
      lifecycle marking, pointer-click reporting, click response population, item sizing, chrome, or
      image props.
      Result: `image_item_controls/behavior.rs` owns pressable behavior installation,
      keyboard-activation lifecycle marking, context-menu key handling, transient clicked reads, and
      `ResponseExt` population. `image_item_controls.rs` keeps a11y props, size props, key
      activation policy for plain images, chrome mounting, and image visual assembly.
- [x] Split the IMUI facade command-presentation default bodies out of
      `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` into the existing button/menu owner modules
      without changing the public IMUI facade trait surface.
      Result: `button_actions.rs` owns `button_command_with_options` presentation/default-enabled
      forwarding, `menu_items.rs` owns `menu_item_command_with_options`
      presentation/default-enabled/default-shortcut forwarding, and the source gate now rejects
      `command_presentation_for_window` from drifting back into the root facade trait hub.
- [x] Converge the dirty `main` and `imui-imgui-editor-grade-refactor` worktrees before continuing
      IMUI feature work.
      Result: `main` checkpoint `d078e25122`, IMUI worktree checkpoint `05727e284b`, and merge
      commit `dee3d48f44` are recorded in `WORKTREE_CONVERGENCE_PLAN_2026-05-26.md` and
      `EVIDENCE_AND_GATES.md`. The merged tree keeps the editor-grade facade/container/listbox
      organization, preserves the `main` image-item owner split, and continues only from `main`.
