use fret_core::Color;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

mod affordance;
mod frame;
mod keying;
mod test_ids;

use self::frame::color_edit_into_element_keyed;
use self::keying::color_edit_into_element;

use super::options::ColorEditOptions;

#[derive(Clone)]
pub struct ColorEdit {
    model: Model<Color>,
    options: ColorEditOptions,
}

impl ColorEdit {
    pub fn new(model: Model<Color>) -> Self {
        Self {
            model,
            options: ColorEditOptions::default(),
        }
    }

    pub fn options(mut self, options: ColorEditOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        color_edit_into_element(self, cx)
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        color_edit_into_element_keyed(self, cx)
    }
}
