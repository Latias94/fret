use std::sync::Arc;

use fret::AppComponentCx;
use fret_ui::element::AnyElement;

pub mod basic_demo;
pub mod code_outline;
pub mod default_demo;
pub mod guide_demo;
pub mod rtl_demo;

pub(super) fn table_cell_text<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement
where
    T: Into<Arc<str>>,
{
    fret_ui_kit::declarative::text::text_table_cell(cx, text)
}
