use fret_core::{Color, Corners, Edges, Px};
use fret_runtime::CommandId;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, MainAlign, PressableProps, RowProps, SpacerProps,
};
use fret_ui::{ElementCx, Invalidation, Theme, UiHost};

use crate::Size;

fn resolve_list_colors(theme: &Theme) -> (Color, Color, Color, Color) {
    let list_bg = theme
        .color_by_key("list.background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or(theme.colors.panel_background);
    let border = theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("list.border"))
        .unwrap_or(theme.colors.panel_border);
    let row_hover = theme
        .color_by_key("list.row.hover")
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or(theme.colors.list_row_hover);
    let row_active = theme
        .color_by_key("list.row.active")
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or(theme.colors.list_row_selected);
    (list_bg, border, row_hover, row_active)
}

fn resolve_row_height(theme: &Theme, size: Size) -> Px {
    let base = theme
        .metric_by_key("component.list.row_height")
        .unwrap_or_else(|| size.list_row_h(theme));
    Px(base.0.max(0.0))
}

/// A small declarative list helper meant for migrating away from `VirtualListRow` fixed schemas.
///
/// This returns a `fret-ui` declarative `VirtualList` element whose rows are composed of arbitrary
/// child elements (icon/text/spacer/trailing), so shadcn-like variants can live in the component
/// layer instead of being forced into runtime list row structs.
pub fn list_from_strings<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    items: Model<Vec<String>>,
    selection: Option<Model<Option<usize>>>,
    size: Size,
    on_select: impl Fn(usize) -> Option<CommandId>,
) -> AnyElement {
    cx.observe_model(items, Invalidation::Layout);
    let values = cx.app.models().get(items).cloned().unwrap_or_default();

    let selected = selection
        .map(|m| {
            cx.observe_model(m, Invalidation::Paint);
            cx.app.models().get(m).cloned().unwrap_or(None)
        })
        .unwrap_or(None);

    let theme = Theme::global(&*cx.app);
    let (list_bg, border, row_hover, row_active) = resolve_list_colors(theme);
    let radius = theme.metrics.radius_md;

    let row_h = resolve_row_height(theme, size);
    let row_px = theme
        .metric_by_key("component.list.row_padding_x")
        .unwrap_or(Px(10.0));
    let row_py = theme
        .metric_by_key("component.list.row_padding_y")
        .unwrap_or(Px(6.0));

    cx.container(
        ContainerProps {
            background: Some(list_bg),
            border: Edges::all(Px(1.0)),
            border_color: Some(border),
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        |cx| {
            vec![cx.virtual_list(values.len(), row_h, 2, |cx, range| {
                range
                    .map(|i| {
                        cx.keyed(i, |cx| {
                            let label = values.get(i).map(String::as_str).unwrap_or("");
                            let leading = if i % 3 == 0 { "●" } else { "○" };
                            let trailing = if i % 5 == 0 { "⌘O" } else { "" };

                            cx.pressable(
                                PressableProps {
                                    enabled: true,
                                    on_click: on_select(i),
                                },
                                |cx, st| {
                                    let is_selected = selected == Some(i);
                                    let bg = if is_selected {
                                        Some(row_active)
                                    } else if st.pressed {
                                        Some(row_active)
                                    } else if st.hovered {
                                        Some(row_hover)
                                    } else {
                                        None
                                    };

                                    vec![cx.container(
                                        ContainerProps {
                                            padding_x: row_px,
                                            padding_y: row_py,
                                            background: bg,
                                            ..Default::default()
                                        },
                                        |cx| {
                                            vec![cx.row(
                                                RowProps {
                                                    gap: Px(8.0),
                                                    justify: MainAlign::Start,
                                                    align: CrossAlign::Center,
                                                    ..Default::default()
                                                },
                                                |cx| {
                                                    let mut out = Vec::new();
                                                    out.push(cx.text(leading));
                                                    out.push(cx.text(label));
                                                    out.push(
                                                        cx.spacer(SpacerProps { min: Px(0.0) }),
                                                    );
                                                    if !trailing.is_empty() {
                                                        out.push(cx.text(trailing));
                                                    }
                                                    out
                                                },
                                            )]
                                        },
                                    )]
                                },
                            )
                        })
                    })
                    .collect()
            })]
        },
    )
}
