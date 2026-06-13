use fret_core::{Edges, Px, SemanticsRole};
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign};
use fret_ui::{ElementContext, UiHost};

use super::{SelectEntry, content_tree::SelectRow};
use crate::select::content_tree::select_group_label;

pub(super) fn render_select_entries<H, F>(
    cx: &mut ElementContext<'_, H>,
    entries: &[SelectEntry],
    out: &mut Vec<AnyElement>,
    row_idx_cursor: &mut usize,
    render_row: &mut F,
) where
    H: UiHost,
    F: FnMut(&mut ElementContext<'_, H>, usize, SelectRow, &mut Vec<AnyElement>),
{
    for entry in entries {
        match entry {
            SelectEntry::Item(item) => {
                let row_idx = *row_idx_cursor;
                *row_idx_cursor = row_idx.saturating_add(1);
                render_row(cx, row_idx, SelectRow::Item(item.clone()), out);
            }
            SelectEntry::Label(label) => {
                let row_idx = *row_idx_cursor;
                *row_idx_cursor = row_idx.saturating_add(1);
                render_row(cx, row_idx, SelectRow::Label(label.clone()), out);
            }
            SelectEntry::Separator(_) => {
                let row_idx = *row_idx_cursor;
                *row_idx_cursor = row_idx.saturating_add(1);
                render_row(cx, row_idx, SelectRow::Separator, out);
            }
            SelectEntry::Group(group) => {
                let label = select_group_label(&group.entries);
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                out.push(cx.semantics(
                    fret_ui::element::SemanticsProps {
                        layout,
                        role: SemanticsRole::Group,
                        label,
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.flex(
                            FlexProps {
                                layout: LayoutStyle::default(),
                                direction: fret_core::Axis::Vertical,
                                gap: Px(0.0).into(),
                                padding: Edges::all(Px(0.0)).into(),
                                justify: MainAlign::Start,
                                align: CrossAlign::Stretch,
                                wrap: false,
                            },
                            |cx| {
                                let mut inner = Vec::new();
                                render_select_entries(
                                    cx,
                                    &group.entries,
                                    &mut inner,
                                    row_idx_cursor,
                                    render_row,
                                );
                                inner
                            },
                        )]
                    },
                ));
            }
        }
    }
}
