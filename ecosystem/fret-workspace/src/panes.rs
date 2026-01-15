use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px};
use fret_ui::action::OnPointerDown;
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, LayoutStyle, Length, PointerRegionProps,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::commands::pane_activate_command;
use crate::layout::{WorkspacePaneLayout, WorkspacePaneTree};

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn split_container_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn pane_container_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn pane_border_color(theme: &Theme, is_active: bool) -> Option<Color> {
    if is_active {
        theme
            .color_by_key("workspace.pane.active_border")
            .or_else(|| theme.color_by_key("ring"))
            .or_else(|| theme.color_by_key("border"))
    } else {
        theme.color_by_key("border")
    }
}

fn pane_border_width(is_active: bool) -> Edges {
    let _ = is_active;
    Edges::all(Px(1.0))
}

fn pane_corner_radius(theme: &Theme) -> Corners {
    let r = theme
        .metric_by_key("workspace.pane.radius")
        .unwrap_or(Px(0.0));
    Corners::all(Px(r.0.max(0.0)))
}

pub fn workspace_pane_tree_element<H: UiHost, F>(
    cx: &mut ElementContext<'_, H>,
    tree: &WorkspacePaneTree,
    active_pane: Option<&str>,
    render_pane: &mut F,
) -> AnyElement
where
    F: FnMut(&mut ElementContext<'_, H>, &WorkspacePaneLayout, bool) -> AnyElement,
{
    render_node(cx, tree, active_pane, render_pane)
}

fn render_node<H: UiHost, F>(
    cx: &mut ElementContext<'_, H>,
    node: &WorkspacePaneTree,
    active_pane: Option<&str>,
    render_pane: &mut F,
) -> AnyElement
where
    F: FnMut(&mut ElementContext<'_, H>, &WorkspacePaneLayout, bool) -> AnyElement,
{
    match node {
        WorkspacePaneTree::Leaf(pane) => render_leaf(cx, pane, active_pane, render_pane),
        WorkspacePaneTree::Split {
            axis,
            fraction,
            a,
            b,
        } => {
            let axis = *axis;
            let fraction = *fraction;

            cx.flex(
                FlexProps {
                    layout: split_container_layout(),
                    direction: axis,
                    ..Default::default()
                },
                |cx| {
                    let mut a_layout = fill_layout();
                    a_layout.flex.grow = fraction.max(0.0);

                    let mut b_layout = fill_layout();
                    b_layout.flex.grow = (1.0 - fraction).max(0.0);

                    vec![
                        cx.container(
                            ContainerProps {
                                layout: a_layout,
                                ..Default::default()
                            },
                            |cx| vec![render_node(cx, a.as_ref(), active_pane, render_pane)],
                        ),
                        cx.container(
                            ContainerProps {
                                layout: b_layout,
                                ..Default::default()
                            },
                            |cx| vec![render_node(cx, b.as_ref(), active_pane, render_pane)],
                        ),
                    ]
                },
            )
        }
    }
}

fn render_leaf<H: UiHost, F>(
    cx: &mut ElementContext<'_, H>,
    pane: &WorkspacePaneLayout,
    active_pane: Option<&str>,
    render_pane: &mut F,
) -> AnyElement
where
    F: FnMut(&mut ElementContext<'_, H>, &WorkspacePaneLayout, bool) -> AnyElement,
{
    let is_active = active_pane.is_some_and(|id| pane.id.as_ref() == id);

    let (border, border_color, corner_radii, background) = {
        let theme = Theme::global(cx.app);
        let background = theme.color_by_key("workspace.pane.bg");
        (
            pane_border_width(is_active),
            pane_border_color(theme, is_active),
            pane_corner_radius(theme),
            background,
        )
    };

    let pane_id = pane.id.clone();
    let activate_cmd = pane_activate_command(pane_id.as_ref());

    cx.pointer_region(
        PointerRegionProps {
            layout: pane_container_layout(),
            enabled: true,
        },
        |cx| {
            if let Some(cmd) = activate_cmd {
                let cmd = cmd.clone();
                let handler: OnPointerDown = Arc::new(move |host, acx, _down| {
                    host.dispatch_command(Some(acx.window), cmd.clone());
                    false
                });
                cx.pointer_region_add_on_pointer_down(handler);
            }

            vec![cx.container(
                ContainerProps {
                    layout: fill_layout(),
                    background,
                    border,
                    border_color,
                    corner_radii,
                    ..Default::default()
                },
                |cx| vec![render_pane(cx, pane, is_active)],
            )]
        },
    )
}
