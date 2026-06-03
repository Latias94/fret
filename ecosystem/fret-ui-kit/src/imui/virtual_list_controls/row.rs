//! Row packing and semantics for immediate virtual lists.

use std::sync::Arc;

use fret_core::{Px, SemanticsRole};
use fret_ui::element::{AnyElement, ContainerProps, Length, Overflow, SemanticsProps};
use fret_ui::{ElementContext, Theme, UiHost};

mod children;
mod metrics;

pub(super) use children::pack_row_children;
pub(super) use metrics::{row_height_for_index, row_test_id};

pub(super) fn wrap_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    index: usize,
    content: AnyElement,
    test_id: Option<Arc<str>>,
    fixed_height: Option<Px>,
) -> AnyElement {
    let mut row = ContainerProps::default();
    row.layout.size.width = Length::Fill;
    row.layout.size.height = fixed_height.map_or(Length::Auto, Length::Px);
    if fixed_height.is_some() {
        row.layout.overflow = Overflow::Clip;
    }

    let theme = Theme::global(&*cx.app);
    if index % 2 == 1 {
        let mut background = theme
            .color_by_key("list.row.striped")
            .or_else(|| theme.color_by_key("muted"))
            .unwrap_or_else(|| theme.color_token("muted"));
        background.a *= 0.18;
        row.background = Some(background);
    }

    let row = cx.container(row, move |_cx| vec![content]);
    if let Some(test_id) = test_id {
        let mut semantics = SemanticsProps::default();
        semantics.role = SemanticsRole::ListItem;
        semantics.test_id = Some(test_id);
        cx.semantics(semantics, move |_cx| vec![row])
    } else {
        row
    }
}
