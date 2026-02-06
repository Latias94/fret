//! Material 3 search bar (MVP).
//!
//! Token-driven outcome alignment via `md.comp.search-bar.*` (Material Web v30).

use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, SemanticsRole, SvgFit};
use fret_icons::{IconId, IconRegistry, MISSING_ICON_SVG, ResolvedSvgOwned};
use fret_runtime::Model;
use fret_ui::action::{PressablePointerDownResult, UiPointerActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, SvgIconProps, TextInputProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{GlobalElementId, SvgSource, Theme, UiHost};

use crate::foundation::elevation::shadow_for_elevation_with_color;
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::tokens::search_bar as search_bar_tokens;

#[derive(Debug, Clone)]
pub struct SearchBar {
    model: Model<String>,
    placeholder: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    disabled: bool,
    leading_icon: Option<IconId>,
    trailing_icon: Option<IconId>,
    test_id: Option<Arc<str>>,
}

impl SearchBar {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            placeholder: None,
            a11y_label: None,
            disabled: false,
            leading_icon: None,
            trailing_icon: None,
            test_id: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
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

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            cx.pressable_with_id_props(|cx, st, pressable_id| {
                let enabled = !self.disabled;

                let container_height = search_bar_tokens::container_height(&theme);
                let corner_radii = search_bar_tokens::container_shape(&theme);

                let now_frame = cx.frame_id.0;
                let pressed = enabled && st.pressed;
                let hovered = enabled && st.hovered;

                let state_layer_target = if pressed {
                    search_bar_tokens::pressed_state_layer_opacity(&theme)
                } else if hovered {
                    search_bar_tokens::hover_state_layer_opacity(&theme)
                } else {
                    0.0
                };

                let state_layer_color = if pressed {
                    search_bar_tokens::pressed_state_layer_color(&theme)
                } else {
                    search_bar_tokens::hover_state_layer_color(&theme)
                };

                let ripple_base_opacity = search_bar_tokens::pressed_state_layer_opacity(&theme);
                let config = material_pressable_indication_config(&theme, None);
                let overlay = material_ink_layer_for_pressable(
                    cx,
                    pressable_id,
                    now_frame,
                    corner_radii,
                    RippleClip::Bounded,
                    state_layer_color,
                    pressed,
                    state_layer_target,
                    ripple_base_opacity,
                    config,
                    false,
                );

                let mut input_id = GlobalElementId(0);
                let input = cx.text_input_with_id_props(|_cx, id| {
                    input_id = id;

                    let mut props = TextInputProps::new(self.model.clone());
                    props.enabled = enabled;
                    props.focusable = enabled;
                    props.a11y_role = Some(SemanticsRole::TextField);
                    props.a11y_label = self.a11y_label.clone();
                    props.test_id = self.test_id.clone();
                    props.placeholder = self.placeholder.clone();
                    props.text_style = search_bar_tokens::input_text_style(&theme);
                    props.chrome = search_bar_text_input_chrome(&theme, hovered, pressed);
                    props.layout.size.width = Length::Fill;
                    props.layout.size.height = Length::Fill;
                    props.layout.flex.grow = 1.0;
                    props
                });

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
                        cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                        let mut row = FlexProps::default();
                        row.layout.size.width = Length::Fill;
                        row.layout.size.height = Length::Fill;
                        row.justify = MainAlign::Start;
                        row.align = CrossAlign::Center;
                        row.gap = Px(12.0);

                        let leading_icon = self.leading_icon;
                        let trailing_icon = self.trailing_icon;
                        let leading_color = search_bar_tokens::leading_icon_color(&theme);
                        let trailing_color = search_bar_tokens::trailing_icon_color(&theme);

                        let content = cx.flex(row, move |cx| {
                            let mut children: Vec<AnyElement> = Vec::new();
                            if let Some(icon) = leading_icon.as_ref() {
                                children.push(material_search_bar_icon(
                                    cx,
                                    icon,
                                    Px(24.0),
                                    leading_color,
                                ));
                            }
                            children.push(input);
                            if let Some(icon) = trailing_icon.as_ref() {
                                children.push(material_search_bar_icon(
                                    cx,
                                    icon,
                                    Px(24.0),
                                    trailing_color,
                                ));
                            }
                            children
                        });

                        let elevation = search_bar_tokens::container_elevation(&theme);
                        let shadow =
                            shadow_for_elevation_with_color(&theme, elevation, None, corner_radii);

                        let mut container = ContainerProps::default();
                        container.layout.size.width = Length::Fill;
                        container.layout.size.height = Length::Px(container_height);
                        container.layout.overflow = Overflow::Visible;
                        container.padding = Edges {
                            left: Px(16.0),
                            right: Px(16.0),
                            top: Px(0.0),
                            bottom: Px(0.0),
                        };
                        container.background = Some(search_bar_tokens::container_color(&theme));
                        container.shadow = shadow;
                        container.corner_radii = corner_radii;
                        container.focus_within = true;
                        container.focus_ring = Some(material_focus_ring_for_component(
                            &theme,
                            "md.comp.search-bar",
                            corner_radii,
                        ));

                        vec![cx.container(container, move |_cx| vec![overlay, content])]
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
    hovered: bool,
    pressed: bool,
) -> fret_ui::TextInputStyle {
    fn alpha_mul(mut c: Color, mul: f32) -> Color {
        c.a = (c.a * mul).clamp(0.0, 1.0);
        c
    }

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

    style.text_color = search_bar_tokens::input_text_color(theme);
    style.placeholder_color = search_bar_tokens::supporting_text_color(theme, hovered, pressed);

    style.selection_color = theme
        .color_by_key("md.sys.color.primary")
        .map(|c| alpha_mul(c, 0.35))
        .unwrap_or(style.selection_color);
    style.caret_color = theme
        .color_by_key("md.sys.color.primary")
        .unwrap_or(style.caret_color);
    style.preedit_color = style.caret_color;

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
                .resolve_svg_owned(icon)
                .unwrap_or(ResolvedSvgOwned::Static(MISSING_ICON_SVG))
        });

    match resolved {
        ResolvedSvgOwned::Static(bytes) => SvgSource::Static(bytes),
        ResolvedSvgOwned::Bytes(bytes) => SvgSource::Bytes(bytes),
    }
}
