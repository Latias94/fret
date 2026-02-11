use fret_app::App;
use fret_core::{AppWindowId, NodeId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::tree::UiTree;
use fret_ui_kit::headless::form_state::{FormFieldId, FormState};
#[path = "web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

fn collect_tag<'a>(node: &'a WebNode, tag: &str, out: &mut Vec<&'a WebNode>) {
    if node.tag == tag {
        out.push(node);
    }
    for child in &node.children {
        collect_tag(child, tag, out);
    }
}

#[derive(Default)]
struct StyleAwareServices;

impl fret_core::TextService for StyleAwareServices {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
        let style = match input {
            fret_core::TextInput::Plain { style, .. } => style,
            fret_core::TextInput::Attributed { base, .. } => base,
            _ => {
                debug_assert!(false, "unsupported TextInput variant");
                return (
                    fret_core::TextBlobId::default(),
                    fret_core::TextMetrics {
                        size: CoreSize::new(Px(0.0), Px(0.0)),
                        baseline: Px(0.0),
                    },
                );
            }
        };

        let line_height = style
            .line_height
            .unwrap_or(Px((style.size.0 * 1.4).max(0.0)));

        let text_len = match input {
            fret_core::TextInput::Plain { text, .. } => text.chars().count(),
            fret_core::TextInput::Attributed { text, .. } => text.chars().count(),
            _ => 0,
        };

        let char_w = (style.size.0 * 0.55).max(1.0);
        let est_w = Px(char_w * text_len as f32);

        let max_w = constraints.max_width.unwrap_or(est_w);
        let (lines, w) = match constraints.wrap {
            fret_core::TextWrap::Word if max_w.0.is_finite() && max_w.0 > 0.0 => {
                let lines = (est_w.0 / max_w.0).ceil().max(1.0) as u32;
                (lines, Px(est_w.0.min(max_w.0)))
            }
            _ => (1, est_w),
        };

        let h = Px(line_height.0 * lines as f32);

        (
            fret_core::TextBlobId::default(),
            fret_core::TextMetrics {
                size: CoreSize::new(w, h),
                baseline: Px(h.0 * 0.8),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for StyleAwareServices {
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

impl fret_core::SvgService for StyleAwareServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        true
    }
}

impl fret_core::MaterialService for StyleAwareServices {
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

fn run_fret_root_with_ui_and_services(
    bounds: Rect,
    services: &mut dyn fret_core::UiServices,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (UiTree<App>, fret_core::SemanticsSnapshot, NodeId) {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        services,
        window,
        bounds,
        "web-vs-fret-form",
        f,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    (ui, snap, root)
}

fn assert_close_px(label: &str, actual: Px, expected: f32, tol: f32) {
    let delta = (actual.0 - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected≈{expected} (±{tol}) got={}",
        actual.0
    );
}

const FORM_KEYS: &[&str] = &[
    "form-rhf-array",
    "form-rhf-checkbox",
    "form-rhf-complex",
    "form-rhf-demo",
    "form-rhf-input",
    "form-rhf-password",
    "form-rhf-radiogroup",
    "form-rhf-select",
    "form-rhf-switch",
    "form-rhf-textarea",
    "form-tanstack-array",
    "form-tanstack-checkbox",
    "form-tanstack-complex",
    "form-tanstack-demo",
    "form-tanstack-input",
    "form-tanstack-radiogroup",
    "form-tanstack-select",
    "form-tanstack-switch",
    "form-tanstack-textarea",
];

#[test]
fn shadcn_form_goldens_are_targeted_gates() {
    for &key in FORM_KEYS {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        let mut forms = Vec::new();
        collect_tag(&theme.root, "form", &mut forms);
        assert!(!forms.is_empty(), "expected at least one <form> in {key}");

        let mut labels = Vec::new();
        collect_tag(&theme.root, "label", &mut labels);
        let mut legends = Vec::new();
        collect_tag(&theme.root, "legend", &mut legends);
        assert!(
            !labels.is_empty() || !legends.is_empty(),
            "expected at least one <label> or <legend> in {key}"
        );

        let mut inputs = Vec::new();
        collect_tag(&theme.root, "input", &mut inputs);
        let mut textareas = Vec::new();
        collect_tag(&theme.root, "textarea", &mut textareas);
        let mut buttons = Vec::new();
        collect_tag(&theme.root, "button", &mut buttons);
        let mut selects = Vec::new();
        collect_tag(&theme.root, "select", &mut selects);
        assert!(
            !inputs.is_empty()
                || !textareas.is_empty()
                || !buttons.is_empty()
                || !selects.is_empty(),
            "expected at least one form control element in {key}"
        );
    }
}

#[test]
fn web_vs_fret_form_rhf_input_control_height_matches() {
    let web = read_web_golden("form-rhf-input");
    let theme = web.themes.get("light").expect("missing light theme");

    let web_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");
    let web_label = find_first(&theme.root, &|n| n.tag == "label").expect("web label");
    let label_text = web_label
        .text
        .clone()
        .unwrap_or_else(|| "Username".to_string());

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (_ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        let input_model: Model<String> = cx.app.models_mut().insert(String::new());
        let form_state: Model<FormState> = cx.app.models_mut().insert(FormState::default());

        let id = FormFieldId::from("username");
        let input = fret_ui_shadcn::Input::new(input_model)
            .a11y_label(label_text.clone())
            .into_element(cx);
        let field = fret_ui_shadcn::FormField::new(form_state, id, vec![input])
            .label(label_text.clone())
            .into_element(cx);

        vec![field]
    });

    let fret_input = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::TextField)
        .expect("fret text field");
    assert_close_px(
        "form-rhf-input control height",
        fret_input.bounds.size.height,
        web_input.rect.h,
        1.0,
    );
}
