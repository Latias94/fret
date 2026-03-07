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
#[cfg(feature = "gallery-dev")]
use fret_code_editor as code_editor;
#[cfg(feature = "gallery-dev")]
use fret_code_editor_view as code_editor_view;
#[cfg(feature = "gallery-dev")]
use fret_code_view as code_view;
use fret_core::{
    AttributedText, CaretAffinity, Color as CoreColor, Corners, DrawOrder, Edges, FontId, ImageId,
    Point, Px, Rect, SceneOp, Size, TextConstraints, TextOverflow, TextSpan, TextStyle, TextWrap,
};
#[cfg(feature = "gallery-dev")]
use fret_markdown as markdown;
use fret_ui::Theme;
use fret_ui::element::{CanvasProps, SemanticsDecoration};
use fret_ui::elements::ContinuousFrames;
use fret_ui::scroll::VirtualListScrollHandle;
#[cfg(feature = "gallery-dev")]
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::CachedSubtreeExt as _;
pub(super) use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::ui;
#[cfg(feature = "gallery-material3")]
use fret_ui_material3 as material3;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, OnceLock};
use time::Date;

use crate::spec::*;

mod content;
mod doc_layout;
mod models;
mod nav;
mod pages;
mod previews;
pub(in crate::ui) mod snippets;

pub(crate) use content::content_view;
pub(crate) use models::UiGalleryModels;
pub(crate) use nav::{nav_visibility_summary, sidebar_view};
pub(crate) use pages::card_doc_scaffold_metrics_json;
use pages::preview_motion_presets;
#[cfg(feature = "gallery-dev")]
use previews::gallery::*;
#[cfg(feature = "gallery-dev")]
use previews::magic::*;
use previews::pages::*;
