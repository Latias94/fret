use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_ui::element::{AnyElement, ContainerProps, Length, SemanticsProps};
use fret_ui::{ElementContext, UiHost};

use super::super::{TableOptions, body::TablePalette};

pub(super) fn table_root_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: Vec<AnyElement>,
    palette: &TablePalette,
    options: TableOptions,
) -> AnyElement {
    let row_gap = options.row_gap.clone();
    let test_id = options.test_id;

    let mut root = ContainerProps::default();
    root.layout.size.width = Length::Fill;
    root.layout.size.height = Length::Auto;
    root.background = Some(palette.table_bg);
    root.border = Edges::all(Px(1.0));
    root.border_color = Some(palette.border);
    root.corner_radii = Corners::all(Px(6.0));

    let table = cx.container(root, move |cx| {
        vec![
            crate::ui::v_flex(move |_cx| children)
                .gap_metric(row_gap.clone())
                .justify(crate::Justify::Start)
                .items(crate::Items::Stretch)
                .no_wrap()
                .into_element(cx),
        ]
    });

    if let Some(test_id) = test_id {
        let mut semantics = SemanticsProps::default();
        semantics.role = SemanticsRole::Group;
        semantics.test_id = Some(test_id);
        cx.semantics(semantics, move |_cx| vec![table])
    } else {
        table
    }
}
