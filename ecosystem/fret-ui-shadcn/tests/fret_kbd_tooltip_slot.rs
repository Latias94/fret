use fret_app::App;
use fret_core::{
    AppWindowId, Color, Paint, Point, Px, Rect, Scene, SceneOp, Size as CoreSize, TextBlobId,
    TextConstraints, TextInput, TextMetrics,
};
use fret_ui::Theme;
use fret_ui::tree::UiTree;
use fret_ui_shadcn::prelude::UiElementTestIdExt;
use fret_ui_shadcn::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york};

fn assert_color_close(label: &str, actual: Color, expected: Color, tol: f32) {
    let dr = (actual.r - expected.r).abs();
    let dg = (actual.g - expected.g).abs();
    let db = (actual.b - expected.b).abs();
    let da = (actual.a - expected.a).abs();
    assert!(
        dr <= tol && dg <= tol && db <= tol && da <= tol,
        "{label}: expected≈({:.3},{:.3},{:.3},{:.3}) got=({:.3},{:.3},{:.3},{:.3}) tol={tol}",
        expected.r,
        expected.g,
        expected.b,
        expected.a,
        actual.r,
        actual.g,
        actual.b,
        actual.a
    );
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn find_best_solid_quad(scene: &Scene, target: Rect) -> Option<(Rect, Color)> {
    let mut best: Option<(Rect, Color)> = None;
    let mut best_score = f32::INFINITY;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect, background, ..
        } = *op
        else {
            continue;
        };
        let Paint::Solid(color) = background.paint else {
            continue;
        };
        if color.a <= 0.0 {
            continue;
        }

        let score = (rect.origin.x.0 - target.origin.x.0).abs()
            + (rect.origin.y.0 - target.origin.y.0).abs()
            + (rect.size.width.0 - target.size.width.0).abs()
            + (rect.size.height.0 - target.size.height.0).abs();

        if score < best_score {
            best_score = score;
            best = Some((rect, color));
        }
    }

    best
}

#[derive(Default)]
struct RecordingServices {
    last_text: Option<String>,
    last_style_px: Option<f32>,
    last_style_line_height_px: Option<f32>,
}

impl fret_core::TextService for RecordingServices {
    fn prepare(
        &mut self,
        input: &TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        match input {
            TextInput::Plain { text, style } => {
                self.last_text = Some(text.to_string());
                self.last_style_px = Some(style.size.0);
                self.last_style_line_height_px = style.line_height.map(|v| v.0);
            }
            TextInput::Attributed { text, base, .. } => {
                self.last_text = Some(text.to_string());
                self.last_style_px = Some(base.size.0);
                self.last_style_line_height_px = base.line_height.map(|v| v.0);
            }
            _ => {}
        }

        (
            TextBlobId::default(),
            TextMetrics {
                size: CoreSize::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl fret_core::PathService for RecordingServices {
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

impl fret_core::SvgService for RecordingServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        true
    }
}

impl fret_core::MaterialService for RecordingServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Ok(fret_core::MaterialId::default())
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
}

#[test]
fn fret_kbd_in_tooltip_content_overrides_bg_and_fg() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(240.0), Px(120.0)),
    );
    let window = AppWindowId::default();

    let mut app = App::new();
    apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
    let theme = Theme::global(&app).snapshot();
    let expected_text_fg = theme.color_token("background");
    let expected_kbd_bg = alpha_mul(expected_text_fg, 0.20);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = RecordingServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "fret-kbd-tooltip-slot",
        |cx| {
            vec![
                fret_ui_shadcn::TooltipContent::build(cx, |_cx| {
                    [fret_ui_shadcn::Kbd::new("S").test_id("kbd-tooltip-key")]
                })
                .into_element(cx),
            ]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let kbd_node = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("kbd-tooltip-key"))
        .expect("expected semantics node for kbd");

    let (_rect, actual_kbd_bg) =
        find_best_solid_quad(&scene, kbd_node.bounds).expect("expected quad for kbd background");
    assert_color_close("kbd.tooltip.bg", actual_kbd_bg, expected_kbd_bg, 0.02);

    let actual_text_fg = scene
        .ops()
        .iter()
        .find_map(|op| match *op {
            SceneOp::Text { paint, .. } => match paint.paint {
                Paint::Solid(c) => Some(c),
                _ => None,
            },
            _ => None,
        })
        .expect("expected painted text op");
    assert_color_close("kbd.tooltip.fg", actual_text_fg, expected_text_fg, 0.02);

    assert_eq!(services.last_text.as_deref(), Some("S"));
    assert_eq!(services.last_style_px, Some(12.0));
    assert_eq!(services.last_style_line_height_px, Some(16.0));
}
