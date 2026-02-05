use std::sync::Arc;

use fret_core::{FontWeight, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};

use fret_ui_shadcn::{Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Card};

use crate::model::SourceItem;

#[derive(Clone)]
pub struct SourcesBlock {
    items: Arc<[SourceItem]>,
    title: Arc<str>,
    on_open_url: Option<fret_markdown::OnLinkActivate>,
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
                                btn = btn.test_id(id);
                            }
                            btn.into_element(cx)
                        }
                        _ => cx.text(item.title.clone()),
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
                            row.push(title_el);
                            if let Some(excerpt_el) = excerpt_el {
                                row.push(excerpt_el);
                            }
                            row
                        },
                    );

                    out.push(row);
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
