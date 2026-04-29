//! Optional integration helpers for immediate-mode authoring frontends.
//!
//! This module lives in `fret-ui-kit` (not `fret-imui`) to preserve dependency direction:
//!
//! - `fret-imui` stays policy-light and depends only on `fret-ui` (+ `fret-authoring` contract).
//! - `fret-ui-kit` can optionally provide bridges that allow `UiBuilder<T>` patch vocabulary to be
//!   used from immediate-style control flow.

#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::too_many_arguments)]

use std::cell::Cell;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;

use fret_authoring::UiWriter;
use fret_core::{Point, Size};
use fret_runtime::{ActionId, CommandId};
use fret_ui::element::{AnyElement, PointerRegionProps};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

mod active_trigger_behavior;
pub mod adapters;
mod boolean_controls;
mod bullet_text_controls;
mod button_controls;
mod child_region;
mod combo_controls;
mod combo_model_controls;
mod containers;
mod control_chrome;
mod disclosure_controls;
mod drag_drop;
mod facade_support;
mod facade_writer;
mod floating_options;
mod floating_surface;
mod floating_window;
mod floating_window_on_area;
mod interaction_runtime;
mod item_behavior;
mod label_identity;
mod menu_controls;
mod menu_family_controls;
mod multi_select;
mod options;
mod popup_overlay;
mod popup_store;
mod response;
mod selectable_controls;
mod separator_text_controls;
mod slider_controls;
mod tab_family_controls;
mod table_controls;
mod text_controls;
mod tooltip_overlay;
mod virtual_list_controls;

use containers::{
    grid_container_element, horizontal_container_element, scroll_container_element,
    vertical_container_element,
};
pub use facade_support::UiWriterUiKitExt;
#[allow(unused_imports)]
use facade_support::{
    DEFAULT_DISABLED_ALPHA, DEFAULT_DRAG_THRESHOLD_PX, DRAG_KIND_MASK, HOVER_DELAY_NORMAL,
    HOVER_DELAY_SHORT, HOVER_STATIONARY_DELAY, KEY_ACTIVATED, KEY_CHANGED, KEY_CLICKED,
    KEY_CONTEXT_MENU_REQUESTED, KEY_DEACTIVATED, KEY_DEACTIVATED_AFTER_EDIT, KEY_DOUBLE_CLICKED,
    KEY_DRAG_STARTED, KEY_DRAG_STOPPED, KEY_HOVER_DELAY_NORMAL_MET, KEY_HOVER_DELAY_SHORT_MET,
    KEY_HOVER_STATIONARY_MET, KEY_LONG_PRESSED, KEY_POINTER_CLICKED, KEY_SECONDARY_CLICKED,
    LONG_PRESS_DELAY, fnv1a64, model_value_changed_for, point_add, point_sub,
    prepare_imui_runtime_for_frame, slider_clamp_and_snap, slider_normalize_range,
    slider_step_or_default, slider_value_from_pointer, snap_point_to_device_pixels,
    snap_size_to_device_pixels,
};
pub use facade_writer::{ImUiFacade, UiWriterImUiFacadeExt};
pub use floating_options::{
    FloatingAreaContext, FloatingAreaOptions, FloatingWindowOptions, FloatingWindowResizeOptions,
    WindowOptions,
};
use floating_surface::{
    FloatWindowResizeHandle, FloatWindowState, FloatingAreaState, FloatingWindowChromeResponse,
    KEY_FLOAT_WINDOW_ACTIVATE, KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED, OnFloatingAreaLeftDoubleClick,
    float_layer_bring_to_front_if_activated, float_window_drag_kind_for_element,
    float_window_resize_kind_for_element, floating_area_drag_surface_element,
};
use floating_surface::{floating_area_element, floating_layer_element};
pub use fret_ui::element::{VirtualListKeyCacheMode, VirtualListMeasureMode};
pub use fret_ui::scroll::VirtualListScrollHandle;
use interaction_runtime::{
    DisabledScopeGuard, active_item_model_for_window, clear_active_item_on_left_pointer_up,
    context_menu_anchor_model_for, disabled_alpha_for, disabled_scope_depth_for,
    drag_kind_for_element, drag_threshold_for, finish_pointer_region_drag,
    finish_pressable_drag_on_pointer_up, float_window_collapsed_model_for,
    handle_pointer_region_drag_move_with_threshold, handle_pressable_drag_move_with_threshold,
    hover_blocked_by_active_item_for, imui_is_disabled, install_hover_query_hooks_for_pressable,
    lifecycle_session_model_for, long_press_signal_model_for,
    mark_active_item_on_left_pointer_down, mark_lifecycle_activated_on_left_pointer_down,
    mark_lifecycle_deactivated_on_left_pointer_up, mark_lifecycle_edit,
    mark_lifecycle_instant_if_inactive, pointer_click_modifiers_model_for,
    populate_pressable_drag_response, populate_response_lifecycle_from_active_state,
    populate_response_lifecycle_transients, prepare_pointer_region_drag_on_left_down,
    prepare_pressable_drag_on_pointer_down, sanitize_response_for_enabled,
};
pub use multi_select::{ImUiMultiSelectState, multi_select_use_model};
pub use options::{
    BeginMenuOptions, BeginSubmenuOptions, BulletTextOptions, ButtonArrowDirection, ButtonOptions,
    ButtonVariant, CheckboxOptions, ChildRegionChrome, ChildRegionOptions, CollapsingHeaderOptions,
    ComboModelOptions, ComboOptions, DragSourceOptions, DropTargetOptions, GridOptions,
    HorizontalOptions, InputTextMode, InputTextOptions, MenuBarOptions, MenuItemOptions,
    PopupMenuOptions, PopupModalOptions, RadioOptions, ScrollOptions, SelectableOptions,
    SeparatorTextOptions, SliderOptions, SwitchOptions, TabBarOptions, TabItemOptions, TableColumn,
    TableColumnWidth, TableOptions, TableRowOptions, TableSortDirection, TextAreaOptions,
    TooltipOptions, TreeNodeOptions, VerticalOptions, VirtualListOptions,
};
use popup_store::{
    drop_popup_scope_for_id, popup_render_generation_for_window, with_popup_store_for_id,
};
pub use response::{
    ComboResponse, DisclosureResponse, DragResponse, DragSourceResponse, DropTargetResponse,
    FloatingAreaResponse, FloatingWindowResponse, ImUiHoveredFlags, ResponseExt, TabBarResponse,
    TabTriggerResponse, TableHeaderResponse, TableResponse, VirtualListResponse,
};
pub use tab_family_controls::ImUiTabBar;
pub use table_controls::{ImUiTable, ImUiTableRow};
