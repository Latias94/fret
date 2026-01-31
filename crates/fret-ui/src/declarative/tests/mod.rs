use std::sync::Arc;

use super::render_root;
use crate::UiHost;
use crate::action::{ActivateReason, DismissReason};
use crate::element::{AnyElement, CrossAlign, Length, MainAlign, TextInputProps};
use crate::elements::{ContinuousFrames, ElementContext};
use crate::test_host::TestHost;
use crate::tree::UiTree;
use crate::widget::Invalidation;
use crate::widget::{LayoutCx, PaintCx, Widget};
use fret_core::{
    AppWindowId, Color, Modifiers, MouseButton, MouseButtons, NodeId, Point, Px, Rect, Scene,
    SceneOp, Size, TextConstraints, TextMetrics, TextService, TextStyle, Transform2D,
};
use fret_runtime::{CommandId, Effect};

#[derive(Default)]
struct FakeTextService {
    prepare_calls: usize,
    release_calls: usize,
    path_prepare_calls: usize,
    path_release_calls: usize,
    svg_register_calls: usize,
    svg_unregister_calls: usize,
}

impl TextService for FakeTextService {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        self.prepare_calls += 1;
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {
        self.release_calls += 1;
    }

    fn selection_rects_clipped(
        &mut self,
        _blob: fret_core::TextBlobId,
        range: (usize, usize),
        clip: Rect,
        out: &mut Vec<Rect>,
    ) {
        let (start, end) = range;
        if start >= end {
            return;
        }

        let width = Px((end.saturating_sub(start)) as f32);
        let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(width, Px(10.0)));

        let ix0 = rect.origin.x.0.max(clip.origin.x.0);
        let iy0 = rect.origin.y.0.max(clip.origin.y.0);
        let ix1 = (rect.origin.x.0 + rect.size.width.0).min(clip.origin.x.0 + clip.size.width.0);
        let iy1 = (rect.origin.y.0 + rect.size.height.0).min(clip.origin.y.0 + clip.size.height.0);

        if ix1 <= ix0 || iy1 <= iy0 {
            return;
        }

        out.push(Rect::new(
            Point::new(Px(ix0), Px(iy0)),
            Size::new(Px(ix1 - ix0), Px(iy1 - iy0)),
        ));
    }
}

impl fret_core::PathService for FakeTextService {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        self.path_prepare_calls += 1;
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {
        self.path_release_calls += 1;
    }
}

impl fret_core::SvgService for FakeTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        self.svg_register_calls += 1;
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        self.svg_unregister_calls += 1;
        true
    }
}

#[derive(Default)]
struct FillStack;

impl<H: UiHost> Widget<H> for FillStack {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout(child, cx.available);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            cx.paint(child, cx.bounds);
        }
    }
}

#[track_caller]
fn build_keyed_rows(
    cx: &mut ElementContext<'_, TestHost>,
    items: &[u64],
    ids: &mut Vec<(u64, crate::elements::GlobalElementId)>,
) -> Vec<crate::element::AnyElement> {
    let mut out = Vec::new();
    for &item in items {
        let el = cx.keyed(item, |cx| cx.text("row"));
        ids.push((item, el.id));
        out.push(el);
    }
    out
}

mod anchored;
mod canvas;
mod command_hooks;
mod core;
mod element_state_gc;
mod identity;
mod interactions;
mod layout;
mod selection_indices;
mod semantics;
mod text_cache;
mod view_cache;
mod virtual_list;
