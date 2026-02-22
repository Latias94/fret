use std::sync::Arc;

use fret_core::{
    Corners, Edges, FontWeight, Px, SemanticsRole, TextAlign, TextOverflow, TextSlant, TextStyle,
    TextWrap,
};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, PressableA11y, PressableKeyActivation, PressableProps,
    SemanticsDecoration, SemanticsProps, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Justify, LayoutRefinement, Radius, Space,
};

use fret_ui_shadcn::{Badge, BadgeVariant, Collapsible, CollapsibleContent, CollapsibleTrigger};

use crate::model::SourceItem;

fn text_xs_style(theme: &Theme, weight: FontWeight) -> TextStyle {
    let mut style =
        typography::TypographyPreset::control_ui(typography::UiTextSize::Xs).resolve(theme);
    style.weight = weight;
    style.slant = TextSlant::Normal;
    style
}

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
            layout: LayoutRefinement::default().mb(Space::N4),
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
        let excerpt_color = theme
            .color_by_key("muted-foreground")
            .unwrap_or_else(|| theme.color_token("muted-foreground"));
        let title_color = theme.color_token("foreground");

        let icon_color = ColorRef::Token {
            key: "foreground",
            fallback: ColorFallback::ThemeTextPrimary,
        };
        let list_theme = theme.clone();
        let trigger_theme = theme.clone();
        let list_icon_color = icon_color.clone();
        let trigger_icon_color = icon_color.clone();

        let highlight_bg = theme
            .color_by_key("accent")
            .unwrap_or_else(|| theme.color_token("muted"));
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
                .layout(LayoutRefinement::default())
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

                    let icon = decl_icon::icon_with(
                        cx,
                        ids::ui::BOOK,
                        Some(Px(16.0)),
                        Some(list_icon_color.clone()),
                    );

                    let title_text = cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: item.title.clone(),
                        style: Some(text_xs_style(&list_theme, FontWeight::MEDIUM)),
                        color: Some(title_color),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Ellipsis,
                        align: TextAlign::Start,
                        ink_overflow: Default::default(),
                    });

                    let title_row = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        move |_cx| vec![icon, title_text],
                    );

                    let title_el: AnyElement = match (&item.url, on_open_url.clone()) {
                        (Some(url), Some(handler)) => {
                            let link = fret_markdown::LinkInfo {
                                href: url.clone(),
                                text: item.title.clone(),
                            };

                            let on_activate: OnActivate = Arc::new(move |host, cx, reason| {
                                handler(host, cx, reason, link.clone());
                            });

                            let link_test_id = row_test_id
                                .clone()
                                .map(|id| Arc::<str>::from(format!("{id}-link")));

                            cx.pressable(
                                PressableProps {
                                    key_activation: PressableKeyActivation::EnterOnly,
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::Link),
                                        label: Some(item.title.clone()),
                                        test_id: link_test_id,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                move |cx, _state| {
                                    cx.pressable_on_activate(on_activate.clone());
                                    [title_row]
                                },
                            )
                        }
                        _ => title_row,
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
                            .layout(LayoutRefinement::default())
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
                            style: Some(text_xs_style(&list_theme, FontWeight::NORMAL)),
                            color: Some(excerpt_color),
                            wrap: TextWrap::Word,
                            overflow: TextOverflow::Clip,
                            align: fret_core::TextAlign::Start,
                            ink_overflow: Default::default(),
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
                let label = cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: Arc::<str>::from(format!("Used {count} sources")),
                    style: Some(text_xs_style(&trigger_theme, FontWeight::MEDIUM)),
                    color: Some(title_color),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    align: TextAlign::Start,
                    ink_overflow: Default::default(),
                });
                let chevron_id = if is_open {
                    ids::ui::CHEVRON_UP
                } else {
                    ids::ui::CHEVRON_DOWN
                };
                let chevron = decl_icon::icon_with(
                    cx,
                    chevron_id,
                    Some(Px(16.0)),
                    Some(trigger_icon_color.clone()),
                );
                let row = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default())
                        .gap(Space::N2)
                        .items_center(),
                    move |_cx| vec![label, chevron],
                );

                let trigger = CollapsibleTrigger::new(open_model, [row]).into_element(cx, is_open);
                let Some(test_id) = trigger_test_id.clone() else {
                    return trigger;
                };

                trigger.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Button)
                        .test_id(test_id),
                )
            },
            move |cx| {
                let content = CollapsibleContent::new(vec![list.clone()])
                    .refine_style(chrome.clone())
                    .refine_layout(LayoutRefinement::default().mt(Space::N3))
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
