//! Material 3 search bar (MVP).
//!
//! Token-driven outcome alignment via `md.comp.search-bar.*` (Material Web v30).

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, SemanticsRole, SvgFit, TextStyle};
use fret_icons::{IconId, IconRegistry, MISSING_ICON_SVG, ResolvedSvgOwned};
use fret_runtime::Model;
use fret_ui::action::{PointerDownCx, PressablePointerDownResult, UiPointerActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, SvgIconProps, TextInputProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{GlobalElementId, SvgSource, Theme, UiHost};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetState, WidgetStateProperty, WidgetStates,
    resolve_override_slot_with,
};

use crate::foundation::elevation::shadow_for_elevation_with_color;
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable_with_last_down,
    material_pressable_indication_config_in_scope,
};
use crate::foundation::strings::{
    material_search_bar_search_label, material_search_bar_suggestions_available_label,
};
use crate::foundation::style_overrides::merge_style_override_slots;
use crate::foundation::test_id::part_test_id;
use crate::tokens::search_bar as search_bar_tokens;
use crate::tokens::search_view as search_view_tokens;

fn search_bar_color_override(
    theme: &Theme,
    slot: &OverrideSlot<ColorRef>,
    states: WidgetStates,
    fallback: impl FnOnce() -> Color,
) -> Color {
    resolve_override_slot_with(
        slot.as_ref(),
        states,
        |color| color.resolve(theme),
        fallback,
    )
}

fn search_bar_metric_override(
    slot: &OverrideSlot<Px>,
    states: WidgetStates,
    fallback: impl FnOnce() -> Px,
) -> Px {
    resolve_override_slot_with(slot.as_ref(), states, |value| *value, fallback)
}

fn search_bar_edges_override(
    slot: &OverrideSlot<Edges>,
    states: WidgetStates,
    fallback: impl FnOnce() -> Edges,
) -> Edges {
    resolve_override_slot_with(slot.as_ref(), states, |value| *value, fallback)
}

fn search_bar_corners_override(
    slot: &OverrideSlot<Corners>,
    states: WidgetStates,
    fallback: impl FnOnce() -> Corners,
) -> Corners {
    resolve_override_slot_with(slot.as_ref(), states, |value| *value, fallback)
}

fn search_bar_text_style_override(
    slot: &OverrideSlot<TextStyle>,
    states: WidgetStates,
    fallback: impl FnOnce() -> TextStyle,
) -> TextStyle {
    resolve_override_slot_with(slot.as_ref(), states, |style| style.clone(), fallback)
}

fn search_bar_widget_states(
    enabled: bool,
    hovered: bool,
    pressed: bool,
    expanded: bool,
) -> WidgetStates {
    let mut states = WidgetStates::empty();
    states.set(WidgetState::Disabled, !enabled);
    states.set(WidgetState::Hovered, enabled && hovered);
    states.set(WidgetState::Active, enabled && pressed);
    states.set(WidgetState::Open, enabled && expanded);
    states
}

#[derive(Debug, Clone)]
struct SearchBarPartTestIds {
    chrome: Arc<str>,
    leading_icon: Arc<str>,
    trailing_icon: Arc<str>,
}

impl SearchBarPartTestIds {
    fn from_base(base: &Arc<str>) -> Self {
        Self {
            chrome: part_test_id(base, "chrome"),
            leading_icon: part_test_id(base, "leading-icon"),
            trailing_icon: part_test_id(base, "trailing-icon"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum SearchBarHeaderTokens {
    #[default]
    SearchBar,
    SearchView,
}

#[derive(Debug, Clone, Default)]
pub struct SearchBarStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub container_elevation: OverrideSlot<Px>,
    pub container_corner_radii: OverrideSlot<Corners>,
    pub container_height: OverrideSlot<Px>,
    pub container_min_width: OverrideSlot<Px>,
    pub container_max_width: OverrideSlot<Px>,
    pub content_padding: OverrideSlot<Edges>,
    pub content_gap: OverrideSlot<Px>,
    pub input_text_color: OverrideSlot<ColorRef>,
    pub supporting_text_color: OverrideSlot<ColorRef>,
    pub input_text_style: OverrideSlot<TextStyle>,
    pub leading_icon_color: OverrideSlot<ColorRef>,
    pub trailing_icon_color: OverrideSlot<ColorRef>,
    pub state_layer_color: OverrideSlot<ColorRef>,
}

impl SearchBarStyle {
    pub fn container_background(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.container_background = Some(color);
        self
    }

    pub fn container_elevation(mut self, elevation: WidgetStateProperty<Option<Px>>) -> Self {
        self.container_elevation = Some(elevation);
        self
    }

    pub fn container_corner_radii(mut self, corners: WidgetStateProperty<Option<Corners>>) -> Self {
        self.container_corner_radii = Some(corners);
        self
    }

    pub fn container_height(mut self, height: WidgetStateProperty<Option<Px>>) -> Self {
        self.container_height = Some(height);
        self
    }

    pub fn container_min_width(mut self, width: WidgetStateProperty<Option<Px>>) -> Self {
        self.container_min_width = Some(width);
        self
    }

    pub fn container_max_width(mut self, width: WidgetStateProperty<Option<Px>>) -> Self {
        self.container_max_width = Some(width);
        self
    }

    pub fn content_padding(mut self, padding: WidgetStateProperty<Option<Edges>>) -> Self {
        self.content_padding = Some(padding);
        self
    }

    pub fn content_gap(mut self, gap: WidgetStateProperty<Option<Px>>) -> Self {
        self.content_gap = Some(gap);
        self
    }

    pub fn input_text_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.input_text_color = Some(color);
        self
    }

    pub fn supporting_text_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.supporting_text_color = Some(color);
        self
    }

    pub fn input_text_style(mut self, style: WidgetStateProperty<Option<TextStyle>>) -> Self {
        self.input_text_style = Some(style);
        self
    }

    pub fn leading_icon_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.leading_icon_color = Some(color);
        self
    }

    pub fn trailing_icon_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.trailing_icon_color = Some(color);
        self
    }

    pub fn state_layer_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.state_layer_color = Some(color);
        self
    }

    pub fn merged(self, other: Self) -> Self {
        merge_style_override_slots!(
            self,
            other,
            [
                container_background,
                container_elevation,
                container_corner_radii,
                container_height,
                container_min_width,
                container_max_width,
                content_padding,
                content_gap,
                input_text_color,
                supporting_text_color,
                input_text_style,
                leading_icon_color,
                trailing_icon_color,
                state_layer_color,
            ]
        )
    }
}

#[derive(Debug, Clone)]
pub struct SearchBar {
    model: Model<String>,
    style: SearchBarStyle,
    placeholder: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    disabled: bool,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    test_id: Option<Arc<str>>,
    input_id_out: Option<Rc<Cell<Option<GlobalElementId>>>>,
    controls_element_id: Option<Rc<Cell<Option<GlobalElementId>>>>,
    expanded_model: Option<Model<bool>>,
    header_tokens: SearchBarHeaderTokens,
}

impl SearchBar {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            style: SearchBarStyle::default(),
            placeholder: None,
            a11y_label: None,
            disabled: false,
            leading_icon: None,
            trailing_icon: None,
            test_id: None,
            input_id_out: None,
            controls_element_id: None,
            expanded_model: None,
            header_tokens: SearchBarHeaderTokens::default(),
        }
    }

    pub fn placeholder_opt(mut self, placeholder: Option<Arc<str>>) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn style(mut self, style: SearchBarStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn a11y_label_opt(mut self, label: Option<Arc<str>>) -> Self {
        self.a11y_label = label;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn trailing_icon(mut self, icon: IconId) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    pub fn test_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.test_id = id;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn input_id_out(mut self, input_id_out: Rc<Cell<Option<GlobalElementId>>>) -> Self {
        self.input_id_out = Some(input_id_out);
        self
    }

    pub(crate) fn controls_element_id(
        mut self,
        controls_element_id: Rc<Cell<Option<GlobalElementId>>>,
    ) -> Self {
        self.controls_element_id = Some(controls_element_id);
        self
    }

    pub fn expanded_model(mut self, model: Model<bool>) -> Self {
        self.expanded_model = Some(model);
        self
    }

    pub(crate) fn header_tokens(mut self, header_tokens: SearchBarHeaderTokens) -> Self {
        self.header_tokens = header_tokens;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let part_test_ids = self.test_id.as_ref().map(SearchBarPartTestIds::from_base);
            let chrome_test_id = part_test_ids.as_ref().map(|ids| ids.chrome.clone());
            let leading_icon_test_id = part_test_ids.as_ref().map(|ids| ids.leading_icon.clone());
            let trailing_icon_test_id = part_test_ids.as_ref().map(|ids| ids.trailing_icon.clone());
            let pointer_down_state: Rc<Cell<Option<PointerDownCx>>> =
                cx.slot_state(|| Rc::new(Cell::new(None)), |state| state.clone());

            let expanded = self
                .expanded_model
                .as_ref()
                .and_then(|m| cx.get_model_copied(m, fret_ui::Invalidation::Layout))
                .unwrap_or(false);

            cx.pressable_with_id_props(|cx, st, pressable_id| {
                let enabled = !self.disabled;

                let now_frame = cx.frame_id.0;
                let pointer_down = pointer_down_state.get();
                let pressed = enabled && (st.pressed || pointer_down.is_some());
                let hovered = enabled && st.hovered;

                let (
                    container_height,
                    container_min_width,
                    container_max_width,
                    corner_radii,
                    state_layer_target,
                    state_layer_color,
                    ripple_base_opacity,
                    indication_config,
                    input_text_style,
                    input_chrome,
                    leading_color,
                    trailing_color,
                    shadow,
                    container_color,
                    focus_ring,
                    content_padding,
                    content_gap,
                ) = {
                    let theme = Theme::global(&*cx.app);
                    let states = search_bar_widget_states(enabled, hovered, pressed, expanded);

                    let container_height =
                        search_bar_metric_override(&self.style.container_height, states, || {
                            search_bar_tokens::container_height(theme)
                        });
                    let container_min_width =
                        search_bar_metric_override(&self.style.container_min_width, states, || {
                            search_bar_tokens::container_min_width(theme)
                        });
                    let container_max_width =
                        search_bar_metric_override(&self.style.container_max_width, states, || {
                            search_bar_tokens::container_max_width(theme)
                        });
                    let corner_radii = search_bar_corners_override(
                        &self.style.container_corner_radii,
                        states,
                        || search_bar_tokens::container_shape(theme),
                    );

                    let state_layer_target = if pressed {
                        search_bar_tokens::pressed_state_layer_opacity(theme)
                    } else if hovered {
                        search_bar_tokens::hover_state_layer_opacity(theme)
                    } else {
                        0.0
                    };

                    let state_layer_color = search_bar_color_override(
                        theme,
                        &self.style.state_layer_color,
                        states,
                        || {
                            if pressed {
                                search_bar_tokens::pressed_state_layer_color(theme)
                            } else {
                                search_bar_tokens::hover_state_layer_color(theme)
                            }
                        },
                    );

                    let ripple_base_opacity = search_bar_tokens::pressed_state_layer_opacity(theme);
                    let indication_config =
                        material_pressable_indication_config_in_scope(&*cx, None);

                    let input_text_style = search_bar_text_style_override(
                        &self.style.input_text_style,
                        states,
                        || match self.header_tokens {
                            SearchBarHeaderTokens::SearchView => {
                                search_view_tokens::header_input_text_style(theme)
                            }
                            SearchBarHeaderTokens::SearchBar => {
                                search_bar_tokens::input_text_style(theme)
                            }
                        },
                    );
                    let input_chrome = search_bar_text_input_chrome(
                        theme,
                        self.header_tokens,
                        hovered,
                        pressed,
                        &self.style,
                        states,
                    );

                    let (leading_color, trailing_color) = match self.header_tokens {
                        SearchBarHeaderTokens::SearchView => (
                            search_view_tokens::header_leading_icon_color(theme),
                            search_view_tokens::header_trailing_icon_color(theme),
                        ),
                        SearchBarHeaderTokens::SearchBar => (
                            search_bar_tokens::leading_icon_color(theme),
                            search_bar_tokens::trailing_icon_color(theme),
                        ),
                    };
                    let leading_color = search_bar_color_override(
                        theme,
                        &self.style.leading_icon_color,
                        states,
                        || leading_color,
                    );
                    let trailing_color = search_bar_color_override(
                        theme,
                        &self.style.trailing_icon_color,
                        states,
                        || trailing_color,
                    );

                    let elevation =
                        search_bar_metric_override(&self.style.container_elevation, states, || {
                            search_bar_tokens::container_elevation(theme)
                        });
                    let shadow =
                        shadow_for_elevation_with_color(theme, elevation, None, corner_radii);
                    let container_color = search_bar_color_override(
                        theme,
                        &self.style.container_background,
                        states,
                        || search_bar_tokens::container_color(theme),
                    );
                    let focus_ring = material_focus_ring_for_component(
                        theme,
                        "md.comp.search-bar",
                        corner_radii,
                    );
                    let content_padding =
                        search_bar_edges_override(&self.style.content_padding, states, || Edges {
                            left: Px(16.0),
                            right: Px(16.0),
                            top: Px(0.0),
                            bottom: Px(0.0),
                        });
                    let content_gap =
                        search_bar_metric_override(&self.style.content_gap, states, || Px(12.0));

                    (
                        container_height,
                        container_min_width,
                        container_max_width,
                        corner_radii,
                        state_layer_target,
                        state_layer_color,
                        ripple_base_opacity,
                        indication_config,
                        input_text_style,
                        input_chrome,
                        leading_color,
                        trailing_color,
                        shadow,
                        container_color,
                        focus_ring,
                        content_padding,
                        content_gap,
                    )
                };
                let overlay = material_ink_layer_for_pressable_with_last_down(
                    cx,
                    pressable_id,
                    now_frame,
                    pointer_down,
                    corner_radii,
                    RippleClip::Bounded,
                    state_layer_color,
                    pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    indication_config,
                    false,
                );

                let mut input_id = GlobalElementId(0);
                let a11y_label = self
                    .a11y_label
                    .clone()
                    .unwrap_or_else(|| material_search_bar_search_label(&*cx.app));
                let a11y_state_description =
                    expanded.then(|| material_search_bar_suggestions_available_label(&*cx.app));
                let input = cx.text_input_with_id_props(|_cx, id| {
                    input_id = id;

                    let mut props = TextInputProps::new(self.model.clone());
                    props.enabled = enabled;
                    props.focusable = enabled;
                    props.a11y_role = Some(SemanticsRole::TextField);
                    props.a11y_label = Some(a11y_label.clone());
                    props.a11y_state_description = a11y_state_description.clone();
                    props.test_id = self.test_id.clone();
                    props.placeholder = self.placeholder.clone();
                    props.expanded = Some(expanded);
                    props.controls_element = self
                        .controls_element_id
                        .as_ref()
                        .and_then(|id| id.get().map(|id| id.0));
                    props.text_style = input_text_style;
                    props.chrome = input_chrome;
                    props.layout.size.width = Length::Fill;
                    props.layout.size.height = Length::Fill;
                    props.layout.flex.grow = 1.0;
                    props
                });

                if let Some(out) = self.input_id_out.as_ref() {
                    out.set(Some(input_id));
                }

                if enabled && input_id != GlobalElementId(0) {
                    let input_id_for_focus = input_id;
                    cx.pressable_on_pointer_down(Arc::new(
                        move |host: &mut dyn UiPointerActionHost, _action_cx, _down| {
                            host.request_focus(input_id_for_focus);
                            PressablePointerDownResult::Continue
                        },
                    ));
                }

                let pointer_region = cx.named("pointer_region", |cx| {
                    let mut props = PointerRegionProps::default();
                    props.enabled = enabled;
                    props.layout.size.width = Length::Fill;
                    props.layout.size.height = Length::Fill;
                    cx.pointer_region(props, move |cx| {
                        let pointer_down_for_down = pointer_down_state.clone();
                        cx.pointer_region_on_pointer_down(Arc::new(
                            move |host, action_cx, down| {
                                pointer_down_for_down.set(Some(down));
                                host.invalidate(fret_ui::Invalidation::Paint);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                                false
                            },
                        ));

                        let pointer_down_for_up = pointer_down_state.clone();
                        cx.pointer_region_on_pointer_up(Arc::new(move |host, action_cx, up| {
                            if pointer_down_for_up
                                .get()
                                .is_some_and(|down| down.pointer_id == up.pointer_id)
                            {
                                pointer_down_for_up.set(None);
                                host.invalidate(fret_ui::Invalidation::Paint);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            }
                            false
                        }));

                        let pointer_down_for_cancel = pointer_down_state.clone();
                        cx.pointer_region_on_pointer_cancel(Arc::new(
                            move |host, action_cx, cancel| {
                                if pointer_down_for_cancel
                                    .get()
                                    .is_some_and(|down| down.pointer_id == cancel.pointer_id)
                                {
                                    pointer_down_for_cancel.set(None);
                                    host.invalidate(fret_ui::Invalidation::Paint);
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }
                                false
                            },
                        ));

                        let mut row = FlexProps::default();
                        row.layout.size.width = Length::Fill;
                        row.layout.size.height = Length::Fill;
                        row.justify = MainAlign::Start;
                        row.align = CrossAlign::Center;
                        row.gap = content_gap.into();

                        let leading_icon = self.leading_icon;
                        let trailing_icon = self.trailing_icon;
                        let leading_icon_test_id = leading_icon_test_id.clone();
                        let trailing_icon_test_id = trailing_icon_test_id.clone();

                        let content = cx.flex(row, move |cx| {
                            let mut children: Vec<AnyElement> = Vec::new();
                            if let Some(icon) = leading_icon.as_ref() {
                                let mut icon =
                                    material_search_bar_icon(cx, icon, Px(24.0), leading_color);
                                if let Some(test_id) = leading_icon_test_id.clone() {
                                    icon = icon.test_id(test_id);
                                }
                                children.push(icon);
                            }
                            children.push(input);
                            if let Some(icon) = trailing_icon.as_ref() {
                                let mut icon =
                                    material_search_bar_icon(cx, icon, Px(24.0), trailing_color);
                                if let Some(test_id) = trailing_icon_test_id.clone() {
                                    icon = icon.test_id(test_id);
                                }
                                children.push(icon);
                            }
                            children
                        });

                        let mut content_container = ContainerProps::default();
                        content_container.layout.size.width = Length::Fill;
                        content_container.layout.size.height = Length::Fill;
                        content_container.layout.overflow = Overflow::Visible;
                        content_container.padding = content_padding.into();
                        let content_layer =
                            cx.container(content_container, move |_cx| vec![content]);

                        let mut container = ContainerProps::default();
                        container.layout.size.width = Length::Fill;
                        container.layout.size.height = Length::Px(container_height);
                        container.layout.overflow = Overflow::Visible;
                        container.background = Some(container_color);
                        container.shadow = shadow;
                        container.corner_radii = corner_radii;
                        container.focus_within = true;
                        container.focus_ring = Some(focus_ring);

                        let mut chrome =
                            cx.container(container, move |_cx| vec![overlay, content_layer]);
                        if let Some(test_id) = chrome_test_id.clone() {
                            chrome = chrome.test_id(test_id);
                        }
                        vec![chrome]
                    })
                });

                let pressable_props = PressableProps {
                    enabled,
                    focusable: false,
                    a11y: PressableA11y {
                        role: None,
                        label: None,
                        test_id: None,
                        ..Default::default()
                    },
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.overflow = Overflow::Visible;
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Px(container_height);
                        if matches!(self.header_tokens, SearchBarHeaderTokens::SearchBar) {
                            layout.size.min_width = Some(Length::Px(container_min_width));
                            layout.size.max_width = Some(Length::Px(container_max_width));
                        }
                        layout
                    },
                    ..Default::default()
                };

                (pressable_props, vec![pointer_region])
            })
        })
    }
}

fn search_bar_text_input_chrome(
    theme: &Theme,
    header_tokens: SearchBarHeaderTokens,
    hovered: bool,
    pressed: bool,
    style_override: &SearchBarStyle,
    states: WidgetStates,
) -> fret_ui::TextInputStyle {
    let mut style = fret_ui::TextInputStyle::default();
    style.padding = Edges::all(Px(0.0));
    style.border = Edges::all(Px(0.0));
    style.border_color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    style.border_color_focused = style.border_color;
    style.focus_ring = None;
    style.corner_radii = Corners::all(Px(0.0));
    style.background = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    style.text_color =
        search_bar_color_override(theme, &style_override.input_text_color, states, || {
            match header_tokens {
                SearchBarHeaderTokens::SearchView => {
                    search_view_tokens::header_input_text_color(theme)
                }
                SearchBarHeaderTokens::SearchBar => search_bar_tokens::input_text_color(theme),
            }
        });
    style.placeholder_color =
        search_bar_color_override(theme, &style_override.supporting_text_color, states, || {
            match header_tokens {
                SearchBarHeaderTokens::SearchView => {
                    search_view_tokens::header_supporting_text_color(theme)
                }
                SearchBarHeaderTokens::SearchBar => {
                    search_bar_tokens::supporting_text_color(theme, hovered, pressed)
                }
            }
        });

    style.selection_color = search_bar_tokens::selection_color(theme);
    style.caret_color = search_bar_tokens::caret_color(theme);
    style.preedit_color = style.caret_color;
    style.preedit_underline_color = style.preedit_color;

    style
}

fn material_search_bar_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: &IconId,
    size: Px,
    color: Color,
) -> AnyElement {
    let svg = svg_source_for_icon(cx, icon);

    let mut props = SvgIconProps::new(svg);
    props.fit = SvgFit::Contain;
    props.layout.size.width = Length::Px(size);
    props.layout.size.height = Length::Px(size);
    props.color = color;
    cx.svg_icon_props(props)
}

fn svg_source_for_icon<H: UiHost>(cx: &mut ElementContext<'_, H>, icon: &IconId) -> SvgSource {
    let resolved = cx
        .app
        .with_global_mut(IconRegistry::default, |icons, _app| {
            icons
                .resolve_owned(icon)
                .unwrap_or(ResolvedSvgOwned::Static(MISSING_ICON_SVG))
        });

    match resolved {
        ResolvedSvgOwned::Static(bytes) => SvgSource::Static(bytes),
        ResolvedSvgOwned::Bytes(bytes) => SvgSource::Bytes(bytes),
    }
}
