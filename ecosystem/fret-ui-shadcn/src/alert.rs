use std::sync::Arc;

use fret_core::{Color, FontWeight, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_ui::element::{AnyElement, ElementKind, SemanticsDecoration};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space, ui};

use crate::layout as shadcn_layout;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertVariant {
    #[default]
    Default,
    Destructive,
}

#[derive(Debug, Clone)]
pub struct Alert {
    children: Vec<AnyElement>,
    variant: AlertVariant,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
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

fn maybe_patch_svg_icon(el: &mut AnyElement, color: Color, size: Px) {
    let ElementKind::SvgIcon(props) = &mut el.kind else {
        return;
    };

    props.color = color;
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

fn alert_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: AlertVariant,
    mut children: Vec<AnyElement>,
    chrome_override: ChromeRefinement,
    layout_override: LayoutRefinement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let bg = theme.color_token("card");
    let border = theme.color_token("border");
    let destructive = theme.color_token("destructive");
    let card_fg = theme.color_token("card-foreground");
    let muted_fg = theme.color_token("muted-foreground");
    let fg = match variant {
        AlertVariant::Default => card_fg,
        AlertVariant::Destructive => destructive,
    };
    let destructive_description = alpha_mul(destructive, 0.9);

    let icon = match children.first() {
        Some(first) if matches!(first.kind, ElementKind::SvgIcon(_)) => Some(children.remove(0)),
        _ => None,
    };
    let mut body_children = children;

    let props = decl_style::container_props(
        &theme,
        ChromeRefinement::default()
            .px(Space::N4)
            .py(Space::N3)
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Color(bg))
            .border_color(ColorRef::Color(border))
            .text_color(ColorRef::Color(fg))
            .merge(chrome_override),
        LayoutRefinement::default().merge(layout_override),
    );

    if let Some(from) = theme.color_by_key("foreground") {
        if let Some(title) = body_children.first_mut() {
            maybe_patch_text_color(title, from, fg);
        }
    }

    if let Some(description) = body_children.get_mut(1) {
        match variant {
            AlertVariant::Default => maybe_patch_text_color(description, muted_fg, muted_fg),
            AlertVariant::Destructive => {
                maybe_patch_text_color(description, muted_fg, destructive_description);
            }
        }
    }

    let body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N0p5)
            .layout(LayoutRefinement::default().flex_1().min_w_0()),
        |_cx| body_children,
    );

    let container = if let Some(mut icon) = icon {
        maybe_patch_svg_icon(&mut icon, fg, Px(16.0));
        let icon = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default(),
                LayoutRefinement::default().mt(Space::N0p5),
            ),
            move |_cx| [icon],
        );

        shadcn_layout::container_hstack(
            cx,
            props,
            stack::HStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            vec![icon, body],
        )
    } else {
        shadcn_layout::container_vstack(
            cx,
            props,
            stack::VStackProps::default()
                .gap(Space::N0p5)
                .layout(LayoutRefinement::default().w_full()),
            vec![body],
        )
    };
    container.attach_semantics(SemanticsDecoration::default().role(SemanticsRole::Alert))
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
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.color_token("foreground");
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
            .wrap(TextWrap::Word)
            .overflow(TextOverflow::Clip)
            .text_color(ColorRef::Color(fg))
            .into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct AlertDescription {
    text: Arc<str>,
}

impl AlertDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.color_token("muted-foreground");
        let px = theme
            .metric_by_key("component.alert.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let line_height = theme
            .metric_by_key("component.alert.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_weight(FontWeight::NORMAL)
            .wrap(TextWrap::Word)
            .text_color(ColorRef::Color(fg))
            .into_element(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::ElementKind;

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
}
