use std::sync::Arc;

use fret_core::{Corners, Edges, SemanticsRole, TextOverflow, TextWrap};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, Justify, LayoutRefinement, Radius, Space};

use fret_ui_shadcn::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Collapsible, CollapsibleContent,
};

use crate::model::SourceItem;

#[derive(Clone)]
/// A collapsible list view for assistant sources/references (AI Elements `sources.tsx`-style).
///
/// Apps still own effects. When `on_open_url` is set, link activation emits an intent with
/// `LinkInfo { href, text }`.
pub struct SourcesBlock {
    items: Arc<[SourceItem]>,
    title: Arc<str>,
    default_open: bool,
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
            .field("default_open", &self.default_open)
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
            title: Arc::<str>::from("Used sources"),
            default_open: false,
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

    /// Sets whether the Collapsible content is open by default.
    ///
    /// Upstream AI Elements hides sources by default.
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
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
        let root_layout = decl_style::layout_style(&theme, self.layout);
        let excerpt_color = theme.color_by_key("muted-foreground");

        let highlight_bg = theme
            .color_by_key("accent")
            .unwrap_or_else(|| theme.color_required("muted"));
        let highlight_padding = decl_style::space(&theme, Space::N2);
        let highlight_radius = decl_style::radius(&theme, Radius::Md);
        let highlight_layout =
            decl_style::layout_style(&theme, LayoutRefinement::default().w_full());

        let trigger_test_id = self
            .test_id_root
            .as_ref()
            .map(|id| Arc::<str>::from(format!("{id}-trigger")));
        let content_test_id = self
            .test_id_root
            .as_ref()
            .map(|id| Arc::<str>::from(format!("{id}-content")));

        let highlighted_source_id = self
            .highlighted_source_model
            .as_ref()
            .and_then(|m| cx.get_model_cloned(m, Invalidation::Paint).unwrap_or(None))
            .or(self.highlighted_source_id);

        let items = self.items;
        let count = items.len();
        let items_for_list = items.clone();
        let on_open_url = self.on_open_url;
        let row_prefix = self.test_id_row_prefix;
        let chrome = self.chrome;

        let list = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2),
            move |cx| {
                let mut out = Vec::new();
                for (index, item) in items_for_list.iter().enumerate() {
                    let row_test_id = row_prefix
                        .clone()
                        .map(|p| Arc::<str>::from(format!("{p}{index}")));

                    let is_highlighted = highlighted_source_id
                        .as_ref()
                        .is_some_and(|id| id.as_ref() == item.id.as_ref());

                    let icon = decl_icon::icon(cx, IconId::new_static("lucide.book"));

                    let title_el: AnyElement = match (&item.url, on_open_url.clone()) {
                        (Some(url), Some(handler)) => {
                            let link = fret_markdown::LinkInfo {
                                href: url.clone(),
                                text: item.title.clone(),
                            };

                            let on_activate: OnActivate = Arc::new(move |host, cx, reason| {
                                handler(host, cx, reason, link.clone());
                            });

                            let title = cx.text(item.title.clone());
                            let mut btn = Button::new(item.title.clone())
                                .variant(ButtonVariant::Link)
                                .size(ButtonSize::Sm)
                                .children([icon, title])
                                .on_activate(on_activate);
                            if let Some(id) = row_test_id.clone() {
                                btn = btn.test_id(Arc::<str>::from(format!("{id}-link")));
                            }
                            btn.into_element(cx)
                        }
                        _ => stack::hstack(
                            cx,
                            stack::HStackProps::default().gap(Space::N2),
                            move |cx| vec![icon, cx.text(item.title.clone())],
                        ),
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

                    let row = stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N2)
                            .justify(Justify::Between),
                        move |_cx| {
                            let mut row = Vec::new();
                            row.push(title_el);
                            if let Some(active_badge) = active_badge {
                                row.push(active_badge);
                            }
                            row
                        },
                    );

                    let excerpt_el = item.excerpt.clone().map(|excerpt| {
                        cx.text_props(TextProps {
                            layout: LayoutStyle::default(),
                            text: excerpt,
                            style: None,
                            color: excerpt_color,
                            wrap: TextWrap::Word,
                            overflow: TextOverflow::Clip,
                        })
                    });

                    let body = stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N1),
                        move |_cx| {
                            let mut body = Vec::new();
                            body.push(row);
                            if let Some(excerpt_el) = excerpt_el {
                                body.push(excerpt_el);
                            }
                            body
                        },
                    );

                    let body: AnyElement = if is_highlighted {
                        cx.container(
                            ContainerProps {
                                layout: highlight_layout,
                                padding: Edges::all(highlight_padding),
                                background: Some(highlight_bg),
                                corner_radii: Corners::all(highlight_radius),
                                ..Default::default()
                            },
                            move |_cx| vec![body],
                        )
                    } else {
                        body
                    };

                    let Some(test_id) = row_test_id else {
                        out.push(body);
                        continue;
                    };

                    out.push(cx.semantics(
                        SemanticsProps {
                            role: SemanticsRole::Group,
                            test_id: Some(test_id),
                            ..Default::default()
                        },
                        move |_cx| vec![body],
                    ));
                }
                out
            },
        );

        let default_open = self.default_open;

        let collapsible = Collapsible::uncontrolled(default_open).into_element_with_open_model(
            cx,
            move |cx, open_model, is_open| {
                let debug_toggle = std::env::var_os("FRET_DIAG_DEBUG_AI_SOURCES_TOGGLE")
                    .is_some_and(|v| !v.is_empty() && v != "0");
                let debug_test_id = trigger_test_id.clone();
                let on_activate: OnActivate = Arc::new(move |host, cx, reason| {
                    let store = host.models_mut();
                    let prev = store.get_copied(&open_model).unwrap_or(false);
                    let _ = store.update(&open_model, |v| *v = !*v);
                    let next = store.get_copied(&open_model).unwrap_or(!prev);

                    if debug_toggle {
                        eprintln!(
                            "ai sources toggle activated test_id={} prev={} next={} reason={reason:?}",
                            debug_test_id.as_deref().unwrap_or("<none>"),
                            prev,
                            next
                        );
                    }
                    host.notify(cx);
                });

                let label = cx.text(format!("Used {count} sources"));
                let chevron_id = if is_open {
                    IconId::new_static("lucide.chevron-up")
                } else {
                    IconId::new_static("lucide.chevron-down")
                };
                let chevron = decl_icon::icon(cx, chevron_id);
                let row = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default())
                        .gap(Space::N2),
                    move |_cx| vec![label, chevron],
                );

                let mut btn = Button::new(format!("Used {count} sources"))
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .children([row])
                    .on_activate(on_activate)
                    .refine_layout(LayoutRefinement::default());
                if let Some(test_id) = trigger_test_id.clone() {
                    btn = btn.test_id(test_id);
                }
                btn.into_element(cx)
            },
            move |cx| {
                let content = CollapsibleContent::new(vec![list.clone()])
                    .refine_style(ChromeRefinement::default().p(Space::N2).pt(Space::N3))
                    .refine_style(chrome.clone())
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx);

                let Some(test_id) = content_test_id.clone() else {
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
            },
        );

        let root = cx.container(
            ContainerProps {
                layout: root_layout,
                padding: Edges::all(fret_core::Px(0.0)),
                background: None,
                corner_radii: Corners::all(fret_core::Px(0.0)),
                ..Default::default()
            },
            move |_cx| vec![collapsible],
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
