use std::sync::Arc;

use fret_runtime::{InputContext, KeyChord, format_chord, format_sequence};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, LayoutStyle, Length, MarginEdge, MarginEdges,
};
use fret_ui::pending_shortcut::PendingShortcutContinuation;
use fret_ui::{ElementContext, Theme, UiHost};

pub fn pending_shortcut_hint_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    top_inset: fret_core::Px,
    input_ctx: &InputContext,
    sequence: &[KeyChord],
    continuations: &[PendingShortcutContinuation],
) -> Option<AnyElement> {
    if sequence.is_empty() {
        return None;
    }

    let theme = Theme::global(cx.app);
    let bg = theme
        .color_by_key("popover")
        .or_else(|| theme.color_by_key("background"));
    let border = theme.color_by_key("border");
    let radius = theme.metric_by_key("radius").unwrap_or(fret_core::Px(8.0));

    let platform = input_ctx.platform;
    let header = Arc::<str>::from(format!("{} …", format_sequence(platform, sequence)));

    let mut lines: Vec<Arc<str>> = Vec::with_capacity(1 + continuations.len());
    lines.push(header);
    for cont in continuations.iter().cloned() {
        let key = format_chord(platform, cont.next);
        let title: Arc<str> = cont
            .command
            .clone()
            .and_then(|cmd| cx.app.commands().get(cmd.clone()).map(|m| m.title.clone()))
            .unwrap_or_else(|| Arc::<str>::from("…"));
        let suffix = if cont.has_continuation { " …" } else { "" };
        lines.push(Arc::<str>::from(format!("{key} → {title}{suffix}")));
    }

    let mut layout = LayoutStyle::default();
    layout.position = fret_ui::element::PositionStyle::Absolute;
    layout.inset.top = Some(top_inset);
    layout.inset.left = Some(fret_core::Px(0.0));
    layout.inset.right = Some(fret_core::Px(0.0));
    layout.size.width = Length::Px(fret_core::Px(520.0));
    layout.margin = MarginEdges {
        left: MarginEdge::Auto,
        right: MarginEdge::Auto,
        ..Default::default()
    };

    Some(cx.container(
        ContainerProps {
            layout,
            padding: fret_core::Edges::all(fret_core::Px(10.0)),
            background: bg,
            border: fret_core::Edges::all(fret_core::Px(1.0)),
            border_color: border,
            corner_radii: fret_core::Corners::all(radius),
            ..Default::default()
        },
        |cx| {
            let col = cx.column(
                ColumnProps {
                    gap: fret_core::Px(6.0),
                    ..Default::default()
                },
                |cx| {
                    lines
                        .into_iter()
                        .map(|line| cx.text(line))
                        .collect::<Vec<_>>()
                },
            );
            vec![col]
        },
    ))
}
