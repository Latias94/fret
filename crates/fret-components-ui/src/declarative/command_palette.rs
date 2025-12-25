use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, TextOverflow, TextStyle, TextWrap};
use fret_runtime::CommandId;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, ElementKind, FlexProps, LayoutStyle,
    MainAlign, PressableProps, TextProps,
};
use fret_ui::{ElementCx, Invalidation, Theme, UiHost};

use super::style;
use crate::command::CommandItem;
use crate::{LayoutRefinement, Size, Space};

#[derive(Debug, Clone)]
enum RowKind {
    Header {
        label: Arc<str>,
    },
    Item {
        id: Arc<str>,
        label: Arc<str>,
        detail: Option<Arc<str>>,
        shortcut: Option<Arc<str>>,
        enabled: bool,
    },
}

#[derive(Debug, Clone)]
struct Row {
    key: u64,
    kind: RowKind,
}

#[derive(Default)]
struct CommandRowsState {
    last_items_revision: Option<u64>,
    last_query_revision: Option<u64>,
    rows: Vec<Row>,
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn stable_key(kind: &str, s: &str) -> u64 {
    let mut buf = Vec::with_capacity(kind.len() + 1 + s.len());
    buf.extend_from_slice(kind.as_bytes());
    buf.push(b'|');
    buf.extend_from_slice(s.as_bytes());
    fnv1a_64(&buf)
}

fn matches_query(item: &CommandItem, q: &str) -> bool {
    if q.is_empty() {
        return true;
    }
    let q = q.trim();
    if q.is_empty() {
        return true;
    }

    let label = item.label.as_ref().to_ascii_lowercase();
    if label.contains(q) {
        return true;
    }
    item.keywords
        .iter()
        .any(|k| k.as_ref().to_ascii_lowercase().contains(q))
}

fn rebuild_rows(items: Vec<CommandItem>, query: String) -> Vec<Row> {
    let q = query.trim().to_ascii_lowercase();
    let mut filtered: Vec<CommandItem> =
        items.into_iter().filter(|i| matches_query(i, &q)).collect();

    filtered.sort_by(|a, b| {
        let ag = a.group.as_deref().unwrap_or("");
        let bg = b.group.as_deref().unwrap_or("");
        ag.cmp(bg)
            .then_with(|| a.label.as_ref().cmp(b.label.as_ref()))
    });

    let mut rows: Vec<Row> = Vec::new();
    let mut current_group: Option<Arc<str>> = None;

    for item in filtered {
        let group = item.group.clone().unwrap_or_else(|| Arc::<str>::from(""));
        let group_changed = current_group
            .as_ref()
            .is_none_or(|g| g.as_ref() != group.as_ref());

        if group_changed {
            if !group.is_empty() {
                rows.push(Row {
                    key: stable_key("header", group.as_ref()),
                    kind: RowKind::Header {
                        label: group.clone(),
                    },
                });
            }
            current_group = Some(group);
        }

        rows.push(Row {
            key: stable_key("item", item.id.as_ref()),
            kind: RowKind::Item {
                id: item.id,
                label: item.label,
                detail: item.detail,
                shortcut: item.shortcut,
                enabled: item.enabled,
            },
        });
    }

    rows
}

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
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or(theme.colors.list_row_hover);
    let row_active = theme
        .color_by_key("list.active.background")
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or(theme.colors.list_row_selected);
    (list_bg, border, row_hover, row_active)
}

fn text_element<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    key: &'static str,
    text: Arc<str>,
    style: TextStyle,
    color: Color,
    layout: LayoutStyle,
) -> AnyElement {
    cx.keyed(key, |cx| {
        let id = cx.root_id();
        AnyElement::new(
            id,
            ElementKind::Text(TextProps {
                layout,
                text,
                style: Some(style),
                color: Some(color),
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
            }),
            Vec::new(),
        )
    })
}

/// Declarative command palette list body (group headers + selectable items) designed for embedding
/// under `CommandPaletteOverlay`.
///
/// This intentionally does not rely on `VirtualListRow { text/secondary/trailing... }` schemas:
/// each row is composed out of low-level declarative primitives so shadcn-like variants can evolve
/// entirely in the component layer (MVP 50).
pub fn command_palette_list<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    items: Model<Vec<CommandItem>>,
    query: Model<String>,
    selection: Model<Option<Arc<str>>>,
    size: Size,
) -> AnyElement {
    cx.observe_model(items, Invalidation::Layout);
    cx.observe_model(query, Invalidation::Layout);
    cx.observe_model(selection, Invalidation::Paint);

    let items_rev = cx.app.models().revision(items);
    let query_rev = cx.app.models().revision(query);

    let items_value = cx.app.models().get(items).cloned().unwrap_or_default();
    let query_value = cx.app.models().get(query).cloned().unwrap_or_default();

    let rows = cx.with_state(CommandRowsState::default, |st| {
        if st.last_items_revision != items_rev || st.last_query_revision != query_rev {
            st.last_items_revision = items_rev;
            st.last_query_revision = query_rev;
            st.rows = rebuild_rows(items_value, query_value);
        }
        st.rows.clone()
    });

    let selected = cx.app.models().get(selection).cloned().unwrap_or(None);
    let scroll_to_index = selected.as_ref().and_then(|id| {
        rows.iter().position(|r| match &r.kind {
            RowKind::Item { id: row_id, .. } => row_id.as_ref() == id.as_ref(),
            _ => false,
        })
    });

    let theme = Theme::global(&*cx.app).clone();
    let (list_bg, border, row_hover, row_active) = resolve_list_colors(&theme);
    let radius = theme.metrics.radius_md;
    let fg = theme
        .color_by_key("foreground")
        .unwrap_or(theme.colors.text_primary);
    let muted_fg = theme
        .color_by_key("muted.foreground")
        .unwrap_or(theme.colors.text_muted);
    let disabled_fg = theme.colors.text_disabled;

    let text_px = size.control_text_px(&theme);
    let small_px = Px((text_px.0 - 1.0).max(0.0));

    // Fixed row height for now (measured rows are a follow-up); choose a slightly taller height
    // when any visible item has `detail`.
    let any_detail = rows.iter().any(|r| {
        matches!(
            r.kind,
            RowKind::Item {
                detail: Some(_),
                ..
            }
        )
    });
    let base_row_h = size.list_row_h(&theme);
    let detail_extra = style::space(&theme, Space::N1p5);
    let row_h = if any_detail {
        Px((base_row_h.0 + (small_px.0 + detail_extra.0)).max(base_row_h.0))
    } else {
        base_row_h
    };

    let row_px = size.list_px(&theme);
    let row_py = size.list_py(&theme);
    let row_gap = style::space(&theme, Space::N2);
    let col_gap = style::space(&theme, Space::N0p5);

    let row_left_layout =
        style::layout_style(&theme, LayoutRefinement::default().flex_1().min_w_0());
    let row_shortcut_layout = style::layout_style(
        &theme,
        LayoutRefinement::default().flex_none().flex_shrink_0(),
    );

    cx.container(
        ContainerProps {
            background: Some(list_bg),
            border: Edges::all(Px(1.0)),
            border_color: Some(border),
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        |cx| {
            vec![cx.virtual_list_keyed(
                rows.len(),
                row_h,
                2,
                scroll_to_index,
                |i| rows.get(i).map(|r| r.key).unwrap_or_default(),
                |cx, i| {
                    let Some(row) = rows.get(i) else {
                        return cx.text("");
                    };

                    match &row.kind {
                        RowKind::Header { label } => cx.container(
                            ContainerProps {
                                padding_x: row_px,
                                padding_y: Px((row_py.0 * 0.75).max(0.0)),
                                ..Default::default()
                            },
                            |cx| {
                                vec![text_element(
                                    cx,
                                    "header",
                                    label.clone(),
                                    TextStyle {
                                        font: fret_core::FontId::default(),
                                        size: small_px,
                                        ..Default::default()
                                    },
                                    muted_fg,
                                    Default::default(),
                                )]
                            },
                        ),
                        RowKind::Item {
                            id,
                            label,
                            detail,
                            shortcut,
                            enabled,
                        } => {
                            let is_selected = selected
                                .as_ref()
                                .is_some_and(|sel| sel.as_ref() == id.as_ref());

                            cx.pressable(
                                PressableProps {
                                    enabled: *enabled,
                                    on_click: enabled.then(|| {
                                        CommandId::new(format!("command_palette.select.{}", id))
                                    }),
                                    ..Default::default()
                                },
                                |cx, st| {
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
                                            vec![cx.flex(
                                                FlexProps {
                                                    direction: fret_core::Axis::Horizontal,
                                                    gap: row_gap,
                                                    justify: MainAlign::Start,
                                                    align: CrossAlign::Center,
                                                    ..Default::default()
                                                },
                                                |cx| {
                                                    let mut out = Vec::new();

                                                    out.push(cx.column(
                                                        ColumnProps {
                                                            layout: row_left_layout.clone(),
                                                            gap: col_gap,
                                                            justify: MainAlign::Start,
                                                            align: CrossAlign::Start,
                                                            ..Default::default()
                                                        },
                                                        |cx| {
                                                            let mut col = Vec::new();
                                                            let label_color = if *enabled {
                                                                fg
                                                            } else {
                                                                disabled_fg
                                                            };

                                                            col.push(text_element(
                                                                cx,
                                                                "label",
                                                                label.clone(),
                                                                TextStyle {
                                                                    font:
                                                                        fret_core::FontId::default(),
                                                                    size: text_px,
                                                                    ..Default::default()
                                                                },
                                                                label_color,
                                                                Default::default(),
                                                            ));

                                                            if let Some(detail) = detail.clone() {
                                                                col.push(text_element(
                                                                    cx,
                                                                    "detail",
                                                                    detail,
                                                                    TextStyle {
                                                                        font: fret_core::FontId::default(),
                                                                        size: small_px,
                                                                        ..Default::default()
                                                                    },
                                                                    muted_fg,
                                                                    Default::default(),
                                                                ));
                                                            }

                                                            col
                                                        },
                                                    ));

                                                    if let Some(sc) = shortcut.clone()
                                                        && !sc.is_empty()
                                                    {
                                                        out.push(text_element(
                                                            cx,
                                                            "shortcut",
                                                            sc,
                                                            TextStyle {
                                                                font: fret_core::FontId::default(),
                                                                size: small_px,
                                                                ..Default::default()
                                                            },
                                                            muted_fg,
                                                            row_shortcut_layout.clone(),
                                                        ));
                                                    }

                                                    out
                                                },
                                            )]
                                        },
                                    )]
                                },
                            )
                        }
                    }
                },
            )]
        },
    )
}
