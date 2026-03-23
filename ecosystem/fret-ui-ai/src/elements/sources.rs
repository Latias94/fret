use std::sync::Arc;

use fret_core::{Corners, Edges, FontWeight, Px, SemanticsRole, TextAlign, TextOverflow, TextWrap};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, PressableA11y, PressableKeyActivation, PressableProps,
    SemanticsDecoration, SemanticsProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, Space,
};
use fret_ui_shadcn::facade::{Collapsible, CollapsibleContent, CollapsibleTrigger};

fn resolve_count_title(template: &Arc<str>, count: usize) -> Arc<str> {
    if template.contains("{count}") {
        Arc::<str>::from(template.replace("{count}", &count.to_string()))
    } else {
        template.clone()
    }
}

/// Docs-shaped root for AI Elements `Sources`.
///
/// This is the typed Fret analogue of:
///
/// - `<Sources>`
/// - `<SourcesTrigger />`
/// - `<SourcesContent />`
///
/// Keep transcript-oriented list conveniences on `SourcesBlock`; use this surface when you want the
/// same composition story as the official AI Elements docs/examples.
#[derive(Clone, Debug)]
pub struct Sources {
    default_open: bool,
    test_id_root: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl Sources {
    pub fn new() -> Self {
        Self {
            default_open: false,
            test_id_root: None,
            layout: LayoutRefinement::default().mb(Space::N4),
        }
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element_parts<H: UiHost>(
        self,
        trigger: SourcesTrigger,
        content: SourcesContent,
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let root_layout = decl_style::layout_style(&theme, self.layout);
        let trigger_test_id = self
            .test_id_root
            .as_ref()
            .map(|id| Arc::<str>::from(format!("{id}-trigger")));
        let content_test_id = self
            .test_id_root
            .as_ref()
            .map(|id| Arc::<str>::from(format!("{id}-content")));
        let default_open = self.default_open;

        let collapsible = Collapsible::uncontrolled(default_open).into_element_with_open_model(
            cx,
            move |cx, open_model, is_open| {
                trigger.into_element(cx, open_model, is_open, trigger_test_id.clone())
            },
            move |cx| content.into_element(cx, content_test_id.clone()),
        );

        // The UI Gallery doc layout uses stretch-heavy column wrappers. Keep the upstream
        // `w-fit`-like intrinsic width by aligning the whole collapsible inside a start-aligned row.
        let aligned = ui::h_row(move |_cx| vec![collapsible])
            .layout(LayoutRefinement::default().w_full())
            .justify(Justify::Start)
            .items(Items::Start)
            .into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: root_layout,
                padding: Edges::all(Px(0.0)).into(),
                background: None,
                corner_radii: Corners::all(Px(0.0)),
                ..Default::default()
            },
            move |_cx| vec![aligned],
        );

        let Some(test_id) = self.test_id_root else {
            return root;
        };

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![root],
        )
    }
}

impl Default for Sources {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SourcesTrigger {
    count: usize,
    title: Arc<str>,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for SourcesTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourcesTrigger")
            .field("count", &self.count)
            .field("title", &self.title.as_ref())
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl SourcesTrigger {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            title: Arc::<str>::from("Used {count} sources"),
            children: Vec::new(),
            test_id: None,
        }
    }

    pub fn title(mut self, title: impl Into<Arc<str>>) -> Self {
        self.title = title.into();
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        open_model: Model<bool>,
        is_open: bool,
        fallback_test_id: Option<Arc<str>>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let label_text = resolve_count_title(&self.title, self.count);
        let icon_color = ColorRef::Token {
            key: "primary",
            fallback: ColorFallback::ThemeAccent,
        };

        let row = if self.children.is_empty() {
            let label = cx.text_props(TextProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default().flex_grow(1.0).min_w_0(),
                ),
                text: label_text.clone(),
                style: Some(typography::preset_text_style_with_overrides(
                    &theme,
                    typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                    Some(FontWeight::MEDIUM),
                    None,
                )),
                color: Some(theme.color_token("primary")),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: TextAlign::Start,
                ink_overflow: Default::default(),
            });

            let chevron = decl_icon::icon_with(
                cx,
                if is_open {
                    ids::ui::CHEVRON_UP
                } else {
                    ids::ui::CHEVRON_DOWN
                },
                Some(Px(16.0)),
                Some(icon_color),
            );

            ui::h_row(move |_cx| vec![label, chevron])
                .layout(LayoutRefinement::default().min_w_0())
                .gap(Space::N2)
                .items(Items::Center)
                .into_element(cx)
        } else {
            let children = self.children;
            ui::h_row(move |_cx| children)
                .layout(LayoutRefinement::default().min_w_0())
                .gap(Space::N2)
                .items(Items::Center)
                .into_element(cx)
        };

        let trigger = CollapsibleTrigger::new(open_model, [row])
            .a11y_label(label_text.clone())
            .into_element(cx, is_open);

        let test_id = self.test_id.or(fallback_test_id);
        let Some(test_id) = test_id else {
            return trigger;
        };

        trigger.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Button)
                .test_id(test_id),
        )
    }
}

pub struct SourcesContent {
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for SourcesContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourcesContent")
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl SourcesContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default().mt(Space::N3),
            children: children.into_iter().collect(),
            test_id: None,
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children.extend(children);
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

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        fallback_test_id: Option<Arc<str>>,
    ) -> AnyElement {
        let list = ui::v_stack(move |_cx| self.children)
            .layout(LayoutRefinement::default().min_w_0())
            .gap(Space::N2)
            .into_element(cx);

        let content = CollapsibleContent::new([list])
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .into_element(cx);

        let test_id = self.test_id.or(fallback_test_id);
        let Some(test_id) = test_id else {
            return content;
        };

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![content],
        )
    }
}

pub struct Source {
    title: Arc<str>,
    href: Option<Arc<str>>,
    on_open_url: Option<fret_markdown::OnLinkActivate>,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Source")
            .field("title", &self.title.as_ref())
            .field("href", &self.href.as_deref())
            .field("has_on_open_url", &self.on_open_url.is_some())
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl Source {
    pub fn new(title: impl Into<Arc<str>>) -> Self {
        Self {
            title: title.into(),
            href: None,
            on_open_url: None,
            children: Vec::new(),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn href(mut self, href: impl Into<Arc<str>>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn on_open_url(mut self, on_open_url: fret_markdown::OnLinkActivate) -> Self {
        self.on_open_url = Some(on_open_url);
        self
    }

    pub fn with_open_url(mut self) -> Self {
        self.on_open_url = Some(fret_markdown::on_link_activate_open_url());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children.extend(children);
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
        let layout = decl_style::layout_style(&theme, self.layout);
        let title = self.title.clone();

        let row = if self.children.is_empty() {
            let icon = decl_icon::icon_with(
                cx,
                ids::ui::BOOK,
                Some(Px(16.0)),
                Some(ColorRef::Token {
                    key: "primary",
                    fallback: ColorFallback::ThemeAccent,
                }),
            );
            let title_text = cx.text_props(TextProps {
                layout: decl_style::layout_style(&theme, LayoutRefinement::default().min_w_0()),
                text: title.clone(),
                style: Some(typography::preset_text_style_with_overrides(
                    &theme,
                    typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                    Some(FontWeight::MEDIUM),
                    None,
                )),
                color: Some(theme.color_token("primary")),
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
                align: TextAlign::Start,
                ink_overflow: Default::default(),
            });

            ui::h_row(move |_cx| vec![icon, title_text])
                .layout(LayoutRefinement::default().min_w_0())
                .gap(Space::N2)
                .items(Items::Center)
                .into_element(cx)
        } else {
            let children = self.children;
            ui::h_row(move |_cx| children)
                .layout(LayoutRefinement::default().min_w_0())
                .gap(Space::N2)
                .items(Items::Center)
                .into_element(cx)
        };

        match (self.href, self.on_open_url) {
            (Some(href), Some(handler)) => {
                let link = fret_markdown::LinkInfo {
                    href,
                    text: title.clone(),
                };
                let on_activate: OnActivate = Arc::new(move |host, cx, reason| {
                    handler(host, cx, reason, link.clone());
                });

                cx.pressable(
                    PressableProps {
                        layout,
                        key_activation: PressableKeyActivation::EnterOnly,
                        a11y: PressableA11y {
                            role: Some(SemanticsRole::Link),
                            label: Some(title),
                            test_id: self.test_id,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx, _state| {
                        cx.pressable_on_activate(on_activate.clone());
                        [row]
                    },
                )
            }
            _ => {
                let root = cx.container(
                    ContainerProps {
                        layout,
                        padding: Edges::all(Px(0.0)).into(),
                        background: None,
                        corner_radii: Corners::all(Px(0.0)),
                        ..Default::default()
                    },
                    move |_cx| vec![row],
                );

                let Some(test_id) = self.test_id else {
                    return root;
                };

                cx.semantics(
                    SemanticsProps {
                        role: SemanticsRole::Group,
                        test_id: Some(test_id),
                        ..Default::default()
                    },
                    move |_cx| vec![root],
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::{ElementKind, Length, PressableProps};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(220.0)),
        )
    }

    fn find_text_by_content<'a>(el: &'a AnyElement, text: &str) -> Option<&'a TextProps> {
        match &el.kind {
            ElementKind::Text(props) if props.text.as_ref() == text => Some(props),
            _ => el
                .children
                .iter()
                .find_map(|child| find_text_by_content(child, text)),
        }
    }

    fn find_pressable_by_label<'a>(el: &'a AnyElement, label: &str) -> Option<&'a PressableProps> {
        match &el.kind {
            ElementKind::Pressable(props) if props.a11y.label.as_deref() == Some(label) => {
                Some(props)
            }
            _ => el
                .children
                .iter()
                .find_map(|child| find_pressable_by_label(child, label)),
        }
    }

    #[test]
    fn sources_trigger_label_can_truncate_within_row() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let title =
            "Used {count} sources with a very long trigger title that should truncate cleanly";
        let resolved = "Used 1 sources with a very long trigger title that should truncate cleanly";

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "sources", |cx| {
            Sources::new().into_element_parts(
                SourcesTrigger::new(1).title(title),
                SourcesContent::new([cx.text("Hidden")]),
                cx,
            )
        });

        let label = find_text_by_content(&el, resolved).expect("sources trigger label");
        assert_eq!(label.wrap, TextWrap::None);
        assert_eq!(label.overflow, TextOverflow::Clip);
        assert_eq!(label.layout.flex.grow, 1.0);
        assert_eq!(label.layout.flex.shrink, 1.0);
        assert_eq!(label.layout.flex.basis, Length::Auto);
        assert_eq!(label.layout.size.min_width, Some(Length::Px(Px(0.0))));
    }

    #[test]
    fn sources_trigger_custom_children_render_inside_button() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "sources", |cx| {
            Sources::new().into_element_parts(
                SourcesTrigger::new(3)
                    .title("Using {count} citations")
                    .children([cx.text("Using 3 citations")]),
                SourcesContent::new([cx.text("Hidden")]),
                cx,
            )
        });

        assert!(
            find_text_by_content(&el, "Using 3 citations").is_some(),
            "custom trigger text should render"
        );
    }

    #[test]
    fn source_link_uses_custom_children_and_link_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let on_open_url: fret_markdown::OnLinkActivate = Arc::new(|_, _, _, _| {});

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "source", |cx| {
            Source::new("Example title")
                .href("https://example.com")
                .on_open_url(on_open_url.clone())
                .children([cx.text("Custom content")])
                .into_element(cx)
        });

        assert!(
            find_text_by_content(&el, "Custom content").is_some(),
            "custom child content should render"
        );
        assert!(
            find_text_by_content(&el, "Example title").is_none(),
            "default title text should not render when custom children are provided"
        );

        let pressable =
            find_pressable_by_label(&el, "Example title").expect("source link pressable");
        assert_eq!(pressable.a11y.role, Some(SemanticsRole::Link));
        assert_eq!(pressable.key_activation, PressableKeyActivation::EnterOnly);
    }
}
