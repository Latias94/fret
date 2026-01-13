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
    SceneOp, Size, TextConstraints, TextMetrics, TextService, Transform2D,
};
use fret_runtime::{CommandId, Effect};

#[derive(Default)]
struct FakeTextService {}

impl TextService for FakeTextService {
    fn prepare(
        &mut self,
        _input: fret_core::TextInput<'_>,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for FakeTextService {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl fret_core::SvgService for FakeTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
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
mod core;
mod interactions;
mod layout;
mod semantics;
mod virtual_list;
