//! Editor composites (higher-level compositions such as inspector, tree, table).

pub mod gradient_editor;
pub mod inspector_panel;
pub mod property_grid;
pub mod property_grid_virtualized;
pub mod property_group;
pub mod property_row;

pub use gradient_editor::{
    GradientEditor, GradientEditorOptions, GradientStopBinding, OnGradientAction,
    OnGradientStopAction,
};
pub use inspector_panel::{InspectorPanel, InspectorPanelCx, InspectorPanelOptions};
pub use property_grid::{PropertyGrid, PropertyGridOptions, PropertyGridRow, PropertyGridRowCx};
pub use property_grid_virtualized::{PropertyGridVirtualized, PropertyGridVirtualizedOptions};
pub use property_group::{OnPropertyGroupToggle, PropertyGroup, PropertyGroupOptions};
pub use property_row::{
    OnPropertyRowReset, PropertyRow, PropertyRowLayoutVariant, PropertyRowOptions,
    PropertyRowReset, PropertyRowResetOptions,
};
