use fret_core::{
    Color, Corners, DrawOrder, Edges, Paint, Point, Px, Rect, SceneOp, Size, Transform2D,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ClipKind {
    Rect,
    RRect,
}

#[derive(Debug, Clone, Copy)]
struct ClipEntry {
    kind: ClipKind,
    pushed_transform: Transform2D,
}

#[derive(Debug, Clone)]
struct Interpreter {
    transform_stack: Vec<Transform2D>,
    clip_stack: Vec<ClipEntry>,
    opacity_stack: Vec<f32>,
}

impl Interpreter {
    fn new() -> Self {
        Self {
            transform_stack: vec![Transform2D::IDENTITY],
            clip_stack: Vec::new(),
            opacity_stack: vec![1.0],
        }
    }

    fn current_transform(&self) -> Transform2D {
        *self
            .transform_stack
            .last()
            .expect("transform stack must be non-empty")
    }

    fn current_opacity(&self) -> f32 {
        *self
            .opacity_stack
            .last()
            .expect("opacity stack must be non-empty")
    }

    fn apply(&mut self, op: SceneOp) {
        match op {
            SceneOp::PushTransform { transform } => {
                let current = self.current_transform();
                self.transform_stack.push(current * transform);
            }
            SceneOp::PopTransform => {
                if self.transform_stack.len() > 1 {
                    self.transform_stack.pop();
                }
            }
            SceneOp::PushOpacity { opacity } => {
                let current = self.current_opacity();
                self.opacity_stack.push(current * opacity);
            }
            SceneOp::PopOpacity => {
                if self.opacity_stack.len() > 1 {
                    self.opacity_stack.pop();
                }
            }
            SceneOp::PushClipRect { .. } => {
                self.clip_stack.push(ClipEntry {
                    kind: ClipKind::Rect,
                    pushed_transform: self.current_transform(),
                });
            }
            SceneOp::PushClipRRect { .. } => {
                self.clip_stack.push(ClipEntry {
                    kind: ClipKind::RRect,
                    pushed_transform: self.current_transform(),
                });
            }
            SceneOp::PopClip => {
                self.clip_stack.pop();
            }
            SceneOp::PushLayer { .. }
            | SceneOp::PopLayer
            | SceneOp::PushMask { .. }
            | SceneOp::PopMask
            | SceneOp::PushEffect { .. }
            | SceneOp::PopEffect
            | SceneOp::PushCompositeGroup { .. }
            | SceneOp::PopCompositeGroup
            | SceneOp::Quad { .. }
            | SceneOp::Image { .. }
            | SceneOp::ImageRegion { .. }
            | SceneOp::MaskImage { .. }
            | SceneOp::SvgMaskIcon { .. }
            | SceneOp::SvgImage { .. }
            | SceneOp::Text { .. }
            | SceneOp::Path { .. }
            | SceneOp::ViewportSurface { .. } => {}
        }
    }
}

fn quad(rect: Rect) -> SceneOp {
    SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: Paint::Solid(Color::TRANSPARENT),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Corners::all(Px(0.0)),
    }
}

#[test]
fn transform_stack_composes_with_left_multiplication() {
    let t_translate = Transform2D::translation(Point::new(Px(10.0), Px(0.0)));
    let t_scale = Transform2D::scale_uniform(2.0);

    let ops = [
        SceneOp::PushTransform {
            transform: t_translate,
        },
        SceneOp::PushTransform { transform: t_scale },
    ];

    let mut it = Interpreter::new();
    for op in ops {
        it.apply(op);
    }

    // `current * t` means: apply `t` first, then apply `current` (ADR 0078).
    //
    // So translating by +10 after a scale-by-2 yields: (1 * 2) + 10 = 12.
    let p = Point::new(Px(1.0), Px(0.0));
    assert_eq!(
        it.current_transform().apply_point(p),
        Point::new(Px(12.0), Px(0.0))
    );
}

#[test]
fn opacity_stack_is_multiplicative_and_balanced() {
    let ops = [
        SceneOp::PushOpacity { opacity: 0.5 },
        SceneOp::PushOpacity { opacity: 0.5 },
        SceneOp::PopOpacity,
        SceneOp::PopOpacity,
    ];

    let mut it = Interpreter::new();

    it.apply(ops[0]);
    assert_eq!(it.current_opacity(), 0.5);

    it.apply(ops[1]);
    assert_eq!(it.current_opacity(), 0.25);

    it.apply(ops[2]);
    assert_eq!(it.current_opacity(), 0.5);

    it.apply(ops[3]);
    assert_eq!(it.current_opacity(), 1.0);
}

#[test]
fn clip_is_captured_at_push_time_and_not_affected_by_later_transforms() {
    let clip_rect = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    let content_rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));

    let scroll = Transform2D::translation(Point::new(Px(20.0), Px(15.0)));

    let ops = [
        SceneOp::PushClipRect { rect: clip_rect },
        SceneOp::PushTransform { transform: scroll },
        quad(content_rect),
    ];

    let mut it = Interpreter::new();
    for op in ops {
        it.apply(op);
    }

    let top = it.clip_stack.last().expect("clip entry");
    assert_eq!(top.kind, ClipKind::Rect);
    assert_eq!(
        top.pushed_transform,
        Transform2D::IDENTITY,
        "clip pushed before scroll should remain in parent space"
    );
    assert_eq!(
        it.current_transform(),
        scroll,
        "content should be drawn under the scroll transform"
    );
}

#[test]
fn clip_moves_with_content_when_transform_is_pushed_before_clip() {
    let clip_rect = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    let content_rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));

    let t = Transform2D::translation(Point::new(Px(3.0), Px(4.0)));

    let ops = [
        SceneOp::PushTransform { transform: t },
        SceneOp::PushClipRect { rect: clip_rect },
        quad(content_rect),
    ];

    let mut it = Interpreter::new();
    for op in ops {
        it.apply(op);
    }

    let top = it.clip_stack.last().expect("clip entry");
    assert_eq!(top.kind, ClipKind::Rect);
    assert_eq!(top.pushed_transform, t);
}

#[test]
fn nested_clips_capture_transforms_independently() {
    let root_clip = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    let inner_clip = Rect::new(Point::new(Px(5.0), Px(6.0)), Size::new(Px(10.0), Px(11.0)));

    let t = Transform2D::translation(Point::new(Px(7.0), Px(8.0)));

    let ops = [
        SceneOp::PushClipRect { rect: root_clip },
        SceneOp::PushTransform { transform: t },
        SceneOp::PushClipRRect {
            rect: inner_clip,
            corner_radii: Corners::all(Px(2.0)),
        },
        quad(inner_clip),
    ];

    let mut it = Interpreter::new();
    for op in ops {
        it.apply(op);
    }

    assert_eq!(it.clip_stack.len(), 2);
    assert_eq!(it.clip_stack[0].kind, ClipKind::Rect);
    assert_eq!(it.clip_stack[0].pushed_transform, Transform2D::IDENTITY);
    assert_eq!(it.clip_stack[1].kind, ClipKind::RRect);
    assert_eq!(it.clip_stack[1].pushed_transform, t);
}

#[test]
fn layer_markers_do_not_affect_transform_clip_or_opacity() {
    let t = Transform2D::translation(Point::new(Px(3.0), Px(4.0)));

    let ops = [
        SceneOp::PushOpacity { opacity: 0.5 },
        SceneOp::PushLayer { layer: 1 },
        SceneOp::PushTransform { transform: t },
        SceneOp::PushClipRect {
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0))),
        },
        SceneOp::PopClip,
        SceneOp::PopTransform,
        SceneOp::PopLayer,
        SceneOp::PopOpacity,
    ];

    let mut it = Interpreter::new();
    for op in ops {
        it.apply(op);
    }

    assert_eq!(it.clip_stack.len(), 0);
    assert_eq!(it.current_transform(), Transform2D::IDENTITY);
    assert_eq!(it.current_opacity(), 1.0);
}
