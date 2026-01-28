//!
//! Outcome-oriented Material 3 Select (MVP).
//!
//! This intentionally focuses on:
//! - token-driven trigger/container outcomes via `md.comp.{outlined,filled}-select.*`,
//! - a minimal listbox overlay anchored to the trigger,
//! - an ADR 1159-shaped `SelectStyle` override surface.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, Edges, KeyCode, Point, Px, Rect, SemanticsRole, Size, SvgFit,
    TextOverflow, TextWrap,
};
use fret_icons::{IconId, IconRegistry, MISSING_ICON_SVG, ResolvedSvgOwned, ids};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CanvasProps, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, RovingFlexProps, ScrollProps,
    SemanticsProps, SvgIconProps, TextProps,
};
use fret_ui::elements::{ElementContext, GlobalElementId};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{Invalidation, SvgSource, Theme, UiHost};
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::{ColorRef, OverlayController, OverlayPresence};
use fret_ui_kit::{
    OverrideSlot, WidgetStateProperty, WidgetStates, merge_override_slot,
    resolve_override_slot_opt_with, resolve_override_slot_with,
};

use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::surface::material_surface_style;
use crate::motion::ms_to_frames;
use crate::tokens::dropdown_menu as dropdown_menu_tokens;
use crate::tokens::select as select_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectVariant {
    #[default]
    Outlined,
    Filled,
}

#[derive(Debug, Clone)]
pub struct SelectItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub disabled: bool,
    pub test_id: Option<Arc<str>>,
}

impl SelectItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
            test_id: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct SelectStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub outline_color: OverrideSlot<ColorRef>,
    pub active_indicator_color: OverrideSlot<ColorRef>,
    pub text_color: OverrideSlot<ColorRef>,
    pub trailing_icon_color: OverrideSlot<ColorRef>,
    pub menu_selected_container_color: OverrideSlot<ColorRef>,
}

impl SelectStyle {
    pub fn container_background(
        mut self,
        background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.container_background = Some(background);
        self
    }

    pub fn outline_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.outline_color = Some(color);
        self
    }

    pub fn active_indicator_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.active_indicator_color = Some(color);
        self
    }

    pub fn text_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn trailing_icon_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.trailing_icon_color = Some(color);
        self
    }

    pub fn menu_selected_container_color(
        mut self,
        color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.menu_selected_container_color = Some(color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        self.container_background =
            merge_override_slot(self.container_background, other.container_background);
        self.outline_color = merge_override_slot(self.outline_color, other.outline_color);
        self.active_indicator_color =
            merge_override_slot(self.active_indicator_color, other.active_indicator_color);
        self.text_color = merge_override_slot(self.text_color, other.text_color);
        self.trailing_icon_color =
            merge_override_slot(self.trailing_icon_color, other.trailing_icon_color);
        self.menu_selected_container_color = merge_override_slot(
            self.menu_selected_container_color,
            other.menu_selected_container_color,
        );
        self
    }
}

#[derive(Clone)]
pub struct Select {
    model: Model<Option<Arc<str>>>,
    items: Arc<[SelectItem]>,
    variant: SelectVariant,
    disabled: bool,
    placeholder: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    style: SelectStyle,
}

impl std::fmt::Debug for Select {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Select").finish()
    }
}

impl Select {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model,
            items: Arc::from([]),
            variant: SelectVariant::default(),
            disabled: false,
            placeholder: None,
            a11y_label: None,
            test_id: None,
            style: SelectStyle::default(),
        }
    }

    pub fn items(mut self, items: impl Into<Arc<[SelectItem]>>) -> Self {
        self.items = items.into();
        self
    }

    pub fn variant(mut self, variant: SelectVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn style(mut self, style: SelectStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        select_into_element(cx, self)
    }
}

#[derive(Clone)]
struct SelectRuntimeModels {
    open: Model<bool>,
}

fn select_runtime_models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> SelectRuntimeModels {
    #[derive(Default)]
    struct State {
        models: Option<SelectRuntimeModels>,
    }

    let existing = cx.with_state(State::default, |st| st.models.clone());
    if let Some(models) = existing {
        return models;
    }

    let models = SelectRuntimeModels {
        open: cx.app.models_mut().insert(false),
    };

    cx.with_state(State::default, |st| st.models = Some(models.clone()));
    models
}

fn select_into_element<H: UiHost>(cx: &mut ElementContext<'_, H>, select: Select) -> AnyElement {
    cx.scope(|cx| {
        let theme = Theme::global(&*cx.app).clone();
        let runtime = select_runtime_models(cx);

        let is_open = cx
            .get_model_copied(&runtime.open, Invalidation::Layout)
            .unwrap_or(false);

        let open_ticks = ms_to_frames(dropdown_menu_tokens::open_duration_ms(&theme));
        let close_ticks = ms_to_frames(dropdown_menu_tokens::close_duration_ms(&theme));
        let easing = dropdown_menu_tokens::easing(&theme);
        let motion = OverlayController::transition_with_durations_and_cubic_bezier(
            cx,
            is_open,
            open_ticks,
            close_ticks,
            easing,
        );
        let overlay_presence = OverlayPresence {
            present: motion.present,
            interactive: is_open,
        };

        let selected = cx
            .get_model_cloned(&select.model, Invalidation::Layout)
            .unwrap_or(None);
        let display_label = selected
            .as_ref()
            .and_then(|value| {
                select
                    .items
                    .iter()
                    .find(|it| it.value.as_ref() == value.as_ref())
            })
            .map(|it| it.label.clone())
            .or_else(|| selected.clone())
            .or_else(|| select.placeholder.clone())
            .unwrap_or_else(|| Arc::<str>::from(""));

        let trigger = select_trigger_element(
            cx,
            &theme,
            select.variant,
            select.disabled,
            overlay_presence.interactive,
            display_label,
            select.a11y_label.clone(),
            select.test_id.clone(),
            runtime.open.clone(),
            select.style.clone(),
        );

        if overlay_presence.present {
            let trigger_id = trigger.id;
            let Some(anchor) = fret_ui_kit::overlay::anchor_bounds_for_element(cx, trigger_id)
            else {
                return trigger;
            };

            let outer = fret_ui_kit::overlay::outer_bounds_with_window_margin(cx.bounds, Px(0.0));

            let min_width = Px(anchor.size.width.0.max(128.0));
            let item_height = select_tokens::menu_list_item_height(&theme, select.variant);
            let max_visible = 7.0;
            let max_height = Px(item_height.0 * max_visible);
            let estimated_height =
                Px((item_height.0 * (select.items.len().max(1) as f32)).min(max_height.0));
            let estimated = Size::new(min_width, estimated_height);

            let direction = direction_prim::use_direction_in_scope(cx, None);
            let placement =
                popper::PopperContentPlacement::new(direction, Side::Bottom, Align::Start, Px(4.0))
                    .with_collision_padding(dropdown_menu_tokens::collision_padding(&theme));

            let layout = popper::popper_content_layout_sized(outer, anchor, estimated, placement);

            let initial_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
            let initial_focus_id_for_list = initial_focus_id.clone();

            let open_model = runtime.open.clone();
            let open_model_for_panel = open_model.clone();
            let overlay_root = popper_content::popper_wrapper_panel_at(
                cx,
                layout.rect,
                Edges::all(Px(0.0)),
                Overflow::Visible,
                move |cx| {
                    vec![select_listbox_panel(
                        cx,
                        select.variant,
                        select.model.clone(),
                        open_model_for_panel.clone(),
                        select.items.clone(),
                        selected.clone(),
                        select.a11y_label.clone(),
                        select.test_id.clone(),
                        initial_focus_id_for_list,
                        select.style.clone(),
                    )]
                },
            );

            let opacity = motion.progress;
            let scale = 0.95 + 0.05 * motion.progress;
            let origin = popper::popper_content_transform_origin(&layout, anchor, None);
            let origin_inv = fret_core::Point::new(Px(-origin.x.0), Px(-origin.y.0));
            let transform = fret_core::Transform2D::translation(origin)
                * fret_core::Transform2D::scale_uniform(scale)
                * fret_core::Transform2D::translation(origin_inv);
            let overlay_root =
                fret_ui_kit::declarative::overlay_motion::wrap_opacity_and_render_transform_gated(
                    cx,
                    opacity,
                    transform,
                    overlay_presence.interactive,
                    vec![overlay_root],
                );

            let mut request = fret_ui_kit::overlay_controller::OverlayRequest::dismissible_menu(
                trigger_id,
                trigger_id,
                open_model.clone(),
                overlay_presence,
                vec![overlay_root],
            );
            request.root_name = Some(format!("material3.select.{}", trigger_id.0));
            request.close_on_window_focus_lost = true;
            request.close_on_window_resize = true;
            request.initial_focus = initial_focus_id.get();

            OverlayController::request(cx, request);
        }

        trigger
    })
}

fn select_trigger_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    variant: SelectVariant,
    disabled: bool,
    open: bool,
    label: Arc<str>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    open_model: Model<bool>,
    style: SelectStyle,
) -> AnyElement {
    cx.pressable_with_id_props(move |cx, st, pressable_id| {
        let enabled = !disabled;

        let mut states = WidgetStates::from_pressable(cx, st, enabled);
        if open {
            states |= WidgetStates::OPEN;
        }

        let toggle_open = open_model.clone();
        let enabled_for_toggle = enabled;
        cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
            if !enabled_for_toggle {
                return;
            }
            let _ = host.models_mut().update(&toggle_open, |v| *v = !*v);
            host.request_redraw(action_cx.window);
        }));

        let enabled_for_key = enabled;
        let open_for_key = open;
        let toggle_open_on_key = open_model.clone();
        cx.key_add_on_key_down_for(
            pressable_id,
            Arc::new(move |host, action_cx, down| {
                if !enabled_for_key {
                    return false;
                }

                match down.key {
                    KeyCode::ArrowDown | KeyCode::ArrowUp => {
                        if !open_for_key {
                            let _ = host.models_mut().update(&toggle_open_on_key, |v| *v = true);
                            host.request_redraw(action_cx.window);
                        }
                        true
                    }
                    KeyCode::Escape => {
                        if open_for_key {
                            let _ = host
                                .models_mut()
                                .update(&toggle_open_on_key, |v| *v = false);
                            host.request_redraw(action_cx.window);
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            }),
        );

        let corner = select_tokens::container_corner(theme, variant);
        let token_container_bg = Some(select_tokens::container_background(
            theme, variant, !enabled,
        ));
        let container_bg = resolve_override_slot_opt_with(
            style.container_background.as_ref(),
            states,
            |color| color.resolve(theme),
            || token_container_bg,
        );

        let focused = states.contains(WidgetStates::FOCUS_VISIBLE);
        let hovered = enabled && st.hovered;

        let (token_text_color, token_text_opacity) =
            select_tokens::input_text_color(theme, variant, focused, hovered, !enabled);
        let text_color = resolve_override_slot_with(
            style.text_color.as_ref(),
            states,
            |color| color.resolve(theme),
            || token_text_color,
        );
        let text_color = with_opacity(text_color, token_text_opacity);

        let (token_icon_color, token_icon_opacity) =
            select_tokens::trailing_icon_color(theme, variant, focused, hovered, !enabled);
        let icon_color = resolve_override_slot_with(
            style.trailing_icon_color.as_ref(),
            states,
            |color| color.resolve(theme),
            || token_icon_color,
        );
        let icon_opacity = token_icon_opacity.clamp(0.0, 1.0);

        let outline = select_tokens::outline(theme, variant, focused, hovered, !enabled)
            .map(|(w, c, opacity)| (w, with_opacity(c, opacity)));
        let outline = outline.map(|(w, c)| {
            let c = resolve_override_slot_opt_with(
                style.outline_color.as_ref(),
                states,
                |color| color.resolve(theme),
                || Some(c),
            )
            .unwrap_or(c);
            (w, c)
        });

        let indicator = select_tokens::active_indicator(theme, variant, focused, hovered, !enabled)
            .map(|(h, c, opacity)| (h, with_opacity(c, opacity)));
        let indicator = indicator.map(|(h, c)| {
            let c = resolve_override_slot_opt_with(
                style.active_indicator_color.as_ref(),
                states,
                |color| color.resolve(theme),
                || Some(c),
            )
            .unwrap_or(c);
            (h, c)
        });

        let a11y = PressableA11y {
            role: Some(SemanticsRole::ComboBox),
            label: a11y_label.clone(),
            test_id: test_id.clone(),
            expanded: Some(open),
            ..Default::default()
        };

        let pressable_props = PressableProps {
            enabled,
            focusable: enabled,
            a11y,
            layout: {
                let mut l = fret_ui::element::LayoutStyle::default();
                l.size.width = Length::Fill;
                l.size.height = Length::Px(select_tokens::container_height(theme, variant));
                l.overflow = Overflow::Visible;
                l
            },
            focus_ring: None,
            focus_ring_bounds: None,
        };

        let pointer_region = cx.named("pointer_region", |cx| {
            let mut props = PointerRegionProps::default();
            props.enabled = enabled;
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                let now_frame = cx.frame_id.0;
                let (state_layer_color, hover_opacity) =
                    select_tokens::hover_state_layer(theme, variant);

                let state_layer_target = if enabled && st.pressed {
                    theme
                        .number_by_key("md.sys.state.pressed.state-layer.opacity")
                        .unwrap_or(0.1)
                } else if enabled && focused {
                    theme
                        .number_by_key("md.sys.state.focus.state-layer.opacity")
                        .unwrap_or(0.1)
                } else if hovered {
                    hover_opacity
                } else {
                    0.0
                };

                let ripple_base_opacity = theme
                    .number_by_key("md.sys.state.pressed.state-layer.opacity")
                    .unwrap_or(0.1);
                let config = material_pressable_indication_config(theme, None);
                let overlay = material_ink_layer_for_pressable(
                    cx,
                    pressable_id,
                    now_frame,
                    corner,
                    RippleClip::Bounded,
                    state_layer_color,
                    enabled && st.pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    false,
                );

                let text_el = {
                    let mut props = TextProps::new(label.clone());
                    props.style = select_tokens::input_text_style(theme, variant);
                    props.color = Some(text_color);
                    props.wrap = TextWrap::None;
                    props.overflow = TextOverflow::Ellipsis;
                    cx.text_props(props)
                };

                let icon_el = chevron_down_icon(
                    cx,
                    icon_color,
                    icon_opacity,
                    select_tokens::trailing_icon_size(theme, variant),
                );

                let mut row = FlexProps::default();
                row.layout.size.width = Length::Fill;
                row.layout.size.height = Length::Fill;
                row.layout.overflow = Overflow::Clip;
                row.direction = Axis::Horizontal;
                row.justify = MainAlign::SpaceBetween;
                row.align = CrossAlign::Center;
                row.padding = Edges {
                    left: Px(16.0),
                    right: Px(12.0),
                    top: Px(0.0),
                    bottom: Px(0.0),
                };

                let mut chrome = ContainerProps::default();
                chrome.layout.size.width = Length::Fill;
                chrome.layout.size.height = Length::Fill;
                chrome.layout.overflow = Overflow::Clip;
                chrome.background = container_bg;
                chrome.corner_radii = corner;
                if let Some((outline_width, outline_color)) = outline {
                    if outline_width.0 > 0.0 {
                        chrome.border = Edges::all(outline_width);
                        chrome.border_color = Some(outline_color);
                    }
                }

                let indicator_el = indicator.map(|(h, c)| {
                    cx.canvas(CanvasProps::default(), move |p| {
                        let bounds = p.bounds();
                        let y = Px(bounds.origin.y.0 + bounds.size.height.0 - h.0);
                        let rect = Rect::new(
                            Point::new(bounds.origin.x, y),
                            Size::new(bounds.size.width, h),
                        );
                        p.scene().push(fret_core::SceneOp::Quad {
                            order: fret_core::DrawOrder(0),
                            rect,
                            background: c,
                            border: Edges::all(Px(0.0)),
                            border_color: Color::TRANSPARENT,
                            corner_radii: Corners::all(Px(0.0)),
                        });
                    })
                });

                vec![cx.container(chrome, move |cx| {
                    let mut children =
                        vec![overlay, cx.flex(row, move |_cx| vec![text_el, icon_el])];
                    if let Some(indicator_el) = indicator_el {
                        children.push(indicator_el);
                    }
                    children
                })]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}

fn chevron_down_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    opacity: f32,
    size: Px,
) -> AnyElement {
    let svg = svg_source_for_icon(cx, &ids::ui::CHEVRON_DOWN);

    let mut props = SvgIconProps::new(svg);
    props.fit = SvgFit::Contain;
    props.color = color;
    props.opacity = opacity;
    props.layout.size.width = Length::Px(size);
    props.layout.size.height = Length::Px(size);
    cx.svg_icon_props(props)
}

fn svg_source_for_icon<H: UiHost>(cx: &mut ElementContext<'_, H>, icon: &IconId) -> SvgSource {
    let resolved = cx
        .app
        .with_global_mut(IconRegistry::default, |icons, _app| {
            icons
                .resolve_svg_owned(icon)
                .unwrap_or(ResolvedSvgOwned::Static(MISSING_ICON_SVG))
        });

    match resolved {
        ResolvedSvgOwned::Static(bytes) => SvgSource::Static(bytes),
        ResolvedSvgOwned::Bytes(bytes) => SvgSource::Bytes(bytes),
    }
}

fn with_opacity(mut color: Color, opacity: f32) -> Color {
    color.a = (color.a * opacity).clamp(0.0, 1.0);
    color
}

fn select_listbox_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: SelectVariant,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    items: Arc<[SelectItem]>,
    selected: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    initial_focus_id_out: Rc<Cell<Option<GlobalElementId>>>,
    style: SelectStyle,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let listbox_test_id = test_id.as_ref().map(|id| {
        let id = id.as_ref();
        Arc::<str>::from(format!("{id}-listbox"))
    });

    let sem = SemanticsProps {
        role: SemanticsRole::ListBox,
        label: a11y_label.clone(),
        test_id: listbox_test_id.or_else(|| Some(Arc::<str>::from("material3-select-listbox"))),
        ..Default::default()
    };

    let disabled: Arc<[bool]> = Arc::from(items.iter().map(|it| it.disabled).collect::<Vec<_>>());
    let count = items.len();

    let tab_stop_idx = selected
        .as_ref()
        .and_then(|value| {
            items
                .iter()
                .position(|it| it.value.as_ref() == value.as_ref())
        })
        .filter(|&idx| !items.get(idx).is_some_and(|it| it.disabled))
        .or_else(|| disabled.iter().position(|&d| !d))
        .unwrap_or(0);

    let mut roving = RovingFlexProps::default();
    roving.flex.direction = Axis::Vertical;
    roving.flex.gap = Px(0.0);
    roving.flex.align = CrossAlign::Stretch;
    roving.flex.justify = MainAlign::Start;
    roving.roving = fret_ui::element::RovingFocusProps {
        enabled: true,
        wrap: true,
        disabled: disabled.clone(),
    };

    let container_bg = select_tokens::menu_container_background(&theme, variant);
    let elevation = select_tokens::menu_container_elevation(&theme, variant);
    let shadow_color = select_tokens::menu_container_shadow_color(&theme, variant);
    let r = select_tokens::menu_container_shape_radius(&theme, variant);
    let corner = Corners::all(r);
    let surface =
        material_surface_style(&theme, container_bg, elevation, Some(shadow_color), corner);

    let selected_bg_token = select_tokens::menu_list_item_selected_container_color(&theme, variant);
    let selected_bg = resolve_override_slot_with(
        style.menu_selected_container_color.as_ref(),
        WidgetStates::SELECTED,
        |color| color.resolve(&theme),
        || selected_bg_token,
    );

    cx.semantics(sem, move |cx| {
        vec![cx.container(
            ContainerProps {
                background: Some(surface.background),
                shadow: surface.shadow,
                corner_radii: corner,
                layout: {
                    let mut l = fret_ui::element::LayoutStyle::default();
                    l.size.width = Length::Fill;
                    l.size.height = Length::Fill;
                    l.overflow = Overflow::Clip;
                    l
                },
                ..Default::default()
            },
            move |cx| {
                vec![cx.scroll(
                    ScrollProps {
                        layout: {
                            let mut l = fret_ui::element::LayoutStyle::default();
                            l.size.width = Length::Fill;
                            l.size.height = Length::Fill;
                            l.overflow = Overflow::Clip;
                            l
                        },
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.roving_flex(roving, move |cx| {
                            cx.roving_on_navigate(Arc::new(|_host, _cx, it| {
                                use fret_ui::action::RovingNavigateResult;

                                let is_disabled = |idx: usize| -> bool {
                                    it.disabled.get(idx).copied().unwrap_or(false)
                                };

                                let forward = match it.key {
                                    KeyCode::ArrowDown => Some(true),
                                    KeyCode::ArrowUp => Some(false),
                                    _ => None,
                                };

                                if it.key == KeyCode::Home {
                                    let target = (0..it.len).find(|&i| !is_disabled(i));
                                    return RovingNavigateResult::Handled { target };
                                }
                                if it.key == KeyCode::End {
                                    let target = (0..it.len).rev().find(|&i| !is_disabled(i));
                                    return RovingNavigateResult::Handled { target };
                                }

                                let Some(forward) = forward else {
                                    return RovingNavigateResult::NotHandled;
                                };

                                let current = it
                                    .current
                                    .or_else(|| (0..it.len).find(|&i| !is_disabled(i)));
                                let Some(current) = current else {
                                    return RovingNavigateResult::Handled { target: None };
                                };

                                let len = it.len;
                                let mut target: Option<usize> = None;
                                if it.wrap {
                                    for step in 1..=len {
                                        let idx = if forward {
                                            (current + step) % len
                                        } else {
                                            (current + len - (step % len)) % len
                                        };
                                        if !is_disabled(idx) {
                                            target = Some(idx);
                                            break;
                                        }
                                    }
                                } else if forward {
                                    target = ((current + 1)..len).find(|&i| !is_disabled(i));
                                } else if current > 0 {
                                    target = (0..current).rev().find(|&i| !is_disabled(i));
                                }

                                RovingNavigateResult::Handled { target }
                            }));

                            let mut out: Vec<AnyElement> = Vec::with_capacity(count);
                            for (idx, item) in items.iter().cloned().enumerate() {
                                let tab_stop = idx == tab_stop_idx;
                                out.push(select_list_item(
                                    cx,
                                    &theme,
                                    variant,
                                    item,
                                    model.clone(),
                                    open.clone(),
                                    selected.clone(),
                                    selected_bg,
                                    tab_stop,
                                    idx,
                                    count,
                                    initial_focus_id_out.clone(),
                                ));
                            }
                            out
                        })]
                    },
                )]
            },
        )]
    })
}

fn select_list_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    variant: SelectVariant,
    item: SelectItem,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    selected: Option<Arc<str>>,
    selected_bg: Color,
    tab_stop: bool,
    idx: usize,
    set_size: usize,
    initial_focus_id_out: Rc<Cell<Option<GlobalElementId>>>,
) -> AnyElement {
    let height = select_tokens::menu_list_item_height(theme, variant);

    cx.pressable_with_id_props(move |cx, st, pressable_id| {
        let enabled = !item.disabled;

        if enabled && tab_stop && initial_focus_id_out.get().is_none() {
            initial_focus_id_out.set(Some(pressable_id));
        }

        let is_selected = selected
            .as_ref()
            .is_some_and(|value| value.as_ref() == item.value.as_ref());

        let a11y = PressableA11y {
            role: Some(SemanticsRole::ListBoxOption),
            label: Some(item.label.clone()),
            test_id: item.test_id.clone(),
            selected: is_selected,
            pos_in_set: Some((idx + 1) as u32),
            set_size: Some(set_size as u32),
            ..Default::default()
        };

        let model_for_select = model.clone();
        let open_for_close = open.clone();
        let value_for_select = item.value.clone();
        let enabled_for_select = enabled;
        cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
            if !enabled_for_select {
                return;
            }
            let _ = host
                .models_mut()
                .update(&model_for_select, |v| *v = Some(value_for_select.clone()));
            let _ = host.models_mut().update(&open_for_close, |v| *v = false);
            host.request_redraw(action_cx.window);
        }));

        let pressable_props = PressableProps {
            enabled,
            focusable: enabled && tab_stop,
            a11y,
            layout: {
                let mut l = fret_ui::element::LayoutStyle::default();
                l.size.width = Length::Fill;
                l.size.height = Length::Px(height);
                l.overflow = Overflow::Visible;
                l
            },
            focus_ring: None,
            focus_ring_bounds: None,
        };

        let pointer_region = cx.named("pointer_region", |cx| {
            let mut props = PointerRegionProps::default();
            props.enabled = enabled;
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            cx.pointer_region(props, |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                let now_frame = cx.frame_id.0;
                let state_layer_color = theme
                    .color_by_key("md.sys.color.on-surface")
                    .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
                let state_layer_target = if enabled && st.pressed {
                    theme
                        .number_by_key("md.sys.state.pressed.state-layer.opacity")
                        .unwrap_or(0.1)
                } else if enabled && st.hovered {
                    theme
                        .number_by_key("md.sys.state.hover.state-layer.opacity")
                        .unwrap_or(0.08)
                } else if enabled
                    && st.focused
                    && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window))
                {
                    theme
                        .number_by_key("md.sys.state.focus.state-layer.opacity")
                        .unwrap_or(0.1)
                } else {
                    0.0
                };
                let ripple_base_opacity = theme
                    .number_by_key("md.sys.state.pressed.state-layer.opacity")
                    .unwrap_or(0.1);
                let config = material_pressable_indication_config(theme, None);
                let overlay = material_ink_layer_for_pressable(
                    cx,
                    pressable_id,
                    now_frame,
                    Corners::all(Px(0.0)),
                    RippleClip::Bounded,
                    state_layer_color,
                    enabled && st.pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    false,
                );

                let label_color = select_tokens::menu_list_item_label_text_color(theme, variant);
                let label_style = select_tokens::menu_list_item_label_text_style(theme, variant);

                let label_el = {
                    let mut props = TextProps::new(item.label.clone());
                    props.style = label_style;
                    props.color = Some(label_color);
                    props.wrap = TextWrap::None;
                    props.overflow = TextOverflow::Clip;
                    cx.text_props(props)
                };

                let bg = if is_selected { Some(selected_bg) } else { None };
                let mut chrome = ContainerProps::default();
                chrome.background = bg;
                chrome.layout.size.width = Length::Fill;
                chrome.layout.size.height = Length::Px(height);
                chrome.layout.overflow = Overflow::Clip;

                let mut row = FlexProps::default();
                row.layout.size.width = Length::Fill;
                row.layout.size.height = Length::Px(height);
                row.layout.overflow = Overflow::Clip;
                row.direction = Axis::Horizontal;
                row.justify = MainAlign::Start;
                row.align = CrossAlign::Center;
                row.padding = Edges {
                    left: Px(12.0),
                    right: Px(12.0),
                    top: Px(0.0),
                    bottom: Px(0.0),
                };

                vec![cx.container(chrome, move |cx| {
                    vec![cx.flex(row, move |_cx| vec![overlay, label_el])]
                })]
            })
        });

        (pressable_props, vec![pointer_region])
    })
}
