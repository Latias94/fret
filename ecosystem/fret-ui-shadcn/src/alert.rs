use std::sync::Arc;

use fret_core::{Color, FontWeight, Px, SemanticsRole, TextWrap};
use fret_ui::element::{
    AnyElement, ContainerProps, ElementKind, InsetStyle, LayoutStyle, Length, PositionStyle,
    SemanticsDecoration,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

const ALERT_ACTION_MARKER_TEST_ID: &str = "__fret_shadcn.alert_action";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertVariant {
    #[default]
    Default,
    Destructive,
}

#[derive(Debug)]
pub struct Alert {
    children: Vec<AnyElement>,
    variant: AlertVariant,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

#[derive(Debug)]
pub struct AlertAction {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl AlertAction {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let inset = MetricRef::space(Space::N4).resolve(theme);

        let mut layout = LayoutStyle::default();
        layout.position = PositionStyle::Absolute;
        layout.inset = InsetStyle {
            top: Some(inset).into(),
            right: Some(inset).into(),
            bottom: None.into(),
            left: None.into(),
        };
        layout.size.width = Length::Auto;
        layout.size.height = Length::Auto;

        cx.container(
            ContainerProps {
                layout,
                ..Default::default()
            },
            move |_cx| self.children,
        )
        .test_id(ALERT_ACTION_MARKER_TEST_ID)
    }
}

impl Alert {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            variant: AlertVariant::Default,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
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
        alert_with_patch(cx, self.variant, self.children, self.chrome, self.layout)
    }
}

pub fn alert<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: AlertVariant,
    children: impl IntoIterator<Item = AnyElement>,
) -> AnyElement {
    let children = children.into_iter().collect();
    alert_with_patch(
        cx,
        variant,
        children,
        ChromeRefinement::default(),
        LayoutRefinement::default(),
    )
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn patch_svg_icon_to_inherit_current_color(el: &mut AnyElement, fallback: Color, size: Px) {
    let ElementKind::SvgIcon(props) = &mut el.kind else {
        return;
    };

    props.color = fallback;
    props.inherit_color = true;
    props.layout.size.width = fret_ui::element::Length::Px(size);
    props.layout.size.height = fret_ui::element::Length::Px(size);
}

fn maybe_patch_text_color(el: &mut AnyElement, from: Color, to: Color) {
    match &mut el.kind {
        ElementKind::Text(props) if props.color == Some(from) => {
            props.color = Some(to);
        }
        ElementKind::StyledText(props) if props.color == Some(from) => {
            props.color = Some(to);
        }
        ElementKind::SelectableText(props) if props.color == Some(from) => {
            props.color = Some(to);
        }
        _ => {}
    }
}

fn patch_text_color_recursive(el: &mut AnyElement, from: Color, to: Color) {
    maybe_patch_text_color(el, from, to);
    for child in &mut el.children {
        patch_text_color_recursive(child, from, to);
    }
}

fn patch_foreground_scope_recursive(el: &mut AnyElement, from: Color, to: Color) {
    if let ElementKind::ForegroundScope(props) = &mut el.kind {
        if props.foreground == Some(from) {
            props.foreground = Some(to);
        }
    }

    for child in &mut el.children {
        patch_foreground_scope_recursive(child, from, to);
    }
}

fn alert_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: AlertVariant,
    mut children: Vec<AnyElement>,
    chrome_override: ChromeRefinement,
    layout_override: LayoutRefinement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    let bg = theme.color_token("card");
    let border = theme.color_token("border");
    let destructive = theme.color_token("destructive");
    let card_fg = theme.color_token("card-foreground");
    let muted_fg = theme.color_token("muted-foreground");

    let fg_default = match variant {
        AlertVariant::Default => card_fg,
        AlertVariant::Destructive => destructive,
    };
    let fg = chrome_override
        .text_color
        .as_ref()
        .map(|c| c.resolve(&theme))
        .unwrap_or(fg_default);
    let destructive_description = alpha_mul(destructive, 0.9);

    let icon = match children.first() {
        Some(first) if matches!(first.kind, ElementKind::SvgIcon(_)) => Some(children.remove(0)),
        _ => None,
    };
    let mut body_children = children;

    let action_idx = body_children.iter().position(|child| {
        child
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
            == Some(ALERT_ACTION_MARKER_TEST_ID)
    });
    let action = action_idx.map(|idx| body_children.remove(idx));

    if variant == AlertVariant::Destructive {
        if let Some(description) = body_children.get_mut(1) {
            patch_text_color_recursive(description, muted_fg, destructive_description);
            patch_foreground_scope_recursive(description, muted_fg, destructive_description);
        }
    }

    let props = decl_style::container_props(
        &theme,
        ChromeRefinement::default()
            .px(Space::N4)
            .py(Space::N3)
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Color(bg))
            .border_color(ColorRef::Color(border))
            .merge(chrome_override),
        // shadcn/ui v4: Alert root uses `w-full` by default.
        LayoutRefinement::default().w_full().merge(layout_override),
    );

    let body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N0p5)
            .layout(LayoutRefinement::default().w_full().flex_1().min_w_0()),
        |_cx| body_children,
    );

    let main = if let Some(mut icon) = icon {
        patch_svg_icon_to_inherit_current_color(&mut icon, fg, Px(16.0));
        let icon = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default(),
                LayoutRefinement::default().mt(Space::N0p5),
            ),
            move |_cx| [icon],
        );

        stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |_cx| vec![icon, body],
        )
    } else {
        body
    };

    let mut props = props;
    props.layout.position = PositionStyle::Relative;

    cx.container(props, move |cx| {
        let mut out: Vec<AnyElement> = vec![cx.foreground_scope(fg, move |_cx| vec![main])];
        if let Some(action) = action {
            out.push(action);
        }
        out
    })
    .attach_semantics(SemanticsDecoration::default().role(SemanticsRole::Alert))
}

#[derive(Debug, Clone)]
pub struct AlertTitle {
    text: Arc<str>,
}

impl AlertTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let px = theme
            .metric_by_key("component.alert.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let line_height = theme
            .metric_by_key("component.alert.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_weight(FontWeight::MEDIUM)
            // Tailwind: `tracking-tight` ~= `-0.025em`.
            .letter_spacing_em(-0.025)
            // Upstream shadcn/ui `AlertTitle` uses `line-clamp-1` (single-line truncation).
            .truncate()
            .into_element(cx)
    }
}

#[derive(Debug)]
pub struct AlertDescription {
    content: AlertDescriptionContent,
}

#[derive(Debug)]
enum AlertDescriptionContent {
    Text(Arc<str>),
    Children(Vec<AnyElement>),
}

impl AlertDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            content: AlertDescriptionContent::Text(text.into()),
        }
    }

    pub fn new_children(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            content: AlertDescriptionContent::Children(children.into_iter().collect()),
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let fg = theme.color_token("muted-foreground");
        let px = theme
            .metric_by_key("component.alert.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let line_height = theme
            .metric_by_key("component.alert.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        match self.content {
            AlertDescriptionContent::Text(text) => cx.foreground_scope(fg, move |cx| {
                vec![
                    ui::text(cx, text)
                        .text_size_px(px)
                        .line_height_px(line_height)
                        .font_weight(FontWeight::NORMAL)
                        .wrap(TextWrap::Word)
                        .into_element(cx),
                ]
            }),
            AlertDescriptionContent::Children(children) => cx.foreground_scope(fg, move |cx| {
                vec![stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N1)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full()),
                    move |_cx| children,
                )]
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Color, Point, Px, Rect, Size, TextOverflow};
    use fret_icons::IconId;
    use fret_ui::element::ElementKind;
    use fret_ui_kit::declarative::icon as decl_icon;

    #[test]
    fn alert_stamps_role_without_layout_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Alert::new([cx.text("Hello")]).into_element(cx)
        });

        assert!(
            !matches!(element.kind, ElementKind::Semantics(_)),
            "expected Alert to avoid `Semantics` wrappers; use `attach_semantics` instead"
        );
        assert_eq!(
            element.semantics_decoration.as_ref().and_then(|d| d.role),
            Some(SemanticsRole::Alert)
        );
    }

    #[test]
    fn alert_title_defaults_to_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            AlertTitle::new("A very long alert title that should truncate").into_element(cx)
        });

        let ElementKind::Text(props) = &element.kind else {
            panic!(
                "expected AlertTitle to be a Text element, got {:?}",
                element.kind
            );
        };

        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
    }

    #[test]
    fn alert_forces_icon_to_inherit_current_color() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let icon = decl_icon::icon_with(
                cx,
                IconId::new_static("lucide.terminal"),
                None,
                Some(ColorRef::Color(Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                })),
            );

            Alert::new([
                icon,
                AlertTitle::new("Heads up!").into_element(cx),
                AlertDescription::new("You can add components to your app.").into_element(cx),
            ])
            .into_element(cx)
        });

        fn find_first_svg_icon(el: &AnyElement) -> Option<&fret_ui::element::SvgIconProps> {
            if let ElementKind::SvgIcon(props) = &el.kind {
                return Some(props);
            }
            for child in &el.children {
                if let Some(found) = find_first_svg_icon(child) {
                    return Some(found);
                }
            }
            None
        }

        let icon = find_first_svg_icon(&element).expect("expected an svg icon under Alert");
        assert!(
            icon.inherit_color,
            "expected Alert icon to inherit currentColor via ForegroundScope"
        );
    }
}
