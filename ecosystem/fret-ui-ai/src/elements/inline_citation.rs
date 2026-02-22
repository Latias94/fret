use std::sync::Arc;

use fret_core::{
    FontWeight, Px, SemanticsRole, TextAlign, TextOverflow, TextSlant, TextStyle, TextWrap,
};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, LayoutStyle, PressableA11y, PressableProps, SemanticsProps, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, Radius, Space};

use fret_ui_shadcn::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, HoverCard, HoverCardContent,
};

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

fn text_sm_style(theme: &Theme, weight: FontWeight) -> TextStyle {
    let mut style =
        typography::TypographyPreset::control_ui(typography::UiTextSize::Sm).resolve(theme);
    style.weight = weight;
    style.slant = TextSlant::Normal;
    style
}

fn text_xs_style(theme: &Theme, weight: FontWeight, slant: TextSlant) -> TextStyle {
    let mut style =
        typography::TypographyPreset::control_ui(typography::UiTextSize::Xs).resolve(theme);
    style.weight = weight;
    style.slant = slant;
    style
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

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

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

        let badge_layout = LayoutRefinement::default().ml(Space::N1).merge(self.layout);

        let badge = Badge::new(trigger_text.clone())
            .variant(BadgeVariant::Secondary)
            .refine_style(ChromeRefinement::default().rounded(Radius::Full))
            .refine_layout(badge_layout)
            .into_element(cx);

        let on_activate = self.on_activate.clone();
        let trigger_test_id = self.test_id.clone();
        let trigger_label: Arc<str> = Arc::from(trigger_text);

        let trigger = cx.pressable(
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
            let icon = decl_icon::icon_with(
                cx,
                ids::ui::CHEVRON_LEFT,
                Some(Px(16.0)),
                Some(ColorRef::Token {
                    key: "muted-foreground",
                    fallback: ColorFallback::ThemeTextMuted,
                }),
            );
            let mut btn = Button::new("Previous source")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .disabled(prev_disabled)
                .children([icon])
                .on_activate(on_prev);
            if let Some(id) = prev_test_id.clone() {
                btn = btn.test_id(id);
            }
            btn.into_element(cx)
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
            let mut btn = Button::new("Next source")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .disabled(next_disabled)
                .children([icon])
                .on_activate(on_next);
            if let Some(id) = next_test_id.clone() {
                btn = btn.test_id(id);
            }
            btn.into_element(cx)
        };

        let index_label = match index_test_id.clone() {
            Some(test_id) => {
                let index_text = cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: Arc::<str>::from(format!("{current}/{total}")),
                    style: Some(text_xs_style(&theme, FontWeight::NORMAL, TextSlant::Normal)),
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

                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().flex_grow(1.0))
                        .justify_end()
                        .items_center(),
                    move |_cx| vec![labelled],
                )
            }
            None => {
                let index_text = cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: Arc::<str>::from(format!("{current}/{total}")),
                    style: Some(text_xs_style(&theme, FontWeight::NORMAL, TextSlant::Normal)),
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

                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().flex_grow(1.0))
                        .justify_end()
                        .items_center(),
                    move |_cx| vec![index_inner],
                )
            }
        };

        let header_row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2)
                .items_center(),
            move |_cx| vec![prev_btn, next_btn, index_label],
        );

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
            let title_text = cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: source.title.clone(),
                style: Some(text_sm_style(&theme, FontWeight::MEDIUM)),
                color: Some(theme.color_token("foreground")),
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
                align: TextAlign::Start,
                ink_overflow: Default::default(),
            });

            let url_text = match (source.url.clone(), self.on_open_url.clone()) {
                (Some(url), Some(handler)) => {
                    let link = fret_markdown::LinkInfo {
                        href: url.clone(),
                        text: url.clone(),
                    };
                    let on_activate: OnActivate = Arc::new(move |host, cx, reason| {
                        handler(host, cx, reason, link.clone());
                    });

                    let url_text = cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: url.clone(),
                        style: Some(text_xs_style(&theme, FontWeight::NORMAL, TextSlant::Normal)),
                        color: Some(theme.color_token("muted-foreground")),
                        wrap: TextWrap::Grapheme,
                        overflow: TextOverflow::Ellipsis,
                        align: TextAlign::Start,
                        ink_overflow: Default::default(),
                    });

                    Some(cx.pressable(
                        PressableProps {
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
                    layout: LayoutStyle::default(),
                    text: url,
                    style: Some(text_xs_style(&theme, FontWeight::NORMAL, TextSlant::Normal)),
                    color: Some(theme.color_token("muted-foreground")),
                    wrap: TextWrap::Grapheme,
                    overflow: TextOverflow::Ellipsis,
                    align: TextAlign::Start,
                    ink_overflow: Default::default(),
                })),
                _ => None,
            };

            let quote = source.excerpt.clone().map(|excerpt| {
                let mut style = text_sm_style(&theme, FontWeight::NORMAL);
                style.slant = TextSlant::Italic;

                let quote_text = cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: excerpt,
                    style: Some(style),
                    color: Some(theme.color_token("muted-foreground")),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                    align: TextAlign::Start,
                    ink_overflow: Default::default(),
                });

                cx.container(
                    fret_ui::element::ContainerProps {
                        layout: LayoutStyle::default(),
                        padding: fret_core::Edges {
                            top: fret_core::Px(0.0),
                            right: fret_core::Px(0.0),
                            bottom: fret_core::Px(0.0),
                            left: fret_core::Px(12.0),
                        },
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
                        focus_border_color: None,
                        focus_within: false,
                        corner_radii: fret_core::Corners::all(fret_core::Px(0.0)),
                        snap_to_device_pixels: false,
                    },
                    move |_cx| vec![quote_text],
                )
            });

            let source_block = stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N1),
                move |_cx| {
                    let mut out = Vec::new();
                    out.push(title_text);
                    if let Some(url_text) = url_text {
                        out.push(url_text);
                    }
                    out
                },
            );

            let body_inner = stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N2),
                move |_cx| {
                    let mut out = vec![source_block];
                    if let Some(quote) = quote {
                        out.push(quote);
                    }
                    out
                },
            );

            cx.container(
                decl_style::container_props(
                    &theme,
                    ChromeRefinement::default().p(Space::N4).pl(Space::N8),
                    LayoutRefinement::default().w_full(),
                ),
                move |_cx| vec![body_inner],
            )
        } else {
            cx.text(self.label)
        };

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N0),
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
