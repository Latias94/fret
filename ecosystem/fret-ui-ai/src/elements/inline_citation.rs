use std::sync::Arc;

use fret_core::{FontWeight, Px, SemanticsRole, TextAlign, TextOverflow, TextSlant, TextWrap};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, InteractivityGateProps, LayoutStyle, PressableA11y, PressableProps, SemanticsProps,
    TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, IntoUiElement, Items, Justify, LayoutRefinement,
    Radius, Space,
};

use fret_ui_shadcn::facade::{Badge, BadgeVariant, HoverCard, HoverCardContent};

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

fn hidden<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.interactivity_gate_props(
        InteractivityGateProps {
            layout: LayoutStyle::default(),
            present: false,
            interactive: false,
        },
        |_cx| Vec::new(),
    )
}

fn inline_citation_title_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    title: Arc<str>,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0()),
        text: title,
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
    })
}

fn inline_citation_url_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    url: Arc<str>,
    on_open_url: Option<fret_markdown::OnLinkActivate>,
) -> AnyElement {
    match on_open_url {
        Some(handler) => {
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

            cx.pressable(
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
            )
        }
        None => cx.text_props(TextProps {
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
        }),
    }
}

fn inline_citation_description_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    description: Arc<str>,
) -> AnyElement {
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
}

fn inline_citation_quote_block<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    quote: Arc<str>,
) -> AnyElement {
    let style = typography::preset_text_style_with_overrides(
        theme,
        typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
        Some(FontWeight::NORMAL),
        Some(TextSlant::Italic),
    );

    let quote_text = cx.text_props(TextProps {
        layout: decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0()),
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
            layout: decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0()),
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
}

// Legacy helper kept only for unit tests; production code uses the compound parts below.
#[cfg(test)]
fn inline_citation_source_body<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    source: SourceItem,
    on_open_url: Option<fret_markdown::OnLinkActivate>,
) -> AnyElement {
    let title_text = inline_citation_title_text(cx, theme, source.title.clone());

    let url_text = source
        .url
        .clone()
        .map(|url| inline_citation_url_element(cx, theme, url, on_open_url));

    let description = source
        .description
        .clone()
        .map(|description| inline_citation_description_text(cx, theme, description));

    let quote = source
        .quote
        .clone()
        .or_else(|| source.excerpt.clone())
        .map(|quote| inline_citation_quote_block(cx, theme, quote));

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
/// Shared state provided by `InlineCitationRoot` for compound-part authoring.
///
/// This mirrors the AI Elements "compound components" architecture, while keeping the mechanism
/// (hover-card open state, paging index, resolved sources) policy-owned in `fret-ui-ai`.
pub struct InlineCitationParts {
    resolved_sources: Arc<[SourceItem]>,
    source_ids_len: usize,
    open_model: Model<bool>,
    index_model: Model<usize>,
    on_open_url: Option<fret_markdown::OnLinkActivate>,
    on_activate: Option<OnActivate>,
    test_id_base: Option<Arc<str>>,
}

impl std::fmt::Debug for InlineCitationParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InlineCitationParts")
            .field("resolved_sources_len", &self.resolved_sources.len())
            .field("source_ids_len", &self.source_ids_len)
            .field("has_on_open_url", &self.on_open_url.is_some())
            .field("has_on_activate", &self.on_activate.is_some())
            .field("test_id_base", &self.test_id_base.as_deref())
            .finish()
    }
}

pub fn use_inline_citation_parts<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<InlineCitationParts> {
    cx.provided::<InlineCitationParts>().cloned()
}

impl InlineCitationParts {
    pub fn resolved_sources(&self) -> &[SourceItem] {
        &self.resolved_sources
    }

    pub fn open_model(&self) -> Model<bool> {
        self.open_model.clone()
    }

    pub fn index_model(&self) -> Model<usize> {
        self.index_model.clone()
    }

    pub fn on_open_url(&self) -> Option<fret_markdown::OnLinkActivate> {
        self.on_open_url.clone()
    }

    pub fn on_activate(&self) -> Option<OnActivate> {
        self.on_activate.clone()
    }

    pub fn badge_count(&self) -> usize {
        self.source_ids_len.max(self.resolved_sources.len()).max(1)
    }

    pub fn derived_test_id(&self, suffix: &str) -> Option<Arc<str>> {
        self.test_id_base
            .as_ref()
            .map(|id| Arc::<str>::from(format!("{id}-{suffix}")))
    }

    pub fn base_test_id(&self) -> Option<Arc<str>> {
        self.test_id_base.clone()
    }
}

fn inline_citation_clamped_index<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    index_model: &Model<usize>,
    len: usize,
) -> usize {
    let idx = cx
        .get_model_copied(index_model, Invalidation::Paint)
        .unwrap_or(0);
    let clamped = idx.min(len.saturating_sub(1));
    if clamped != idx {
        let _ = cx.app.models_mut().update(index_model, |v| *v = clamped);
    }
    clamped
}

#[derive(Clone)]
/// Compound-parts root aligned with AI Elements `InlineCitation`.
///
/// This root:
/// - resolves `source_ids + sources` into a pager-friendly list
/// - provides `InlineCitationParts` to descendant parts via `cx.provide(...)`
/// - owns the hover-card open model and paging index model
///
/// The default visual chrome is implemented by the part types below. Apps can override structure
/// by supplying a custom `children` builder in `into_element_with_children`.
pub struct InlineCitationRoot {
    source_ids: Arc<[Arc<str>]>,
    sources: Option<Arc<[SourceItem]>>,
    on_open_url: Option<fret_markdown::OnLinkActivate>,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for InlineCitationRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InlineCitationRoot")
            .field("source_ids_len", &self.source_ids.len())
            .field("has_sources", &self.sources.is_some())
            .field("has_on_open_url", &self.on_open_url.is_some())
            .field("has_on_activate", &self.on_activate.is_some())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl InlineCitationRoot {
    pub fn new() -> Self {
        Self {
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

    pub fn sources(mut self, sources: Arc<[SourceItem]>) -> Self {
        self.sources = Some(sources);
        self
    }

    pub fn on_open_url(mut self, on_open_url: fret_markdown::OnLinkActivate) -> Self {
        self.on_open_url = Some(on_open_url);
        self
    }

    /// When activated, sets the provided model to `Some(source_id)`.
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

    /// Eager helper for the upstream-shaped compound-parts usage lane.
    ///
    /// This is equivalent to calling `into_element_with_children(...)` and landing the provided
    /// part values inside the nearest `InlineCitationParts` provider scope.
    #[track_caller]
    pub fn into_element_parts<H, TText, TCard>(
        self,
        text: TText,
        card: TCard,
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement
    where
        H: UiHost + 'static,
        TText: IntoUiElement<H> + 'static,
        TCard: IntoUiElement<H> + 'static,
    {
        self.into_element_with_children(cx, move |cx| {
            vec![text.into_element(cx), card.into_element(cx)]
        })
    }

    pub fn into_element_with_children<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement> + 'static,
    ) -> AnyElement {
        let resolved_sources: Arc<[SourceItem]> = self
            .sources
            .as_ref()
            .map(|sources| {
                let mut out: Vec<SourceItem> = Vec::new();
                for id in self.source_ids.iter() {
                    if let Some(item) = sources.iter().find(|s| s.id.as_ref() == id.as_ref()) {
                        out.push(item.clone());
                    }
                }
                out.into()
            })
            .unwrap_or_else(|| Arc::<[SourceItem]>::from(Vec::<SourceItem>::new()));

        let open_model = cx.local_model(|| false);
        let index_model = cx.local_model(|| 0usize);

        let parts = InlineCitationParts {
            resolved_sources: resolved_sources.clone(),
            source_ids_len: self.source_ids.len(),
            open_model: open_model.clone(),
            index_model: index_model.clone(),
            on_open_url: self.on_open_url.clone(),
            on_activate: self.on_activate.clone(),
            test_id_base: self.test_id.clone(),
        };

        let root_layout = LayoutRefinement::default().min_w_0().merge(self.layout);

        cx.provide(parts.clone(), |cx| {
            let _ = inline_citation_clamped_index(cx, &index_model, resolved_sources.len());

            let mut children = children(cx);
            if children.is_empty() {
                children.push(InlineCitationCard::new().into_element(cx));
            }

            ui::h_row(move |_cx| children)
                .layout(root_layout)
                .gap(Space::N1)
                .items(Items::Center)
                .into_element(cx)
        })
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_with_children(cx, |_cx| Vec::new())
    }
}

impl Default for InlineCitationRoot {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost + 'static> IntoUiElement<H> for InlineCitationText {
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        InlineCitationText::into_element(self, cx)
    }
}

impl<H: UiHost + 'static> IntoUiElement<H> for InlineCitationCard {
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        InlineCitationCard::into_element(self, cx)
    }
}

#[derive(Debug)]
pub struct InlineCitationText {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl InlineCitationText {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
        }
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(parts) = use_inline_citation_parts(cx) else {
            return hidden(cx);
        };

        let theme = Theme::global(&*cx.app).clone();
        let open_now = cx
            .get_model_copied(&parts.open_model, Invalidation::Paint)
            .unwrap_or(false);

        let mut chrome = ChromeRefinement::default()
            .rounded(Radius::Sm)
            .merge(self.chrome);
        if open_now {
            chrome = chrome.bg(ColorRef::Token {
                key: "accent",
                fallback: ColorFallback::ThemeHoverBackground,
            });
        }

        let layout = LayoutRefinement::default().min_w_0().merge(self.layout);

        let children = self.children;
        let inner = match children.len() {
            0 => cx.text(""),
            1 => children
                .into_iter()
                .next()
                .expect("inline citation text child"),
            _ => ui::h_row(move |_cx| children)
                .layout(LayoutRefinement::default().min_w_0())
                .gap(Space::N0)
                .items(Items::Center)
                .into_element(cx),
        };

        let el = cx.container(
            decl_style::container_props(&theme, chrome, layout),
            move |_cx| vec![inner],
        );

        let test_id = self.test_id.or_else(|| parts.derived_test_id("label"));
        if let Some(test_id) = test_id {
            return cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: Some(test_id),
                    ..Default::default()
                },
                move |_cx| vec![el],
            );
        }

        el
    }
}

impl Default for InlineCitationText {
    fn default() -> Self {
        Self::new([])
    }
}

#[derive(Debug, Clone)]
pub struct InlineCitationCardTrigger {
    sources: Option<Arc<[Arc<str>]>>,
    test_id: Option<Arc<str>>,
}

impl InlineCitationCardTrigger {
    pub fn new() -> Self {
        Self {
            sources: None,
            test_id: None,
        }
    }

    pub fn sources(mut self, sources: Arc<[Arc<str>]>) -> Self {
        self.sources = Some(sources);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(parts) = use_inline_citation_parts(cx) else {
            return hidden(cx);
        };

        let sources_override = self.sources.is_some();
        let sources = self.sources.unwrap_or_else(|| {
            parts
                .resolved_sources
                .iter()
                .filter_map(|s| s.url.as_ref().cloned())
                .collect::<Vec<_>>()
                .into()
        });

        let count = if sources_override {
            sources.len().max(1)
        } else {
            parts.badge_count()
        };

        let host = sources
            .first()
            .map(|url| url.as_ref())
            .and_then(hostname_for_url);

        let trigger_text = match host {
            Some(host) if count > 1 => format!("{host} +{}", count.saturating_sub(1)),
            Some(host) => host.to_string(),
            None if count > 1 => format!("unknown +{}", count.saturating_sub(1)),
            None => "unknown".to_string(),
        };

        let badge = Badge::new(trigger_text.clone())
            .variant(BadgeVariant::Secondary)
            .refine_style(ChromeRefinement::default().rounded(Radius::Full))
            .into_element(cx);

        let on_activate = parts.on_activate();
        let trigger_label: Arc<str> = Arc::from(trigger_text);
        let trigger_test_id = self.test_id.or_else(|| parts.base_test_id());

        cx.pressable(
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
        )
    }
}

impl Default for InlineCitationCardTrigger {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct InlineCitationCarouselPrev {
    test_id: Option<Arc<str>>,
}

impl InlineCitationCarouselPrev {
    pub fn new() -> Self {
        Self { test_id: None }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(parts) = use_inline_citation_parts(cx) else {
            return hidden(cx);
        };

        let icon = decl_icon::icon_with(
            cx,
            ids::ui::CHEVRON_LEFT,
            Some(Px(16.0)),
            Some(ColorRef::Token {
                key: "muted-foreground",
                fallback: ColorFallback::ThemeTextMuted,
            }),
        );

        let on_prev: OnActivate = Arc::new({
            let index_model = parts.index_model();
            move |host, _cx, _reason| {
                let _ = host.models_mut().update(&index_model, |v| {
                    *v = v.saturating_sub(1);
                });
            }
        });

        let test_id = self.test_id.or_else(|| parts.derived_test_id("prev"));

        cx.pressable(
            PressableProps {
                a11y: PressableA11y {
                    role: Some(SemanticsRole::Button),
                    label: Some(Arc::<str>::from("Previous")),
                    test_id,
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, _state| {
                cx.pressable_on_activate(on_prev.clone());
                [icon]
            },
        )
    }
}

impl Default for InlineCitationCarouselPrev {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct InlineCitationCarouselNext {
    test_id: Option<Arc<str>>,
}

impl InlineCitationCarouselNext {
    pub fn new() -> Self {
        Self { test_id: None }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(parts) = use_inline_citation_parts(cx) else {
            return hidden(cx);
        };

        let icon = decl_icon::icon_with(
            cx,
            ids::ui::CHEVRON_RIGHT,
            Some(Px(16.0)),
            Some(ColorRef::Token {
                key: "muted-foreground",
                fallback: ColorFallback::ThemeTextMuted,
            }),
        );

        let on_next: OnActivate = Arc::new({
            let index_model = parts.index_model();
            move |host, _cx, _reason| {
                let _ = host.models_mut().update(&index_model, |v| {
                    *v = v.saturating_add(1);
                });
            }
        });

        let test_id = self.test_id.or_else(|| parts.derived_test_id("next"));

        cx.pressable(
            PressableProps {
                a11y: PressableA11y {
                    role: Some(SemanticsRole::Button),
                    label: Some(Arc::<str>::from("Next")),
                    test_id,
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, _state| {
                cx.pressable_on_activate(on_next.clone());
                [icon]
            },
        )
    }
}

impl Default for InlineCitationCarouselNext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct InlineCitationCarouselIndex {
    test_id: Option<Arc<str>>,
}

impl InlineCitationCarouselIndex {
    pub fn new() -> Self {
        Self { test_id: None }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(parts) = use_inline_citation_parts(cx) else {
            return hidden(cx);
        };

        let theme = Theme::global(&*cx.app).clone();
        let index_now =
            inline_citation_clamped_index(cx, &parts.index_model, parts.resolved_sources.len());
        let current = index_now.saturating_add(1);
        let total = parts.resolved_sources.len().max(1);

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

        let test_id = self.test_id.or_else(|| parts.derived_test_id("index"));
        let index_el = if let Some(test_id) = test_id {
            cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: Some(test_id),
                    ..Default::default()
                },
                move |_cx| vec![index_inner],
            )
        } else {
            index_inner
        };

        ui::h_row(move |_cx| vec![index_el])
            .layout(LayoutRefinement::default().flex_grow(1.0).min_w_0())
            .justify(Justify::End)
            .items(Items::Center)
            .into_element(cx)
    }
}

impl Default for InlineCitationCarouselIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct InlineCitationCarouselHeader {
    children: Vec<AnyElement>,
}

impl InlineCitationCarouselHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
        }
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let mut children = self.children;
        if children.is_empty() {
            children = vec![
                InlineCitationCarouselPrev::new().into_element(cx),
                InlineCitationCarouselNext::new().into_element(cx),
                InlineCitationCarouselIndex::new().into_element(cx),
            ];
        }

        let header_row = ui::h_row(move |_cx| children)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2)
            .items(Items::Center)
            .into_element(cx);

        cx.container(
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
        )
    }
}

impl Default for InlineCitationCarouselHeader {
    fn default() -> Self {
        Self::new([])
    }
}

#[derive(Debug)]
pub struct InlineCitationSource {
    source: Option<SourceItem>,
    children: Vec<AnyElement>,
}

impl InlineCitationSource {
    pub fn new(source: SourceItem) -> Self {
        Self {
            source: Some(source),
            children: Vec::new(),
        }
    }

    pub fn from_context() -> Self {
        Self {
            source: None,
            children: Vec::new(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(parts) = use_inline_citation_parts(cx) else {
            return hidden(cx);
        };

        let theme = Theme::global(&*cx.app).clone();
        let source = self.source.or_else(|| {
            let index_now =
                inline_citation_clamped_index(cx, &parts.index_model, parts.resolved_sources.len());
            parts.resolved_sources.get(index_now).cloned()
        });

        let Some(source) = source else {
            return hidden(cx);
        };

        let mut out: Vec<AnyElement> = Vec::new();
        out.push(inline_citation_title_text(cx, &theme, source.title.clone()));
        if let Some(url) = source.url.clone() {
            out.push(inline_citation_url_element(
                cx,
                &theme,
                url,
                parts.on_open_url(),
            ));
        }
        if let Some(description) = source.description.clone() {
            out.push(inline_citation_description_text(cx, &theme, description));
        }
        out.extend(self.children);

        ui::v_stack(move |_cx| out)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N1)
            .into_element(cx)
    }
}

#[derive(Debug)]
pub struct InlineCitationQuote {
    quote: Option<Arc<str>>,
}

impl InlineCitationQuote {
    pub fn new(quote: impl Into<Arc<str>>) -> Self {
        Self {
            quote: Some(quote.into()),
        }
    }

    pub fn from_context() -> Self {
        Self { quote: None }
    }

    pub fn quote(mut self, quote: impl Into<Arc<str>>) -> Self {
        self.quote = Some(quote.into());
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(parts) = use_inline_citation_parts(cx) else {
            return hidden(cx);
        };

        let theme = Theme::global(&*cx.app).clone();

        let quote = self.quote.or_else(|| {
            let index_now =
                inline_citation_clamped_index(cx, &parts.index_model, parts.resolved_sources.len());
            parts
                .resolved_sources
                .get(index_now)
                .and_then(|s| s.quote.clone().or_else(|| s.excerpt.clone()))
        });

        let Some(quote) = quote else {
            return hidden(cx);
        };

        inline_citation_quote_block(cx, &theme, quote)
    }
}

impl Default for InlineCitationQuote {
    fn default() -> Self {
        Self::from_context()
    }
}

#[derive(Debug)]
pub struct InlineCitationCarouselItem {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl InlineCitationCarouselItem {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
        }
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
        let body_inner = ui::v_stack(move |_cx| self.children)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2)
            .into_element(cx);

        cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .p(Space::N4)
                    .pl(Space::N8)
                    .merge(self.chrome),
                LayoutRefinement::default()
                    .w_full()
                    .min_w_0()
                    .merge(self.layout),
            ),
            move |_cx| vec![body_inner],
        )
    }
}

impl Default for InlineCitationCarouselItem {
    fn default() -> Self {
        Self::new([])
    }
}

#[derive(Debug)]
pub struct InlineCitationCarouselContent {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl InlineCitationCarouselContent {
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        if self.children.len() == 1 {
            return self
                .children
                .into_iter()
                .next()
                .expect("inline citation content");
        }

        ui::v_stack(move |_cx| self.children)
            .layout(
                LayoutRefinement::default()
                    .w_full()
                    .min_w_0()
                    .merge(self.layout),
            )
            .gap(Space::N0)
            .into_element(cx)
    }
}

impl Default for InlineCitationCarouselContent {
    fn default() -> Self {
        Self::new([])
    }
}

#[derive(Debug)]
pub struct InlineCitationCarousel {
    header: Option<AnyElement>,
    content: Option<AnyElement>,
    layout: LayoutRefinement,
}

impl InlineCitationCarousel {
    pub fn new() -> Self {
        Self {
            header: None,
            content: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn header(mut self, header: AnyElement) -> Self {
        self.header = Some(header);
        self
    }

    pub fn content(mut self, content: AnyElement) -> Self {
        self.content = Some(content);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let header = self
            .header
            .unwrap_or_else(|| InlineCitationCarouselHeader::default().into_element(cx));

        let content = self.content.unwrap_or_else(|| {
            InlineCitationCarouselContent::new([InlineCitationCarouselItem::new([
                InlineCitationSource::from_context().into_element(cx),
                InlineCitationQuote::from_context().into_element(cx),
            ])
            .into_element(cx)])
            .into_element(cx)
        });

        ui::v_stack(move |_cx| vec![header, content])
            .layout(
                LayoutRefinement::default()
                    .w_full()
                    .min_w_0()
                    .merge(self.layout),
            )
            .gap(Space::N0)
            .into_element(cx)
    }
}

impl Default for InlineCitationCarousel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct InlineCitationCardBody {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl InlineCitationCardBody {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
        }
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(parts) = use_inline_citation_parts(cx) else {
            return hidden(cx);
        };

        let mut children = self.children;
        if children.is_empty() {
            children.push(InlineCitationCarousel::default().into_element(cx));
        }

        let content = HoverCardContent::new(children)
            .refine_style(ChromeRefinement::default().p(Space::N0).merge(self.chrome))
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(Px(320.0))
                    .merge(self.layout),
            );

        let content = content.into_element(cx);
        let test_id = self.test_id.or_else(|| parts.derived_test_id("card"));
        if let Some(test_id) = test_id {
            return cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: Some(test_id),
                    ..Default::default()
                },
                move |_cx| vec![content],
            );
        }

        content
    }
}

impl Default for InlineCitationCardBody {
    fn default() -> Self {
        Self::new([])
    }
}

#[derive(Debug)]
pub struct InlineCitationCard {
    trigger: Option<AnyElement>,
    body: Option<AnyElement>,
    open_delay_frames: u32,
    close_delay_frames: u32,
    layout: LayoutRefinement,
}

impl InlineCitationCard {
    pub fn new() -> Self {
        Self {
            trigger: None,
            body: None,
            open_delay_frames: 0,
            close_delay_frames: 0,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn trigger(mut self, trigger: AnyElement) -> Self {
        self.trigger = Some(trigger);
        self
    }

    pub fn body(mut self, body: AnyElement) -> Self {
        self.body = Some(body);
        self
    }

    pub fn open_delay_frames(mut self, frames: u32) -> Self {
        self.open_delay_frames = frames;
        self
    }

    pub fn close_delay_frames(mut self, frames: u32) -> Self {
        self.close_delay_frames = frames;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(parts) = use_inline_citation_parts(cx) else {
            return hidden(cx);
        };

        let trigger = self
            .trigger
            .unwrap_or_else(|| InlineCitationCardTrigger::default().into_element(cx));

        if parts.resolved_sources.is_empty() {
            return trigger;
        }

        let body = self
            .body
            .unwrap_or_else(|| InlineCitationCardBody::default().into_element(cx));

        HoverCard::new(cx, trigger, body)
            .open(Some(parts.open_model()))
            .open_delay_frames(self.open_delay_frames)
            .close_delay_frames(self.close_delay_frames)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}

impl Default for InlineCitationCard {
    fn default() -> Self {
        Self::new()
    }
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
        let InlineCitation {
            label,
            source_ids,
            sources,
            on_open_url,
            on_activate,
            test_id,
            layout,
        } = self;

        let label_fallback = label.clone();

        let mut root = InlineCitationRoot::new()
            .source_ids(source_ids)
            .refine_layout(layout);
        if let Some(sources) = sources {
            root = root.sources(sources);
        }
        if let Some(handler) = on_open_url {
            root = root.on_open_url(handler);
        }
        if let Some(on_activate) = on_activate {
            root = root.on_activate(on_activate);
        }
        if let Some(test_id) = test_id {
            root = root.test_id(test_id);
        }

        root.into_element_with_children(cx, move |cx| {
            let mut inline_children = children(cx);
            if inline_children.is_empty() {
                inline_children.push(cx.text(label_fallback.clone()));
            }

            vec![
                InlineCitationText::new(inline_children).into_element(cx),
                InlineCitationCard::new().into_element(cx),
            ]
        })
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
        let sources: Arc<[SourceItem]> = Arc::from(vec![SourceItem::new(
            "source-1",
            "Advances in Natural Language Processing",
        )
        .url("https://example.com/nlp-advances")]);

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

    #[test]
    fn inline_citation_root_into_element_parts_renders_hostname_trigger() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let sources: Arc<[SourceItem]> =
            Arc::from(vec![
                SourceItem::new("source-1", "Alpha source").url("https://example.com/foo")
            ]);

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "citation-root-parts",
            |cx| {
                let text = InlineCitationText::new([cx.text("Inline copy")]);
                InlineCitationRoot::new()
                    .source_id("source-1")
                    .sources(sources.clone())
                    .test_id("citation")
                    .into_element_parts(text, InlineCitationCard::new(), cx)
            },
        );

        let trigger =
            find_pressable_by_label(&el, "example.com").expect("inline citation hostname trigger");
        assert_eq!(trigger.a11y.test_id.as_deref(), Some("citation"));
    }
}
