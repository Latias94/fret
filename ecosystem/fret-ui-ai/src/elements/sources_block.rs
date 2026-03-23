use std::sync::Arc;

use fret_core::{Corners, Edges, FontWeight, Px, SemanticsRole, TextAlign, TextOverflow, TextWrap};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, PressableA11y, PressableKeyActivation, PressableProps,
    SemanticsDecoration, SemanticsProps, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, Radius, Space,
};

use fret_ui_shadcn::facade::{
    Badge, BadgeVariant, Collapsible, CollapsibleContent, CollapsibleTrigger,
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
            // Upstream `SourcesTrigger` defaults to: "Used {count} sources".
            //
            // Reference: `repo-ref/ai-elements/packages/elements/src/sources.tsx`.
            title: Arc::<str>::from("Used {count} sources"),
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let root_layout = decl_style::layout_style(&theme, self.layout);
        let excerpt_color = theme
            .color_by_key("muted-foreground")
            .unwrap_or_else(|| theme.color_token("muted-foreground"));
        // Upstream `sources.tsx` styles the entire block as `text-primary text-xs`.
        // Reference: `repo-ref/ai-elements/packages/elements/src/sources.tsx`.
        let title_color = theme.color_token("primary");

        let icon_color = ColorRef::Token {
            key: "primary",
            fallback: ColorFallback::ThemeAccent,
        };
        let list_theme = theme.clone();
        let trigger_theme = theme.clone();
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
        let title = self.title;
        let items_for_list = items.clone();
        let on_open_url = self.on_open_url;
        let row_prefix = self.test_id_row_prefix;
        let chrome = self.chrome;

        let list = ui::v_stack(move |cx| {
            let mut out = Vec::new();
            for (index, item) in items_for_list.iter().enumerate() {
                let row_test_id = row_prefix
                    .clone()
                    .map(|p| Arc::<str>::from(format!("{p}{index}")));

                let is_highlighted = highlighted_source_id
                    .as_ref()
                    .is_some_and(|id| id.as_ref() == item.id.as_ref());

                let row_title_color = if is_highlighted {
                    list_theme.color_token("accent-foreground")
                } else {
                    title_color
                };
                let row_icon_color = if is_highlighted {
                    ColorRef::Token {
                        key: "accent-foreground",
                        fallback: ColorFallback::ThemeTextPrimary,
                    }
                } else {
                    icon_color.clone()
                };

                let icon =
                    decl_icon::icon_with(cx, ids::ui::BOOK, Some(Px(16.0)), Some(row_icon_color));

                let title_text = cx.text_props(TextProps {
                    layout: decl_style::layout_style(
                        &list_theme,
                        LayoutRefinement::default().flex_grow(1.0).min_w_0(),
                    ),
                    text: item.title.clone(),
                    style: Some(typography::preset_text_style_with_overrides(
                        &list_theme,
                        typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                        Some(FontWeight::MEDIUM),
                        None,
                    )),
                    color: Some(row_title_color),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Ellipsis,
                    align: TextAlign::Start,
                    ink_overflow: Default::default(),
                });

                let title_row = ui::h_row(move |_cx| vec![icon, title_text])
                    .layout(LayoutRefinement::default().flex_grow(1.0).min_w_0())
                    .gap(Space::N2)
                    .items(Items::Center)
                    .into_element(cx);

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
                                layout: decl_style::layout_style(
                                    &list_theme,
                                    LayoutRefinement::default().flex_grow(1.0).min_w_0(),
                                ),
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

                let row = ui::h_row(move |_cx| {
                    let mut row = Vec::new();
                    row.push(title_el);
                    if let Some(active_badge) = active_badge {
                        row.push(active_badge);
                    }
                    row
                })
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2)
                .justify(Justify::Between)
                .items(Items::Center)
                .into_element(cx);

                let excerpt_el = item.excerpt.clone().map(|excerpt| {
                    cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: excerpt,
                        style: Some(typography::preset_text_style_with_overrides(
                            &list_theme,
                            typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                            Some(FontWeight::NORMAL),
                            None,
                        )),
                        color: Some(excerpt_color),
                        wrap: TextWrap::Word,
                        overflow: TextOverflow::Clip,
                        align: fret_core::TextAlign::Start,
                        ink_overflow: Default::default(),
                    })
                });

                let body = ui::v_stack(move |_cx| {
                    let mut body = Vec::new();
                    body.push(row);
                    if let Some(excerpt_el) = excerpt_el {
                        body.push(excerpt_el);
                    }
                    body
                })
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N1)
                .into_element(cx);

                let body: AnyElement = if is_highlighted {
                    cx.container(
                        ContainerProps {
                            layout: highlight_layout,
                            padding: Edges::all(highlight_padding).into(),
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
        })
        .layout(LayoutRefinement::default())
        .gap(Space::N2)
        .into_element(cx);

        let default_open = self.default_open;

        let collapsible = Collapsible::uncontrolled(default_open).into_element_with_open_model(
            cx,
            move |cx, open_model, is_open| {
                let label_text = if title.contains("{count}") {
                    Arc::<str>::from(title.replace("{count}", &count.to_string()))
                } else {
                    title.clone()
                };
                let label = cx.text_props(TextProps {
                    layout: decl_style::layout_style(
                        &trigger_theme,
                        LayoutRefinement::default().flex_grow(1.0).min_w_0(),
                    ),
                    text: label_text,
                    style: Some(typography::preset_text_style_with_overrides(
                        &trigger_theme,
                        typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                        Some(FontWeight::MEDIUM),
                        None,
                    )),
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
                let row = ui::h_row(move |_cx| vec![label, chevron])
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N2)
                    .justify(Justify::Between)
                    .items(Items::Center)
                    .into_element(cx);

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
                let content = CollapsibleContent::new(vec![list])
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

        // The UI gallery doc layout (and many transcript shells) uses `items: stretch` for vertical
        // stacks, which would otherwise make the sources block appear full-width. Upstream AI
        // Elements renders sources content as `w-fit`, so we wrap the block in a row to preserve
        // intrinsic width while still participating in the parent layout.
        let aligned = ui::h_row(move |_cx| vec![collapsible])
            .layout(LayoutRefinement::default().w_full())
            .justify(Justify::Start)
            .items(Items::Start)
            .into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: root_layout,
                padding: Edges::all(fret_core::Px(0.0)).into(),
                background: None,
                corner_radii: Corners::all(fret_core::Px(0.0)),
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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::{ElementKind, Length, PressableProps};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(260.0)),
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
    fn sources_block_item_title_can_shrink_within_row() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let title =
            "A very long cited source title that should truncate instead of overflowing the row";
        let on_open_url: fret_markdown::OnLinkActivate = Arc::new(|_, _, _, _| {});

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "sources", |cx| {
            SourcesBlock::new([SourceItem::new("source-1", title)
                .url("https://example.com/source-1")
                .excerpt("Supporting excerpt text that can wrap under the title row.")])
            .default_open(true)
            .highlighted_source_id("source-1")
            .on_open_url(on_open_url.clone())
            .into_element(cx)
        });

        let title_text = find_text_by_content(&el, title).expect("sources block title text");
        assert_eq!(title_text.wrap, TextWrap::None);
        assert_eq!(title_text.overflow, TextOverflow::Ellipsis);
        assert_eq!(title_text.layout.flex.grow, 1.0);
        assert_eq!(title_text.layout.flex.shrink, 1.0);
        assert_eq!(title_text.layout.flex.basis, Length::Auto);
        assert_eq!(title_text.layout.size.min_width, Some(Length::Px(Px(0.0))));

        let pressable =
            find_pressable_by_label(&el, title).expect("sources block title link pressable");
        assert_eq!(pressable.layout.flex.grow, 1.0);
        assert_eq!(pressable.layout.flex.shrink, 1.0);
        assert_eq!(pressable.layout.flex.basis, Length::Auto);
        assert_eq!(pressable.layout.size.min_width, Some(Length::Px(Px(0.0))));
    }

    #[test]
    fn sources_block_trigger_label_can_shrink_within_row() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let title =
            "Used {count} sources with a very long trigger title that should truncate cleanly";
        let resolved = "Used 1 sources with a very long trigger title that should truncate cleanly";

        let el = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "sources-trigger",
            |cx| {
                SourcesBlock::new([SourceItem::new("source-1", "Alpha")])
                    .title(title)
                    .into_element(cx)
            },
        );

        let label = find_text_by_content(&el, resolved).expect("sources block trigger label");
        assert_eq!(label.wrap, TextWrap::None);
        assert_eq!(label.overflow, TextOverflow::Clip);
        assert_eq!(label.layout.flex.grow, 1.0);
        assert_eq!(label.layout.flex.shrink, 1.0);
        assert_eq!(label.layout.flex.basis, Length::Auto);
        assert_eq!(label.layout.size.min_width, Some(Length::Px(Px(0.0))));
    }

    #[test]
    fn sources_block_uses_shared_xs_typography_preset() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let title = "Alpha";
        let excerpt = "Supporting excerpt text.";
        let trigger = "Used 1 sources";

        let el =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "sources-style", |cx| {
                SourcesBlock::new([SourceItem::new("source-1", title).excerpt(excerpt)])
                    .default_open(true)
                    .into_element(cx)
            });

        let theme = Theme::global(&app).clone();
        let title_text = find_text_by_content(&el, title).expect("sources block title text");
        let excerpt_text = find_text_by_content(&el, excerpt).expect("sources block excerpt text");
        let trigger_text = find_text_by_content(&el, trigger).expect("sources block trigger text");

        assert_eq!(
            title_text.style,
            Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                Some(FontWeight::MEDIUM),
                None,
            ))
        );
        assert_eq!(
            excerpt_text.style,
            Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                Some(FontWeight::NORMAL),
                None,
            ))
        );
        assert_eq!(
            trigger_text.style,
            Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                Some(FontWeight::MEDIUM),
                None,
            ))
        );
    }
}
