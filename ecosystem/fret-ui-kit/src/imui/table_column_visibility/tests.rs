use std::sync::Arc;

use fret_core::Px;

use super::menu::{menu_column_id, menu_test_id_suffix, visible_menu_label};
use super::{
    ImUiTableColumnVisibilityState, TableColumnVisibilityEntry, TableColumnVisibilitySnapshot,
};
use crate::imui::{TableColumn, TableColumnWidth};

mod menu;
mod state;
