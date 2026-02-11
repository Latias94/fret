#![allow(clippy::arc_with_non_send_sync)]

use super::*;
use fret_runtime::GlobalsHost as _;
use std::sync::Arc;

fn attributed_plain(text: &str) -> fret_core::AttributedText {
    fret_core::AttributedText::new(
        Arc::<str>::from(text),
        [fret_core::TextSpan {
            len: text.len(),
            ..Default::default()
        }],
    )
}

mod dismissible;
mod pointer_regions;
mod pressable;
mod resizable_panel_group;
mod roving_flex;
mod selectable_text;
mod text_area;
mod text_input;
