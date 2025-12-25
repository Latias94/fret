use fret_core::{Color, Corners, Edges, Px};
use fret_runtime::CommandId;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, MainAlign, PressableProps, RowProps, SpacerProps,
};
use fret_ui::{ElementCx, Invalidation, Theme, UiHost};

use crate::{MetricRef, Size, Space};

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

fn resolve_row_padding_x(theme: &Theme) -> Px {
    // Prefer component-level Tailwind-like tokens; fall back to baseline metrics to avoid drift.
    MetricRef::space(Space::N2p5).resolve(theme)
}

fn resolve_row_padding_y(theme: &Theme) -> Px {
    MetricRef::space(Space::N1p5).resolve(theme)
}

fn resolve_row_gap(theme: &Theme) -> Px {
    MetricRef::space(Space::N2).resolve(theme)
}

/// Declarative virtualized list helper (component-friendly, row content is fully composable).
///
/// This intentionally avoids a fixed row schema (`VirtualListRow { text/secondary/trailing... }`)
/// so higher-level shadcn-like components can be built in the component layer via composition.
pub fn list_virtualized<H: UiHost, K: std::hash::Hash>(
    cx: &mut ElementCx<'_, H>,
    selection: Option<Model<Option<usize>>>,
    size: Size,
    row_height: Option<Px>,
    len: usize,
    overscan: usize,
    scroll_to_index: Option<usize>,
    key_at: impl FnMut(usize) -> K,
    on_select: impl Fn(usize) -> Option<CommandId>,
    mut row_contents: impl FnMut(&mut ElementCx<'_, H>, usize) -> Vec<AnyElement>,
) -> AnyElement {
    let selected = selection
        .map(|m| {
            cx.observe_model(m, Invalidation::Paint);
            cx.app.models().get(m).cloned().unwrap_or(None)
        })
        .unwrap_or(None);

    let scroll_to_index = scroll_to_index.or(selected);

    let theme = Theme::global(&*cx.app);
    let (list_bg, border, row_hover, row_active) = resolve_list_colors(theme);
    let radius = theme.metrics.radius_md;

    let row_h = row_height.unwrap_or_else(|| resolve_row_height(theme, size));
    let row_px = resolve_row_padding_x(theme);
    let row_py = resolve_row_padding_y(theme);
    let row_gap = resolve_row_gap(theme);

    cx.container(
        ContainerProps {
            background: Some(list_bg),
            border: Edges::all(Px(1.0)),
            border_color: Some(border),
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        |cx| {
            vec![
                cx.virtual_list_keyed(len, row_h, overscan, scroll_to_index, key_at, |cx, i| {
                    let cmd = on_select(i);
                    let enabled = cmd.is_some();

                    cx.pressable(
                        PressableProps {
                            enabled,
                            on_click: cmd,
                            ..Default::default()
                        },
                        |cx, st| {
                            let is_selected = selected == Some(i);
                            let bg = if is_selected {
                                Some(row_active)
                            } else if enabled && st.pressed {
                                Some(row_active)
                            } else if enabled && st.hovered {
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
                                            gap: row_gap,
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            ..Default::default()
                                        },
                                        |cx| row_contents(cx, i),
                                    )]
                                },
                            )]
                        },
                    )
                }),
            ]
        },
    )
}

/// Compatibility helper for simple string lists (used in demos).
pub fn list_from_strings<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    items: Model<Vec<String>>,
    selection: Option<Model<Option<usize>>>,
    size: Size,
    on_select: impl Fn(usize) -> Option<CommandId>,
) -> AnyElement {
    cx.observe_model(items, Invalidation::Layout);
    let values = cx.app.models().get(items).cloned().unwrap_or_default();

    list_virtualized(
        cx,
        selection,
        size,
        None,
        values.len(),
        2,
        None,
        |i| i,
        on_select,
        |cx, i| {
            let label = values.get(i).map(String::as_str).unwrap_or("");
            let leading = if i % 3 == 0 { "●" } else { "○" };
            let trailing = if i % 5 == 0 { "⌘O" } else { "" };

            let mut out = Vec::new();
            out.push(cx.text(leading));
            out.push(cx.text(label));
            out.push(cx.spacer(SpacerProps {
                min: Px(0.0),
                ..Default::default()
            }));
            if !trailing.is_empty() {
                out.push(cx.text(trailing));
            }
            out
        },
    )
}
