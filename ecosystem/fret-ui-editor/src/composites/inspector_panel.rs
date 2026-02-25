//! Inspector panel recipe (search + toolbar + sections).
//!
//! This is a composition-only surface intended for editor apps:
//! - it does not define data models beyond an optional search `Model<String>`,
//! - it stays renderer/platform agnostic,
//! - it provides stable slots so apps can assemble an inspector without re-rolling layout.

use std::sync::Arc;

use fret_core::{Axis, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacerProps,
    SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::controls::{MiniSearchBox, MiniSearchBoxOptions};
use crate::primitives::EditorDensity;

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
            let (density, gap, header_gap, padding) = {
                let theme = Theme::global(&*cx.app);
                let density = EditorDensity::resolve(theme);
                let gap = self.options.gap.unwrap_or(Px(8.0));
                let header_gap = self.options.header_gap.unwrap_or(Px(6.0));
                let padding = self.options.padding.unwrap_or_else(|| Edges::all(Px(0.0)));
                (density, gap, header_gap, padding)
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
                    let search_el = MiniSearchBox::new(search)
                        .options(MiniSearchBoxOptions {
                            enabled: self.options.enabled,
                            focusable: self.options.enabled,
                            test_id: self.options.search_test_id.clone(),
                            clear_test_id: self.options.search_clear_test_id.clone(),
                            ..Default::default()
                        })
                        .into_element(cx);

                    out.push(search_el);
                }

                let mut header = cx.flex(
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

            let mut root = cx.flex(
                FlexProps {
                    layout: self.options.layout,
                    direction: Axis::Vertical,
                    gap: SpacingLength::Px(gap),
                    padding: padding.into(),
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
            );

            if let Some(test_id) = self.options.test_id.as_ref() {
                root = root.test_id(test_id.clone());
            }

            root
        })
    }
}
