//! Public text-assist field element assembly owner.

use std::panic::Location;
use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::headless::text_assist::TextAssistItem;

use super::model::{OnTextAssistFieldAccept, TextAssistFieldOptions};

mod body;
mod keyboard;

#[derive(Clone)]
pub struct TextAssistField {
    query_model: Model<String>,
    dismissed_query_model: Model<String>,
    active_item_id_model: Model<Option<Arc<str>>>,
    items: Arc<[TextAssistItem]>,
    on_accept: Option<OnTextAssistFieldAccept>,
    options: TextAssistFieldOptions,
}

impl TextAssistField {
    pub fn new(
        query_model: Model<String>,
        dismissed_query_model: Model<String>,
        active_item_id_model: Model<Option<Arc<str>>>,
        items: Arc<[TextAssistItem]>,
    ) -> Self {
        Self {
            query_model,
            dismissed_query_model,
            active_item_id_model,
            items,
            on_accept: None,
            options: TextAssistFieldOptions::default(),
        }
    }

    pub fn options(mut self, options: TextAssistFieldOptions) -> Self {
        self.options = options;
        self
    }

    pub fn on_accept(mut self, on_accept: Option<OnTextAssistFieldAccept>) -> Self {
        self.on_accept = on_accept;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model_id = self.query_model.id();
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.field.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(
                ("fret-ui-editor.text_assist_field", id_source, model_id),
                |cx| self.into_element_keyed(cx),
            )
        } else {
            cx.keyed(
                ("fret-ui-editor.text_assist_field", callsite, model_id),
                |cx| self.into_element_keyed(cx),
            )
        }
    }
}
