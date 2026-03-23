use std::sync::Arc;

use fret_core::{FontWeight, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{Items, Justify, LayoutRefinement, Space};

/// A centered empty-state card for transcript/conversation surfaces.
pub struct ConversationEmptyState {
    title: Arc<str>,
    description: Option<Arc<str>>,
    icon: Option<AnyElement>,
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ConversationEmptyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConversationEmptyState")
            .field("title", &self.title.as_ref())
            .field("has_description", &self.description.is_some())
            .field("has_icon", &self.icon.is_some())
            .field("has_children", &self.children.is_some())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl Default for ConversationEmptyState {
    fn default() -> Self {
        Self {
            title: Arc::<str>::from("No messages yet"),
            description: Some(Arc::<str>::from(
                "Start a conversation to see messages here",
            )),
            icon: None,
            children: None,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }
}

impl ConversationEmptyState {
    pub fn new(title: impl Into<Arc<str>>) -> Self {
        Self {
            title: title.into(),
            description: None,
            icon: None,
            children: None,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn icon(mut self, icon: AnyElement) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let title_style = typography::as_control_text(TextStyle {
            font: Default::default(),
            size: theme.metric_token("font.size"),
            weight: FontWeight::MEDIUM,
            slant: Default::default(),
            line_height: Some(theme.metric_token("font.line_height")),
            letter_spacing_em: None,
            ..Default::default()
        });

        let title = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.title,
            style: Some(title_style),
            color: Some(theme.color_token("foreground")),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Center,
            ink_overflow: Default::default(),
        });

        let description = self.description.map(|text| {
            typography::scope_description_text_with_fallbacks(
                cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text,
                    style: None,
                    color: None,
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                    align: fret_core::TextAlign::Center,
                    ink_overflow: Default::default(),
                }),
                &theme,
                "component.conversation.empty_state.description",
                Some("component.text.sm_px"),
                Some("component.text.sm_line_height"),
            )
        });

        let text_block = ui::v_stack(move |_cx| {
            let mut text_children = vec![title];
            if let Some(description) = description {
                text_children.push(description);
            }
            text_children
        })
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .items(Items::Center)
        .gap(Space::N1)
        .into_element(cx);

        let body = if let Some(children) = self.children {
            ui::v_stack(move |_cx| children)
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .items(Items::Center)
                .gap(Space::N3)
                .into_element(cx)
        } else {
            let icon = self.icon;
            ui::v_stack(move |_cx| {
                let mut out = Vec::new();
                if let Some(icon) = icon {
                    out.push(icon);
                }
                out.push(text_block);
                out
            })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .items(Items::Center)
            .gap(Space::N3)
            .into_element(cx)
        };

        let centered = ui::v_stack(move |_cx| vec![body])
            .layout(LayoutRefinement::default().w_full().h_full())
            .justify(Justify::Center)
            .items(Items::Center)
            .gap(Space::N4)
            .into_element(cx);

        let container = ui::v_stack(move |_cx| vec![centered])
            .layout(
                LayoutRefinement::default()
                    .w_full()
                    .h_full()
                    .min_h(theme.metric_token("metric.size.lg"))
                    .merge(self.layout),
            )
            .items(Items::Center)
            .into_element(cx);

        let Some(test_id) = self.test_id else {
            return container;
        };

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![container],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Point, Px, Rect, Size, TextAlign, TextLineHeightPolicy, TextWrap,
    };
    use fret_ui::ThemeConfig;
    use fret_ui::element::ElementKind;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(480.0), Px(320.0)),
        )
    }

    fn has_text(element: &AnyElement, expected: &str) -> bool {
        match &element.kind {
            ElementKind::Text(TextProps { text, .. }) if text.as_ref() == expected => true,
            _ => element
                .children
                .iter()
                .any(|child| has_text(child, expected)),
        }
    }

    fn find_text_element<'a>(element: &'a AnyElement, expected: &str) -> Option<&'a AnyElement> {
        match &element.kind {
            ElementKind::Text(TextProps { text, .. }) if text.as_ref() == expected => Some(element),
            _ => element
                .children
                .iter()
                .find_map(|child| find_text_element(child, expected)),
        }
    }

    #[test]
    fn empty_state_default_copy_matches_docs() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                ConversationEmptyState::default().into_element(cx)
            });

        assert!(has_text(&element, "No messages yet"));
        assert!(has_text(
            &element,
            "Start a conversation to see messages here"
        ));
    }

    #[test]
    fn empty_state_custom_children_replace_default_copy() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                ConversationEmptyState::default()
                    .children([cx.text("Custom content")])
                    .into_element(cx)
            });

        assert!(has_text(&element, "Custom content"));
        assert!(!has_text(&element, "No messages yet"));
    }
    #[test]
    fn empty_state_description_scopes_inherited_description_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "ConversationEmptyState Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("component.text.sm_px".to_string(), 13.0),
                    ("component.text.sm_line_height".to_string(), 18.0),
                ]),
                colors: std::collections::HashMap::from([(
                    "muted-foreground".to_string(),
                    "#667788".to_string(),
                )]),
                ..ThemeConfig::default()
            });
        });

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                ConversationEmptyState::default().into_element(cx)
            });

        let description = find_text_element(&element, "Start a conversation to see messages here")
            .expect("expected description text node");
        let ElementKind::Text(props) = &description.kind else {
            panic!("expected description to be a text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.align, TextAlign::Center);

        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            description.inherited_foreground,
            Some(typography::muted_foreground_color(&theme))
        );
        assert_eq!(
            description.inherited_text_style,
            Some(typography::description_text_refinement_with_fallbacks(
                &theme,
                "component.conversation.empty_state.description",
                Some("component.text.sm_px"),
                Some("component.text.sm_line_height"),
            ))
        );
        assert_eq!(
            description
                .inherited_text_style
                .as_ref()
                .and_then(|style| style.line_height_policy),
            Some(TextLineHeightPolicy::FixedFromStyle)
        );
    }
}
