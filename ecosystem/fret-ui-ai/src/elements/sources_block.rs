use std::sync::Arc;

use fret_core::{Corners, Edges, FontWeight, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};

use fret_ui_shadcn::{Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Card};

use crate::model::SourceItem;

#[derive(Clone)]
pub struct SourcesBlock {
    items: Arc<[SourceItem]>,
    title: Arc<str>,
    on_open_url: Option<fret_markdown::OnLinkActivate>,
    highlighted_source_id: Option<Arc<str>>,
    highlighted_source_model: Option<Model<Option<Arc<str>>>>,
    test_id_root: Option<Arc<str>>,
    test_id_row_prefix: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for SourcesBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourcesBlock")
            .field("items_len", &self.items.len())
            .field("title", &self.title.as_ref())
            .field("has_on_open_url", &self.on_open_url.is_some())
            .field(
                "highlighted_source_id",
                &self.highlighted_source_id.as_deref(),
            )
            .field(
                "has_highlighted_source_model",
                &self.highlighted_source_model.is_some(),
            )
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("test_id_row_prefix", &self.test_id_row_prefix.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl SourcesBlock {
    pub fn new(items: impl Into<Arc<[SourceItem]>>) -> Self {
        Self {
            items: items.into(),
            title: Arc::<str>::from("Sources"),
            on_open_url: None,
            highlighted_source_id: None,
            highlighted_source_model: None,
            test_id_root: None,
            test_id_row_prefix: None,
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn title(mut self, title: impl Into<Arc<str>>) -> Self {
        self.title = title.into();
        self
    }

    pub fn on_open_url(mut self, on_open_url: fret_markdown::OnLinkActivate) -> Self {
        self.on_open_url = Some(on_open_url);
        self
    }

    pub fn highlighted_source_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.highlighted_source_id = Some(id.into());
        self
    }

    pub fn highlighted_source_model(mut self, model: Model<Option<Arc<str>>>) -> Self {
        self.highlighted_source_model = Some(model);
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn test_id_row_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_row_prefix = Some(prefix.into());
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let highlighted_source_id = self
            .highlighted_source_model
            .as_ref()
            .and_then(|m| cx.get_model_cloned(m, Invalidation::Paint).unwrap_or(None))
            .or(self.highlighted_source_id);

        let title = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2),
            |cx| {
                let label_style = TextStyle {
                    font: Default::default(),
                    size: theme.metric_required("font.size_sm"),
                    weight: FontWeight::MEDIUM,
                    slant: Default::default(),
                    line_height: Some(theme.metric_required("font.line_height")),
                    letter_spacing_em: None,
                };

                let label = cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: self.title.clone(),
                    style: Some(label_style),
                    color: theme.color_by_key("muted-foreground"),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                });

                let count = Badge::new(format!("{}", self.items.len()))
                    .variant(BadgeVariant::Secondary)
                    .into_element(cx);

                vec![label, count]
            },
        );

        let items = self.items;
        let on_open_url = self.on_open_url;
        let row_prefix = self.test_id_row_prefix;
        let highlighted_source_id = highlighted_source_id.clone();

        let list = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2),
            |cx| {
                let mut out = Vec::new();
                for (index, item) in items.iter().enumerate() {
                    let row_test_id = row_prefix
                        .clone()
                        .map(|p| Arc::<str>::from(format!("{p}{index}")));

                    let is_highlighted = highlighted_source_id
                        .as_ref()
                        .is_some_and(|id| id.as_ref() == item.id.as_ref());

                    let title_el: AnyElement = match (&item.url, on_open_url.clone()) {
                        (Some(url), Some(handler)) => {
                            let link = fret_markdown::LinkInfo {
                                href: url.clone(),
                                text: item.title.clone(),
                            };

                            let on_activate: OnActivate = Arc::new(move |host, cx, reason| {
                                handler(host, cx, reason, link.clone());
                            });

                            let mut btn = Button::new(item.title.clone())
                                .variant(ButtonVariant::Link)
                                .size(ButtonSize::Sm)
                                .on_activate(on_activate);
                            if let Some(id) = row_test_id.clone() {
                                btn = btn.test_id(Arc::<str>::from(format!("{id}-link")));
                            }
                            btn.into_element(cx)
                        }
                        _ => cx.text(item.title.clone()),
                    };

                    let active_badge = if is_highlighted {
                        row_test_id.clone().map(|id| {
                            let badge = Badge::new("Active")
                                .variant(BadgeVariant::Secondary)
                                .into_element(cx);
                            cx.semantics(
                                SemanticsProps {
                                    role: SemanticsRole::Group,
                                    test_id: Some(Arc::<str>::from(format!("{id}-active"))),
                                    ..Default::default()
                                },
                                move |_cx| vec![badge],
                            )
                        })
                    } else {
                        None
                    };

                    let title_row = if let Some(active_badge) = active_badge {
                        stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full())
                                .gap(Space::N2),
                            |_cx| vec![title_el, active_badge],
                        )
                    } else {
                        title_el
                    };

                    let excerpt_el = item.excerpt.clone().map(|excerpt| {
                        cx.text_props(TextProps {
                            layout: LayoutStyle::default(),
                            text: excerpt,
                            style: None,
                            color: theme.color_by_key("muted-foreground"),
                            wrap: TextWrap::Word,
                            overflow: TextOverflow::Clip,
                        })
                    });

                    let row = stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N1),
                        |_cx| {
                            let mut row = Vec::new();
                            row.push(title_row);
                            if let Some(excerpt_el) = excerpt_el {
                                row.push(excerpt_el);
                            }
                            row
                        },
                    );

                    let row: AnyElement = if is_highlighted {
                        let bg = theme
                            .color_by_key("accent")
                            .unwrap_or_else(|| theme.color_required("muted"));
                        let padding = decl_style::space(&theme, Space::N2);
                        let radius = decl_style::radius(&theme, fret_ui_kit::Radius::Md);
                        let layout =
                            decl_style::layout_style(&theme, LayoutRefinement::default().w_full());

                        cx.container(
                            ContainerProps {
                                layout,
                                padding: Edges::all(padding),
                                background: Some(bg),
                                corner_radii: Corners::all(radius),
                                ..Default::default()
                            },
                            move |_cx| vec![row],
                        )
                    } else {
                        row
                    };

                    let Some(test_id) = row_test_id else {
                        out.push(row);
                        continue;
                    };

                    out.push(cx.semantics(
                        SemanticsProps {
                            role: SemanticsRole::Group,
                            test_id: Some(test_id),
                            ..Default::default()
                        },
                        move |_cx| vec![row],
                    ));
                }
                out
            },
        );

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N3),
            |_cx| vec![title, list],
        );

        let card = Card::new(vec![body])
            .refine_layout(self.layout)
            .refine_style(self.chrome);

        let card = card.into_element(cx);

        let Some(test_id) = self.test_id_root else {
            return card;
        };

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![card],
        )
    }
}
