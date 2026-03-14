//! Filterable enum select control (editor-grade combobox-like widget).
//!
//! This is an ecosystem/policy control:
//! - it uses `fret-ui` mechanisms (pressable, focus, overlays),
//! - and `fret-ui-kit` infrastructure (overlay controller + popper placement),
//! - without depending on any design-system crate.

use std::panic::Location;
use std::sync::{Arc, Mutex};

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Corners, Edges, KeyCode, Px, TextAlign, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, OnKeyDown};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, ScrollAxis, ScrollProps, SizeStyle, SpacingLength, TextProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::{active_descendant as active_desc, combobox as kit_combobox, popper};
use fret_ui_kit::typography;
use fret_ui_kit::{OverlayController, OverlayPresence, OverlayRequest};

use crate::controls::MiniSearchBox;
use crate::primitives::icons::editor_icon_with;
use crate::primitives::input_group::{
    editor_input_group_divider, editor_input_group_frame, editor_input_group_inset,
    editor_input_group_row,
};
use crate::primitives::popup_surface::{
    EditorPopupSurfaceChrome, resolve_editor_popup_surface_chrome,
};
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState};
use crate::primitives::{EditorDensity, EditorTokenKeys};

#[derive(Debug, Clone)]
pub struct EnumSelectItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
}

impl EnumSelectItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnumSelectOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub focusable: bool,
    pub placeholder: Arc<str>,
    pub none_label: Arc<str>,
    pub max_list_height: Option<Px>,
    pub a11y_label: Option<Arc<str>>,
    /// Explicit identity source for internal state (open/filter models, overlay root ids).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when a helper function builds multiple enum selects from the same callsite and
    /// you need stable, per-instance state separation.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub list_test_id: Option<Arc<str>>,
    pub search_test_id: Option<Arc<str>>,
}

impl Default for EnumSelectOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            focusable: true,
            placeholder: Arc::from("Select…"),
            // In inspectors, `None` often means "mixed/indeterminate" rather than "unset".
            none_label: Arc::from("Mixed"),
            max_list_height: None,
            a11y_label: None,
            id_source: None,
            test_id: None,
            list_test_id: None,
            search_test_id: None,
        }
    }
}

#[derive(Clone)]
pub struct EnumSelect {
    model: Model<Option<Arc<str>>>,
    items: Arc<[EnumSelectItem]>,
    options: EnumSelectOptions,
}

impl EnumSelect {
    pub fn new(model: Model<Option<Arc<str>>>, items: Arc<[EnumSelectItem]>) -> Self {
        Self {
            model,
            items,
            options: EnumSelectOptions::default(),
        }
    }

    pub fn options(mut self, options: EnumSelectOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model_id = self.model.id();
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.enum_select", id_source, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.enum_select", callsite, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let open = open_model(cx);
        let filter = filter_model(cx);
        let open_change_reason = cx.local_model_keyed("open_change_reason", || {
            None::<kit_combobox::ComboboxOpenChangeReason>
        });
        let focus_restore_target = cx.slot_state(
            || Arc::new(Mutex::new(None::<GlobalElementId>)),
            |cell| cell.clone(),
        );

        let is_open = cx
            .get_model_copied(&open, Invalidation::Layout)
            .unwrap_or(false);

        let should_clear_filter = cx
            .slot_state(kit_combobox::ClearQueryOnCloseState::default, |state| {
                kit_combobox::should_clear_query_on_close(state, is_open)
            });
        if should_clear_filter {
            let _ = cx.app.models_mut().update(&filter, |s| s.clear());
        }

        let selected_value = cx
            .get_model_cloned(&self.model, Invalidation::Paint)
            .unwrap_or(None);

        let (density, frame_chrome, ring, popup_chrome) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            let density = style.density;
            let frame_chrome = style.frame_chrome_small();
            let ring = theme
                .color_by_key("ring")
                .unwrap_or_else(|| theme.color_token("primary"));
            let popup_chrome = resolve_editor_popup_surface_chrome(theme, true);
            (density, frame_chrome, ring, popup_chrome)
        };

        let selected_label = selected_value
            .as_deref()
            .and_then(|v| self.items.iter().find(|it| it.value.as_ref() == v))
            .map(|it| it.label.clone());

        let trigger_text = match (selected_value.as_ref(), selected_label.as_ref()) {
            (Some(_), Some(label)) => label.clone(),
            (Some(v), None) => Arc::from(format!("<unknown: {v}>")),
            (None, _) => self.options.none_label.clone(),
        };

        let mut trigger_layout = self.options.layout;
        if trigger_layout.size.min_height.is_none() {
            trigger_layout.size.min_height = Some(Length::Px(density.row_height));
        }

        let trigger_model = self.model.clone();
        let items_for_overlay = self.items.clone();
        let options_for_overlay = self.options.clone();
        let open_for_overlay = open.clone();
        let open_change_reason_for_overlay = open_change_reason.clone();
        let enabled_for_paint = self.options.enabled;

        let trigger = cx.pressable(
            PressableProps {
                layout: trigger_layout,
                enabled: self.options.enabled,
                focusable: self.options.focusable,
                a11y: PressableA11y {
                    role: Some(fret_core::SemanticsRole::ComboBox),
                    label: self.options.a11y_label.clone(),
                    expanded: Some(is_open),
                    ..Default::default()
                },
                focus_ring: Some(fret_ui::element::RingStyle {
                    placement: fret_ui::element::RingPlacement::Outset,
                    width: Px(2.0),
                    offset: Px(2.0),
                    color: ring,
                    offset_color: None,
                    corner_radii: Corners::all(frame_chrome.radius),
                }),
                ..Default::default()
            },
            move |cx, _st| {
                cx.pressable_add_on_activate(kit_combobox::set_open_change_reason_on_activate(
                    open_change_reason_for_overlay.clone(),
                    kit_combobox::ComboboxOpenChangeReason::TriggerPress,
                ));

                let open = open_for_overlay.clone();
                let on_activate: OnActivate =
                    Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
                        let prev = host.models_mut().get_copied(&open).unwrap_or(false);
                        let _ = host.models_mut().update(&open, |v| *v = !prev);
                        host.request_redraw(action_cx.window);
                    });
                cx.pressable_add_on_activate(on_activate);

                let caret_icon = if is_open {
                    fret_icons::ids::ui::CHEVRON_UP
                } else {
                    fret_icons::ids::ui::CHEVRON_DOWN
                };

                let divider = frame_chrome.border;

                vec![editor_input_group_frame(
                    cx,
                    LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    density,
                    frame_chrome,
                    EditorFrameState {
                        enabled: enabled_for_paint,
                        hovered: _st.hovered,
                        pressed: _st.pressed,
                        focused: _st.focused,
                        open: is_open,
                        semantic: EditorFrameSemanticState::default(),
                    },
                    move |cx, visuals| {
                        let text_el = cx.text_props(TextProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: trigger_text.clone(),
                            style: Some(typography::as_control_text(TextStyle {
                                size: Px(12.0),
                                line_height: Some(density.row_height),
                                ..Default::default()
                            })),
                            color: Some(visuals.fg),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Ellipsis,
                            align: TextAlign::Start,
                            ink_overflow: Default::default(),
                        });
                        let text = editor_input_group_inset(cx, frame_chrome.padding, text_el);

                        let sep = editor_input_group_divider(cx, divider);

                        let caret = cx.container(
                            ContainerProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Px(density.hit_thickness),
                                        height: Length::Fill,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                padding: Edges::all(Px(0.0)).into(),
                                ..Default::default()
                            },
                            move |cx| {
                                vec![cx.flex(
                                    FlexProps {
                                        layout: LayoutStyle {
                                            size: SizeStyle {
                                                width: Length::Fill,
                                                height: Length::Fill,
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        direction: Axis::Horizontal,
                                        gap: SpacingLength::Px(Px(0.0)),
                                        padding: Edges::all(Px(0.0)).into(),
                                        justify: MainAlign::Center,
                                        align: CrossAlign::Center,
                                        wrap: false,
                                    },
                                    move |cx| {
                                        vec![editor_icon_with(
                                            cx,
                                            density,
                                            caret_icon,
                                            Some(Px(12.0)),
                                            Some(fret_ui_kit::ColorRef::Color(visuals.icon)),
                                        )]
                                    },
                                )]
                            },
                        );

                        vec![editor_input_group_row(cx, Px(0.0), vec![text, sep, caret])]
                    },
                )]
            },
        );

        let trigger_id = trigger.id;
        *focus_restore_target
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(trigger_id);

        let enabled_for_keys = self.options.enabled;
        let on_trigger_open_keys: OnKeyDown = Arc::new({
            let open = open.clone();
            let open_change_reason = open_change_reason.clone();
            move |host, action_cx: ActionCx, down| {
                if !enabled_for_keys {
                    return false;
                }
                if matches!(
                    down.key,
                    KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space | KeyCode::ArrowDown
                ) {
                    let _ = host.models_mut().update(&open_change_reason, |v| {
                        *v = Some(kit_combobox::ComboboxOpenChangeReason::TriggerPress);
                    });
                    let _ = host.models_mut().update(&open, |v| *v = true);
                    host.request_redraw(action_cx.window);
                    return true;
                }
                if down.key == KeyCode::Escape {
                    let was_open = host.models_mut().get_copied(&open).unwrap_or(false);
                    if was_open {
                        let _ = host.models_mut().update(&open_change_reason, |v| {
                            *v = Some(kit_combobox::ComboboxOpenChangeReason::EscapeKey);
                        });
                        let _ = host.models_mut().update(&open, |v| *v = false);
                        host.request_redraw(action_cx.window);
                        return true;
                    }
                }
                false
            }
        });
        cx.key_add_on_key_down_capture_for(trigger_id, on_trigger_open_keys);

        if let Some(test_id) = self.options.test_id.as_ref() {
            // Attach on the returned element, not inside the pressable body, to keep the trigger
            // subtree stable across internal composition changes.
            let trigger = trigger.test_id(test_id.clone());
            request_overlay(
                cx,
                trigger_id,
                trigger_model,
                items_for_overlay,
                open,
                filter,
                open_change_reason,
                focus_restore_target,
                options_for_overlay,
                density,
                popup_chrome,
            );
            trigger
        } else {
            request_overlay(
                cx,
                trigger_id,
                trigger_model,
                items_for_overlay,
                open,
                filter,
                open_change_reason,
                focus_restore_target,
                options_for_overlay,
                density,
                popup_chrome,
            );
            trigger
        }
    }
}

fn request_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: fret_ui::elements::GlobalElementId,
    model: Model<Option<Arc<str>>>,
    items: Arc<[EnumSelectItem]>,
    open: Model<bool>,
    filter: Model<String>,
    open_change_reason: Model<Option<kit_combobox::ComboboxOpenChangeReason>>,
    focus_restore_target: Arc<Mutex<Option<GlobalElementId>>>,
    options: EnumSelectOptions,
    density: EditorDensity,
    popup_chrome: EditorPopupSurfaceChrome,
) {
    let model_for_list = model.clone();
    let open_for_list = open.clone();
    let open_for_dismiss = open.clone();
    let query_for_list = filter.clone();
    let open_change_reason_for_list = open_change_reason.clone();
    let open_change_reason_for_dismiss = open_change_reason.clone();
    let list_test_id = options.list_test_id.clone();
    let list_viewport_test_id = list_test_id
        .as_ref()
        .map(|test_id| enum_select_viewport_test_id(test_id.as_ref()));
    let item_test_id_prefix = list_test_id.clone();
    let search_test_id = options.search_test_id.clone();
    let scroll_handle = cx.slot_state(ScrollHandle::default, |handle| handle.clone());
    let pending_selected_reveal = cx.local_model_keyed("pending_selected_reveal", || false);

    let overlay_id = cx
        .named("enum_select.overlay", |cx| cx.spacer(Default::default()))
        .id;

    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let presence = OverlayPresence::instant(is_open);
    let close_focus = kit_combobox::on_close_auto_focus_with_reason(
        open_change_reason.clone(),
        focus_restore_target,
        enum_select_close_auto_focus_policy(),
    );

    let max_h = {
        let theme = Theme::global(&*cx.app);
        options
            .max_list_height
            .or_else(|| theme.metric_by_key(EditorTokenKeys::ENUM_SELECT_MAX_LIST_HEIGHT))
            .unwrap_or(Px(240.0))
    };

    let filter_text = cx
        .get_model_cloned(&filter, Invalidation::Paint)
        .unwrap_or_default();
    let q = filter_text.trim().to_lowercase();
    let matches = |s: &str| q.is_empty() || s.to_lowercase().contains(&q);

    let filtered: Arc<[EnumSelectItem]> = items
        .iter()
        .filter(|it| matches(it.label.as_ref()) || matches(it.value.as_ref()))
        .cloned()
        .collect::<Vec<_>>()
        .into();
    let queue_selected_reveal = cx.slot_state(
        || false,
        |was_open| {
            let queue = !*was_open && is_open;
            *was_open = is_open;
            queue
        },
    );
    if queue_selected_reveal {
        let _ = cx
            .app
            .models_mut()
            .update(&pending_selected_reveal, |pending| *pending = true);
    } else if !is_open {
        let _ = cx
            .app
            .models_mut()
            .update(&pending_selected_reveal, |pending| *pending = false);
    }
    let should_reveal_selected = cx
        .get_model_copied(&pending_selected_reveal, Invalidation::Layout)
        .unwrap_or(false);

    let placement = popper::PopperContentPlacement::new(
        popper::LayoutDirection::Ltr,
        Side::Bottom,
        Align::Start,
        Px(4.0),
    )
    .with_collision_padding(Edges::all(Px(8.0)));

    let list = cx.anchored_props(
        fret_ui::element::AnchoredProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Auto,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            outer_margin: Edges::all(Px(0.0)),
            anchor_element: Some(trigger_id.0),
            side: placement.side,
            align: placement.align,
            side_offset: placement.side_offset,
            options: placement.options(),
            ..Default::default()
        },
        move |cx| {
            let filtered = filtered.clone();
            let panel = cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(Px(260.0)),
                            height: Length::Auto,
                            ..Default::default()
                        },
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                    padding: Edges::all(Px(8.0)).into(),
                    background: Some(popup_chrome.bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(popup_chrome.border),
                    corner_radii: Corners::all(popup_chrome.radius),
                    shadow: popup_chrome.shadow.clone(),
                    ..Default::default()
                },
                move |cx| {
                    // `Container` does not imply vertical flow layout. Use an explicit column so
                    // the search box and the list do not overlap.
                    vec![cx.flex(
                        FlexProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: Axis::Vertical,
                            gap: SpacingLength::Px(Px(6.0)),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                            wrap: false,
                        },
                        move |cx| {
                            let mut out: Vec<AnyElement> = Vec::new();

                            let mut search = MiniSearchBox::new(filter.clone()).into_element(cx);
                            if let Some(test_id) = search_test_id.as_ref() {
                                search = search.test_id(test_id.clone());
                            }
                            out.push(search);

                            let scroll_handle_for_list = scroll_handle.clone();
                            let pending_selected_reveal_for_list = pending_selected_reveal.clone();
                            let selected_row_element_out = Arc::new(Mutex::new(None));
                            let selected_row_element_out_for_rows =
                                selected_row_element_out.clone();
                            let scroll = cx.scroll(
                                ScrollProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Fill,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    axis: ScrollAxis::Y,
                                    scroll_handle: Some(scroll_handle.clone()),
                                    ..Default::default()
                                },
                                move |cx| {
                                    let filtered = filtered.clone();
                                    vec![cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle {
                                                size: SizeStyle {
                                                    width: Length::Fill,
                                                    height: Length::Auto,
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            direction: Axis::Vertical,
                                            gap: SpacingLength::Px(Px(2.0)),
                                            padding: Edges::all(Px(0.0)).into(),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Stretch,
                                            wrap: false,
                                        },
                                        move |cx| {
                                            if filtered.is_empty() {
                                                return vec![cx.text("No matches")];
                                            }

                                            let item_test_id_prefix = item_test_id_prefix.clone();
                                            let mut rows = Vec::with_capacity(filtered.len());
                                            for (idx, it) in filtered.iter().enumerate() {
                                                let (row, row_id, row_selected) = enum_select_row(
                                                    cx,
                                                    idx,
                                                    filtered.len(),
                                                    model_for_list.clone(),
                                                    open_for_list.clone(),
                                                    query_for_list.clone(),
                                                    open_change_reason_for_list.clone(),
                                                    it.clone(),
                                                    density,
                                                    item_test_id_prefix.clone(),
                                                );
                                                if row_selected {
                                                    *selected_row_element_out_for_rows
                                                        .lock()
                                                        .unwrap_or_else(|e| e.into_inner()) =
                                                        Some(row_id);
                                                }
                                                rows.push(row);
                                            }
                                            rows
                                        },
                                    )]
                                },
                            );
                            let viewport = cx.container(
                                ContainerProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Px(max_h),
                                            ..Default::default()
                                        },
                                        overflow: Overflow::Clip,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                move |_cx| vec![scroll],
                            );
                            let viewport = if let Some(test_id) = list_viewport_test_id.as_ref() {
                                viewport.test_id(test_id.clone())
                            } else {
                                viewport
                            };
                            let viewport_id = viewport.id;
                            if should_reveal_selected {
                                let selected_row_element = *selected_row_element_out
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                if let Some(selected_row_element) = selected_row_element {
                                    let did_reveal = active_desc::scroll_active_element_into_view_y(
                                        cx,
                                        &scroll_handle_for_list,
                                        viewport_id,
                                        selected_row_element,
                                    );
                                    let already_visible = element_visible_within_viewport_y(
                                        cx,
                                        viewport_id,
                                        selected_row_element,
                                    )
                                    .unwrap_or(false);
                                    if did_reveal || already_visible {
                                        let _ =
                                            cx.app.models_mut().update(
                                                &pending_selected_reveal_for_list,
                                                |pending| *pending = false,
                                            );
                                    }
                                } else {
                                    let _ = cx
                                        .app
                                        .models_mut()
                                        .update(&pending_selected_reveal_for_list, |pending| {
                                            *pending = false
                                        });
                                }
                            }
                            out.push(viewport);
                            out
                        },
                    )]
                },
            );

            vec![panel]
        },
    );

    let list = if let Some(test_id) = list_test_id.as_ref() {
        list.test_id(test_id.clone())
    } else {
        list
    };

    // For editor selects, we want menu-like outside press dismissal that does not "click through"
    // (outside press closes the overlay without activating the underlay), but we do not need
    // Radix-style `disableOutsidePointerEvents` occlusion. Keeping occlusion off improves
    // reliability when other layers temporarily hold pointer capture.
    let mut request =
        OverlayRequest::dismissible_popover(overlay_id, trigger_id, open, presence, vec![list]);
    request.consume_outside_pointer_events = true;
    request.disable_outside_pointer_events = false;
    request.close_on_window_focus_lost = true;
    request.close_on_window_resize = true;
    request.on_close_auto_focus = Some(close_focus);
    let set_reason_on_dismiss =
        kit_combobox::set_open_change_reason_on_dismiss_request(open_change_reason_for_dismiss);
    request.dismissible_on_dismiss_request =
        Some(Arc::new(move |host, action_cx: ActionCx, req| {
            set_reason_on_dismiss(host, action_cx, req);
            let _ = host.models_mut().update(&open_for_dismiss, |v| *v = false);
            host.request_redraw(action_cx.window);
        }));

    OverlayController::request(cx, request);
}

fn enum_select_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    idx: usize,
    total: usize,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
    open_change_reason: Model<Option<kit_combobox::ComboboxOpenChangeReason>>,
    item: EnumSelectItem,
    density: EditorDensity,
    item_test_id_prefix: Option<Arc<str>>,
) -> (AnyElement, GlobalElementId, bool) {
    let selected = cx
        .get_model_cloned(&model, Invalidation::Paint)
        .unwrap_or(None)
        .as_deref()
        .is_some_and(|v| v == item.value.as_ref());
    let item_test_id = item_test_id_prefix.as_ref().map(|prefix| {
        Arc::<str>::from(format!(
            "{prefix}.item.{}",
            sanitize_test_id_segment(item.value.as_ref())
        ))
    });

    let (bg_hover, fg_hover, fg) = {
        let theme = Theme::global(&*cx.app);
        (
            theme.color_token("accent"),
            theme.color_token("accent-foreground"),
            theme.color_token("foreground"),
        )
    };

    let value_for_activate = item.value.clone();
    let model_for_activate = model.clone();
    let open_for_activate = open.clone();
    let query_for_activate = query.clone();
    let open_change_reason_for_activate = open_change_reason.clone();

    let mut el = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(density.row_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            focusable: true,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::ListBoxOption),
                label: Some(item.label.clone()),
                test_id: item_test_id.clone(),
                selected,
                pos_in_set: Some((idx as u32) + 1),
                set_size: Some(total as u32),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_activate(kit_combobox::commit_selection_on_activate(
                enum_select_selection_commit_policy(),
                model_for_activate.clone(),
                open_for_activate.clone(),
                query_for_activate.clone(),
                open_change_reason_for_activate.clone(),
                value_for_activate.clone(),
            ));

            let hovered = st.hovered || st.hovered_raw;
            let bg = if hovered || selected {
                Some(bg_hover)
            } else {
                None
            };
            let text_color = if hovered || selected { fg_hover } else { fg };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    padding: Edges::symmetric(density.padding_x, Px(0.0)).into(),
                    background: bg,
                    corner_radii: Corners::all(Px(6.0)),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.text_props(TextProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        text: item.label.clone(),
                        style: Some(typography::as_control_text(TextStyle {
                            size: Px(12.0),
                            line_height: Some(density.row_height),
                            ..Default::default()
                        })),
                        color: Some(text_color),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Ellipsis,
                        align: TextAlign::Start,
                        ink_overflow: Default::default(),
                    })]
                },
            )]
        },
    );

    if let Some(test_id) = item_test_id.as_ref() {
        el = el.test_id(test_id.clone());
    }

    let el_id = el.id;
    (el, el_id, selected)
}

fn enum_select_selection_commit_policy() -> kit_combobox::SelectionCommitPolicy {
    kit_combobox::SelectionCommitPolicy {
        toggle_selected_to_none: false,
        close_on_commit: true,
        clear_query_on_commit: true,
    }
}

fn enum_select_close_auto_focus_policy() -> kit_combobox::ComboboxCloseAutoFocusPolicy {
    kit_combobox::ComboboxCloseAutoFocusPolicy::default()
}

fn enum_select_viewport_test_id(list_test_id: &str) -> Arc<str> {
    Arc::from(format!("{list_test_id}.viewport"))
}

fn element_visible_within_viewport_y<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    viewport_element: GlobalElementId,
    child_element: GlobalElementId,
) -> Option<bool> {
    let viewport = cx.last_bounds_for_element(viewport_element)?;
    let child = cx.last_bounds_for_element(child_element)?;
    Some(rect_visible_within_viewport_y(viewport, child))
}

fn rect_visible_within_viewport_y(viewport: fret_core::Rect, child: fret_core::Rect) -> bool {
    let viewport_h = viewport.size.height.0.max(0.0);
    if viewport_h <= 0.0 {
        return false;
    }

    let view_top = viewport.origin.y.0;
    let view_bottom = view_top + viewport_h;
    let child_top = child.origin.y.0;
    let child_h = child.size.height.0.max(0.0);
    let child_bottom = child_top + child_h;

    if child_h >= viewport_h - 0.01 {
        child_top >= view_top - 0.01
    } else {
        child_top >= view_top - 0.01 && child_bottom <= view_bottom + 0.01
    }
}

fn sanitize_test_id_segment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut prev_dash = false;

    for ch in raw.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    while out.starts_with('-') {
        out.remove(0);
    }
    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        out.push_str("item");
    }

    out
}

#[track_caller]
fn open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    cx.local_model(|| false)
}

#[cfg(test)]
mod tests {
    use super::{
        enum_select_close_auto_focus_policy, enum_select_selection_commit_policy,
        enum_select_viewport_test_id, rect_visible_within_viewport_y, sanitize_test_id_segment,
    };
    use fret_core::{Point, Px, Rect, Size};
    use fret_ui_kit::primitives::combobox::{
        ComboboxCloseAutoFocusDecision, ComboboxCloseAutoFocusPolicy,
    };

    #[test]
    fn enum_select_item_test_id_segment_is_stable_ascii() {
        assert_eq!(sanitize_test_id_segment("Lit"), "lit");
        assert_eq!(
            sanitize_test_id_segment("Material / Matcap"),
            "material-matcap"
        );
        assert_eq!(sanitize_test_id_segment("  "), "item");
    }

    #[test]
    fn enum_select_commit_policy_does_not_toggle_selected_to_none() {
        let policy = enum_select_selection_commit_policy();

        assert!(!policy.toggle_selected_to_none);
        assert!(policy.close_on_commit);
        assert!(policy.clear_query_on_commit);
    }

    #[test]
    fn enum_select_close_focus_policy_matches_trigger_owned_combobox() {
        let policy: ComboboxCloseAutoFocusPolicy = enum_select_close_auto_focus_policy();

        assert_eq!(
            policy.on_item_press,
            ComboboxCloseAutoFocusDecision::RestoreTrigger
        );
        assert_eq!(
            policy.on_escape,
            ComboboxCloseAutoFocusDecision::RestoreTrigger
        );
        assert_eq!(
            policy.on_trigger_press,
            ComboboxCloseAutoFocusDecision::RestoreTrigger
        );
        assert_eq!(
            policy.on_outside_press,
            ComboboxCloseAutoFocusDecision::RestoreTrigger
        );
        assert_eq!(
            policy.on_focus_out,
            ComboboxCloseAutoFocusDecision::PreventDefault
        );
    }

    #[test]
    fn enum_select_viewport_test_id_suffixes_list_test_id() {
        assert_eq!(
            enum_select_viewport_test_id("editor.enum.list").as_ref(),
            "editor.enum.list.viewport"
        );
    }

    #[test]
    fn rect_visible_within_viewport_y_matches_nearest_visibility_contract() {
        let viewport = Rect::new(Point::new(Px(0.0), Px(10.0)), Size::new(Px(40.0), Px(40.0)));

        let fully_visible = Rect::new(Point::new(Px(0.0), Px(20.0)), Size::new(Px(40.0), Px(12.0)));
        assert!(rect_visible_within_viewport_y(viewport, fully_visible));

        let clipped_bottom =
            Rect::new(Point::new(Px(0.0), Px(42.0)), Size::new(Px(40.0), Px(16.0)));
        assert!(!rect_visible_within_viewport_y(viewport, clipped_bottom));

        let tall_child = Rect::new(Point::new(Px(0.0), Px(10.0)), Size::new(Px(40.0), Px(60.0)));
        assert!(rect_visible_within_viewport_y(viewport, tall_child));

        let tall_child_top_hidden =
            Rect::new(Point::new(Px(0.0), Px(4.0)), Size::new(Px(40.0), Px(60.0)));
        assert!(!rect_visible_within_viewport_y(
            viewport,
            tall_child_top_hidden
        ));
    }
}

#[track_caller]
fn filter_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    cx.local_model(String::new)
}
