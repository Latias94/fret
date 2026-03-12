use std::sync::Arc;

use fret_core::window::ColorScheme;
use fret_core::{FontWeight, Px};
use fret_icons::IconId;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, IntoUiElement, LayoutRefinement, MetricRef, Radius, Space, ui,
};

use crate::surface_slot::{ShadcnSurfaceSlot, surface_slot_in_scope};

#[derive(Debug)]
enum KbdContent {
    Text(Arc<str>),
    Children(Vec<AnyElement>),
}

#[derive(Debug)]
pub struct Kbd {
    content: KbdContent,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Kbd {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            content: KbdContent::Text(text.into()),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn from_children(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            content: KbdContent::Children(children.into_iter().collect()),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Overrides the contents of this `Kbd` with an explicit child list.
    ///
    /// This is primarily used for icon-first keycap rendering (shadcn `&>svg` patterns), so
    /// demos can avoid relying on platform fonts having `⌘`/`⌥` glyphs.
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.content = KbdContent::Children(children.into_iter().collect());
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        kbd_with_patch(cx, self.content, self.chrome, self.layout)
    }
}

pub fn kbd<H: UiHost, T>(text: T) -> impl IntoUiElement<H> + use<H, T>
where
    T: Into<Arc<str>>,
{
    Kbd::new(text)
}

/// Returns a pre-landed icon element for `Kbd::from_children(...)`.
///
/// This intentionally stays raw because `Kbd::from_children(...)` stores an explicit
/// `Vec<AnyElement>` child list for icon-first keycap composition.
#[track_caller]
pub fn kbd_icon<H: UiHost>(cx: &mut ElementContext<'_, H>, icon: IconId) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let px = theme
        .metric_by_key("component.kbd.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    decl_icon::icon_with(cx, icon, Some(px), None)
}

fn kbd_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    content: KbdContent,
    chrome_override: ChromeRefinement,
    layout_override: LayoutRefinement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    let (bg, fg) = if surface_slot_in_scope(cx) == Some(ShadcnSurfaceSlot::TooltipContent) {
        // Upstream shadcn/ui (new-york-v4):
        // - default: `[[data-slot=tooltip-content]_&]:bg-background/20`
        // - dark: `dark:[[data-slot=tooltip-content]_&]:bg-background/10`
        let alpha = if theme.color_scheme == Some(ColorScheme::Dark) {
            0.10
        } else {
            0.20
        };
        (
            alpha_mul(theme.color_token("background"), alpha),
            theme.color_token("background"),
        )
    } else {
        (
            theme.color_token("muted"),
            theme.color_token("muted-foreground"),
        )
    };

    let chrome = ChromeRefinement::default()
        .px(Space::N1)
        .py(Space::N0p5)
        .rounded(Radius::Sm)
        .bg(ColorRef::Color(bg))
        .merge(chrome_override);

    let layout_override = LayoutRefinement::default()
        .h_px(Px(20.0))
        .min_h(Px(20.0))
        .min_w(Px(20.0))
        .merge(layout_override);

    let props = decl_style::container_props(&theme, chrome, layout_override);

    let gap = MetricRef::space(Space::N1).resolve(&theme);

    let px = theme
        .metric_by_key("component.kbd.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.kbd.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));

    cx.container(props, |cx| {
        vec![cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                gap: gap.into(),
                padding: fret_core::Edges::all(Px(0.0)).into(),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| match content {
                KbdContent::Text(text) => vec![ui::label( text)
                    .text_size_px(px)
                    .fixed_line_box_px(line_height)
                    .line_box_in_bounds()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(ColorRef::Color(fg))
                    .into_element(cx)],
                KbdContent::Children(children) => children,
            },
        )]
    })
}

fn alpha_mul(mut c: fret_core::Color, mul: f32) -> fret_core::Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug)]
pub struct KbdGroup {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl KbdGroup {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let gap = MetricRef::space(Space::N1).resolve(&theme);
        let direction = crate::use_direction(cx, None);
        let children = crate::rtl::reverse_in_rtl(direction, self.children);
        let layout = decl_style::layout_style(&theme, self.layout);

        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Horizontal,
                gap: gap.into(),
                padding: fret_core::Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::LayoutDirection;
    use fret_app::App;
    use fret_core::{AppWindowId, Color, Point, Px, Rect, Size as CoreSize};
    use fret_ui::element::{ElementKind, Length, MarginEdge, SpacingLength};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(200.0), Px(120.0)),
        )
    }

    fn alpha_mul(mut c: Color, mul: f32) -> Color {
        c.a = (c.a * mul).clamp(0.0, 1.0);
        c
    }

    fn find_first_text_style(el: &AnyElement) -> Option<&fret_core::TextStyle> {
        match &el.kind {
            ElementKind::Text(props) => props.style.as_ref(),
            ElementKind::StyledText(props) => props.style.as_ref(),
            ElementKind::SelectableText(props) => props.style.as_ref(),
            _ => None,
        }
        .or_else(|| el.children.iter().find_map(find_first_text_style))
    }

    #[test]
    fn kbd_defaults_match_shadcn_constraints_and_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Kbd::new("K").into_element(cx)
            });

        let ElementKind::Container(props) = &element.kind else {
            panic!("expected Kbd to render a container");
        };

        assert_eq!(props.layout.size.height, Length::Px(Px(20.0)));
        assert_eq!(props.layout.size.min_height, Some(Length::Px(Px(20.0))));
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(20.0))));

        let theme = Theme::global(&app);
        let expected_px = MetricRef::space(Space::N1).resolve(theme);
        let expected_py = MetricRef::space(Space::N0p5).resolve(theme);
        assert_eq!(props.padding.left, SpacingLength::Px(expected_px));
        assert_eq!(props.padding.right, SpacingLength::Px(expected_px));
        assert_eq!(props.padding.top, SpacingLength::Px(expected_py));
        assert_eq!(props.padding.bottom, SpacingLength::Px(expected_py));

        let expected_bg = theme.color_token("muted");
        assert_eq!(props.background, Some(expected_bg));

        let expected_radius = MetricRef::radius(Radius::Sm).resolve(theme);
        assert_eq!(props.corner_radii.top_left, expected_radius);
        assert_eq!(props.corner_radii.top_right, expected_radius);
        assert_eq!(props.corner_radii.bottom_left, expected_radius);
        assert_eq!(props.corner_radii.bottom_right, expected_radius);

        let style = find_first_text_style(&element).expect("text style");
        assert_eq!(style.size, Px(12.0));
        assert_eq!(style.line_height, Some(Px(16.0)));
        assert_eq!(style.weight, FontWeight::MEDIUM);
    }

    #[test]
    fn kbd_tooltip_slot_uses_background_alpha_mapping() {
        let window = AppWindowId::default();

        for (scheme, expected_alpha) in [
            (crate::shadcn_themes::ShadcnColorScheme::Light, 0.20),
            (crate::shadcn_themes::ShadcnColorScheme::Dark, 0.10),
        ] {
            let mut app = App::new();
            crate::shadcn_themes::apply_shadcn_new_york(
                &mut app,
                crate::shadcn_themes::ShadcnBaseColor::Neutral,
                scheme,
            );

            let element =
                fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                    crate::surface_slot::with_surface_slot_provider(
                        cx,
                        ShadcnSurfaceSlot::TooltipContent,
                        |cx| Kbd::new("T").into_element(cx),
                    )
                });
            let ElementKind::Container(props) = &element.kind else {
                panic!("expected Kbd to render a container");
            };

            let theme = Theme::global(&app);
            let expected_bg = alpha_mul(theme.color_token("background"), expected_alpha);
            assert_eq!(props.background, Some(expected_bg));
        }
    }

    #[test]
    fn kbd_group_gap_matches_shadcn() {
        let window = AppWindowId::default();
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                KbdGroup::new([
                    Kbd::new("A").into_element(cx),
                    Kbd::new("B").into_element(cx),
                ])
                .into_element(cx)
            });

        let ElementKind::Flex(props) = &element.kind else {
            panic!("expected KbdGroup to render as a flex element");
        };
        let theme = Theme::global(&app);
        let expected_gap = MetricRef::space(Space::N1).resolve(theme);
        assert_eq!(props.gap, SpacingLength::Px(expected_gap));
    }

    #[test]
    fn kbd_group_reverses_children_in_rtl() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                crate::with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
                    KbdGroup::new([
                        Kbd::new("A").into_element(cx).test_id("a"),
                        Kbd::new("B").into_element(cx).test_id("b"),
                    ])
                    .into_element(cx)
                })
            });

        assert_eq!(element.children.len(), 2);
        let first = element.children[0]
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref());
        let second = element.children[1]
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref());
        assert_eq!(first, Some("b"));
        assert_eq!(second, Some("a"));
    }

    #[test]
    fn kbd_icon_default_size_matches_text_px_metric() {
        let window = AppWindowId::default();
        let mut app = App::new();

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                kbd_icon(cx, fret_icons::ids::ui::CHECK)
            });

        let ElementKind::SvgIcon(props) = &element.kind else {
            panic!("expected kbd_icon to render an SvgIcon element");
        };
        assert_eq!(props.layout.size.width, Length::Px(Px(12.0)));
        assert_eq!(props.layout.size.height, Length::Px(Px(12.0)));
    }

    #[test]
    fn kbd_layout_does_not_apply_vertical_margins() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Kbd::new("K").into_element(cx)
            });
        let ElementKind::Container(props) = &element.kind else {
            panic!("expected Kbd to render a container");
        };
        assert_eq!(props.layout.margin.top, MarginEdge::Px(Px(0.0)));
        assert_eq!(props.layout.margin.bottom, MarginEdge::Px(Px(0.0)));
    }
}
