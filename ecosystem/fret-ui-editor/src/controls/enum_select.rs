//! Filterable enum select control (editor-grade combobox-like widget).
//!
//! This is an ecosystem/policy control:
//! - it uses `fret-ui` mechanisms (pressable, focus, overlays),
//! - and `fret-ui-kit` infrastructure (overlay controller + popper placement),
//! - without depending on any design-system crate.

use std::panic::Location;
use std::sync::{Arc, Mutex};

use fret_core::{Axis, Corners, Edges, KeyCode, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, OnKeyDown};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, SizeStyle, SpacingLength,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::combobox as kit_combobox;

use crate::primitives::icons::editor_icon_with;
use crate::primitives::input_group::{
    editor_input_group_divider, editor_input_group_frame, editor_input_group_inset,
    editor_input_group_row, editor_input_value_text,
};
use crate::primitives::popup_surface::resolve_editor_popup_surface_chrome;
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState};

mod options;
mod overlay;
mod row;

pub use options::EnumSelectOptions;

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
                        let text_el = editor_input_value_text(
                            cx,
                            density,
                            Px(12.0),
                            trigger_text.clone(),
                            visuals.fg,
                            Length::Auto,
                        );
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
            overlay::request_overlay(
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
            overlay::request_overlay(
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

#[track_caller]
fn open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    cx.local_model(|| false)
}

#[track_caller]
fn filter_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    cx.local_model(String::new)
}
