//! Immediate-mode option structs and canonical defaults.

mod collections;
mod containers;
mod controls;
mod menus;
mod misc;

pub use collections::{
    TableColumn, TableColumnWidth, TableOptions, TableRowOptions, VirtualListOptions,
};
pub use containers::{
    ChildRegionOptions, GridOptions, HorizontalOptions, ScrollOptions, VerticalOptions,
};
pub use controls::{
    ButtonArrowDirection, ButtonOptions, ButtonVariant, CheckboxOptions, CollapsingHeaderOptions,
    ComboModelOptions, ComboOptions, InputTextMode, InputTextOptions, RadioOptions,
    SelectableOptions, SliderOptions, SwitchOptions, TabItemOptions, TextAreaOptions,
    TreeNodeOptions,
};
pub use menus::{
    BeginMenuOptions, BeginSubmenuOptions, MenuBarOptions, MenuItemOptions, PopupMenuOptions,
    PopupModalOptions, TabBarOptions, TooltipOptions,
};
pub use misc::{BulletTextOptions, DragSourceOptions, DropTargetOptions, SeparatorTextOptions};
