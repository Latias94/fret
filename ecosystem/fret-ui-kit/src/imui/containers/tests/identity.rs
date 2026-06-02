use std::sync::Arc;

use fret_app::App;
use fret_core::AppWindowId;
use fret_ui::element::ElementKind;

use crate::imui::{
    GridOptions, HorizontalOptions, ScrollOptions, UiWriterImUiFacadeExt as _, VerticalOptions,
};

use super::super::{
    grid_container_element, horizontal_container_element, scroll_container_element,
    vertical_container_element,
};
use super::bounds;

mod outer;
mod viewport;
