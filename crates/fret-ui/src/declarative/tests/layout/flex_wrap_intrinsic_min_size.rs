use super::*;

#[derive(Default)]
struct VariableWidthTextService {
    prepare_calls: usize,
    release_calls: usize,
    path_prepare_calls: usize,
    path_release_calls: usize,
    svg_register_calls: usize,
    svg_unregister_calls: usize,
}

impl VariableWidthTextService {
    fn width_for_text(text: &str) -> Px {
        Px(text.chars().count() as f32 * 10.0)
    }
}

impl TextService for VariableWidthTextService {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        self.prepare_calls += 1;
        let w = Self::width_for_text(input.text());
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(w, Px(10.0)),
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

impl fret_core::PathService for VariableWidthTextService {
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

impl fret_core::SvgService for VariableWidthTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        self.svg_register_calls += 1;
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        self.svg_unregister_calls += 1;
        true
    }
}

impl fret_core::MaterialService for VariableWidthTextService {
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

fn flex_1_item(
    cx: &mut ElementContext<'_, TestHost>,
    label: &'static str,
    min_w_0: bool,
) -> AnyElement {
    let mut container = crate::element::ContainerProps::default();
    container.layout.flex.grow = 1.0;
    container.layout.flex.shrink = 1.0;
    container.layout.flex.basis = Length::Px(Px(0.0));
    container.layout.size.height = Length::Px(Px(20.0));
    if min_w_0 {
        container.layout.size.min_width = Some(Px(0.0));
    }

    let text = Arc::<str>::from(label);
    cx.container(container, move |cx| {
        vec![cx.text_props(crate::element::TextProps {
            layout: crate::element::LayoutStyle::default(),
            text: text.clone(),
            style: None,
            color: None,
            wrap: fret_core::TextWrap::None,
            overflow: fret_core::TextOverflow::Clip,
            align: fret_core::TextAlign::Center,
        })]
    })
}

#[test]
fn flex_wrap_keeps_flex_1_items_above_intrinsic_min_content_width() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(200.0)),
    );
    let mut services = VariableWidthTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "flex-wrap-intrinsic-min",
        |cx| {
            let mut props = crate::element::FlexProps::default();
            props.layout.size.width = crate::element::Length::Fill;
            props.wrap = true;
            vec![cx.flex(props, |cx| {
                vec![
                    flex_1_item(cx, "123456", false),
                    flex_1_item(cx, "123456", false),
                ]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let children = ui.children(flex_node);
    assert_eq!(children.len(), 2);

    let b0 = ui.debug_node_bounds(children[0]).expect("child0 bounds");
    let b1 = ui.debug_node_bounds(children[1]).expect("child1 bounds");

    let min_w = VariableWidthTextService::width_for_text("123456").0;
    assert!(
        b0.size.width.0 + 0.01 >= min_w,
        "expected child0 width >= intrinsic min-content width, got {:?}",
        b0.size.width
    );
    assert!(
        b1.size.width.0 + 0.01 >= min_w,
        "expected child1 width >= intrinsic min-content width, got {:?}",
        b1.size.width
    );
    assert!(
        b1.origin.y.0 > 0.01,
        "expected child1 to wrap onto a new line; got y={:?}",
        b1.origin.y
    );
}

#[test]
fn flex_wrap_allows_opt_out_with_min_w_0() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(200.0)),
    );
    let mut services = VariableWidthTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "flex-wrap-intrinsic-min-optout",
        |cx| {
            let mut props = crate::element::FlexProps::default();
            props.layout.size.width = crate::element::Length::Fill;
            props.wrap = true;
            vec![cx.flex(props, |cx| {
                vec![
                    flex_1_item(cx, "123456", false),
                    flex_1_item(cx, "123456", true),
                ]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let flex_node = ui.children(root)[0];
    let children = ui.children(flex_node);
    assert_eq!(children.len(), 2);

    let b0 = ui.debug_node_bounds(children[0]).expect("child0 bounds");
    let b1 = ui.debug_node_bounds(children[1]).expect("child1 bounds");

    let min_w = VariableWidthTextService::width_for_text("123456").0;
    assert!(
        b0.size.width.0 + 0.01 >= min_w,
        "expected child0 width >= intrinsic min-content width, got {:?}",
        b0.size.width
    );
    assert!(
        b1.origin.y.0.abs() < 0.01,
        "expected child1 not to wrap with min-w-0; got y={:?}",
        b1.origin.y
    );
    assert!(
        b1.size.width.0 + 0.01 < min_w,
        "expected child1 to be allowed to shrink below intrinsic width; got {:?}",
        b1.size.width
    );
}
