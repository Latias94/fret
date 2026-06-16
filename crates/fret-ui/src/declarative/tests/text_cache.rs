use super::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[test]
fn theme_color_change_does_not_reprepare_text_in_paint() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    // Ensure the theme is stored as a global so we can mutate it between frames.
    app.set_global(crate::Theme::global(&app).clone());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-cache",
        |cx| vec![cx.keyed(1u64, |cx| cx.text("hello"))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepares_after_first_paint = services.prepare_calls;

    // Paint-only theme change: should not invalidate the text blob cache path.
    crate::Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = crate::ThemeConfig::default();
        cfg.colors
            .insert("foreground".to_string(), "#ff0000".to_string());
        theme.extend_tokens_from_config(&cfg);
    });

    // Intentionally skip `render_root`/`layout_all` so the only possible text service work is from paint.
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(
        services.prepare_calls, prepares_after_first_paint,
        "paint-only theme changes should not force re-preparing text blobs"
    );
}

#[test]
fn text_color_prop_changes_are_paint_only_in_declarative_diff() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let red = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let green = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    let styled = fret_core::AttributedText::new(
        std::sync::Arc::<str>::from("styled"),
        [fret_core::TextSpan {
            len: "styled".len(),
            ..Default::default()
        }],
    );
    let selectable = fret_core::AttributedText::new(
        std::sync::Arc::<str>::from("selectable"),
        [fret_core::TextSpan {
            len: "selectable".len(),
            ..Default::default()
        }],
    );

    let root = render_colored_text_diff_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        red,
        &styled,
        &selectable,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(
        ui.debug_stats().layout_nodes_performed,
        0,
        "expected the baseline tree to be clean before the color-only rerender"
    );
    let prepares_after_first_paint = services.prepare_calls;

    app.advance_frame();
    let root = render_colored_text_diff_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        green,
        &styled,
        &selectable,
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(
        ui.debug_stats().layout_nodes_performed,
        0,
        "text color changes should invalidate paint without forcing layout"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(
        services.prepare_calls, prepares_after_first_paint,
        "text color changes should not force text blob preparation"
    );
}

#[test]
fn stable_unwrapped_text_content_changes_are_paint_only_in_declarative_diff() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_stable_text_content_diff_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "Query: 2",
        fret_core::TextWrap::None,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let children = ui.children(root);
    assert_eq!(children.len(), 1);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert!(
        ui.semantics_snapshot()
            .expect("initial semantics snapshot")
            .nodes
            .iter()
            .any(|n| n.role == fret_core::SemanticsRole::Text
                && n.label.as_deref() == Some("Query: 2")),
        "baseline text semantics should expose the initial label"
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(
        ui.debug_stats().layout_nodes_performed,
        0,
        "expected the baseline tree to be clean before the text-content rerender"
    );
    let prepares_after_first_paint = services.prepare_calls;

    app.advance_frame();
    let root = render_stable_text_content_diff_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "Query: 249",
        fret_core::TextWrap::None,
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(
        ui.debug_stats().layout_nodes_performed,
        0,
        "single-line clipped text content changes should invalidate paint without forcing layout"
    );
    assert!(
        ui.request_semantics_snapshot_if_dirty(),
        "text content changes should mark semantics dirty even when layout is skipped"
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert!(
        ui.semantics_snapshot()
            .expect("updated semantics snapshot")
            .nodes
            .iter()
            .any(|n| n.role == fret_core::SemanticsRole::Text
                && n.label.as_deref() == Some("Query: 249")),
        "updated text semantics should expose the new label"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(
        services.prepare_calls,
        prepares_after_first_paint + 1,
        "text content changes should prepare new blobs during paint, not through layout"
    );
}

#[test]
fn inherited_fixed_line_height_text_content_changes_are_paint_only_in_declarative_diff() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_inherited_stable_text_content_diff_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "Ready",
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(
        ui.debug_stats().layout_nodes_performed,
        0,
        "expected the baseline tree to be clean before the inherited-style rerender"
    );

    app.advance_frame();
    let root = render_inherited_stable_text_content_diff_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "Running",
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(
        ui.debug_stats().layout_nodes_performed,
        0,
        "fixed single-line text content changes should not force layout when the stable line box comes from inherited typography"
    );
}

#[test]
fn wrapped_text_content_changes_still_invalidate_layout_in_declarative_diff() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_stable_text_content_diff_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "short",
        fret_core::TextWrap::Word,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(
        ui.debug_stats().layout_nodes_performed,
        0,
        "expected the baseline tree to be clean before the wrapped text-content rerender"
    );

    app.advance_frame();
    let root = render_stable_text_content_diff_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a longer status label whose wrapped height may depend on content",
        fret_core::TextWrap::Word,
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert!(
        ui.debug_stats().layout_nodes_performed > 0,
        "wrapped text content changes must keep invalidating layout because height can change"
    );
}

fn render_colored_text_diff_root(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    window: AppWindowId,
    bounds: Rect,
    color: Color,
    styled: &fret_core::AttributedText,
    selectable: &fret_core::AttributedText,
) -> NodeId {
    let styled = styled.clone();
    let selectable = selectable.clone();
    render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "text-color-diff",
        move |cx| {
            let mut plain_props = crate::element::TextProps::new("plain");
            plain_props.color = Some(color);

            let mut styled_props = crate::element::StyledTextProps::new(styled);
            styled_props.color = Some(color);

            let mut selectable_props = crate::element::SelectableTextProps::new(selectable);
            selectable_props.color = Some(color);

            vec![
                cx.text_props(plain_props),
                cx.styled_text_props(styled_props),
                cx.selectable_text_props(selectable_props),
            ]
        },
    )
}

fn render_stable_text_content_diff_root(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    window: AppWindowId,
    bounds: Rect,
    text: &'static str,
    wrap: fret_core::TextWrap,
) -> NodeId {
    render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "text-content-diff",
        move |cx| {
            let mut plain_props = crate::element::TextProps::new(text);
            plain_props.wrap = wrap;
            plain_props.overflow = fret_core::TextOverflow::Clip;
            plain_props.layout.size.width = Length::Fill;
            plain_props.layout.size.height = Length::Px(Px(20.0));

            vec![cx.text_props(plain_props)]
        },
    )
}

fn render_inherited_stable_text_content_diff_root(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    window: AppWindowId,
    bounds: Rect,
    text: &'static str,
) -> NodeId {
    render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "inherited-text-content-diff",
        move |cx| {
            let mut inherited = fret_core::TextStyleRefinement::default();
            inherited.line_height = Some(Px(18.0));
            inherited.line_height_policy = Some(fret_core::TextLineHeightPolicy::FixedFromStyle);

            let mut props = crate::element::TextProps::new(text);
            props.wrap = fret_core::TextWrap::None;
            props.overflow = fret_core::TextOverflow::Ellipsis;
            props.layout.size.width = Length::Fill;
            props.layout.size.min_width = Some(Length::Px(Px(0.0)));

            vec![cx.text_props(props).inherit_text_style(inherited)]
        },
    )
}

#[test]
fn unwrapped_start_clip_text_reuses_prepared_blobs_across_paint_width_changes() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_paint_cache_enabled(false);

    let mut services = FakeTextService::default();
    let nodes = render_text_width_cache_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        fret_core::TextOverflow::Clip,
        fret_core::TextAlign::Start,
    );

    paint_text_width_cache_nodes(&mut ui, &mut app, &mut services, &nodes, Px(100.0));
    assert_eq!(
        services.prepare_calls, 3,
        "expected the first paint to prepare plain, styled, and selectable text"
    );

    services.prepare_calls = 0;
    paint_text_width_cache_nodes(&mut ui, &mut app, &mut services, &nodes, Px(180.0));
    assert_eq!(
        services.prepare_calls, 0,
        "unwrapped start-aligned clipped text blobs are width-insensitive in paint"
    );
}

#[test]
fn unwrapped_ellipsis_text_reprepares_across_paint_width_changes() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_paint_cache_enabled(false);

    let mut services = FakeTextService::default();
    let nodes = render_text_width_cache_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        fret_core::TextOverflow::Ellipsis,
        fret_core::TextAlign::Start,
    );

    paint_text_width_cache_nodes(&mut ui, &mut app, &mut services, &nodes, Px(100.0));
    services.prepare_calls = 0;
    paint_text_width_cache_nodes(&mut ui, &mut app, &mut services, &nodes, Px(180.0));
    assert_eq!(
        services.prepare_calls, 3,
        "ellipsis shaping is width-sensitive and must be prepared for the new paint width"
    );
}

#[test]
fn unwrapped_center_aligned_text_reprepares_across_paint_width_changes() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_paint_cache_enabled(false);

    let mut services = FakeTextService::default();
    let nodes = render_text_width_cache_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        fret_core::TextOverflow::Clip,
        fret_core::TextAlign::Center,
    );

    paint_text_width_cache_nodes(&mut ui, &mut app, &mut services, &nodes, Px(100.0));
    services.prepare_calls = 0;
    paint_text_width_cache_nodes(&mut ui, &mut app, &mut services, &nodes, Px(180.0));
    assert_eq!(
        services.prepare_calls, 3,
        "non-start alignment is width-sensitive and must be prepared for the new paint width"
    );
}

fn render_text_width_cache_root(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    window: AppWindowId,
    overflow: fret_core::TextOverflow,
    align: fret_core::TextAlign,
) -> [NodeId; 3] {
    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let styled = fret_core::AttributedText::new(
        std::sync::Arc::<str>::from("styled"),
        [fret_core::TextSpan {
            len: "styled".len(),
            ..Default::default()
        }],
    );
    let selectable = fret_core::AttributedText::new(
        std::sync::Arc::<str>::from("selectable"),
        [fret_core::TextSpan {
            len: "selectable".len(),
            ..Default::default()
        }],
    );

    let root = render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "text-width-cache",
        move |cx| {
            let mut plain_props = crate::element::TextProps::new("plain");
            plain_props.wrap = fret_core::TextWrap::None;
            plain_props.overflow = overflow;
            plain_props.align = align;

            let mut styled_props = crate::element::StyledTextProps::new(styled);
            styled_props.wrap = fret_core::TextWrap::None;
            styled_props.overflow = overflow;
            styled_props.align = align;

            let mut selectable_props = crate::element::SelectableTextProps::new(selectable);
            selectable_props.wrap = fret_core::TextWrap::None;
            selectable_props.overflow = overflow;
            selectable_props.align = align;

            vec![
                cx.text_props(plain_props),
                cx.styled_text_props(styled_props),
                cx.selectable_text_props(selectable_props),
            ]
        },
    );
    ui.set_root(root);

    let children = ui.children(root);
    assert_eq!(children.len(), 3);
    [children[0], children[1], children[2]]
}

fn paint_text_width_cache_nodes(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    nodes: &[NodeId; 3],
    width: Px,
) {
    let mut scene = Scene::default();
    for &node in nodes {
        ui.paint(
            app,
            services,
            node,
            Rect::new(
                fret_core::Point::new(Px(0.0), Px(0.0)),
                Size::new(width, Px(24.0)),
            ),
            &mut scene,
            1.0,
        );
    }
}

fn fingerprint_text_style(style: &TextStyle, h: &mut impl Hasher) {
    style.font.hash(h);
    style.size.0.to_bits().hash(h);
    style.weight.hash(h);
    style.slant.hash(h);
    style.line_height.map(|v| v.0.to_bits()).hash(h);
    style.letter_spacing_em.map(f32::to_bits).hash(h);
}

fn fingerprint_shaping_style(style: &fret_core::TextShapingStyle, h: &mut impl Hasher) {
    style.font.hash(h);
    style.weight.hash(h);
    style.slant.hash(h);
    style.letter_spacing_em.map(f32::to_bits).hash(h);
}

fn fingerprint_text_input(input: &fret_core::TextInput) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    input.text().hash(&mut h);
    match input {
        fret_core::TextInput::Plain { style, .. } => {
            0u8.hash(&mut h);
            fingerprint_text_style(style, &mut h);
        }
        fret_core::TextInput::Attributed { base, spans, .. } => {
            1u8.hash(&mut h);
            fingerprint_text_style(base, &mut h);
            for span in spans.iter() {
                span.len.hash(&mut h);
                fingerprint_shaping_style(&span.shaping, &mut h);
            }
        }
        _ => {
            2u8.hash(&mut h);
        }
    }
    h.finish()
}

#[derive(Default)]
struct FingerprintingServices {
    calls: Vec<(String, u64)>,
}

impl TextService for FingerprintingServices {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        self.calls
            .push((input.text().to_string(), fingerprint_text_input(input)));
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

impl fret_core::PathService for FingerprintingServices {
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

impl fret_core::SvgService for FingerprintingServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

impl fret_core::MaterialService for FingerprintingServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        false
    }
}

#[derive(Default)]
struct WidthSensitiveTextService {
    prepare_calls: usize,
}

impl TextService for WidthSensitiveTextService {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        self.prepare_calls += 1;
        let width = constraints.max_width.map(|w| w.0).unwrap_or(200.0);
        let height = if width < 100.0 { 40.0 } else { 10.0 };
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(width), Px(height)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for WidthSensitiveTextService {
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

impl fret_core::SvgService for WidthSensitiveTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

impl fret_core::MaterialService for WidthSensitiveTextService {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        false
    }
}

#[derive(Default)]
struct UnderestimatedMeasureTextService {
    measure_calls: usize,
    prepare_calls: usize,
}

impl TextService for UnderestimatedMeasureTextService {
    fn measure(
        &mut self,
        _input: &fret_core::TextInput,
        constraints: TextConstraints,
    ) -> TextMetrics {
        self.measure_calls += 1;
        let width = constraints.max_width.map(|w| w.0).unwrap_or(160.0);
        TextMetrics {
            size: Size::new(Px(width), Px(10.0)),
            baseline: Px(8.0),
        }
    }

    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        self.prepare_calls += 1;
        let width = constraints.max_width.map(|w| w.0).unwrap_or(160.0);
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(width), Px(40.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for UnderestimatedMeasureTextService {
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

impl fret_core::SvgService for UnderestimatedMeasureTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

impl fret_core::MaterialService for UnderestimatedMeasureTextService {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        false
    }
}

#[test]
fn wrapped_text_paint_width_shrink_reinvalidates_layout_when_height_grows() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_paint_cache_enabled(false);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut services = WidthSensitiveTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-paint-width-layout-repair",
        |cx| {
            let mut props = crate::element::TextProps::new(
                "A long text run whose wrapped height depends on available width",
            );
            props.layout.size.width = Length::Fill;
            props.wrap = fret_core::TextWrap::Word;
            vec![cx.text_props(props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let text_node = ui.children(root)[0];
    ui.test_clear_node_invalidations(text_node);

    let mut scene = Scene::default();
    ui.paint(
        &mut app,
        &mut services,
        text_node,
        Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(80.0), Px(10.0)),
        ),
        &mut scene,
        1.0,
    );

    assert!(
        ui.node_layout_invalidated(text_node),
        "paint-time text reprepare that increases auto-height must schedule a layout repair"
    );
    assert!(
        app.flush_effects()
            .into_iter()
            .any(|effect| matches!(effect, Effect::Redraw(w) if w == window)),
        "layout repair should request another frame"
    );

    let ops = scene.ops();
    assert!(
        matches!(
            ops,
            [
                fret_core::SceneOp::PushClipRRect { rect, .. },
                fret_core::SceneOp::Text { .. },
                fret_core::SceneOp::PopClip,
            ] if *rect
                == Rect::new(
                    fret_core::Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(80.0), Px(10.0)),
                )
        ),
        "the repair frame should clip the taller paint-prepared text to the stale layout bounds"
    );
}

#[test]
fn wrapped_text_measure_uses_prepare_metrics_for_startup_layout() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_paint_cache_enabled(false);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(160.0), Px(80.0)),
    );
    let mut services = UnderestimatedMeasureTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-startup-prepared-measure",
        |cx| {
            let mut props = crate::element::TextProps::new(
                "A long text run whose backend measure path underestimates wrapped height",
            );
            props.layout.size.width = Length::Fill;
            props.wrap = fret_core::TextWrap::Word;
            vec![cx.text_props(props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let text_node = ui.children(root)[0];
    let text_bounds = ui.debug_node_bounds(text_node).expect("text bounds");

    assert_eq!(
        services.measure_calls, 0,
        "wrapped text should not trust a separate measure path for startup layout"
    );
    assert_eq!(services.prepare_calls, 1);
    assert_eq!(
        text_bounds.size.height,
        Px(40.0),
        "startup layout should reserve the prepared text height"
    );

    ui.test_clear_node_invalidations(text_node);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        !ui.node_layout_invalidated(text_node),
        "first paint should reuse prepared metrics instead of scheduling a repair frame"
    );
    assert!(
        app.flush_effects()
            .into_iter()
            .all(|effect| !matches!(effect, Effect::Redraw(w) if w == window)),
        "first paint should not request an extra redraw for text height repair"
    );
}

#[test]
fn wrapped_text_cached_prepared_metrics_reinvalidate_when_bounds_height_shrinks() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_paint_cache_enabled(false);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(160.0), Px(80.0)),
    );
    let mut services = UnderestimatedMeasureTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-cached-prepared-layout-repair",
        |cx| {
            let mut props = crate::element::TextProps::new(
                "A long text run whose cached prepared height exceeds stale paint bounds",
            );
            props.layout.size.width = Length::Fill;
            props.wrap = fret_core::TextWrap::Word;
            vec![cx.text_props(props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let text_node = ui.children(root)[0];
    ui.test_clear_node_invalidations(text_node);

    let mut scene = Scene::default();
    ui.paint(
        &mut app,
        &mut services,
        text_node,
        Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(160.0), Px(10.0)),
        ),
        &mut scene,
        1.0,
    );

    assert!(
        ui.node_layout_invalidated(text_node),
        "cached prepared metrics that exceed auto-height bounds must schedule a layout repair"
    );
    assert!(
        app.flush_effects()
            .into_iter()
            .any(|effect| matches!(effect, Effect::Redraw(w) if w == window)),
        "cached prepared metrics layout repair should request another frame"
    );

    let ops = scene.ops();
    assert!(
        matches!(
            ops,
            [
                fret_core::SceneOp::PushClipRRect { rect, .. },
                fret_core::SceneOp::Text { .. },
                fret_core::SceneOp::PopClip,
            ] if *rect
                == Rect::new(
                    fret_core::Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(160.0), Px(10.0)),
                )
        ),
        "cached prepared text should be clipped to stale bounds while the repair frame is pending"
    );
}

#[test]
fn wrapped_text_first_paint_reinvalidates_layout_when_height_grows() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_paint_cache_enabled(false);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut services = WidthSensitiveTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-first-paint-layout-repair",
        |cx| {
            let mut props = crate::element::TextProps::new(
                "A long text run whose first prepared height exceeds stale startup bounds",
            );
            props.layout.size.width = Length::Fill;
            props.wrap = fret_core::TextWrap::Word;
            vec![cx.text_props(props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let text_node = ui.children(root)[0];
    ui.test_clear_node_invalidations(text_node);

    let mut scene = Scene::default();
    ui.paint(
        &mut app,
        &mut services,
        text_node,
        Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(80.0), Px(10.0)),
        ),
        &mut scene,
        1.0,
    );

    assert!(
        ui.node_layout_invalidated(text_node),
        "first paint prepare that increases auto-height must schedule a layout repair"
    );
    assert!(
        app.flush_effects()
            .into_iter()
            .any(|effect| matches!(effect, Effect::Redraw(w) if w == window)),
        "first paint layout repair should request another frame"
    );

    let ops = scene.ops();
    assert!(
        matches!(
            ops,
            [
                fret_core::SceneOp::PushClipRRect { rect, .. },
                fret_core::SceneOp::Text { .. },
                fret_core::SceneOp::PopClip,
            ] if *rect
                == Rect::new(
                    fret_core::Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(80.0), Px(10.0)),
                )
        ),
        "the startup repair frame should clip taller prepared text to stale layout bounds"
    );
}

#[test]
fn theme_color_change_does_not_change_text_input_fingerprints() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut services = FingerprintingServices::default();

    app.set_global(crate::Theme::global(&app).clone());

    let rich_a = fret_core::AttributedText::new(
        std::sync::Arc::<str>::from("styled"),
        [fret_core::TextSpan {
            len: "styled".len(),
            ..Default::default()
        }],
    );
    let rich_b = fret_core::AttributedText::new(
        std::sync::Arc::<str>::from("selectable"),
        [fret_core::TextSpan {
            len: "selectable".len(),
            ..Default::default()
        }],
    );

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-fingerprint",
        move |cx| {
            vec![
                cx.text("plain"),
                cx.styled_text(rich_a.clone()),
                cx.selectable_text(rich_b.clone()),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let mut baseline: HashMap<String, u64> = HashMap::new();
    for (text, fp) in services.calls.drain(..) {
        if let Some(existing) = baseline.get(&text) {
            assert_eq!(
                *existing, fp,
                "expected measure/paint to use the same text input fingerprint for {text:?}"
            );
        } else {
            baseline.insert(text, fp);
        }
    }

    crate::Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = crate::ThemeConfig::default();
        cfg.colors
            .insert("foreground".to_string(), "#00ff00".to_string());
        theme.extend_tokens_from_config(&cfg);
    });

    // Paint-only theme change: should not force any new text preparation.
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        services.calls.is_empty(),
        "expected paint-only theme changes to avoid re-preparing any text blobs"
    );
}
