use std::sync::Arc;

use fret_core::{Color, FontWeight, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_ui::element::{
    AnyElement, ContainerProps, ElementKind, Length, PositionStyle, SemanticsDecoration,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;

use fret_ui_kit::typography::scope_description_text;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, PaddingRefinement, Radius, Space, ui,
};

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
        let mut layout = decl_style::layout_style(
            theme,
            LayoutRefinement::default()
                .absolute()
                .top(Space::N2p5)
                .right(Space::N3)
                .merge(self.layout),
        );
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

fn patch_inherited_foreground_recursive(el: &mut AnyElement, from: Color, to: Color) {
    if el.inherited_foreground == Some(from) {
        el.inherited_foreground = Some(to);
    }

    if let ElementKind::ForegroundScope(props) = &mut el.kind {
        if props.foreground == Some(from) {
            props.foreground = Some(to);
        }
    }

    for child in &mut el.children {
        patch_inherited_foreground_recursive(child, from, to);
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
    let has_action = children.iter().any(|child| {
        child
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
            == Some(ALERT_ACTION_MARKER_TEST_ID)
    });

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
            patch_inherited_foreground_recursive(description, muted_fg, destructive_description);
        }
    }

    let props = decl_style::container_props(
        &theme,
        ChromeRefinement::default()
            .px(Space::N4)
            .py(Space::N3)
            .merge(ChromeRefinement {
                padding: Some(PaddingRefinement {
                    right: Some(if has_action {
                        MetricRef::Px(Px(72.0))
                    } else {
                        MetricRef::space(Space::N4)
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Color(bg))
            .border_color(ColorRef::Color(border))
            .merge(chrome_override),
        // shadcn/ui v4: Alert root uses `w-full` by default.
        LayoutRefinement::default().w_full().merge(layout_override),
    );

    let body = ui::v_flex(move |_cx| body_children)
        .gap(Space::N0p5)
        .layout(LayoutRefinement::default().w_full().flex_1().min_w_0())
        .into_element(cx);

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

        ui::h_flex(move |_cx| vec![icon, body])
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full())
            .into_element(cx)
    } else {
        body
    };

    let mut props = props;
    props.layout.position = PositionStyle::Relative;

    cx.container(props, move |_cx| {
        let mut out: Vec<AnyElement> = vec![main.inherit_foreground(fg)];
        if let Some(action) = action {
            out.push(action);
        }
        out
    })
    .attach_semantics(SemanticsDecoration::default().role(SemanticsRole::Alert))
}

#[derive(Debug)]
pub struct AlertTitle {
    content: AlertTitleContent,
}

#[derive(Debug)]
enum AlertTitleContent {
    Text(Arc<str>),
    Children(Vec<AnyElement>),
}

impl AlertTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            content: AlertTitleContent::Text(text.into()),
        }
    }

    pub fn new_children(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            content: AlertTitleContent::Children(children.into_iter().collect()),
        }
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

        match self.content {
            AlertTitleContent::Text(text) => ui::text(text)
                .text_size_px(px)
                .line_height_px(line_height)
                .font_weight(FontWeight::MEDIUM)
                // Tailwind: `tracking-tight` ~= `-0.025em`.
                .letter_spacing_em(-0.025)
                .wrap(TextWrap::Word)
                .into_element(cx),
            AlertTitleContent::Children(mut children) => {
                for child in &mut children {
                    patch_alert_title_text_style_recursive(child, px, line_height);
                }

                match children.len() {
                    0 => ui::text("")
                        .text_size_px(px)
                        .line_height_px(line_height)
                        .font_weight(FontWeight::MEDIUM)
                        .letter_spacing_em(-0.025)
                        .wrap(TextWrap::Word)
                        .into_element(cx),
                    1 => children.pop().expect("children.len() == 1"),
                    _ => ui::v_flex(move |_cx| children)
                        .gap(Space::N0)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx),
                }
            }
        }
    }
}

fn patch_alert_text_style_recursive(
    el: &mut AnyElement,
    px: Px,
    line_height: Px,
    weight: FontWeight,
) {
    fn patch_text_style(
        style: &mut Option<fret_core::TextStyle>,
        px: Px,
        line_height: Px,
        weight: FontWeight,
    ) {
        let mut style_value = style.take().unwrap_or_default();
        style_value.size = px;
        style_value.weight = weight;
        style_value.line_height = Some(line_height);
        style_value.line_height_em = None;
        style_value.letter_spacing_em = Some(if weight == FontWeight::MEDIUM {
            -0.025
        } else {
            0.0
        });
        *style = Some(style_value);
    }

    match &mut el.kind {
        ElementKind::Text(props) => {
            patch_text_style(&mut props.style, px, line_height, weight);
            props.wrap = TextWrap::Word;
            props.overflow = TextOverflow::Clip;
        }
        ElementKind::StyledText(props) => {
            patch_text_style(&mut props.style, px, line_height, weight);
            props.wrap = TextWrap::Word;
            props.overflow = TextOverflow::Clip;
        }
        ElementKind::SelectableText(props) => {
            patch_text_style(&mut props.style, px, line_height, weight);
            props.wrap = TextWrap::Word;
            props.overflow = TextOverflow::Clip;
        }
        _ => {}
    }

    for child in &mut el.children {
        patch_alert_text_style_recursive(child, px, line_height, weight);
    }
}

fn patch_alert_title_text_style_recursive(el: &mut AnyElement, px: Px, line_height: Px) {
    patch_alert_text_style_recursive(el, px, line_height, FontWeight::MEDIUM);
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

        match self.content {
            AlertDescriptionContent::Text(text) => scope_description_text(
                ui::raw_text(text)
                    .wrap(TextWrap::Word)
                    .overflow(TextOverflow::Clip)
                    .into_element(cx),
                &theme,
                "component.alert.description",
            ),
            AlertDescriptionContent::Children(children) => scope_description_text(
                ui::v_flex(move |_cx| children)
                    .gap(Space::N1)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
                &theme,
                "component.alert.description",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, AttributedText, Color, Point, Px, Rect, Size, TextOverflow, TextSpan,
    };
    use fret_icons::IconId;
    use fret_ui::element::{ElementKind, InsetEdge, SpacingLength};
    use fret_ui_kit::declarative::icon as decl_icon;

    fn contains_foreground_scope(el: &AnyElement) -> bool {
        matches!(el.kind, ElementKind::ForegroundScope(_))
            || el.children.iter().any(contains_foreground_scope)
    }

    fn find_first_inherited_foreground_node(el: &AnyElement) -> Option<&AnyElement> {
        if el.inherited_foreground.is_some() {
            return Some(el);
        }
        el.children
            .iter()
            .find_map(find_first_inherited_foreground_node)
    }

    fn find_text_element<'a>(el: &'a AnyElement, needle: &str) -> Option<&'a AnyElement> {
        match &el.kind {
            ElementKind::Text(props) if props.text.as_ref() == needle => Some(el),
            _ => el
                .children
                .iter()
                .find_map(|child| find_text_element(child, needle)),
        }
    }

    fn find_first_styled_text(el: &AnyElement) -> Option<&fret_ui::element::StyledTextProps> {
        if let ElementKind::StyledText(props) = &el.kind {
            return Some(props);
        }
        el.children.iter().find_map(find_first_styled_text)
    }

    fn find_first_selectable_text(
        el: &AnyElement,
    ) -> Option<&fret_ui::element::SelectableTextProps> {
        if let ElementKind::SelectableText(props) = &el.kind {
            return Some(props);
        }
        el.children.iter().find_map(find_first_selectable_text)
    }

    #[test]
    fn alert_description_children_scope_inherited_text_style() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            AlertDescription::new_children([cx.text("Nested body")]).into_element(cx)
        });

        let text = find_text_element(&element, "Nested body").expect("expected nested text node");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected nested alert description child to be text");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());

        let theme = fret_ui::Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::description_text_refinement(
                &theme,
                "component.alert.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );
    }

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
    fn alert_title_wraps_by_default_like_shadcn() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            AlertTitle::new("A very long alert title that should wrap").into_element(cx)
        });

        let ElementKind::Text(props) = &element.kind else {
            panic!(
                "expected AlertTitle to be a Text element, got {:?}",
                element.kind
            );
        };

        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
    }

    #[test]
    fn alert_title_children_patch_rich_text_with_title_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(100.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let rich = AttributedText::new(
                Arc::<str>::from("Alert title rendered from a rich text child"),
                Arc::<[TextSpan]>::from([TextSpan::new(
                    "Alert title rendered from a rich text child".len(),
                )]),
            );

            AlertTitle::new_children([cx.styled_text(rich)]).into_element(cx)
        });

        let ElementKind::StyledText(props) = &element.kind else {
            panic!(
                "expected AlertTitle::new_children(single child) to keep the rich text node, got {:?}",
                element.kind
            );
        };

        let style = props
            .style
            .as_ref()
            .expect("expected AlertTitle children to receive explicit title text style");
        let theme = Theme::global(&app).snapshot();
        let expected_px = theme
            .metric_by_key("component.alert.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let expected_line_height = theme
            .metric_by_key("component.alert.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        assert_eq!(style.size, expected_px);
        assert_eq!(style.weight, FontWeight::MEDIUM);
        assert_eq!(style.line_height, Some(expected_line_height));
        assert_eq!(style.letter_spacing_em, Some(-0.025));
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
    }

    #[test]
    fn alert_description_children_scope_rich_text_with_description_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let rich = AttributedText::new(
                Arc::<str>::from("Alert description rendered from a rich text child"),
                Arc::<[TextSpan]>::from([TextSpan::new(
                    "Alert description rendered from a rich text child".len(),
                )]),
            );

            AlertDescription::new_children([cx.styled_text(rich)]).into_element(cx)
        });

        let props = find_first_styled_text(&element)
            .expect("expected AlertDescription children to keep the rich text node");
        assert!(props.style.is_none());
        assert!(props.color.is_none());

        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::description_text_refinement(
                &theme,
                "component.alert.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
    }

    #[test]
    fn alert_description_children_preserve_interactive_spans_under_description_scope() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(260.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let rich = AttributedText::new(
                Arc::<str>::from("Open support article"),
                Arc::<[TextSpan]>::from([TextSpan::new("Open support article".len())]),
            );

            let mut props = fret_ui::element::SelectableTextProps::new(rich);
            props.interactive_spans =
                Arc::from([fret_ui::element::SelectableTextInteractiveSpan {
                    range: 0.."Open support article".len(),
                    tag: Arc::<str>::from("support-article"),
                }]);

            AlertDescription::new_children([cx.selectable_text_props(props)]).into_element(cx)
        });

        let props = find_first_selectable_text(&element)
            .expect("expected AlertDescription children to keep selectable text nodes");
        assert!(props.style.is_none());
        assert!(props.color.is_none());

        let theme = Theme::global(&app).snapshot();
        assert_eq!(props.interactive_spans.len(), 1);
        assert_eq!(props.interactive_spans[0].tag.as_ref(), "support-article");
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::description_text_refinement(
                &theme,
                "component.alert.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );
    }

    #[test]
    fn alert_with_action_reserves_right_padding_like_shadcn() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Alert::new([
                AlertTitle::new("Heads up!").into_element(cx),
                AlertDescription::new("You can add components to your app.").into_element(cx),
                AlertAction::new([cx.text("Undo")]).into_element(cx),
            ])
            .into_element(cx)
        });

        let ElementKind::Container(props) = &element.kind else {
            panic!(
                "expected Alert root to be a Container, got {:?}",
                element.kind
            );
        };

        assert_eq!(props.padding.right, SpacingLength::Px(Px(72.0)));
    }

    #[test]
    fn alert_action_uses_upstream_offsets_and_merges_layout_refinement() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            AlertAction::new([cx.text("Undo")])
                .refine_layout(LayoutRefinement::default().w_px(Px(88.0)))
                .into_element(cx)
        });

        let ElementKind::Container(props) = &element.kind else {
            panic!(
                "expected AlertAction root to be a Container, got {:?}",
                element.kind
            );
        };

        assert_eq!(props.layout.position, PositionStyle::Absolute);
        let theme = Theme::global(&app);
        assert_eq!(
            props.layout.inset.top,
            InsetEdge::Px(MetricRef::space(Space::N2p5).resolve(theme))
        );
        assert_eq!(
            props.layout.inset.right,
            InsetEdge::Px(MetricRef::space(Space::N3).resolve(theme))
        );
        assert_eq!(props.layout.size.width, Length::Px(Px(88.0)));
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

    #[test]
    fn alert_attaches_foreground_to_main_content_without_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let expected_fg = Theme::global(&*cx.app).color_token("foreground");
            let el = Alert::new([
                decl_icon::icon_with(cx, IconId::new_static("lucide.terminal"), None, None),
                AlertTitle::new("Heads up!").into_element(cx),
                AlertDescription::new("You can add components to your app.").into_element(cx),
            ])
            .into_element(cx);

            let inherited = find_first_inherited_foreground_node(&el)
                .expect("expected alert subtree to carry inherited foreground");
            assert_eq!(inherited.inherited_foreground, Some(expected_fg));
            assert!(
                !contains_foreground_scope(&el),
                "expected Alert to attach inherited foreground without inserting a ForegroundScope"
            );
        });
    }
}
