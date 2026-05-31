//! Inspector panel recipe (search + toolbar + sections).
//!
//! This is a composition-only surface intended for editor apps:
//! - it does not define data models beyond an optional search `Model<String>` and optional
//!   search-assist state when apps opt into history/completion,
//! - it stays renderer/platform agnostic,
//! - it provides stable slots so apps can assemble an inspector without re-rolling layout.
mod element;

use std::sync::Arc;

use fret_core::{Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle};
use fret_ui::{ElementContext, ElementContextAccess, UiHost};
use fret_ui_kit::headless::text_assist::{InputOwnedTextAssistKeyOptions, TextAssistItem};

use crate::primitives::EditorDensity;

use element::inspector_panel_element;

#[cfg(test)]
mod tests;

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
    density: EditorDensity,
    query: Arc<str>,
    query_lower: Arc<str>,
}

impl InspectorPanelCx {
    pub fn density(&self) -> EditorDensity {
        self.density
    }

    pub fn query(&self) -> &str {
        self.query.as_ref()
    }

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
        inspector_panel_element(cx, self.search, self.options, toolbar, contents)
    }

    #[track_caller]
    pub fn into_element_in<'a, H: UiHost + 'a, Cx>(
        self,
        cx: &mut Cx,
        toolbar: impl FnOnce(&mut ElementContext<'_, H>, &InspectorPanelCx) -> Vec<AnyElement>,
        contents: impl FnOnce(&mut ElementContext<'_, H>, &InspectorPanelCx) -> Vec<AnyElement>,
    ) -> AnyElement
    where
        Cx: ElementContextAccess<'a, H>,
    {
        self.into_element(cx.elements(), toolbar, contents)
    }
}
