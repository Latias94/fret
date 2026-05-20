use std::sync::Arc;

use fret::AppComponentCx;
use fret_ui::element::AnyElement;

pub mod actions;
pub mod children;
pub mod demo;
pub mod footer;
pub mod rtl;
pub mod usage;

pub(super) fn table_cell_text<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement
where
    T: Into<Arc<str>>,
{
    fret_ui_kit::declarative::text::text_table_cell(cx, text)
}

pub(super) fn table_cell_text_emphasis<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement
where
    T: Into<Arc<str>>,
{
    fret_ui_kit::declarative::text::text_table_cell_emphasis(cx, text)
}
