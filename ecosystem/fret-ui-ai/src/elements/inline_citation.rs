use std::sync::Arc;

use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, SemanticsProps};
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{ChromeRefinement, Justify, LayoutRefinement, Radius, Space};

use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant, HoverCard, HoverCardContent};

use crate::model::SourceItem;

fn hostname_for_url(url: &str) -> Option<&str> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }

    let url = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let host_port = url.split('/').next().unwrap_or("");
    let host = host_port.split(':').next().unwrap_or("");
    (!host.is_empty()).then_some(host)
}

#[derive(Clone)]
/// A small inline citation chip.
///
/// Upstream AI Elements renders citations as hoverable badges that show the source hostname and open
/// a hover card with a small pager. In Fret, this is implemented as a composition of shadcn
/// primitives (`HoverCard`, `Button`).
///
/// Optional: Call `select_source_model` to emit a “selected source” intent on activation (e.g. to
/// highlight the matching row in a nearby `SourcesBlock`).
pub struct InlineCitation {
    label: Arc<str>,
    source_ids: Arc<[Arc<str>]>,
    sources: Option<Arc<[SourceItem]>>,
    on_open_url: Option<fret_markdown::OnLinkActivate>,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for InlineCitation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InlineCitation")
            .field("label", &self.label.as_ref())
            .field("source_ids_len", &self.source_ids.len())
            .field("has_sources", &self.sources.is_some())
            .field("has_on_open_url", &self.on_open_url.is_some())
            .field("has_on_activate", &self.on_activate.is_some())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl InlineCitation {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            source_ids: Vec::<Arc<str>>::new().into(),
            sources: None,
            on_open_url: None,
            on_activate: None,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn source_id(mut self, source_id: impl Into<Arc<str>>) -> Self {
        self.source_ids = vec![source_id.into()].into();
        self
    }

    pub fn source_ids(mut self, source_ids: Arc<[Arc<str>]>) -> Self {
        self.source_ids = source_ids;
        self
    }

    /// Optional resolver list used to render hover-card content and hostname.
    pub fn sources(mut self, sources: Arc<[SourceItem]>) -> Self {
        self.sources = Some(sources);
        self
    }

    pub fn on_open_url(mut self, on_open_url: fret_markdown::OnLinkActivate) -> Self {
        self.on_open_url = Some(on_open_url);
        self
    }

    /// When activated, sets the provided model to `Some(source_id)`.
    ///
    /// This is intended to support “jump/highlight” behaviors by letting apps (or parent
    /// components) observe the selected `source_id` and respond appropriately.
    pub fn select_source_model(mut self, model: Model<Option<Arc<str>>>) -> Self {
        let Some(source_id) = self.source_ids.first().cloned() else {
            return self;
        };

        let on_activate: OnActivate = Arc::new(move |host, _cx, _reason| {
            let _ = host
                .models_mut()
                .update(&model, |v| *v = Some(source_id.clone()));
        });

        self.on_activate = Some(on_activate);
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        #[derive(Default)]
        struct PagerState {
            index: Option<Model<usize>>,
        }

        let resolved_sources = self.sources.as_ref().map(|sources| {
            let mut out: Vec<SourceItem> = Vec::new();
            for id in self.source_ids.iter() {
                if let Some(item) = sources.iter().find(|s| s.id.as_ref() == id.as_ref()) {
                    out.push(item.clone());
                }
            }
            out
        });

        let resolved_sources = resolved_sources.unwrap_or_default();
        let count = self.source_ids.len().max(resolved_sources.len()).max(1);

        let trigger_text = resolved_sources
            .first()
            .and_then(|s| s.url.as_deref())
            .and_then(|url| hostname_for_url(url))
            .map(|host| {
                if count > 1 {
                    format!("{host} +{}", count.saturating_sub(1))
                } else {
                    host.to_string()
                }
            })
            .unwrap_or_else(|| {
                if count > 1 {
                    format!("unknown +{}", count.saturating_sub(1))
                } else {
                    "unknown".to_string()
                }
            });

        let mut trigger = Button::new(trigger_text)
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .refine_style(ChromeRefinement::default().rounded(Radius::Full))
            .refine_layout(self.layout);
        if let Some(on_activate) = self.on_activate.clone() {
            trigger = trigger.on_activate(on_activate);
        }
        if let Some(test_id) = self.test_id.clone() {
            trigger = trigger.test_id(test_id);
        }
        let trigger = trigger.into_element(cx);

        if resolved_sources.is_empty() {
            return trigger;
        }

        let index_model = cx.with_state(PagerState::default, |st| st.index.clone());
        let index_model = match index_model {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(0usize);
                cx.with_state(PagerState::default, |st| st.index = Some(model.clone()));
                model
            }
        };

        let index_now = cx
            .get_model_copied(&index_model, Invalidation::Paint)
            .unwrap_or(0)
            .min(resolved_sources.len().saturating_sub(1));
        if index_now
            != cx
                .get_model_copied(&index_model, Invalidation::Paint)
                .unwrap_or(0)
        {
            let _ = cx.app.models_mut().update(&index_model, |v| *v = index_now);
        }

        let current = index_now.saturating_add(1);
        let total = resolved_sources.len().max(1);

        let base_id = self.test_id.clone();
        let content_test_id = base_id
            .as_ref()
            .map(|id| Arc::<str>::from(format!("{id}-card")));
        let prev_test_id = base_id
            .as_ref()
            .map(|id| Arc::<str>::from(format!("{id}-prev")));
        let next_test_id = base_id
            .as_ref()
            .map(|id| Arc::<str>::from(format!("{id}-next")));
        let index_test_id = base_id
            .as_ref()
            .map(|id| Arc::<str>::from(format!("{id}-index")));

        let prev_disabled = index_now == 0;
        let next_disabled = index_now + 1 >= total;

        let on_prev: OnActivate = Arc::new({
            let index_model = index_model.clone();
            move |host, _cx, _reason| {
                let _ = host.models_mut().update(&index_model, |v| {
                    *v = v.saturating_sub(1);
                });
            }
        });
        let on_next: OnActivate = Arc::new({
            let index_model = index_model.clone();
            move |host, _cx, _reason| {
                let _ = host.models_mut().update(&index_model, |v| {
                    *v = v.saturating_add(1);
                });
            }
        });

        let prev_btn = {
            let mut btn = Button::new("Previous source")
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::IconSm)
                .disabled(prev_disabled)
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .children([decl_icon::icon(
                    cx,
                    IconId::new_static("lucide.chevron-left"),
                )])
                .on_activate(on_prev);
            if let Some(id) = prev_test_id.clone() {
                btn = btn.test_id(id);
            }
            btn.into_element(cx)
        };

        let next_btn = {
            let mut btn = Button::new("Next source")
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::IconSm)
                .disabled(next_disabled)
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .children([decl_icon::icon(
                    cx,
                    IconId::new_static("lucide.chevron-right"),
                )])
                .on_activate(on_next);
            if let Some(id) = next_test_id.clone() {
                btn = btn.test_id(id);
            }
            btn.into_element(cx)
        };

        let index_label = match index_test_id.clone() {
            Some(test_id) => {
                let label = cx.text(format!("{current}/{total}"));
                cx.semantics(
                    SemanticsProps {
                        role: fret_core::SemanticsRole::Group,
                        test_id: Some(test_id),
                        ..Default::default()
                    },
                    move |_cx| vec![label],
                )
            }
            None => cx.text(format!("{current}/{total}")),
        };

        let header = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2)
                .justify(Justify::Between),
            move |_cx| vec![prev_btn, index_label, next_btn],
        );

        let source = resolved_sources.get(index_now).cloned();
        let body = if let Some(source) = source {
            let title = match (source.url.clone(), self.on_open_url.clone()) {
                (Some(url), Some(handler)) => {
                    let link = fret_markdown::LinkInfo {
                        href: url.clone(),
                        text: source.title.clone(),
                    };
                    let on_activate: OnActivate = Arc::new(move |host, cx, reason| {
                        handler(host, cx, reason, link.clone());
                    });
                    Button::new(source.title.clone())
                        .variant(ButtonVariant::Link)
                        .size(ButtonSize::Sm)
                        .on_activate(on_activate)
                        .into_element(cx)
                }
                _ => cx.text(source.title),
            };

            let excerpt = source.excerpt.map(|t| cx.text(t));

            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N2),
                move |_cx| {
                    let mut out = Vec::new();
                    out.push(title);
                    if let Some(excerpt) = excerpt {
                        out.push(excerpt);
                    }
                    out
                },
            )
        } else {
            cx.text(self.label)
        };

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2),
            move |_cx| vec![header, body],
        );

        let card = HoverCardContent::new(vec![content])
            .refine_style(ChromeRefinement::default().p(Space::N0))
            .refine_layout(LayoutRefinement::default().w_px(fret_core::Px(320.0)));

        let card = card.into_element(cx);

        let card = if let Some(test_id) = content_test_id {
            cx.semantics(
                SemanticsProps {
                    role: fret_core::SemanticsRole::Group,
                    test_id: Some(test_id),
                    ..Default::default()
                },
                move |_cx| vec![card],
            )
        } else {
            card
        };

        HoverCard::new(trigger, card)
            .open_delay_frames(0)
            .close_delay_frames(0)
            .into_element(cx)
    }
}
