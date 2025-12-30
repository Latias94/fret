use fret_core::{Color, Corners, Edges, Px};
use fret_runtime::CommandId;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ContainerProps, PressableProps, SpacerProps};
use fret_ui::scroll::{ScrollStrategy, VirtualListScrollHandle};
use fret_ui::{ElementCx, Invalidation, Theme, UiHost};

use crate::declarative::action_hooks::ActionHooksExt;
use crate::declarative::stack;
use crate::{Items, Justify, MetricRef, Size, Space};

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
        .color_by_key("list.hover.background")
        .or_else(|| theme.color_by_key("list.row.hover"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or(theme.colors.list_row_hover);
    let row_active = theme
        .color_by_key("list.active.background")
        .or_else(|| theme.color_by_key("list.row.active"))
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

/// Declarative virtualized list helper (component-friendly, row content is fully composable).
///
/// This intentionally avoids a fixed row schema (`VirtualListRow { text/secondary/trailing... }`)
/// so higher-level shadcn-like components can be built in the component layer via composition.
#[allow(clippy::too_many_arguments)]
pub fn list_virtualized<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    selection: Option<Model<Option<usize>>>,
    size: Size,
    row_height: Option<Px>,
    len: usize,
    overscan: usize,
    scroll_handle: &VirtualListScrollHandle,
    items_revision: u64,
    key_at: impl FnMut(usize) -> u64,
    on_select: impl Fn(usize) -> Option<CommandId>,
    mut row_contents: impl FnMut(&mut ElementCx<'_, H>, usize) -> Vec<AnyElement>,
) -> AnyElement {
    let selected = selection
        .map(|m| {
            cx.observe_model(m, Invalidation::Paint);
            cx.app.models().get(m).cloned().unwrap_or(None)
        })
        .unwrap_or(None);

    if let Some(selected) = selected {
        scroll_handle.scroll_to_item(selected, ScrollStrategy::Nearest);
    }

    let theme = Theme::global(&*cx.app);
    let (list_bg, border, row_hover, row_active) = resolve_list_colors(theme);
    let radius = theme.metrics.radius_md;

    let row_h = row_height.unwrap_or_else(|| resolve_row_height(theme, size));
    let row_px = resolve_row_padding_x(theme);
    let row_py = resolve_row_padding_y(theme);

    let mut options = fret_ui::element::VirtualListOptions::new(row_h, overscan);
    options.items_revision = items_revision;

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
                cx.virtual_list_keyed(len, options, scroll_handle, key_at, |cx, i| {
                    let cmd = on_select(i);
                    let enabled = cmd.is_some();

                    cx.pressable(
                        PressableProps {
                            enabled,
                            ..Default::default()
                        },
                        |cx, st| {
                            cx.pressable_dispatch_command_opt(cmd);
                            let is_selected = selected == Some(i);
                            let bg = if is_selected || (enabled && st.pressed) {
                                Some(row_active)
                            } else if enabled && st.hovered {
                                Some(row_hover)
                            } else {
                                None
                            };

                            vec![cx.container(
                                ContainerProps {
                                    padding: Edges::symmetric(row_px, row_py),
                                    background: bg,
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![stack::hstack(
                                        cx,
                                        stack::HStackProps::default()
                                            .gap_x(Space::N2)
                                            .justify(Justify::Start)
                                            .items(Items::Center),
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

    let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());
    let items_revision = cx.app.models().revision(items).unwrap_or(0);

    list_virtualized(
        cx,
        selection,
        size,
        None,
        values.len(),
        2,
        &scroll_handle,
        items_revision,
        |i| i as u64,
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
