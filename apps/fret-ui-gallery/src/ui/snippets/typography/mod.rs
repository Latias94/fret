use std::sync::Arc;

use fret::AppComponentCx;
use fret_ui::element::AnyElement;

pub mod blockquote;
pub mod demo;
pub mod h1;
pub mod h2;
pub mod h3;
pub mod h4;
pub mod inline_code;
pub mod interactive_links;
pub mod large;
pub mod lead;
pub mod list;
pub mod muted;
pub mod p;
pub mod rtl;
pub mod small;
pub mod table;

pub(super) fn table_cell_text<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement
where
    T: Into<Arc<str>>,
{
    fret_ui_kit::declarative::text::text_table_cell(cx, text)
}
