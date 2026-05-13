use super::paint;
use super::*;
use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, MaterialRegistrationError, MaterialService, Modifiers,
    PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, Point, Px, Rect,
    Size, SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService,
};
use fret_runtime::TickId;
use fret_ui::tree::UiTree;
use fret_ui_kit::declarative::windowed_rows_surface::{
    WindowedRowsSurfaceDiagnosticsStore, WindowedRowsSurfaceWindowTelemetry,
};
use std::sync::Arc;

mod accessibility;
mod caret_navigation;
mod display_navigation;
mod edit_refresh;
mod feature_payloads;
mod fold_lifecycle;
mod geometry;
mod keyboard_commands;
mod paint_guards;
mod platform_text_input;
mod platform_text_input_roundtrip;
mod pointer_helpers;
mod pointer_selection;
mod preedit_paint;
mod row_geom_cache;
mod row_text_cache;
mod scroll_window;
mod state_lifecycle;
mod support;
#[cfg(feature = "syntax-rust")]
mod syntax;
#[cfg(feature = "syntax")]
mod syntax_window;
mod word_navigation;

use support::*;

#[derive(Default)]
struct TestHost {
    models: fret_runtime::ModelStore,
    next_timer: u64,
    next_clipboard: u64,
    next_share_sheet: u64,
}

impl fret_ui::action::UiActionHost for TestHost {
    fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
        &mut self.models
    }

    fn push_effect(&mut self, _effect: fret_runtime::Effect) {}

    fn request_redraw(&mut self, _window: fret_core::AppWindowId) {}

    fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
        self.next_timer = self.next_timer.saturating_add(1);
        fret_runtime::TimerToken(self.next_timer)
    }

    fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
        self.next_clipboard = self.next_clipboard.saturating_add(1);
        fret_runtime::ClipboardToken(self.next_clipboard)
    }

    fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
        self.next_share_sheet = self.next_share_sheet.saturating_add(1);
        fret_runtime::ShareSheetToken(self.next_share_sheet)
    }
}

impl fret_ui::action::UiFocusActionHost for TestHost {
    fn request_focus(&mut self, _target: fret_ui::GlobalElementId) {}
}

#[derive(Default)]
struct FakeServices;

impl TextService for FakeServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(10.0), Px(16.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl PathService for FakeServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
}

impl SvgService for FakeServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

impl MaterialService for FakeServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, MaterialRegistrationError> {
        Ok(fret_core::MaterialId::default())
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
}
