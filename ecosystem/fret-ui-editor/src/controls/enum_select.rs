//! Filterable enum select control (editor-grade combobox-like widget).
//!
//! This is an ecosystem/policy control:
//! - it uses `fret-ui` mechanisms (pressable, focus, overlays),
//! - and `fret-ui-kit` infrastructure (overlay controller + popper placement),
//! - without depending on any design-system crate.

use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Corners, Edges, KeyCode, Px, TextAlign, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, OnCloseAutoFocus, OnKeyDown};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PointerRegionProps, PressableA11y, PressableProps, ScrollAxis, ScrollProps, SizeStyle,
    TextProps,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::popper;
use fret_ui_kit::{OverlayController, OverlayPresence, OverlayRequest};

use crate::controls::MiniSearchBox;
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
        let open = open_model(cx);
        let filter = filter_model(cx);

        let is_open = cx
            .get_model_copied(&open, Invalidation::Layout)
            .unwrap_or(false);

        let should_clear_filter = cx.with_state(
            || false,
            |last_open| {
                let should_clear = !*last_open && is_open;
                *last_open = is_open;
                should_clear
            },
        );
        if should_clear_filter {
            let _ = cx.app.models_mut().update(&filter, |s| s.clear());
        }

        let selected_value = cx
            .get_model_cloned(&self.model, Invalidation::Paint)
            .unwrap_or(None);

        let (density, bg, border, fg, ring) = {
            let theme = Theme::global(&*cx.app);
            let density = EditorDensity::resolve(theme);
            let bg = theme
                .color_by_key("popover")
                .or_else(|| theme.color_by_key("component.input.bg"))
                .unwrap_or_else(|| theme.color_token("background"));
            let border = theme
                .color_by_key("border")
                .or_else(|| theme.color_by_key("component.input.border"))
                .unwrap_or_else(|| theme.color_token("foreground"));
            let fg = theme
                .color_by_key("foreground")
                .unwrap_or_else(|| theme.color_token("foreground"));
            let ring = theme
                .color_by_key("ring")
                .unwrap_or_else(|| theme.color_token("primary"));
            (density, bg, border, fg, ring)
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
            trigger_layout.size.min_height = Some(density.row_height);
        }

        let trigger_model = self.model.clone();
        let items_for_overlay = self.items.clone();
        let options_for_overlay = self.options.clone();
        let open_for_overlay = open.clone();

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
                    corner_radii: Corners::all(Px(6.0)),
                }),
                ..Default::default()
            },
            move |cx, _st| {
                let open = open_for_overlay.clone();
                let on_activate: OnActivate =
                    Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
                        let prev = host.models_mut().get_copied(&open).unwrap_or(false);
                        let _ = host.models_mut().update(&open, |v| *v = !prev);
                        host.request_redraw(action_cx.window);
                    });
                cx.pressable_add_on_activate(on_activate);

                // ASCII fallback (avoid missing-glyph tofu on default fonts).
                let caret = if is_open { "^" } else { "v" };
                let caret_fg = Theme::global(&*cx.app)
                    .color_by_key("muted-foreground")
                    .or_else(|| Theme::global(&*cx.app).color_by_key("muted_foreground"))
                    .unwrap_or_else(|| Theme::global(&*cx.app).color_token("foreground"));

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
                        padding: Edges::symmetric(density.padding_x, density.padding_y),
                        background: Some(bg),
                        border: Edges::all(Px(1.0)),
                        border_color: Some(border),
                        corner_radii: Corners::all(Px(6.0)),
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
                                gap: Px(6.0),
                                padding: Edges::all(Px(0.0)),
                                justify: MainAlign::Start,
                                align: CrossAlign::Center,
                                wrap: false,
                            },
                            move |cx| {
                                vec![
                                    cx.text_props(TextProps {
                                        layout: LayoutStyle {
                                            size: SizeStyle {
                                                width: Length::Fill,
                                                height: Length::Auto,
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        text: trigger_text.clone(),
                                        style: Some(TextStyle {
                                            size: Px(12.0),
                                            line_height: Some(density.row_height),
                                            ..Default::default()
                                        }),
                                        color: Some(fg),
                                        wrap: TextWrap::None,
                                        overflow: TextOverflow::Ellipsis,
                                        align: TextAlign::Start,
                                    }),
                                    cx.spacer(Default::default()),
                                    cx.text_props(TextProps {
                                        layout: LayoutStyle {
                                            size: SizeStyle {
                                                width: Length::Px(density.hit_thickness),
                                                height: Length::Fill,
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        text: Arc::from(caret),
                                        style: Some(TextStyle {
                                            size: Px(12.0),
                                            line_height: Some(density.row_height),
                                            ..Default::default()
                                        }),
                                        color: Some(caret_fg),
                                        wrap: TextWrap::None,
                                        overflow: TextOverflow::Clip,
                                        align: TextAlign::Center,
                                    }),
                                ]
                            },
                        )]
                    },
                )]
            },
        );

        let trigger_id = trigger.id;

        let enabled_for_keys = self.options.enabled;
        let on_trigger_open_keys: OnKeyDown = Arc::new({
            let open = open.clone();
            move |host, action_cx: ActionCx, down| {
                if !enabled_for_keys {
                    return false;
                }
                if matches!(
                    down.key,
                    KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space | KeyCode::ArrowDown
                ) {
                    let _ = host.models_mut().update(&open, |v| *v = true);
                    host.request_redraw(action_cx.window);
                    return true;
                }
                if down.key == KeyCode::Escape {
                    let _ = host.models_mut().update(&open, |v| *v = false);
                    host.request_redraw(action_cx.window);
                    return true;
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
                options_for_overlay,
                density,
                bg,
                border,
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
                options_for_overlay,
                density,
                bg,
                border,
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
    options: EnumSelectOptions,
    density: EditorDensity,
    bg: fret_core::Color,
    border: fret_core::Color,
) {
    let model_for_list = model.clone();
    let open_for_list = open.clone();
    let list_test_id = options.list_test_id.clone();
    let search_test_id = options.search_test_id.clone();

    let overlay_id = cx
        .named("enum_select.overlay", |cx| cx.spacer(Default::default()))
        .id;

    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let presence = OverlayPresence::instant(is_open);

    let close_focus: OnCloseAutoFocus = Arc::new(move |host, _cx, req| {
        req.prevent_default();
        host.request_focus(trigger_id);
    });

    let (max_h, shadow_color) = {
        let theme = Theme::global(&*cx.app);
        let max_h = options
            .max_list_height
            .or_else(|| theme.metric_by_key(EditorTokenKeys::ENUM_SELECT_MAX_LIST_HEIGHT))
            .unwrap_or(Px(240.0));
        let shadow_color = theme.color_token("muted");
        (max_h, shadow_color)
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
                    width: Length::Fill,
                    height: Length::Fill,
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
                        ..Default::default()
                    },
                    padding: Edges::all(Px(8.0)),
                    background: Some(bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border),
                    corner_radii: Corners::all(Px(8.0)),
                    shadow: Some(fret_ui::element::ShadowStyle {
                        primary: fret_ui::element::ShadowLayerStyle {
                            color: shadow_color,
                            offset_x: Px(0.0),
                            offset_y: Px(6.0),
                            blur: Px(16.0),
                            spread: Px(-4.0),
                        },
                        secondary: None,
                        corner_radii: Corners::all(Px(8.0)),
                    }),
                    ..Default::default()
                },
                move |cx| {
                    let mut out = Vec::new();

                    let mut search = MiniSearchBox::new(filter.clone()).into_element(cx);
                    if let Some(test_id) = search_test_id.as_ref() {
                        search = search.test_id(test_id.clone());
                    }
                    out.push(search);

                    let scroll = cx.scroll(
                        ScrollProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Px(max_h),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            axis: ScrollAxis::Y,
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
                                    gap: Px(2.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Stretch,
                                    wrap: false,
                                },
                                move |cx| {
                                    if filtered.is_empty() {
                                        return vec![cx.text("No matches")];
                                    }

                                    filtered
                                        .iter()
                                        .enumerate()
                                        .map(|(idx, it)| {
                                            enum_select_row(
                                                cx,
                                                idx,
                                                filtered.len(),
                                                &model_for_list,
                                                &open_for_list,
                                                it.clone(),
                                                density,
                                            )
                                        })
                                        .collect::<Vec<_>>()
                                },
                            )]
                        },
                    );
                    out.push(scroll);
                    out
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

    let mut request = OverlayRequest::dismissible_menu(
        overlay_id,
        trigger_id,
        open,
        presence,
        vec![cx.pointer_region(
            PointerRegionProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                enabled: true,
                capture_phase_pointer_moves: false,
            },
            move |_cx| vec![list],
        )],
    );
    request.close_on_window_focus_lost = true;
    request.close_on_window_resize = true;
    request.on_close_auto_focus = Some(close_focus);

    OverlayController::request(cx, request);
}

fn enum_select_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    idx: usize,
    total: usize,
    model: &Model<Option<Arc<str>>>,
    open: &Model<bool>,
    item: EnumSelectItem,
    density: EditorDensity,
) -> AnyElement {
    let selected = cx
        .get_model_cloned(model, Invalidation::Paint)
        .unwrap_or(None)
        .as_deref()
        .is_some_and(|v| v == item.value.as_ref());

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

    cx.pressable(
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
                selected,
                pos_in_set: Some((idx as u32) + 1),
                set_size: Some(total as u32),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            let on_activate: OnActivate =
                Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
                    let _ = host.models_mut().update(&model_for_activate, |v| {
                        *v = Some(value_for_activate.clone());
                    });
                    let _ = host.models_mut().update(&open_for_activate, |v| *v = false);
                    host.request_redraw(action_cx.window);
                });
            cx.pressable_add_on_activate(on_activate);

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
                    padding: Edges::symmetric(density.padding_x, Px(0.0)),
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
                        style: Some(TextStyle {
                            size: Px(12.0),
                            line_height: Some(density.row_height),
                            ..Default::default()
                        }),
                        color: Some(text_color),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Ellipsis,
                        align: TextAlign::Start,
                    })]
                },
            )]
        },
    )
}

fn open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let m = cx.with_state(|| None::<Model<bool>>, |st| st.clone());
    match m {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(false);
            cx.with_state(
                || None::<Model<bool>>,
                |st| {
                    if st.is_none() {
                        *st = Some(m.clone());
                    }
                },
            );
            m
        }
    }
}

fn filter_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let m = cx.with_state(|| None::<Model<String>>, |st| st.clone());
    match m {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(String::new());
            cx.with_state(
                || None::<Model<String>>,
                |st| {
                    if st.is_none() {
                        *st = Some(m.clone());
                    }
                },
            );
            m
        }
    }
}
