#![allow(
    clippy::arc_with_non_send_sync,
    clippy::collapsible_if,
    clippy::default_constructed_unit_structs,
    clippy::field_reassign_with_default,
    clippy::if_same_then_else,
    clippy::io_other_error,
    clippy::iter_overeager_cloned,
    clippy::let_and_return,
    clippy::let_unit_value,
    clippy::manual_is_multiple_of,
    clippy::redundant_closure,
    clippy::redundant_locals,
    clippy::reserve_after_initialization,
    clippy::too_many_arguments,
    clippy::unnecessary_cast,
    clippy::unnecessary_lazy_evaluations,
    clippy::useless_format
)]

use fret_app::{App, CommandId, Model};
use fret_code_editor as code_editor;
use fret_code_editor_view as code_editor_view;
use fret_code_view as code_view;
use fret_core::{
    AttributedText, CaretAffinity, Color as CoreColor, Corners, DrawOrder, Edges, FontId,
    FontWeight, ImageId, Point, Px, Rect, SceneOp, Size, TextConstraints, TextOverflow, TextSpan,
    TextStyle, TextWrap,
};
use fret_kit::prelude::ModelWatchExt as _;
use fret_markdown as markdown;
use fret_ui::Theme;
use fret_ui::element::{CanvasProps, SemanticsDecoration, StackProps};
use fret_ui::elements::ContinuousFrames;
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui_ai as ui_ai;
use fret_ui_assets as ui_assets;
use fret_ui_kit::declarative::CachedSubtreeExt as _;
pub(super) use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::ui;
use fret_ui_material3 as material3;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, OnceLock};
use time::Date;

use crate::driver::UiGalleryImageSourceDemoAssets;
use crate::spec::*;

mod content;
mod models;
mod nav;
mod pages;
mod previews;

pub(crate) use content::content_view;
pub(crate) use models::UiGalleryModels;
pub(crate) use nav::sidebar_view;
use previews::gallery::*;
use previews::magic::*;
use previews::material3::*;
use previews::pages::*;
