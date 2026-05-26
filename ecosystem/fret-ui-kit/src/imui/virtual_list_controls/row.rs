//! Row packing and semantics for immediate virtual lists.

use std::sync::Arc;

use fret_core::{Px, SemanticsRole};
use fret_ui::element::{
    AnyElement, ContainerProps, Length, Overflow, SemanticsProps, VirtualListMeasureMode,
};
use fret_ui::{ElementContext, Theme, UiHost};

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

pub(super) fn row_height_for_index(
    index: usize,
    measure_mode: VirtualListMeasureMode,
    estimate_row_height: Px,
    known_row_height_at: Option<&Arc<dyn Fn(usize) -> Px + Send + Sync>>,
) -> Option<Px> {
    match measure_mode {
        VirtualListMeasureMode::Measured => None,
        VirtualListMeasureMode::Fixed => Some(estimate_row_height),
        VirtualListMeasureMode::Known => known_row_height_at
            .map(|f| f(index))
            .or(Some(estimate_row_height)),
    }
}

pub(super) fn row_test_id(base: Option<&Arc<str>>, index: usize) -> Option<Arc<str>> {
    base.map(|base| Arc::from(format!("{base}.row.{index}")))
}

pub(super) fn pack_row_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: Vec<AnyElement>,
) -> AnyElement {
    match children.len() {
        0 => empty_row(cx),
        1 => children.into_iter().next().expect("single row child"),
        _ => crate::ui::v_flex(move |_cx| children)
            .gap_metric(crate::MetricRef::space(crate::Space::N0))
            .justify(crate::Justify::Start)
            .items(crate::Items::Stretch)
            .no_wrap()
            .into_element(cx),
    }
}

fn empty_row<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.container(ContainerProps::default(), |_cx| Vec::new())
}
