use super::*;

type BuildFocus = Option<Rc<Cell<Option<GlobalElementId>>>>;

pub(super) fn items<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    items_with_options(ui, build_focus, ItemFlowOptions::default(), f);
}

pub(super) fn items_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: ItemFlowOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| layout_sugar::items_element(cx, build_focus, options, f));
    ui.add(element);
}

pub(super) fn same_line<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    same_line_with_options(ui, build_focus, SameLineOptions::default(), f);
}

pub(super) fn same_line_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: SameLineOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| layout_sugar::same_line_element(cx, build_focus, options, f));
    ui.add(element);
}

pub(super) fn dummy<H: UiHost, W>(ui: &mut W, size: Size)
where
    W: UiWriter<H> + ?Sized,
{
    dummy_with_options(ui, size, DummyOptions::default());
}

pub(super) fn dummy_with_options<H: UiHost, W>(ui: &mut W, size: Size, options: DummyOptions)
where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| layout_sugar::dummy_element(cx, size, options));
    ui.add(element);
}

pub(super) fn spacing<H: UiHost, W>(ui: &mut W)
where
    W: UiWriter<H> + ?Sized,
{
    spacing_with_options(ui, SpacingOptions::default());
}

pub(super) fn spacing_with_options<H: UiHost, W>(ui: &mut W, options: SpacingOptions)
where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| layout_sugar::spacing_element(cx, options));
    ui.add(element);
}

pub(super) fn indent<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    indent_with_options(ui, build_focus, IndentOptions::default(), f);
}

pub(super) fn indent_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: IndentOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| layout_sugar::indent_element(cx, build_focus, options, f));
    ui.add(element);
}

pub(super) fn horizontal<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    horizontal_with_options(ui, build_focus, HorizontalOptions::default(), f);
}

pub(super) fn horizontal_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: HorizontalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| horizontal_container_element(cx, build_focus, options, f));
    ui.add(element);
}

pub(super) fn menu_bar<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    menu_bar_with_options(ui, build_focus, MenuBarOptions::default(), f);
}

pub(super) fn menu_bar_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: MenuBarOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element =
        ui.with_cx_mut(|cx| menu_family_controls::menu_bar_element(cx, build_focus, options, f));
    ui.add(element);
}

pub(super) fn tab_bar<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
) -> TabBarResponse
where
    W: UiWriter<H> + ?Sized,
{
    tab_bar_with_options(ui, build_focus, id, TabBarOptions::default(), f)
}

pub(super) fn tab_bar_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    options: TabBarOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
) -> TabBarResponse
where
    W: UiWriter<H> + ?Sized,
{
    let (element, response) =
        ui.with_cx_mut(|cx| tab_family_controls::tab_bar_element(cx, id, build_focus, options, f));
    ui.add(element);
    response
}

pub(super) fn vertical<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    vertical_with_options(ui, build_focus, VerticalOptions::default(), f);
}

pub(super) fn vertical_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: VerticalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| vertical_container_element(cx, build_focus, options, f));
    ui.add(element);
}

pub(super) fn list_box<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    label: impl Into<Arc<str>>,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    list_box_with_options(
        ui,
        build_focus,
        id,
        ListBoxOptions {
            label: Some(label.into()),
            ..Default::default()
        },
        f,
    );
}

pub(super) fn list_box_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    options: ListBoxOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element =
        ui.with_cx_mut(|cx| list_box_controls::list_box_element(cx, id, build_focus, options, f));
    ui.add(element);
}

pub(super) fn grid<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    grid_with_options(ui, build_focus, GridOptions::default(), f);
}

pub(super) fn grid_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: GridOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| grid_container_element(cx, build_focus, options, f));
    ui.add(element);
}

pub(super) fn table<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    columns: &[TableColumn],
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
) -> TableResponse
where
    W: UiWriter<H> + ?Sized,
{
    table_with_options(ui, build_focus, id, columns, TableOptions::default(), f)
}

pub(super) fn table_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    columns: &[TableColumn],
    options: TableOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
) -> TableResponse
where
    W: UiWriter<H> + ?Sized,
{
    let (element, response) = ui
        .with_cx_mut(|cx| table_controls::table_element(cx, id, columns, build_focus, options, f));
    ui.add(element);
    response
}

pub(super) fn virtual_list<H: UiHost, W, K, R>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    len: usize,
    key_at: K,
    row: R,
) -> VirtualListResponse
where
    W: UiWriter<H> + ?Sized,
    K: FnMut(usize) -> fret_ui::ItemKey,
    R: for<'cx2, 'a2> FnMut(&mut ImUiFacade<'cx2, 'a2, H>, usize),
{
    virtual_list_with_options(
        ui,
        build_focus,
        id,
        len,
        VirtualListOptions::default(),
        key_at,
        row,
    )
}

pub(super) fn virtual_list_with_options<H: UiHost, W, K, R>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    len: usize,
    options: VirtualListOptions,
    key_at: K,
    row: R,
) -> VirtualListResponse
where
    W: UiWriter<H> + ?Sized,
    K: FnMut(usize) -> fret_ui::ItemKey,
    R: for<'cx2, 'a2> FnMut(&mut ImUiFacade<'cx2, 'a2, H>, usize),
{
    let (element, response) = ui.with_cx_mut(|cx| {
        virtual_list_controls::virtual_list_element(cx, id, len, build_focus, options, key_at, row)
    });
    ui.add(element);
    response
}

pub(super) fn scroll<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    scroll_with_options(ui, build_focus, ScrollOptions::default(), f);
}

pub(super) fn scroll_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: ScrollOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| scroll_container_element(cx, build_focus, options, f));
    ui.add(element);
}

pub(super) fn child_region<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> ChildRegionResponse
where
    W: UiWriter<H> + ?Sized,
{
    child_region_with_options(ui, build_focus, id, ChildRegionOptions::default(), f)
}

pub(super) fn child_region_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    options: ChildRegionOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> ChildRegionResponse
where
    W: UiWriter<H> + ?Sized,
{
    let (element, response) =
        ui.with_cx_mut(|cx| child_region::child_region_element(cx, id, build_focus, options, f));
    ui.add(element);
    response
}
