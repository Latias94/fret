//! Inspector panel recipe (search + toolbar + sections).
//!
//! This is a composition-only surface intended for editor apps:
//! - it does not define data models beyond an optional search `Model<String>` and optional
//!   search-assist state when apps opt into history/completion,
//! - it stays renderer/platform agnostic,
//! - it provides stable slots so apps can assemble an inspector without re-rolling layout.

use std::sync::Arc;

use fret_core::{Axis, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    SpacerProps, SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::headless::text_assist::{InputOwnedTextAssistKeyOptions, TextAssistItem};

use crate::controls::{
    EditorTextCancelBehavior, EditorTextSelectionBehavior, MiniSearchBox, MiniSearchBoxOptions,
    TextAssistField, TextAssistFieldOptions, TextAssistFieldSurface, TextFieldOptions,
};
use crate::primitives::inspector_layout::InspectorLayoutMetrics;
use crate::primitives::{EditorDensity, EditorTokenKeys};

#[derive(Debug, Clone)]
pub struct InspectorPanelSearchAssistOptions {
    pub dismissed_query_model: Model<String>,
    pub active_item_id_model: Model<Option<Arc<str>>>,
    pub items: Arc<[TextAssistItem]>,
    pub list_label: Arc<str>,
    pub empty_label: Arc<str>,
    pub key_options: InputOwnedTextAssistKeyOptions,
    pub list_test_id: Option<Arc<str>>,
    pub item_test_id_prefix: Option<Arc<str>>,
    pub empty_test_id: Option<Arc<str>>,
    pub max_list_height: Option<Px>,
}

#[derive(Debug, Clone)]
pub struct InspectorPanelOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub title: Option<Arc<str>>,
    pub padding: Option<Edges>,
    pub gap: Option<Px>,
    pub header_gap: Option<Px>,
    pub test_id: Option<Arc<str>>,
    pub header_test_id: Option<Arc<str>>,
    pub toolbar_test_id: Option<Arc<str>>,
    pub search_test_id: Option<Arc<str>>,
    pub search_clear_test_id: Option<Arc<str>>,
    pub search_assist: Option<InspectorPanelSearchAssistOptions>,
    pub content_test_id: Option<Arc<str>>,
}

impl Default for InspectorPanelOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            title: None,
            padding: None,
            gap: None,
            header_gap: None,
            test_id: None,
            header_test_id: None,
            toolbar_test_id: None,
            search_test_id: None,
            search_clear_test_id: None,
            search_assist: None,
            content_test_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InspectorPanelCx {
    pub density: EditorDensity,
    pub query: Arc<str>,
    pub query_lower: Arc<str>,
}

impl InspectorPanelCx {
    pub fn is_query_empty(&self) -> bool {
        self.query_lower.is_empty()
    }

    pub fn matches(&self, s: &str) -> bool {
        if self.query_lower.is_empty() {
            return true;
        }
        s.to_lowercase().contains(self.query_lower.as_ref())
    }
}

#[derive(Clone)]
pub struct InspectorPanel {
    search: Option<Model<String>>,
    options: InspectorPanelOptions,
}

impl InspectorPanel {
    pub fn new(search: Option<Model<String>>) -> Self {
        Self {
            search,
            options: InspectorPanelOptions::default(),
        }
    }

    pub fn options(mut self, options: InspectorPanelOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        toolbar: impl FnOnce(&mut ElementContext<'_, H>, &InspectorPanelCx) -> Vec<AnyElement>,
        contents: impl FnOnce(&mut ElementContext<'_, H>, &InspectorPanelCx) -> Vec<AnyElement>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let (
                density,
                gap,
                header_gap,
                padding,
                header_bg,
                header_border,
                panel_bg,
                panel_border,
                radius,
            ) = {
                let theme = Theme::global(&*cx.app);
                let metrics = InspectorLayoutMetrics::resolve(theme);
                let density = metrics.density;
                let gap = self.options.gap.unwrap_or(metrics.panel_gap);
                let header_gap = self.options.header_gap.unwrap_or(metrics.panel_header_gap);
                let padding = self.options.padding.unwrap_or_else(|| Edges::all(Px(0.0)));
                let header_bg = theme
                    .color_by_key(EditorTokenKeys::PROPERTY_HEADER_BG)
                    .or_else(|| theme.color_by_key("muted"))
                    .unwrap_or_else(|| theme.color_token("background"));
                let header_border = theme
                    .color_by_key(EditorTokenKeys::PROPERTY_HEADER_BORDER)
                    .or_else(|| theme.color_by_key("border"))
                    .unwrap_or_else(|| theme.color_token("foreground"));
                let panel_bg = theme
                    .color_by_key(EditorTokenKeys::PROPERTY_PANEL_BG)
                    .or_else(|| theme.color_by_key("card"))
                    .unwrap_or_else(|| theme.color_token("background"));
                let panel_border = theme
                    .color_by_key(EditorTokenKeys::PROPERTY_PANEL_BORDER)
                    .or_else(|| theme.color_by_key("border"))
                    .unwrap_or_else(|| theme.color_token("foreground"));
                let radius = theme.metric_token("metric.radius.sm");
                (
                    density,
                    gap,
                    header_gap,
                    padding,
                    header_bg,
                    header_border,
                    panel_bg,
                    panel_border,
                    radius,
                )
            };

            let query = self
                .search
                .as_ref()
                .and_then(|m| {
                    cx.get_model_cloned(m, Invalidation::Layout)
                        .map(|s| s.trim().to_string())
                })
                .unwrap_or_default();
            let query_lower = query.to_lowercase();

            let panel_cx = InspectorPanelCx {
                density,
                query: Arc::from(query),
                query_lower: Arc::from(query_lower),
            };

            let title = self.options.title.clone();

            let mut toolbar = toolbar(cx, &panel_cx);
            let has_header = title.is_some() || !toolbar.is_empty() || self.search.is_some();

            let header = has_header.then(|| {
                let mut out = Vec::new();
                if let Some(title) = title.clone() {
                    let mut row = {
                        let toolbar = std::mem::take(&mut toolbar);
                        cx.flex(
                            FlexProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Auto,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                direction: Axis::Horizontal,
                                gap: SpacingLength::Px(Px(6.0)),
                                padding: Edges::all(Px(0.0)).into(),
                                justify: MainAlign::Start,
                                align: CrossAlign::Center,
                                wrap: false,
                            },
                            move |cx| {
                                let mut row = Vec::new();
                                row.push(cx.text(title.clone()));
                                let mut spacer = SpacerProps::default();
                                spacer.layout.size.width = Length::Fill;
                                spacer.layout.size.height = Length::Px(Px(0.0));
                                spacer.layout.flex.grow = 1.0;
                                row.push(cx.spacer(spacer));
                                row.extend(toolbar);
                                row
                            },
                        )
                    };

                    if let Some(test_id) = self.options.toolbar_test_id.as_ref() {
                        row = row.test_id(test_id.clone());
                    }

                    out.push(row);
                } else if !toolbar.is_empty() {
                    let toolbar = std::mem::take(&mut toolbar);
                    let mut row = cx.flex(
                        FlexProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: Axis::Horizontal,
                            gap: SpacingLength::Px(Px(6.0)),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::End,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |_cx| toolbar,
                    );

                    if let Some(test_id) = self.options.toolbar_test_id.as_ref() {
                        row = row.test_id(test_id.clone());
                    }

                    out.push(row);
                }

                if let Some(search) = self.search.clone() {
                    let search_el = inspector_panel_search_element(
                        cx,
                        search,
                        self.options.enabled,
                        self.options.search_test_id.clone(),
                        self.options.search_clear_test_id.clone(),
                        self.options.search_assist.clone(),
                    );

                    out.push(search_el);
                }

                let mut header = cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        padding: Edges {
                            top: Px(density.padding_y.0 + 3.0),
                            right: density.padding_x,
                            bottom: Px(density.padding_y.0 + 4.0),
                            left: density.padding_x,
                        }
                        .into(),
                        background: Some(header_bg),
                        corner_radii: Corners {
                            top_left: radius,
                            top_right: radius,
                            bottom_right: Px(0.0),
                            bottom_left: Px(0.0),
                        },
                        border: Edges {
                            top: Px(0.0),
                            right: Px(0.0),
                            bottom: Px(1.0),
                            left: Px(0.0),
                        },
                        border_color: Some(header_border),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.flex(
                            FlexProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Auto,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                direction: Axis::Vertical,
                                gap: SpacingLength::Px(header_gap),
                                padding: Edges::all(Px(0.0)).into(),
                                justify: MainAlign::Start,
                                align: CrossAlign::Stretch,
                                wrap: false,
                            },
                            move |_cx| out,
                        )]
                    },
                );

                if let Some(test_id) = self.options.header_test_id.as_ref() {
                    header = header.test_id(test_id.clone());
                }
                header
            });

            let mut content = cx.flex(
                FlexProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    direction: Axis::Vertical,
                    gap: SpacingLength::Px(gap),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    wrap: false,
                },
                move |cx| contents(cx, &panel_cx),
            );

            if let Some(test_id) = self.options.content_test_id.as_ref() {
                content = content.test_id(test_id.clone());
            }

            let mut root = cx.container(
                ContainerProps {
                    layout: self.options.layout,
                    padding: padding.into(),
                    background: Some(panel_bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(panel_border),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: Axis::Vertical,
                            gap: SpacingLength::Px(gap),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                            wrap: false,
                        },
                        move |_cx| {
                            let mut out = Vec::new();
                            if let Some(header) = header {
                                out.push(header);
                            }
                            out.push(content);
                            out
                        },
                    )]
                },
            );

            if let Some(test_id) = self.options.test_id.as_ref() {
                root = root.test_id(test_id.clone());
            }

            root
        })
    }
}

fn inspector_panel_search_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    search: Model<String>,
    enabled: bool,
    search_test_id: Option<Arc<str>>,
    search_clear_test_id: Option<Arc<str>>,
    search_assist: Option<InspectorPanelSearchAssistOptions>,
) -> AnyElement {
    if let Some(search_assist) = search_assist {
        return TextAssistField::new(
            search,
            search_assist.dismissed_query_model,
            search_assist.active_item_id_model,
            search_assist.items,
        )
        .options(TextAssistFieldOptions {
            field: TextFieldOptions {
                enabled,
                focusable: enabled,
                placeholder: Some(Arc::from("Search…")),
                clear_button: true,
                buffered: false,
                selection_behavior: EditorTextSelectionBehavior::SelectAllOnFocus,
                cancel_behavior: EditorTextCancelBehavior::Clear,
                test_id: search_test_id,
                clear_test_id: search_clear_test_id,
                ..Default::default()
            },
            surface: TextAssistFieldSurface::AnchoredOverlay,
            list_label: search_assist.list_label,
            empty_label: search_assist.empty_label,
            key_options: search_assist.key_options,
            list_test_id: search_assist.list_test_id,
            item_test_id_prefix: search_assist.item_test_id_prefix,
            empty_test_id: search_assist.empty_test_id,
            max_list_height: search_assist.max_list_height,
        })
        .into_element(cx);
    }

    MiniSearchBox::new(search)
        .options(MiniSearchBoxOptions {
            enabled,
            focusable: enabled,
            test_id: search_test_id,
            clear_test_id: search_clear_test_id,
            ..Default::default()
        })
        .into_element(cx)
}
