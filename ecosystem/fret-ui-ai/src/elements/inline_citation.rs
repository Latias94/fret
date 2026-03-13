use std::sync::Arc;

use fret_core::{FontWeight, Px, SemanticsRole, TextAlign, TextOverflow, TextSlant, TextWrap};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, LayoutStyle, PressableA11y, PressableProps, SemanticsProps, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, Radius, Space,
};

use fret_ui_shadcn::{Badge, BadgeVariant, HoverCard, HoverCardContent};

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

fn inline_citation_source_body<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    source: SourceItem,
    on_open_url: Option<fret_markdown::OnLinkActivate>,
) -> AnyElement {
    let title_text = cx.text_props(TextProps {
        layout: decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0()),
        text: source.title.clone(),
        style: Some(typography::preset_text_style_with_overrides(
            theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
            Some(FontWeight::MEDIUM),
            Some(TextSlant::Normal),
        )),
        color: Some(theme.color_token("foreground")),
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    });

    let url_text = match (source.url.clone(), on_open_url) {
        (Some(url), Some(handler)) => {
            let link = fret_markdown::LinkInfo {
                href: url.clone(),
                text: url.clone(),
            };
            let on_activate: OnActivate = Arc::new(move |host, cx, reason| {
                handler(host, cx, reason, link.clone());
            });

            let url_text = cx.text_props(TextProps {
                layout: decl_style::layout_style(
                    theme,
                    LayoutRefinement::default().w_full().min_w_0(),
                ),
                text: url.clone(),
                style: Some(typography::preset_text_style_with_overrides(
                    theme,
                    typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                    Some(FontWeight::NORMAL),
                    Some(TextSlant::Normal),
                )),
                color: Some(theme.color_token("muted-foreground")),
                wrap: TextWrap::Grapheme,
                overflow: TextOverflow::Ellipsis,
                align: TextAlign::Start,
                ink_overflow: Default::default(),
            });

            Some(cx.pressable(
                PressableProps {
                    layout: decl_style::layout_style(
                        theme,
                        LayoutRefinement::default().w_full().min_w_0(),
                    ),
                    key_activation: fret_ui::element::PressableKeyActivation::EnterOnly,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Link),
                        label: Some(Arc::<str>::from("Open source URL")),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |cx, _state| {
                    cx.pressable_on_activate(on_activate.clone());
                    [url_text]
                },
            ))
        }
        (Some(url), None) => Some(cx.text_props(TextProps {
            layout: decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0()),
            text: url,
            style: Some(typography::preset_text_style_with_overrides(
                theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                Some(FontWeight::NORMAL),
                Some(TextSlant::Normal),
            )),
            color: Some(theme.color_token("muted-foreground")),
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: Default::default(),
        })),
        _ => None,
    };

    let description = source.description.clone().map(|description| {
        cx.text_props(TextProps {
            layout: decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0()),
            text: description,
            style: Some(typography::preset_text_style_with_overrides(
                theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                Some(FontWeight::NORMAL),
                Some(TextSlant::Normal),
            )),
            color: Some(theme.color_token("muted-foreground")),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: TextAlign::Start,
            ink_overflow: Default::default(),
        })
    });

    let quote = source
        .quote
        .clone()
        .or_else(|| source.excerpt.clone())
        .map(|quote| {
            let style = typography::preset_text_style_with_overrides(
                theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                Some(FontWeight::NORMAL),
                Some(TextSlant::Italic),
            );

            let quote_text = cx.text_props(TextProps {
                layout: decl_style::layout_style(
                    theme,
                    LayoutRefinement::default().w_full().min_w_0(),
                ),
                text: quote,
                style: Some(style),
                color: Some(theme.color_token("muted-foreground")),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
                align: TextAlign::Start,
                ink_overflow: Default::default(),
            });

            cx.container(
                fret_ui::element::ContainerProps {
                    layout: decl_style::layout_style(
                        theme,
                        LayoutRefinement::default().w_full().min_w_0(),
                    ),
                    padding: fret_core::Edges {
                        top: fret_core::Px(0.0),
                        right: fret_core::Px(0.0),
                        bottom: fret_core::Px(0.0),
                        left: fret_core::Px(12.0),
                    }
                    .into(),
                    background: None,
                    background_paint: None,
                    shadow: None,
                    border: fret_core::Edges {
                        top: fret_core::Px(0.0),
                        right: fret_core::Px(0.0),
                        bottom: fret_core::Px(0.0),
                        left: fret_core::Px(2.0),
                    },
                    border_color: Some(theme.color_token("muted")),
                    border_paint: None,
                    border_dash: None,
                    focus_ring: None,
                    focus_ring_always_paint: false,
                    focus_border_color: None,
                    focus_within: false,
                    corner_radii: fret_core::Corners::all(fret_core::Px(0.0)),
                    snap_to_device_pixels: false,
                },
                move |_cx| vec![quote_text],
            )
        });

    let source_block = ui::v_stack(move |_cx| {
        let mut out = Vec::new();
        out.push(title_text);
        if let Some(url_text) = url_text {
            out.push(url_text);
        }
        out
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N1)
    .into_element(cx);

    let body_inner = ui::v_stack(move |_cx| {
        let mut out = vec![source_block];
        if let Some(description) = description {
            out.push(description);
        }
        if let Some(quote) = quote {
            out.push(quote);
        }
        out
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N2)
    .into_element(cx);

    cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default().p(Space::N4).pl(Space::N8),
            LayoutRefinement::default().w_full().min_w_0(),
        ),
        move |_cx| vec![body_inner],
    )
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

    pub fn with_children<I>(children: I) -> InlineCitationWithChildren
    where
        I: IntoIterator<Item = AnyElement>,
    {
        Self::new(Arc::<str>::from("")).children(children)
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
            #[cfg(debug_assertions)]
            if std::env::var_os("FRET_DIAG_DEBUG_AI_INLINE_CITATION_ACTIVATE").is_some() {
                eprintln!("inline_citation activate: source_id={}", source_id.as_ref());
            }
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

    pub fn children<I>(self, children: I) -> InlineCitationWithChildren
    where
        I: IntoIterator<Item = AnyElement>,
    {
        InlineCitationWithChildren {
            root: self,
            children: children.into_iter().collect(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let label = self.label.clone();
        self.into_element_with_children(cx, move |cx| vec![cx.text(label.clone())])
    }

    fn into_element_with_children<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement> + 'static,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

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

        let badge_layout = LayoutRefinement::default().merge(self.layout);

        let badge = Badge::new(trigger_text.clone())
            .variant(BadgeVariant::Secondary)
            .refine_style(ChromeRefinement::default().rounded(Radius::Full))
            .refine_layout(badge_layout)
            .into_element(cx);

        let on_activate = self.on_activate.clone();
        let trigger_test_id = self.test_id.clone();
        let trigger_label: Arc<str> = Arc::from(trigger_text);

        let badge_trigger = cx.pressable(
            PressableProps {
                a11y: PressableA11y {
                    role: Some(SemanticsRole::Button),
                    label: Some(trigger_label),
                    test_id: trigger_test_id,
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, _state| {
                if let Some(handler) = on_activate.clone() {
                    cx.pressable_on_activate(handler);
                }
                [badge]
            },
        );

        let base_id = self.test_id.clone();
        let label_test_id = base_id
            .as_ref()
            .map(|id| Arc::<str>::from(format!("{id}-label")));

        let open_model = cx.local_model(|| false);
        let open_now = cx
            .get_model_copied(&open_model, Invalidation::Paint)
            .unwrap_or(false);

        let mut label_chrome = ChromeRefinement::default().rounded(Radius::Sm);
        if open_now {
            label_chrome = label_chrome.bg(ColorRef::Token {
                key: "accent",
                fallback: ColorFallback::ThemeHoverBackground,
            });
        }
        let inline_children = children(cx);
        let inline_label = match inline_children.len() {
            0 => cx.text(self.label.clone()),
            1 => inline_children
                .into_iter()
                .next()
                .expect("inline citation inline child"),
            _ => ui::h_row(move |_cx| inline_children)
                .gap(Space::N0)
                .items(Items::Center)
                .into_element(cx),
        };

        let label = cx.container(
            decl_style::container_props(&theme, label_chrome, LayoutRefinement::default()),
            move |_cx| vec![inline_label],
        );
        let label = if let Some(test_id) = label_test_id {
            cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: Some(test_id),
                    ..Default::default()
                },
                move |_cx| vec![label],
            )
        } else {
            label
        };

        if resolved_sources.is_empty() {
            return ui::h_row(move |_cx| vec![label, badge_trigger])
                .gap(Space::N1)
                .items(Items::Center)
                .into_element(cx);
        }

        let index_model = cx.local_model(|| 0usize);

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
            let icon = decl_icon::icon_with(
                cx,
                ids::ui::CHEVRON_LEFT,
                Some(Px(16.0)),
                Some(ColorRef::Token {
                    key: "muted-foreground",
                    fallback: ColorFallback::ThemeTextMuted,
                }),
            );
            cx.pressable(
                PressableProps {
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: Some(Arc::<str>::from("Previous")),
                        test_id: prev_test_id.clone(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |cx, _state| {
                    cx.pressable_on_activate(on_prev.clone());
                    [icon]
                },
            )
        };

        let next_btn = {
            let icon = decl_icon::icon_with(
                cx,
                ids::ui::CHEVRON_RIGHT,
                Some(Px(16.0)),
                Some(ColorRef::Token {
                    key: "muted-foreground",
                    fallback: ColorFallback::ThemeTextMuted,
                }),
            );
            cx.pressable(
                PressableProps {
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: Some(Arc::<str>::from("Next")),
                        test_id: next_test_id.clone(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |cx, _state| {
                    cx.pressable_on_activate(on_next.clone());
                    [icon]
                },
            )
        };

        let index_label = match index_test_id.clone() {
            Some(test_id) => {
                let index_text = cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: Arc::<str>::from(format!("{current}/{total}")),
                    style: Some(typography::preset_text_style_with_overrides(
                        &theme,
                        typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                        Some(FontWeight::NORMAL),
                        Some(TextSlant::Normal),
                    )),
                    color: Some(
                        theme
                            .color_by_key("muted-foreground")
                            .unwrap_or_else(|| theme.color_token("muted-foreground")),
                    ),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    align: TextAlign::End,
                    ink_overflow: Default::default(),
                });

                let index_inner = cx.container(
                    decl_style::container_props(
                        &theme,
                        ChromeRefinement::default().px(Space::N3).py(Space::N1),
                        LayoutRefinement::default(),
                    ),
                    move |_cx| vec![index_text],
                );

                let labelled = cx.semantics(
                    SemanticsProps {
                        role: fret_core::SemanticsRole::Group,
                        test_id: Some(test_id),
                        ..Default::default()
                    },
                    move |_cx| vec![index_inner],
                );

                ui::h_row(move |_cx| vec![labelled])
                    .layout(LayoutRefinement::default().flex_grow(1.0))
                    .justify(Justify::End)
                    .items(Items::Center)
                    .into_element(cx)
            }
            None => {
                let index_text = cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: Arc::<str>::from(format!("{current}/{total}")),
                    style: Some(typography::preset_text_style_with_overrides(
                        &theme,
                        typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                        Some(FontWeight::NORMAL),
                        Some(TextSlant::Normal),
                    )),
                    color: Some(
                        theme
                            .color_by_key("muted-foreground")
                            .unwrap_or_else(|| theme.color_token("muted-foreground")),
                    ),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    align: TextAlign::End,
                    ink_overflow: Default::default(),
                });

                let index_inner = cx.container(
                    decl_style::container_props(
                        &theme,
                        ChromeRefinement::default().px(Space::N3).py(Space::N1),
                        LayoutRefinement::default(),
                    ),
                    move |_cx| vec![index_text],
                );

                ui::h_row(move |_cx| vec![index_inner])
                    .layout(LayoutRefinement::default().flex_grow(1.0).min_w_0())
                    .justify(Justify::End)
                    .items(Items::Center)
                    .into_element(cx)
            }
        };

        let header_row = ui::h_row(move |_cx| vec![prev_btn, next_btn, index_label])
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2)
            .items(Items::Center)
            .into_element(cx);

        let header = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Token {
                        key: "secondary",
                        fallback: ColorFallback::ThemePanelBackground,
                    })
                    .p(Space::N2)
                    .rounded_tl(Radius::Md)
                    .rounded_tr(Radius::Md),
                LayoutRefinement::default().w_full(),
            ),
            move |_cx| vec![header_row],
        );

        let source = resolved_sources.get(index_now).cloned();
        let body = if let Some(source) = source {
            inline_citation_source_body(cx, &theme, source, self.on_open_url.clone())
        } else {
            cx.text(self.label)
        };

        let content = ui::v_stack(move |_cx| vec![header, body])
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N0)
            .into_element(cx);

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

        let hover = HoverCard::new(cx, badge_trigger, card)
            .open(Some(open_model))
            .open_delay_frames(0)
            .close_delay_frames(0)
            .into_element(cx);

        ui::h_row(move |_cx| vec![label, hover])
            .gap(Space::N1)
            .items(Items::Center)
            .into_element(cx)
    }
}

pub struct InlineCitationWithChildren {
    root: InlineCitation,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for InlineCitationWithChildren {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InlineCitationWithChildren")
            .field("root", &self.root)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl InlineCitationWithChildren {
    pub fn source_id(mut self, source_id: impl Into<Arc<str>>) -> Self {
        self.root = self.root.source_id(source_id);
        self
    }

    pub fn source_ids(mut self, source_ids: Arc<[Arc<str>]>) -> Self {
        self.root = self.root.source_ids(source_ids);
        self
    }

    pub fn sources(mut self, sources: Arc<[SourceItem]>) -> Self {
        self.root = self.root.sources(sources);
        self
    }

    pub fn on_open_url(mut self, on_open_url: fret_markdown::OnLinkActivate) -> Self {
        self.root = self.root.on_open_url(on_open_url);
        self
    }

    pub fn select_source_model(mut self, model: Model<Option<Arc<str>>>) -> Self {
        self.root = self.root.select_source_model(model);
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.root = self.root.on_activate(on_activate);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.root = self.root.test_id(id);
        self
    }

    pub fn children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.children.extend(children);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.root = self.root.refine_layout(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Self { root, children } = self;
        root.into_element_with_children(cx, move |_cx| children)
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
            Size::new(Px(420.0), Px(280.0)),
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
    fn inline_citation_title_and_url_can_truncate_within_card_width() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let title =
            "A very long source title that should truncate instead of overflowing the hover card";
        let url = "https://example.com/a/very/long/url/that/should/truncate/inside/the/hover/card";
        let on_open_url: fret_markdown::OnLinkActivate = Arc::new(|_, _, _, _| {});

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "citation", |cx| {
            let theme = Theme::global(&*cx.app).clone();
            inline_citation_source_body(
                cx,
                &theme,
                SourceItem::new("source-1", title).url(url),
                Some(on_open_url.clone()),
            )
        });

        let title_text = find_text_by_content(&el, title).expect("inline citation title text");
        assert_eq!(title_text.wrap, TextWrap::None);
        assert_eq!(title_text.overflow, TextOverflow::Ellipsis);
        assert_eq!(title_text.layout.size.width, Length::Fill);
        assert_eq!(title_text.layout.size.min_width, Some(Length::Px(Px(0.0))));

        let url_text = find_text_by_content(&el, url).expect("inline citation url text");
        assert_eq!(url_text.wrap, TextWrap::Grapheme);
        assert_eq!(url_text.overflow, TextOverflow::Ellipsis);
        assert_eq!(url_text.layout.size.width, Length::Fill);
        assert_eq!(url_text.layout.size.min_width, Some(Length::Px(Px(0.0))));

        let link =
            find_pressable_by_label(&el, "Open source URL").expect("inline citation url pressable");
        assert_eq!(link.layout.size.width, Length::Fill);
        assert_eq!(link.layout.size.min_width, Some(Length::Px(Px(0.0))));
    }

    #[test]
    fn inline_citation_quote_can_wrap_within_card_width() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let excerpt = "A very long quoted excerpt that should wrap inside the hover card instead of overflowing in one line.";

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "citation-quote",
            |cx| {
                let theme = Theme::global(&*cx.app).clone();
                inline_citation_source_body(
                    cx,
                    &theme,
                    SourceItem::new("source-1", "Alpha source").excerpt(excerpt),
                    None,
                )
            },
        );

        let quote = find_text_by_content(&el, excerpt).expect("inline citation quote text");
        assert_eq!(quote.wrap, TextWrap::Word);
        assert_eq!(quote.overflow, TextOverflow::Clip);
        assert_eq!(quote.layout.size.width, Length::Fill);
        assert_eq!(quote.layout.size.min_width, Some(Length::Px(Px(0.0))));
    }

    #[test]
    fn inline_citation_source_body_uses_shared_typography_presets() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let title = "Alpha source";
        let url = "https://example.com/source-1";
        let excerpt = "Quoted excerpt";

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "citation-style",
            |cx| {
                let theme = Theme::global(&*cx.app).clone();
                inline_citation_source_body(
                    cx,
                    &theme,
                    SourceItem::new("source-1", title).url(url).excerpt(excerpt),
                    None,
                )
            },
        );

        let theme = Theme::global(&app).clone();
        let title_text = find_text_by_content(&el, title).expect("inline citation title text");
        let url_text = find_text_by_content(&el, url).expect("inline citation url text");
        let quote_text = find_text_by_content(&el, excerpt).expect("inline citation quote text");

        assert_eq!(
            title_text.style,
            Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                Some(FontWeight::MEDIUM),
                Some(TextSlant::Normal),
            ))
        );
        assert_eq!(
            url_text.style,
            Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                Some(FontWeight::NORMAL),
                Some(TextSlant::Normal),
            ))
        );
        assert_eq!(
            quote_text.style,
            Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                Some(FontWeight::NORMAL),
                Some(TextSlant::Italic),
            ))
        );
    }

    #[test]
    fn inline_citation_source_body_renders_description_and_quote_separately() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let description =
            "A comprehensive study on recent developments in natural language processing.";
        let quote = "The technology continues to evolve rapidly.";

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "citation-description-quote",
            |cx| {
                let theme = Theme::global(&*cx.app).clone();
                inline_citation_source_body(
                    cx,
                    &theme,
                    SourceItem::new("source-1", "Alpha source")
                        .description(description)
                        .quote(quote),
                    None,
                )
            },
        );

        let theme = Theme::global(&app).clone();
        let description_text =
            find_text_by_content(&el, description).expect("inline citation description text");
        let quote_text = find_text_by_content(&el, quote).expect("inline citation quote text");

        assert_eq!(description_text.wrap, TextWrap::Word);
        assert_eq!(description_text.overflow, TextOverflow::Clip);
        assert_eq!(
            description_text.style,
            Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                Some(FontWeight::NORMAL),
                Some(TextSlant::Normal),
            ))
        );
        assert_eq!(
            quote_text.style,
            Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                Some(FontWeight::NORMAL),
                Some(TextSlant::Italic),
            ))
        );
    }

    #[test]
    fn inline_citation_with_children_renders_custom_inline_copy() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let inline_copy = "The technology continues to evolve rapidly, with new breakthroughs being announced regularly";
        let sources: Arc<[SourceItem]> = Arc::from(vec![
            SourceItem::new("source-1", "Advances in Natural Language Processing")
                .url("https://example.com/nlp-advances"),
        ]);

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "citation-inline",
            |cx| {
                InlineCitation::with_children([cx.text(inline_copy)])
                    .source_id("source-1")
                    .sources(sources.clone())
                    .into_element(cx)
            },
        );

        let inline_text =
            find_text_by_content(&el, inline_copy).expect("inline citation custom inline copy");
        assert_eq!(inline_text.text.as_ref(), inline_copy);
    }
}
