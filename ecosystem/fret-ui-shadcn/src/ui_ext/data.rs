use crate::data_grid::DataGrid;
use crate::data_grid_canvas::DataGridCanvas;
use crate::data_table::DataTable;
use crate::data_table_controls::{DataTableGlobalFilterInput, DataTableViewOptions};
use crate::data_table_recipes::{DataTablePagination, DataTableToolbar};

impl_ui_patch_chrome_layout_patch_only!(DataGrid);
impl_ui_patch_chrome_layout_patch_only!(DataGridCanvas);
impl_ui_patch_chrome_layout_patch_only!(DataTable);

impl_ui_patch_passthrough!(DataTableGlobalFilterInput);
impl_ui_patch_passthrough!(DataTableViewOptions);
impl_ui_patch_passthrough!(DataTablePagination);

impl<TData> ::fret_ui_kit::UiPatchTarget for DataTableToolbar<TData> {
    fn apply_ui_patch(self, _patch: ::fret_ui_kit::UiPatch) -> Self {
        self
    }
}

impl<H: ::fret_ui::UiHost, TData: 'static> ::fret_ui_kit::IntoUiElement<H>
    for DataTableToolbar<TData>
{
    fn into_element(
        self,
        cx: &mut ::fret_ui::ElementContext<'_, H>,
    ) -> ::fret_ui::element::AnyElement {
        DataTableToolbar::<TData>::into_element(self, cx)
    }
}
