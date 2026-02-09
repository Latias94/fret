use super::*;
use fret_core::{
    Color, Corners, DrawOrder, Edges, Px, Scene, SceneOp, TextConstraints, TextMetrics,
    TextService, TextStyle, TextWrap,
};
use fret_runtime::{BindingV1, KeySpecV1, Keymap, KeymapFileV1, KeymapService, Model};
use slotmap::KeyData;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

#[derive(Default)]
struct TestStack;

impl<H: UiHost> Widget<H> for TestStack {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}

#[derive(Default)]
struct FakeUiServices;

impl TextService for FakeUiServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
                baseline: fret_core::Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for FakeUiServices {
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

impl fret_core::SvgService for FakeUiServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

struct ObservingWidget {
    model: Model<u32>,
}

struct PaintObservingWidget {
    model: Model<u32>,
}

impl<H: UiHost> Widget<H> for PaintObservingWidget {
    fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
        Size::new(fret_core::Px(10.0), fret_core::Px(10.0))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.model, Invalidation::Paint);
    }
}

struct HitTestObservingWidget {
    model: Model<u32>,
}

impl<H: UiHost> Widget<H> for HitTestObservingWidget {
    fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
        Size::new(fret_core::Px(10.0), fret_core::Px(10.0))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.model, Invalidation::HitTest);
    }
}

impl<H: UiHost> Widget<H> for ObservingWidget {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.model, Invalidation::Layout);
        let _ = cx.services.text().prepare_str(
            "x",
            &TextStyle {
                font: fret_core::FontId::default(),
                size: fret_core::Px(12.0),
                ..Default::default()
            },
            TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            },
        );
        Size::new(fret_core::Px(10.0), fret_core::Px(10.0))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.model, Invalidation::Paint);
        let _ = cx.scene;
    }
}

struct RoundedClipWidget;

impl<H: UiHost> Widget<H> for RoundedClipWidget {
    fn clips_hit_test(&self, _bounds: Rect) -> bool {
        true
    }

    fn clip_hit_test_corner_radii(&self, _bounds: Rect) -> Option<Corners> {
        Some(Corners::all(Px(20.0)))
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

struct CountingPaintWidget {
    paints: Arc<AtomicUsize>,
}

impl<H: UiHost> Widget<H> for CountingPaintWidget {
    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.paints.fetch_add(1, Ordering::SeqCst);
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: Color::TRANSPARENT,
            border: Edges::default(),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::default(),
        });
    }
}

struct ClickCounter {
    clicks: Model<u32>,
}

impl<H: UiHost> Widget<H> for ClickCounter {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if matches!(
            event,
            Event::Pointer(fret_core::PointerEvent::Up {
                button: fret_core::MouseButton::Left,
                ..
            })
        ) {
            let _ = cx
                .app
                .models_mut()
                .update(&self.clicks, |v: &mut u32| *v += 1);
            cx.stop_propagation();
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

mod alt_menu_bar_activation;
mod children;
mod command_availability;
mod command_enabled_service;
mod cursor_icon_query;
mod dispatch_phase;
mod dock_drag;
mod escape_dismiss;
mod focus_scope;
mod focus_traversal_availability;
mod focus_traversal_prepaint_cache;
mod gc_liveness;
mod globals;
mod hit_test;
mod hit_test_cache_reuse_policy;
mod interactivity_gate;
mod measure_in;
mod models;
mod outside_press;
mod paint_cache;
mod platform_text_input;
mod pointer_move_hover;
mod pointer_move_layers;
mod pointer_occlusion;
mod prepaint;
mod prevent_default;
mod scroll_into_view;
mod scroll_invalidation;
mod semantics_focus_shortcuts;
mod stack_safety;
mod transforms;
mod view_cache;
mod window_command_action_availability_snapshot;
mod window_input_arbitration_snapshot;
mod window_input_context_snapshot;
mod window_text_input_snapshot;
