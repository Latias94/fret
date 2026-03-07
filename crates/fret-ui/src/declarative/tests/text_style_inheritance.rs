use super::*;

use crate::element::{
    ContainerProps, SelectableTextProps, StyledTextProps, TextAreaProps, TextInputProps, TextProps,
};
use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};
use fret_core::{
    AttributedText, MaterialDescriptor, MaterialId, MaterialRegistrationError, PathCommand,
    PathConstraints, PathId, PathMetrics, PathStyle, SvgId, TextBlobId, TextInput,
    TextLineHeightPolicy, TextMetrics, TextSpan, TextStyleRefinement, TextWrap,
};
use std::sync::Arc;

#[derive(Default)]
struct StyleRecordingTextService {
    styles: Vec<fret_core::TextStyle>,
}

impl StyleRecordingTextService {
    fn last_style(&self) -> &fret_core::TextStyle {
        self.styles.last().expect("expected recorded text style")
    }
}

impl TextService for StyleRecordingTextService {
    fn prepare(
        &mut self,
        input: &TextInput,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let style = match input {
            TextInput::Plain { style, .. } => style.clone(),
            TextInput::Attributed { base, .. } => base.clone(),
            _ => fret_core::TextStyle::default(),
        };
        self.styles.push(style.clone());
        let line_height = style.line_height.unwrap_or(style.size);
        let width = constraints
            .max_width
            .map(|max_width| Px(style.size.0.min(max_width.0.max(0.0))))
            .unwrap_or(style.size);
        (
            TextBlobId::default(),
            TextMetrics {
                size: Size::new(width, line_height),
                baseline: Px((line_height.0 * 0.8).max(0.0)),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl fret_core::PathService for StyleRecordingTextService {
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

impl fret_core::SvgService for StyleRecordingTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

impl fret_core::MaterialService for StyleRecordingTextService {
    fn register_material(
        &mut self,
        _desc: MaterialDescriptor,
    ) -> Result<MaterialId, MaterialRegistrationError> {
        Err(MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: MaterialId) -> bool {
        false
    }
}

fn bounds() -> Rect {
    Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
    )
}

fn max_content_constraints() -> LayoutConstraints {
    LayoutConstraints::new(
        LayoutSize::new(None, None),
        LayoutSize::new(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
    )
}

fn render_root_for_frame_local<S>(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut S,
    window: AppWindowId,
    bounds: Rect,
    name: &str,
    build_root: impl FnOnce(&mut ElementContext<'_, TestHost>) -> Vec<AnyElement>,
) -> NodeId
where
    S: TextService + fret_core::PathService + fret_core::SvgService + fret_core::MaterialService,
{
    let _ = ui.propagate_pending_model_changes(app);
    let root = render_root(ui, app, services, window, bounds, name, build_root);
    ui.set_root(root);
    root
}

fn inherited_refinement(size: f32, line_height: f32) -> TextStyleRefinement {
    TextStyleRefinement {
        size: Some(Px(size)),
        line_height: Some(Px(line_height)),
        line_height_policy: Some(TextLineHeightPolicy::FixedFromStyle),
        ..Default::default()
    }
}

fn fixed_style(size: f32, line_height: f32) -> fret_core::TextStyle {
    fret_core::TextStyle {
        size: Px(size),
        line_height: Some(Px(line_height)),
        line_height_policy: TextLineHeightPolicy::FixedFromStyle,
        ..Default::default()
    }
}

fn only_child(ui: &UiTree<TestHost>, node: NodeId) -> NodeId {
    let children = ui.children(node);
    assert_eq!(
        children.len(),
        1,
        "expected exactly one child for node {node:?}"
    );
    children[0]
}

#[test]
fn inherited_text_style_affects_passive_text_measurement() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let mut services = StyleRecordingTextService::default();

    let root = render_root_for_frame_local(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        "text-style-inheritance-text-measure",
        |cx| {
            vec![
                cx.container(ContainerProps::default(), |cx| vec![cx.text("hello")])
                    .inherit_text_style(inherited_refinement(24.0, 30.0)),
            ]
        },
    );

    let scope = only_child(&ui, root);
    let text = only_child(&ui, scope);
    let size = ui.measure_in(
        &mut app,
        &mut services,
        text,
        max_content_constraints(),
        1.0,
    );

    assert_eq!(size, Size::new(Px(24.0), Px(30.0)));
    assert_eq!(services.last_style().size, Px(24.0));
    assert_eq!(services.last_style().line_height, Some(Px(30.0)));
}

#[test]
fn explicit_leaf_text_style_wins_over_inherited_refinement() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let mut services = StyleRecordingTextService::default();

    let root = render_root_for_frame_local(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        "text-style-inheritance-explicit-wins",
        |cx| {
            vec![
                cx.container(ContainerProps::default(), |cx| {
                    let mut props = TextProps::new("hello");
                    props.style = Some(fixed_style(11.0, 15.0));
                    vec![cx.text_props(props)]
                })
                .inherit_text_style(inherited_refinement(24.0, 30.0)),
            ]
        },
    );

    let scope = only_child(&ui, root);
    let text = only_child(&ui, scope);
    let size = ui.measure_in(
        &mut app,
        &mut services,
        text,
        max_content_constraints(),
        1.0,
    );

    assert_eq!(size, Size::new(Px(11.0), Px(15.0)));
    assert_eq!(services.last_style().size, Px(11.0));
    assert_eq!(services.last_style().line_height, Some(Px(15.0)));
}

#[test]
fn inherited_text_style_applies_to_styled_and_selectable_text_measurement() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let mut services = StyleRecordingTextService::default();

    let rich = AttributedText::new(
        Arc::<str>::from("hello"),
        Arc::<[TextSpan]>::from([TextSpan::new(5)]),
    );

    let root = render_root_for_frame_local(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        "text-style-inheritance-styled-selectable",
        |cx| {
            vec![
                cx.container(ContainerProps::default(), |cx| {
                    vec![
                        cx.styled_text_props(StyledTextProps::new(rich.clone())),
                        cx.selectable_text_props(SelectableTextProps::new(rich.clone())),
                    ]
                })
                .inherit_text_style(inherited_refinement(21.0, 27.0)),
            ]
        },
    );

    let scope = only_child(&ui, root);
    let styled = ui.children(scope)[0];
    let selectable = ui.children(scope)[1];

    let styled_size = ui.measure_in(
        &mut app,
        &mut services,
        styled,
        max_content_constraints(),
        1.0,
    );
    assert_eq!(styled_size, Size::new(Px(21.0), Px(27.0)));
    assert_eq!(services.last_style().size, Px(21.0));

    services.styles.clear();

    let selectable_size = ui.measure_in(
        &mut app,
        &mut services,
        selectable,
        max_content_constraints(),
        1.0,
    );
    assert_eq!(selectable_size, Size::new(Px(21.0), Px(27.0)));
    assert_eq!(services.last_style().size, Px(21.0));
}

#[test]
fn wrap_none_measure_cache_tracks_inherited_text_style_changes() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let mut services = StyleRecordingTextService::default();

    let render = |ui: &mut UiTree<TestHost>,
                  app: &mut TestHost,
                  services: &mut StyleRecordingTextService,
                  size: f32,
                  line_height: f32| {
        let root = render_root_for_frame_local(
            ui,
            app,
            services,
            window,
            bounds(),
            "text-style-inheritance-wrap-none-cache",
            |cx| {
                vec![
                    cx.container(ContainerProps::default(), |cx| {
                        let mut props = TextProps::new("cache");
                        props.wrap = TextWrap::None;
                        vec![cx.text_props(props)]
                    })
                    .inherit_text_style(inherited_refinement(size, line_height)),
                ]
            },
        );
        let scope = only_child(ui, root);
        let text = only_child(ui, scope);
        ui.measure_in(app, services, text, max_content_constraints(), 1.0)
    };

    let first = render(&mut ui, &mut app, &mut services, 12.0, 16.0);
    let second = render(&mut ui, &mut app, &mut services, 28.0, 36.0);

    assert_eq!(first, Size::new(Px(12.0), Px(16.0)));
    assert_eq!(second, Size::new(Px(28.0), Px(36.0)));
}

fn measure_text_input_size(with_inherited: bool) -> Size {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let mut services = StyleRecordingTextService::default();
    let model = app.models_mut().insert(String::new());

    let root = render_root_for_frame_local(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        "text-style-inheritance-text-input",
        |cx| {
            let input = cx.text_input(TextInputProps::new(model));
            let child = cx.container(ContainerProps::default(), move |_cx| vec![input]);
            if with_inherited {
                vec![child.inherit_text_style(inherited_refinement(30.0, 38.0))]
            } else {
                vec![child]
            }
        },
    );

    let scope = only_child(&ui, root);
    let input = only_child(&ui, scope);
    ui.measure_in(
        &mut app,
        &mut services,
        input,
        max_content_constraints(),
        1.0,
    )
}

fn measure_text_area_size(with_inherited: bool) -> Size {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    let mut services = StyleRecordingTextService::default();
    let model = app.models_mut().insert(String::new());

    let root = render_root_for_frame_local(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        "text-style-inheritance-text-area",
        |cx| {
            let area = cx.text_area(TextAreaProps::new(model));
            let child = cx.container(ContainerProps::default(), move |_cx| vec![area]);
            if with_inherited {
                vec![child.inherit_text_style(inherited_refinement(30.0, 38.0))]
            } else {
                vec![child]
            }
        },
    );

    let scope = only_child(&ui, root);
    let area = only_child(&ui, scope);
    ui.measure_in(
        &mut app,
        &mut services,
        area,
        max_content_constraints(),
        1.0,
    )
}

#[test]
fn text_input_and_text_area_ignore_inherited_text_style_v1() {
    assert_eq!(
        measure_text_input_size(false),
        measure_text_input_size(true),
        "expected TextInput to ignore inherited passive-text refinement in v1"
    );
    assert_eq!(
        measure_text_area_size(false),
        measure_text_area_size(true),
        "expected TextArea to ignore inherited passive-text refinement in v1"
    );
}
