use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Px, SemanticsRole};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, Overflow, SemanticsProps,
    VirtualListMeasureMode, VirtualListOptions as RuntimeVirtualListOptions,
};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, GlobalElementId, Theme, UiHost};

use super::containers::build_imui_children_with_focus;
use super::{ImUiFacade, VirtualListOptions, VirtualListResponse};

pub(super) fn virtual_list_element<H: UiHost, K, R>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    len: usize,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: VirtualListOptions,
    mut key_at: K,
    mut row: R,
) -> (AnyElement, VirtualListResponse)
where
    K: FnMut(usize) -> fret_ui::ItemKey,
    R: for<'cx2, 'a2> FnMut(&mut ImUiFacade<'cx2, 'a2, H>, usize),
{
    cx.keyed(("fret-ui-kit.imui.virtual_list", id), |cx| {
        let handle = options
            .handle
            .clone()
            .unwrap_or_else(|| cx.slot_state(VirtualListScrollHandle::new, |h| h.clone()));
        let root_test_id = options.test_id.clone();
        let row_height_fn = options.known_row_height_at.clone();
        let resolved_measure_mode = resolved_measure_mode(&options);
        let first_rendered = Rc::new(Cell::new(None::<usize>));
        let last_rendered = Rc::new(Cell::new(None::<usize>));
        let first_rendered_out = first_rendered.clone();
        let last_rendered_out = last_rendered.clone();

        let list = cx.virtual_list_keyed_with_layout(
            list_layout(&options),
            len,
            runtime_options(&options, resolved_measure_mode),
            &handle,
            &mut key_at,
            move |cx, index| {
                if first_rendered.get().is_none() {
                    first_rendered.set(Some(index));
                }
                last_rendered.set(Some(index));

                let mut out = Vec::new();
                build_imui_children_with_focus(cx, &mut out, build_focus.clone(), |ui| {
                    row(ui, index);
                });
                let content = pack_row_children(cx, out);
                wrap_row(
                    cx,
                    index,
                    content,
                    row_test_id(root_test_id.as_ref(), index),
                    row_height_for_index(
                        index,
                        resolved_measure_mode,
                        options.estimate_row_height,
                        row_height_fn.as_ref(),
                    ),
                )
            },
        );

        let list = if let Some(test_id) = options.test_id {
            let mut semantics = SemanticsProps::default();
            semantics.role = SemanticsRole::List;
            semantics.test_id = Some(test_id);
            cx.semantics(semantics, move |_cx| vec![list])
        } else {
            list
        };

        (
            list,
            VirtualListResponse {
                handle,
                rendered_range: first_rendered_out.get().zip(last_rendered_out.get()),
            },
        )
    })
}

fn runtime_options(
    options: &VirtualListOptions,
    measure_mode: VirtualListMeasureMode,
) -> RuntimeVirtualListOptions {
    let mut runtime = RuntimeVirtualListOptions::new(options.estimate_row_height, options.overscan);
    runtime.items_revision = options.items_revision;
    runtime.measure_mode = measure_mode;
    runtime.key_cache = options.key_cache;
    runtime.keep_alive = options.keep_alive;
    runtime.gap = options.gap;
    runtime.scroll_margin = options.scroll_margin;
    runtime.known_row_height_at = options.known_row_height_at.clone();
    runtime
}

fn resolved_measure_mode(options: &VirtualListOptions) -> VirtualListMeasureMode {
    if matches!(options.measure_mode, VirtualListMeasureMode::Known)
        && options.known_row_height_at.is_none()
    {
        VirtualListMeasureMode::Measured
    } else {
        options.measure_mode
    }
}

fn list_layout(options: &VirtualListOptions) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Px(options.viewport_height);
    layout.overflow = Overflow::Clip;
    layout
}

fn wrap_row<H: UiHost>(
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

fn row_height_for_index(
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

fn row_test_id(base: Option<&Arc<str>>, index: usize) -> Option<Arc<str>> {
    base.map(|base| Arc::from(format!("{base}.row.{index}")))
}

fn pack_row_children<H: UiHost>(
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::{
        ContainerProps, ElementKind, LayoutStyle, Length, Overflow, VirtualListMeasureMode,
    };

    use super::{row_height_for_index, wrap_row};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        )
    }

    fn oversized_content(
        cx: &mut fret_ui::ElementContext<'_, App>,
    ) -> fret_ui::element::AnyElement {
        let mut props = ContainerProps::default();
        props.layout.size.height = Length::Px(Px(80.0));
        cx.container(props, |_cx| Vec::new())
    }

    #[test]
    fn fixed_virtual_list_rows_clip_content_to_row_height() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let row = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "virtual-list-fixed-row",
            |cx| {
                let content = oversized_content(cx);
                wrap_row(cx, 0, content, None, Some(Px(20.0)))
            },
        );

        let ElementKind::Container(ContainerProps { layout, .. }) = row.kind else {
            panic!("expected fixed virtual-list row to render as a container");
        };

        assert_eq!(layout.size.height, Length::Px(Px(20.0)));
        assert_eq!(layout.overflow, Overflow::Clip);
    }

    #[test]
    fn known_virtual_list_rows_clip_content_to_known_row_height() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let height_at: Arc<dyn Fn(usize) -> Px + Send + Sync> =
            Arc::new(|index| if index == 1 { Px(36.0) } else { Px(20.0) });
        let fixed_height =
            row_height_for_index(1, VirtualListMeasureMode::Known, Px(20.0), Some(&height_at));

        let row = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "virtual-list-known-row",
            |cx| {
                let content = oversized_content(cx);
                wrap_row(cx, 1, content, None, fixed_height)
            },
        );

        let ElementKind::Container(ContainerProps { layout, .. }) = row.kind else {
            panic!("expected known-height virtual-list row to render as a container");
        };

        assert_eq!(layout.size.height, Length::Px(Px(36.0)));
        assert_eq!(layout.overflow, Overflow::Clip);
    }

    #[test]
    fn measured_virtual_list_rows_keep_content_overflow_visible_for_measurement() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let row = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds(),
            "virtual-list-measured-row",
            |cx| {
                let content = oversized_content(cx);
                wrap_row(cx, 0, content, None, None)
            },
        );

        let ElementKind::Container(ContainerProps { layout, .. }) = row.kind else {
            panic!("expected measured virtual-list row to render as a container");
        };

        assert_eq!(layout.size.height, Length::Auto);
        assert_eq!(layout.overflow, LayoutStyle::default().overflow);
    }
}
