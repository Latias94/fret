use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButtons, NodeId, Point, PointerEvent, PointerId,
    PointerType, Px, Rect, SemanticsRole, Size as CoreSize,
};
use fret_core::{Scene, SceneOp, Transform2D};
use fret_runtime::Model;
use fret_ui::Theme;
use fret_ui::tree::UiTree;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod css_color;
use css_color::{Rgba, color_to_rgba, parse_css_color};

#[derive(Debug, Clone, Deserialize)]
struct WebGolden {
    themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebGoldenTheme {
    viewport: WebViewport,
    root: WebNode,
    #[serde(default)]
    portals: Vec<WebNode>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct WebViewport {
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct WebRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct WebNode {
    tag: String,
    #[serde(default)]
    #[serde(rename = "className")]
    class_name: Option<String>,
    #[serde(default)]
    #[serde(rename = "computedStyle")]
    computed_style: BTreeMap<String, String>,
    #[serde(default)]
    attrs: BTreeMap<String, String>,
    rect: WebRect,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    children: Vec<WebNode>,
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

fn web_golden_path(name: &str) -> PathBuf {
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
        .join(format!("{name}.json"))
}

fn read_web_golden(name: &str) -> WebGolden {
    let path = web_golden_path(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing web golden: {}\nerror: {err}\n\nGenerate it via:\n  pnpm -C repo-ref/ui/apps/v4 golden:extract {name} --update\n\nDocs:\n  goldens/README.md\n  docs/shadcn-web-goldens.md",
            path.display()
        )
    });
    serde_json::from_str(&text).unwrap_or_else(|err| {
        panic!(
            "failed to parse web golden: {}\nerror: {err}",
            path.display()
        )
    })
}

fn web_theme<'a>(golden: &'a WebGolden) -> &'a WebGoldenTheme {
    golden
        .themes
        .get("light")
        .or_else(|| golden.themes.get("dark"))
        .expect("missing theme in web golden")
}

fn find_first<'a>(node: &'a WebNode, pred: &impl Fn(&'a WebNode) -> bool) -> Option<&'a WebNode> {
    if pred(node) {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_first(child, pred) {
            return Some(found);
        }
    }
    None
}

fn find_all<'a>(node: &'a WebNode, pred: &impl Fn(&'a WebNode) -> bool) -> Vec<&'a WebNode> {
    let mut out = Vec::new();
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if pred(n) {
            out.push(n);
        }
        for child in &n.children {
            stack.push(child);
        }
    }
    out
}

fn find_first_in_theme<'a>(
    theme: &'a WebGoldenTheme,
    pred: &impl Fn(&'a WebNode) -> bool,
) -> Option<&'a WebNode> {
    find_first(&theme.root, pred).or_else(|| theme.portals.iter().find_map(|p| find_first(p, pred)))
}

fn find_all_in_theme<'a>(
    theme: &'a WebGoldenTheme,
    pred: &impl Fn(&'a WebNode) -> bool,
) -> Vec<&'a WebNode> {
    let mut out = find_all(&theme.root, pred);
    for portal in &theme.portals {
        out.extend(find_all(portal, pred));
    }
    out
}

fn class_has_token(node: &WebNode, token: &str) -> bool {
    node.class_name
        .as_deref()
        .unwrap_or("")
        .split_whitespace()
        .any(|t| t == token)
}

fn web_find_by_class_token<'a>(root: &'a WebNode, token: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| class_has_token(n, token))
}

fn web_find_by_class_token_in_theme<'a>(
    theme: &'a WebGoldenTheme,
    token: &str,
) -> Option<&'a WebNode> {
    find_first_in_theme(theme, &|n| class_has_token(n, token))
}

fn assert_close_px(label: &str, actual: Px, expected: f32, tol: f32) {
    let delta = (actual.0 - expected).abs();
    assert!(
        delta <= tol,
        "{label}: expected≈{expected} (±{tol}) got={}",
        actual.0
    );
}

#[derive(Debug, Clone)]
struct RecordedTextPrepare {
    text: String,
    style: fret_core::TextStyle,
    constraints: fret_core::TextConstraints,
}

#[derive(Default)]
struct StyleAwareServices {
    prepared: Vec<RecordedTextPrepare>,
}

impl fret_core::TextService for StyleAwareServices {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
        let (text, style) = match input {
            fret_core::TextInput::Plain { text, style } => (text.as_ref(), style),
            fret_core::TextInput::Attributed { text, base, .. } => (text.as_ref(), base),
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
        self.prepared.push(RecordedTextPrepare {
            text: text.to_string(),
            style: style.clone(),
            constraints,
        });

        let line_height = style
            .line_height
            .unwrap_or(Px((style.size.0 * 1.4).max(0.0)));

        let char_w = (style.size.0 * 0.55).max(1.0);
        let est_w = Px(char_w * text.chars().count() as f32);

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
        "web-vs-fret-calendar",
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

fn render_calendar_in_bounds(
    bounds: Rect,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (UiTree<App>, fret_core::SemanticsSnapshot, NodeId) {
    let mut services = StyleAwareServices::default();
    run_fret_root_with_ui_and_services(bounds, &mut services, f)
}

fn render_calendar_in_bounds_with_scene(
    bounds: Rect,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-calendar-painted",
        f,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    (snap, scene)
}

fn render_calendar_in_bounds_with_scene_and_scheme(
    bounds: Rect,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    f: impl FnOnce(&mut fret_ui::ElementContext<'_, App>) -> Vec<fret_ui::element::AnyElement>,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        scheme,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-calendar-painted",
        f,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    (snap, scene)
}

fn find_semantics<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    role: SemanticsRole,
    label: Option<&str>,
) -> Option<&'a fret_core::SemanticsNode> {
    snap.nodes.iter().find(|n| {
        if n.role != role {
            return false;
        }
        if let Some(label) = label {
            return n.label.as_deref() == Some(label);
        }
        true
    })
}

struct CalendarRangeWebConfig {
    month: time::Month,
    year: i32,
    origin_x: f32,
    origin_y: f32,
    chrome_override: fret_ui_kit::ChromeRefinement,
    cell_size: Px,
    week_start: time::Weekday,
    today: Option<time::Date>,
    show_week_number: bool,
    show_outside_days: bool,
    disable_outside_days: bool,
    range_min: time::Date,
    range_max: time::Date,
}

fn web_calendar_range_config(theme: &WebGoldenTheme) -> CalendarRangeWebConfig {
    fn parse_css_px(s: &str) -> Option<f32> {
        s.strip_suffix("px")?.parse::<f32>().ok()
    }

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let origin_x = web_rdp_root.rect.x;
    let origin_y = web_rdp_root.rect.y;

    let padding_left = web_rdp_root
        .computed_style
        .get("paddingLeft")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);
    let border_left = web_rdp_root
        .computed_style
        .get("borderLeftWidth")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);

    let web_month_grid = find_first_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    })
    .expect("web month grid");
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_day_buttons = find_all_in_theme(theme, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let selected_dates: Vec<time::Date> = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
        .map(|(d, _)| d)
        .collect();
    assert!(
        selected_dates.len() >= 3,
        "expected at least 3 selected dates for range mode"
    );

    let (range_min, range_max) = selected_dates
        .iter()
        .copied()
        .fold((selected_dates[0], selected_dates[0]), |(min, max), d| {
            (min.min(d), max.max(d))
        });

    let weekday_headers = find_all_in_theme(theme, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let show_week_number =
        find_first_in_theme(theme, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let show_outside_days =
        find_first_in_theme(theme, &|n| class_has_token(n, "rdp-outside")).is_some();

    let disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let cell_size = parse_calendar_cell_size_px(theme).unwrap_or_else(|| {
        let sample = web_day_buttons[0];
        Px(sample.rect.w)
    });

    let chrome_override = {
        let mut chrome = fret_ui_kit::ChromeRefinement::default();
        if (padding_left - 0.0).abs() < 0.5 {
            chrome = chrome.p(fret_ui_kit::Space::N0);
        } else if (padding_left - 12.0).abs() < 0.5 {
            chrome = chrome.p(fret_ui_kit::Space::N3);
        } else if (padding_left - 8.0).abs() < 0.5 {
            chrome = chrome.p(fret_ui_kit::Space::N2);
        }
        if border_left >= 0.5 {
            chrome = chrome.border_1();
        }
        chrome
    };

    CalendarRangeWebConfig {
        month,
        year,
        origin_x,
        origin_y,
        chrome_override,
        cell_size,
        week_start,
        today,
        show_week_number,
        show_outside_days,
        disable_outside_days,
        range_min,
        range_max,
    }
}

fn render_fret_calendar_range_scene(
    config: &CalendarRangeWebConfig,
    viewport: WebViewport,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) -> Scene {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(viewport.w), Px(viewport.h)),
    );

    let (_snap, scene) = render_calendar_in_bounds_with_scene_and_scheme(bounds, scheme, |cx| {
        use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};

        let month_model: Model<CalendarMonth> = cx
            .app
            .models_mut()
            .insert(CalendarMonth::new(config.year, config.month));
        let selected: Model<DateRangeSelection> = cx.app.models_mut().insert(DateRangeSelection {
            from: Some(config.range_min),
            to: Some(config.range_max),
        });

        let mut calendar = fret_ui_shadcn::CalendarRange::new(month_model, selected)
            .week_start(config.week_start)
            .show_outside_days(config.show_outside_days)
            .disable_outside_days(config.disable_outside_days)
            .show_week_number(config.show_week_number)
            .refine_style(config.chrome_override.clone())
            .cell_size(config.cell_size);

        if let Some(today) = config.today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            fret_ui::element::ContainerProps {
                layout: {
                    let mut layout = fret_ui::element::LayoutStyle::default();
                    layout.size.width = fret_ui::element::Length::Fill;
                    layout.size.height = fret_ui::element::Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(config.origin_x),
                    top: Px(config.origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    });

    scene
}

fn assert_calendar_range_day_background_matches_web(
    web_name: &str,
    range_cell_class: &str,
    expected_label: &str,
    web_theme_name: &str,
) {
    let web = read_web_golden(web_name);
    let theme = web
        .themes
        .get(web_theme_name)
        .unwrap_or_else(|| panic!("missing {web_theme_name} theme in web golden {web_name}"));

    let cell = find_first_in_theme(theme, &|n| class_has_token(n, range_cell_class))
        .unwrap_or_else(|| panic!("web missing {range_cell_class} day cell"));
    let button = find_first(cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| label.as_str() == expected_label)
    })
    .unwrap_or_else(|| {
        panic!("web missing {range_cell_class} day button label={expected_label:?}")
    });

    let web_bg_css = button
        .computed_style
        .get("backgroundColor")
        .expect("web day backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let config = web_calendar_range_config(theme);
    let scheme = match web_theme_name {
        "dark" => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        _ => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    };
    let scene = render_fret_calendar_range_scene(&config, theme.viewport, scheme);

    let target = Rect::new(
        Point::new(Px(button.rect.x), Px(button.rect.y)),
        CoreSize::new(Px(button.rect.w), Px(button.rect.h)),
    );
    let quad = find_best_opaque_background_quad(&scene, target)
        .unwrap_or_else(|| panic!("painted opaque {range_cell_class} day background quad"));

    assert_rect_xwh_close_px(
        &format!("{web_name} {web_theme_name} {range_cell_class} day quad"),
        quad.rect,
        button.rect,
        3.0,
    );
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} {range_cell_class} day background"),
        color_to_rgba(quad.background),
        expected_bg,
        0.02,
    );
}

fn parse_calendar_title_label(label: &str) -> Option<(time::Month, i32)> {
    let label = label.trim();
    let (month, year) = label.rsplit_once(' ')?;
    if year.len() != 4 || !year.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let year: i32 = year.parse().ok()?;

    let month_lower = month.to_lowercase();
    let month = match (month, month_lower.as_str()) {
        ("January", _) | (_, "january") | (_, "enero") => time::Month::January,
        ("February", _) | (_, "february") | (_, "febrero") => time::Month::February,
        ("March", _) | (_, "march") | (_, "marzo") => time::Month::March,
        ("April", _) | (_, "april") | (_, "abril") => time::Month::April,
        ("May", _) | (_, "may") | (_, "mayo") => time::Month::May,
        ("June", _) | (_, "june") | (_, "junio") => time::Month::June,
        ("July", _) | (_, "july") | (_, "julio") => time::Month::July,
        ("August", _) | (_, "august") | (_, "agosto") => time::Month::August,
        ("September", _) | (_, "september") | (_, "septiembre") | (_, "setiembre") => {
            time::Month::September
        }
        ("October", _) | (_, "october") | (_, "octubre") => time::Month::October,
        ("November", _) | (_, "november") | (_, "noviembre") => time::Month::November,
        ("December", _) | (_, "december") | (_, "diciembre") => time::Month::December,
        _ => return None,
    };
    Some((month, year))
}

fn parse_calendar_weekday_label(label: &str) -> Option<time::Weekday> {
    let label = label.trim();
    let lower = label.to_lowercase();
    match (label, lower.as_str()) {
        ("Monday", _) | (_, "monday") | (_, "lunes") => Some(time::Weekday::Monday),
        ("Tuesday", _) | (_, "tuesday") | (_, "martes") => Some(time::Weekday::Tuesday),
        ("Wednesday", _) | (_, "wednesday") | (_, "miércoles") | (_, "miercoles") => {
            Some(time::Weekday::Wednesday)
        }
        ("Thursday", _) | (_, "thursday") | (_, "jueves") => Some(time::Weekday::Thursday),
        ("Friday", _) | (_, "friday") | (_, "viernes") => Some(time::Weekday::Friday),
        ("Saturday", _) | (_, "saturday") | (_, "sábado") | (_, "sabado") => {
            Some(time::Weekday::Saturday)
        }
        ("Sunday", _) | (_, "sunday") | (_, "domingo") => Some(time::Weekday::Sunday),
        _ => None,
    }
}

fn parse_calendar_day_aria_label(label: &str) -> Option<(time::Date, bool)> {
    let selected = label.ends_with(", selected");
    let label = label.strip_suffix(", selected").unwrap_or(label);
    let label = label.strip_prefix("Today, ").unwrap_or(label);
    let label = label.strip_prefix("Hoy, ").unwrap_or(label);

    if let Some((prefix, year)) = label.rsplit_once(", ") {
        if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) {
            let year: i32 = year.parse().ok()?;

            let (_weekday, month_and_day) = prefix.split_once(", ")?;
            let (month, day_with_suffix) = month_and_day.split_once(' ')?;
            let (month, _label_year) = parse_calendar_title_label(&format!("{month} {year}"))?;

            let day_digits: String = day_with_suffix
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if day_digits.is_empty() {
                return None;
            }
            let day: u8 = day_digits.parse().ok()?;

            let date = time::Date::from_calendar_date(year, month, day).ok()?;
            return Some((date, selected));
        }
    }

    let (weekday, rest) = label.split_once(", ")?;
    let _weekday = parse_calendar_weekday_label(weekday)?;
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() != 5 || parts[1] != "de" || parts[3] != "de" {
        return None;
    }
    let day: u8 = parts[0].parse().ok()?;
    let (month, year) = parse_calendar_title_label(&format!("{} {}", parts[2], parts[4]))?;
    let date = time::Date::from_calendar_date(year, month, day).ok()?;
    Some((date, selected))
}

fn assert_rgba_close(label: &str, actual: Rgba, expected: Rgba, tol: f32) {
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

fn assert_rect_xwh_close_px(label: &str, actual: Rect, expected: WebRect, tol: f32) {
    assert_close_px(&format!("{label} x"), actual.origin.x, expected.x, tol);
    assert_close_px(&format!("{label} y"), actual.origin.y, expected.y, tol);
    assert_close_px(&format!("{label} w"), actual.size.width, expected.w, tol);
    assert_close_px(&format!("{label} h"), actual.size.height, expected.h, tol);
}

#[derive(Debug, Clone, Copy)]
struct PaintedQuad {
    rect: Rect,
    background: fret_core::Color,
}

fn find_best_opaque_background_quad(scene: &Scene, target: Rect) -> Option<PaintedQuad> {
    let mut best: Option<PaintedQuad> = None;
    let mut best_score = f32::INFINITY;

    for op in scene.ops() {
        let SceneOp::Quad {
            rect, background, ..
        } = *op
        else {
            continue;
        };

        if background.a <= 0.001 {
            continue;
        }

        let score = (rect.origin.x.0 - target.origin.x.0).abs()
            + (rect.origin.y.0 - target.origin.y.0).abs()
            + (rect.size.width.0 - target.size.width.0).abs()
            + (rect.size.height.0 - target.size.height.0).abs();

        if score < best_score {
            best_score = score;
            best = Some(PaintedQuad { rect, background });
        }
    }

    best
}

#[derive(Clone, Copy)]
struct SceneWalkState {
    transform: Transform2D,
    opacity: f32,
}

fn scene_walk(scene: &Scene, mut f: impl FnMut(SceneWalkState, &SceneOp)) {
    let mut transform_stack: Vec<Transform2D> = Vec::new();
    let mut opacity_stack: Vec<f32> = Vec::new();
    let mut st = SceneWalkState {
        transform: Transform2D::IDENTITY,
        opacity: 1.0,
    };

    for op in scene.ops() {
        match *op {
            SceneOp::PushTransform { transform } => {
                transform_stack.push(st.transform);
                st.transform = st.transform.compose(transform);
            }
            SceneOp::PopTransform => {
                st.transform = transform_stack.pop().unwrap_or(Transform2D::IDENTITY);
            }
            SceneOp::PushOpacity { opacity } => {
                opacity_stack.push(st.opacity);
                st.opacity *= opacity;
            }
            SceneOp::PopOpacity => {
                st.opacity = opacity_stack.pop().unwrap_or(1.0);
            }
            _ => f(st, op),
        }
    }
}

fn color_with_opacity(color: fret_core::Color, opacity: f32) -> fret_core::Color {
    fret_core::Color {
        a: (color.a * opacity).clamp(0.0, 1.0),
        ..color
    }
}

fn apply_opacity(rgba: Rgba, opacity: f32) -> Rgba {
    Rgba {
        a: (rgba.a * opacity).clamp(0.0, 1.0),
        ..rgba
    }
}

fn find_best_text_color_in_rect(scene: &Scene, search_within: Rect) -> Option<Rgba> {
    let mut best: Option<Rgba> = None;
    let mut best_score = f32::INFINITY;

    let center = Point::new(
        Px(search_within.origin.x.0 + search_within.size.width.0 * 0.5),
        Px(search_within.origin.y.0 + search_within.size.height.0 * 0.5),
    );

    scene_walk(scene, |st, op| {
        let SceneOp::Text { origin, color, .. } = *op else {
            return;
        };
        let origin = st.transform.apply_point(origin);
        if !search_within.contains(origin) {
            return;
        }
        let rgba = color_to_rgba(color_with_opacity(color, st.opacity));
        if rgba.a <= 0.01 {
            return;
        }
        let score = (origin.x.0 - center.x.0).abs() + (origin.y.0 - center.y.0).abs();
        if score < best_score {
            best_score = score;
            best = Some(rgba);
        }
    });

    best
}

fn transform_rect_bounds(transform: Transform2D, rect: Rect) -> Rect {
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = x0 + rect.size.width.0;
    let y1 = y0 + rect.size.height.0;

    let p0 = transform.apply_point(Point::new(Px(x0), Px(y0)));
    let p1 = transform.apply_point(Point::new(Px(x1), Px(y0)));
    let p2 = transform.apply_point(Point::new(Px(x0), Px(y1)));
    let p3 = transform.apply_point(Point::new(Px(x1), Px(y1)));

    let min_x = p0.x.0.min(p1.x.0).min(p2.x.0).min(p3.x.0);
    let max_x = p0.x.0.max(p1.x.0).max(p2.x.0).max(p3.x.0);
    let min_y = p0.y.0.min(p1.y.0).min(p2.y.0).min(p3.y.0);
    let max_y = p0.y.0.max(p1.y.0).max(p2.y.0).max(p3.y.0);

    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        CoreSize::new(Px(max_x - min_x), Px(max_y - min_y)),
    )
}

fn find_best_icon_color_in_rect(scene: &Scene, search_within: Rect) -> Option<Rgba> {
    let mut best: Option<Rgba> = None;
    let mut best_score = f32::INFINITY;

    let center = Point::new(
        Px(search_within.origin.x.0 + search_within.size.width.0 * 0.5),
        Px(search_within.origin.y.0 + search_within.size.height.0 * 0.5),
    );

    scene_walk(scene, |st, op| match *op {
        SceneOp::SvgMaskIcon {
            rect,
            color,
            opacity,
            ..
        } => {
            let rect = transform_rect_bounds(st.transform, rect);
            let icon_center = Point::new(
                Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
                Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
            );
            if !search_within.contains(icon_center) {
                return;
            }

            let rgba = color_to_rgba(color_with_opacity(color, st.opacity * opacity));
            if rgba.a <= 0.01 {
                return;
            }

            let score = (icon_center.x.0 - center.x.0).abs() + (icon_center.y.0 - center.y.0).abs();
            if score < best_score {
                best_score = score;
                best = Some(rgba);
            }
        }
        SceneOp::Path { origin, color, .. } => {
            let origin = st.transform.apply_point(origin);
            if !search_within.contains(origin) {
                return;
            }
            let rgba = color_to_rgba(color_with_opacity(color, st.opacity));
            if rgba.a <= 0.01 {
                return;
            }

            let score = (origin.x.0 - center.x.0).abs() + (origin.y.0 - center.y.0).abs();
            if score < best_score {
                best_score = score;
                best = Some(rgba);
            }
        }
        _ => {}
    });

    best.or_else(|| find_best_text_color_in_rect(scene, search_within))
}

fn days_in_month(year: i32, month: time::Month) -> u8 {
    match month {
        time::Month::January => 31,
        time::Month::February => {
            let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            if leap { 29 } else { 28 }
        }
        time::Month::March => 31,
        time::Month::April => 30,
        time::Month::May => 31,
        time::Month::June => 30,
        time::Month::July => 31,
        time::Month::August => 31,
        time::Month::September => 30,
        time::Month::October => 31,
        time::Month::November => 30,
        time::Month::December => 31,
    }
}

#[derive(Debug, Clone, Copy)]
struct CalendarChromeConfig {
    locale: fret_ui_shadcn::calendar::CalendarLocale,
    month: time::Month,
    year: i32,
    origin_x: f32,
    origin_y: f32,
    cell_size: Px,
    week_start: time::Weekday,
    today: Option<time::Date>,
    show_week_number: bool,
    show_outside_days: bool,
    disable_outside_days: bool,
    selected: time::Date,
}

#[derive(Debug, Clone)]
enum CalendarSelectionMode {
    Single(time::Date),
    Range { min: time::Date, max: time::Date },
    Multiple(Vec<time::Date>),
}

#[derive(Debug, Clone)]
struct CalendarHoverChromeConfig {
    locale: fret_ui_shadcn::calendar::CalendarLocale,
    month: time::Month,
    year: i32,
    origin_x: f32,
    origin_y: f32,
    cell_size: Px,
    week_start: time::Weekday,
    today: Option<time::Date>,
    show_week_number: bool,
    show_outside_days: bool,
    disable_outside_days: bool,
    selection: CalendarSelectionMode,
    calendar_03_multiple_contract: bool,
}

fn render_calendar_chrome_from_config(
    cx: &mut fret_ui::ElementContext<'_, App>,
    config: CalendarChromeConfig,
) -> Vec<fret_ui::element::AnyElement> {
    use fret_ui_headless::calendar::CalendarMonth;

    let theme = Theme::global(&*cx.app).clone();
    let border = theme.color_required("border");

    let month_model: Model<CalendarMonth> = cx
        .app
        .models_mut()
        .insert(CalendarMonth::new(config.year, config.month));
    let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(Some(config.selected));

    let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
        .locale(config.locale)
        .week_start(config.week_start)
        .show_outside_days(config.show_outside_days)
        .disable_outside_days(config.disable_outside_days)
        .show_week_number(config.show_week_number)
        .refine_style(
            fret_ui_kit::ChromeRefinement::default()
                .rounded(fret_ui_kit::Radius::Lg)
                .border_1()
                .border_color(fret_ui_kit::ColorRef::Color(border))
                .shadow_sm(),
        )
        .cell_size(config.cell_size);
    if let Some(today) = config.today {
        calendar = calendar.today(today);
    }

    let calendar = calendar.into_element(cx);
    let calendar = cx.container(
        fret_ui::element::ContainerProps {
            layout: {
                let mut layout = fret_ui::element::LayoutStyle::default();
                layout.size.width = fret_ui::element::Length::Fill;
                layout.size.height = fret_ui::element::Length::Fill;
                layout
            },
            padding: fret_core::Edges {
                left: Px(config.origin_x),
                top: Px(config.origin_y),
                right: Px(0.0),
                bottom: Px(0.0),
            },
            ..Default::default()
        },
        move |_cx| vec![calendar],
    );

    vec![calendar]
}

fn render_calendar_hover_chrome_from_config(
    cx: &mut fret_ui::ElementContext<'_, App>,
    config: CalendarHoverChromeConfig,
) -> Vec<fret_ui::element::AnyElement> {
    use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};

    let theme = Theme::global(&*cx.app).clone();
    let border = theme.color_required("border");

    let chrome = fret_ui_kit::ChromeRefinement::default()
        .rounded(fret_ui_kit::Radius::Lg)
        .border_1()
        .border_color(fret_ui_kit::ColorRef::Color(border))
        .shadow_sm();

    let calendar = match config.selection {
        CalendarSelectionMode::Single(date) => {
            let month_model: Model<CalendarMonth> = cx
                .app
                .models_mut()
                .insert(CalendarMonth::new(config.year, config.month));
            let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(Some(date));

            let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
                .locale(config.locale)
                .week_start(config.week_start)
                .show_outside_days(config.show_outside_days)
                .disable_outside_days(config.disable_outside_days)
                .show_week_number(config.show_week_number)
                .refine_style(chrome)
                .cell_size(config.cell_size);
            if let Some(today) = config.today {
                calendar = calendar.today(today);
            }
            calendar.into_element(cx)
        }
        CalendarSelectionMode::Range { min, max } => {
            let month_model: Model<CalendarMonth> = cx
                .app
                .models_mut()
                .insert(CalendarMonth::new(config.year, config.month));
            let selected: Model<DateRangeSelection> =
                cx.app.models_mut().insert(DateRangeSelection {
                    from: Some(min),
                    to: Some(max),
                });

            let mut calendar = fret_ui_shadcn::CalendarRange::new(month_model, selected)
                .locale(config.locale)
                .week_start(config.week_start)
                .show_outside_days(config.show_outside_days)
                .disable_outside_days(config.disable_outside_days)
                .show_week_number(config.show_week_number)
                .refine_style(chrome)
                .cell_size(config.cell_size);
            if let Some(today) = config.today {
                calendar = calendar.today(today);
            }
            calendar.into_element(cx)
        }
        CalendarSelectionMode::Multiple(dates) => {
            let month_model: Model<CalendarMonth> = cx
                .app
                .models_mut()
                .insert(CalendarMonth::new(config.year, config.month));
            let selected: Model<Vec<time::Date>> = cx.app.models_mut().insert(dates);

            let mut calendar = fret_ui_shadcn::CalendarMultiple::new(month_model, selected)
                .locale(config.locale)
                .week_start(config.week_start)
                .show_outside_days(config.show_outside_days)
                .disable_outside_days(config.disable_outside_days)
                .show_week_number(config.show_week_number)
                .refine_style(chrome)
                .cell_size(config.cell_size);
            if config.calendar_03_multiple_contract {
                calendar = calendar.required(true).max(5);
            }
            if let Some(today) = config.today {
                calendar = calendar.today(today);
            }
            calendar.into_element(cx)
        }
    };
    let calendar = cx.container(
        fret_ui::element::ContainerProps {
            layout: {
                let mut layout = fret_ui::element::LayoutStyle::default();
                layout.size.width = fret_ui::element::Length::Fill;
                layout.size.height = fret_ui::element::Length::Fill;
                layout
            },
            padding: fret_core::Edges {
                left: Px(config.origin_x),
                top: Px(config.origin_y),
                right: Px(0.0),
                bottom: Px(0.0),
            },
            ..Default::default()
        },
        move |_cx| vec![calendar],
    );

    vec![calendar]
}

fn parse_calendar_cell_size_px(theme: &WebGoldenTheme) -> Option<Px> {
    let rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root")?;
    let class_name = rdp_root.class_name.as_deref().unwrap_or("");

    fn parse_spacing_value(token: &str, prefix: &str) -> Option<f32> {
        let rest = token.strip_prefix(prefix)?;
        let rest = rest.strip_suffix(")]")?;
        rest.parse::<f32>().ok()
    }

    let mut base: Option<f32> = None;
    let mut md: Option<f32> = None;
    for token in class_name.split_whitespace() {
        if let Some(v) = parse_spacing_value(token, "[--cell-size:--spacing(") {
            base = Some(v);
        }
        if let Some(v) = parse_spacing_value(token, "md:[--cell-size:--spacing(") {
            md = Some(v);
        }
    }

    let spacing = if theme.viewport.w >= 768.0 {
        md.or(base)
    } else {
        base
    }?;

    Some(Px(spacing * 4.0))
}

fn assert_calendar_single_month_variant_geometry_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    fn parse_css_px(s: &str) -> Option<f32> {
        s.strip_suffix("px")?.parse::<f32>().ok()
    }

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_padding_left = web_rdp_root
        .computed_style
        .get("paddingLeft")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);
    let web_border_left = web_rdp_root
        .computed_style
        .get("borderLeftWidth")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();

    let web_month_grids = find_all(&theme.root, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(
        web_month_grids.len(),
        1,
        "expected a single month grid for {web_name} (multi-month variants are gated separately)"
    );
    let web_month_grid = web_month_grids[0];
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let web_prev = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Previous Month")
    })
    .expect("web prev-month button");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(
        !web_day_buttons.is_empty(),
        "expected calendar day buttons for {web_name}"
    );

    let web_weekday_headers = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_selected_dates: Vec<time::Date> = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
        .map(|(d, _)| d)
        .collect();
    let selected_date = match web_selected_dates.as_slice() {
        [] => None,
        [d] => Some(*d),
        _ => None,
    };

    let web_show_outside_days = web_day_buttons.len() != days_in_month(year, month) as usize;
    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let web_sample = web_day_buttons[0];
    let web_sample_label = web_sample
        .attrs
        .get("aria-label")
        .expect("web sample day aria-label")
        .clone();

    let cell_size = parse_calendar_cell_size_px(theme);

    let chrome_override = {
        let mut chrome = fret_ui_kit::ChromeRefinement::default();
        if (web_padding_left - 0.0).abs() < 0.5 {
            chrome = chrome.p(fret_ui_kit::Space::N0);
        } else if (web_padding_left - 12.0).abs() < 0.5 {
            chrome = chrome.p(fret_ui_kit::Space::N3);
        } else if (web_padding_left - 8.0).abs() < 0.5 {
            chrome = chrome.p(fret_ui_kit::Space::N2);
        }
        if web_border_left >= 0.5 {
            chrome = chrome.border_1();
        }
        chrome
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;

        let month_model: Model<CalendarMonth> =
            cx.app.models_mut().insert(CalendarMonth::new(year, month));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(selected_date);

        let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
            .week_start(week_start)
            .show_outside_days(web_show_outside_days)
            .disable_outside_days(web_disable_outside_days)
            .show_week_number(web_show_week_number)
            .refine_style(chrome_override);
        if let Some(cell_size) = cell_size {
            calendar = calendar.cell_size(cell_size);
        }
        if let Some(today) = web_today {
            calendar = calendar.today(today);
        }

        let calendar = calendar.into_element(cx);
        let calendar = cx.container(
            fret_ui::element::ContainerProps {
                layout: {
                    let mut layout = fret_ui::element::LayoutStyle::default();
                    layout.size.width = fret_ui::element::Length::Fill;
                    layout.size.height = fret_ui::element::Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |_cx| vec![calendar],
        );

        vec![calendar]
    });

    let prev = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Go to the Previous Month"),
    )
    .expect("fret prev-month semantics node");
    assert_close_px(
        &format!("{web_name} prev button width"),
        prev.bounds.size.width,
        web_prev.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} prev button height"),
        prev.bounds.size.height,
        web_prev.rect.h,
        1.0,
    );

    let day = find_semantics(&snap, SemanticsRole::Button, Some(&web_sample_label)).unwrap_or_else(
        || panic!("fret day semantics node for {web_name} label={web_sample_label:?}"),
    );
    assert_close_px(
        &format!("{web_name} day button width"),
        day.bounds.size.width,
        web_sample.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} day button height"),
        day.bounds.size.height,
        web_sample.rect.h,
        1.0,
    );

    let node_bounds = ui.debug_node_bounds(day.id).expect("fret day node bounds");
    assert_close_px(
        &format!("{web_name} day x"),
        node_bounds.origin.x,
        web_sample.rect.x,
        3.0,
    );
    assert_close_px(
        &format!("{web_name} day y"),
        node_bounds.origin.y,
        web_sample.rect.y,
        3.0,
    );
}

fn assert_calendar_multi_month_variant_geometry_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    fn parse_css_px(s: &str) -> Option<f32> {
        s.strip_suffix("px")?.parse::<f32>().ok()
    }

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_padding_left = web_rdp_root
        .computed_style
        .get("paddingLeft")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);
    let web_border_left = web_rdp_root
        .computed_style
        .get("borderLeftWidth")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();

    let mut web_month_grids = find_all(&theme.root, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    web_month_grids.sort_by(|a, b| {
        let by_y = a.rect.y.total_cmp(&b.rect.y);
        if !matches!(by_y, std::cmp::Ordering::Equal) {
            return by_y;
        }
        a.rect.x.total_cmp(&b.rect.x)
    });
    assert_eq!(
        web_month_grids.len(),
        2,
        "expected two month grids for {web_name}"
    );

    let month_labels: Vec<(time::Month, i32)> = web_month_grids
        .iter()
        .map(|grid| {
            let label = grid
                .attrs
                .get("aria-label")
                .expect("web month grid aria-label");
            parse_calendar_title_label(label).expect("web month label (Month YYYY)")
        })
        .collect();
    let (month_a, year_a) = month_labels[0];
    let (month_b, year_b) = month_labels[1];

    let locale = web_month_grids
        .first()
        .and_then(|grid| grid.attrs.get("aria-label"))
        .and_then(|label| label.chars().next())
        .map(|c| {
            if c.is_ascii_uppercase() {
                fret_ui_shadcn::calendar::CalendarLocale::En
            } else {
                fret_ui_shadcn::calendar::CalendarLocale::Es
            }
        })
        .unwrap_or(fret_ui_shadcn::calendar::CalendarLocale::En);

    let in_view = |d: time::Date| {
        (d.month() == month_a && d.year() == year_a) || (d.month() == month_b && d.year() == year_b)
    };

    let web_prev = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Previous Month")
    })
    .expect("web prev-month button");
    let web_next = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Next Month")
    })
    .expect("web next-month button");

    let web_disable_navigation = web_prev
        .attrs
        .get("aria-disabled")
        .is_some_and(|v| v == "true")
        && web_next
            .attrs
            .get("aria-disabled")
            .is_some_and(|v| v == "true");

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(
        !web_day_buttons.is_empty(),
        "expected calendar day buttons for {web_name}"
    );

    let web_weekday_headers = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_selected_dates: Vec<time::Date> = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
        .map(|(d, _)| d)
        .collect();

    let web_is_range_mode = find_first(&theme.root, &|n| {
        class_has_token(n, "rdp-range_start")
            || class_has_token(n, "rdp-range_middle")
            || class_has_token(n, "rdp-range_end")
    })
    .is_some();

    let web_selected = web_day_buttons
        .iter()
        .find(|n| {
            n.attrs
                .get("aria-label")
                .is_some_and(|label| label.as_str().ends_with(", selected"))
        })
        .copied();
    let selected_date = match web_selected_dates.as_slice() {
        [] => None,
        [d] => Some(*d),
        _ => None,
    };

    let web_show_outside_days =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-outside")).is_some();
    let web_has_out_of_view_days = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).map(|(d, _)| d))
        .any(|d| !in_view(d));

    let web_month_bounds =
        if web_disable_navigation && web_show_outside_days && !web_has_out_of_view_days {
            Some(((month_a, year_a), (month_b, year_b)))
        } else {
            None
        };

    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if in_view(date) {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let web_sample = web_selected.unwrap_or(web_day_buttons[0]);
    let web_sample_label = web_sample
        .attrs
        .get("aria-label")
        .expect("web sample day aria-label")
        .clone();

    let cell_size = parse_calendar_cell_size_px(theme);

    let chrome_override = {
        let mut chrome = fret_ui_kit::ChromeRefinement::default();
        if (web_padding_left - 0.0).abs() < 0.5 {
            chrome = chrome.p(fret_ui_kit::Space::N0);
        } else if (web_padding_left - 12.0).abs() < 0.5 {
            chrome = chrome.p(fret_ui_kit::Space::N3);
        } else if (web_padding_left - 8.0).abs() < 0.5 {
            chrome = chrome.p(fret_ui_kit::Space::N2);
        }
        if web_border_left >= 0.5 {
            chrome = chrome.border_1();
        }
        chrome
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;
        use fret_ui_headless::calendar::DateRangeSelection;

        let month_model: Model<CalendarMonth> = cx
            .app
            .models_mut()
            .insert(CalendarMonth::new(year_a, month_a));

        match web_selected_dates.as_slice() {
            [] | [_] => {
                let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(selected_date);
                let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
                    .number_of_months(2)
                    .locale(locale)
                    .disable_navigation(web_disable_navigation)
                    .week_start(week_start)
                    .show_outside_days(web_show_outside_days)
                    .disable_outside_days(web_disable_outside_days)
                    .show_week_number(web_show_week_number)
                    .refine_style(chrome_override);
                if let Some(((start_month, start_year), (end_month, end_year))) = web_month_bounds {
                    calendar = calendar.month_bounds(
                        CalendarMonth::new(start_year, start_month),
                        CalendarMonth::new(end_year, end_month),
                    );
                }
                if let Some(cell_size) = cell_size {
                    calendar = calendar.cell_size(cell_size);
                }
                if let Some(today) = web_today {
                    calendar = calendar.today(today);
                }
                vec![cx.container(
                    fret_ui::element::ContainerProps {
                        layout: {
                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width = fret_ui::element::Length::Fill;
                            layout.size.height = fret_ui::element::Length::Fill;
                            layout
                        },
                        padding: fret_core::Edges {
                            left: Px(web_origin_x),
                            top: Px(web_origin_y),
                            right: Px(0.0),
                            bottom: Px(0.0),
                        },
                        ..Default::default()
                    },
                    move |cx| vec![calendar.into_element(cx)],
                )]
            }
            _ if web_is_range_mode => {
                let (min, max) = web_selected_dates.iter().fold(
                    (web_selected_dates[0], web_selected_dates[0]),
                    |(min, max), d| (min.min(*d), max.max(*d)),
                );
                let selected: Model<DateRangeSelection> =
                    cx.app.models_mut().insert(DateRangeSelection {
                        from: Some(min),
                        to: Some(max),
                    });
                let mut calendar = fret_ui_shadcn::CalendarRange::new(month_model, selected)
                    .number_of_months(2)
                    .locale(locale)
                    .disable_navigation(web_disable_navigation)
                    .week_start(week_start)
                    .show_outside_days(web_show_outside_days)
                    .disable_outside_days(web_disable_outside_days)
                    .show_week_number(web_show_week_number)
                    .refine_style(chrome_override);
                if let Some(((start_month, start_year), (end_month, end_year))) = web_month_bounds {
                    calendar = calendar.month_bounds(
                        CalendarMonth::new(start_year, start_month),
                        CalendarMonth::new(end_year, end_month),
                    );
                }
                if let Some(cell_size) = cell_size {
                    calendar = calendar.cell_size(cell_size);
                }
                if let Some(today) = web_today {
                    calendar = calendar.today(today);
                }
                vec![cx.container(
                    fret_ui::element::ContainerProps {
                        layout: {
                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width = fret_ui::element::Length::Fill;
                            layout.size.height = fret_ui::element::Length::Fill;
                            layout
                        },
                        padding: fret_core::Edges {
                            left: Px(web_origin_x),
                            top: Px(web_origin_y),
                            right: Px(0.0),
                            bottom: Px(0.0),
                        },
                        ..Default::default()
                    },
                    move |cx| vec![calendar.into_element(cx)],
                )]
            }
            _ => {
                let selected: Model<Vec<time::Date>> =
                    cx.app.models_mut().insert(web_selected_dates.clone());
                let mut calendar = fret_ui_shadcn::CalendarMultiple::new(month_model, selected)
                    .number_of_months(2)
                    .locale(locale)
                    .disable_navigation(web_disable_navigation)
                    .week_start(week_start)
                    .show_outside_days(web_show_outside_days)
                    .disable_outside_days(web_disable_outside_days)
                    .show_week_number(web_show_week_number)
                    .refine_style(chrome_override);

                if web_name == "calendar-03" {
                    calendar = calendar.required(true).max(5);
                }
                if let Some(((start_month, start_year), (end_month, end_year))) = web_month_bounds {
                    calendar = calendar.month_bounds(
                        CalendarMonth::new(start_year, start_month),
                        CalendarMonth::new(end_year, end_month),
                    );
                }
                if let Some(cell_size) = cell_size {
                    calendar = calendar.cell_size(cell_size);
                }
                if let Some(today) = web_today {
                    calendar = calendar.today(today);
                }

                vec![cx.container(
                    fret_ui::element::ContainerProps {
                        layout: {
                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width = fret_ui::element::Length::Fill;
                            layout.size.height = fret_ui::element::Length::Fill;
                            layout
                        },
                        padding: fret_core::Edges {
                            left: Px(web_origin_x),
                            top: Px(web_origin_y),
                            right: Px(0.0),
                            bottom: Px(0.0),
                        },
                        ..Default::default()
                    },
                    move |cx| vec![calendar.into_element(cx)],
                )]
            }
        }
    });

    let day = find_semantics(&snap, SemanticsRole::Button, Some(&web_sample_label)).unwrap_or_else(
        || panic!("fret day semantics node for {web_name} label={web_sample_label:?}"),
    );
    assert_close_px(
        &format!("{web_name} day button width"),
        day.bounds.size.width,
        web_sample.rect.w,
        1.0,
    );
    assert_close_px(
        &format!("{web_name} day button height"),
        day.bounds.size.height,
        web_sample.rect.h,
        1.0,
    );

    let node_bounds = ui.debug_node_bounds(day.id).expect("fret day node bounds");
    assert_close_px(
        &format!("{web_name} day x"),
        node_bounds.origin.x,
        web_sample.rect.x,
        3.0,
    );
    assert_close_px(
        &format!("{web_name} day y"),
        node_bounds.origin.y,
        web_sample.rect.y,
        3.0,
    );
}

#[test]
fn web_vs_fret_calendar_demo_day_grid_geometry_and_a11y_labels_match_web_targeted() {
    let web = read_web_golden("calendar-demo");
    let theme = web_theme(&web);

    let web_prev = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Previous Month")
    })
    .expect("web prev-month button");

    let web_day = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Sunday, December 28th, 2025")
    })
    .expect("web day button");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        use fret_ui_headless::calendar::CalendarMonth;
        use time::{Month, Weekday};

        let month: Model<CalendarMonth> = cx
            .app
            .models_mut()
            .insert(CalendarMonth::new(2026, Month::January));
        let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);

        vec![
            fret_ui_shadcn::Calendar::new(month, selected)
                .week_start(Weekday::Sunday)
                .disable_outside_days(false)
                .into_element(cx),
        ]
    });

    fn is_calendar_day_label(label: &str) -> bool {
        let label = label.strip_suffix(", selected").unwrap_or(label);
        let label = label.strip_prefix("Today, ").unwrap_or(label);
        if !label.contains(',') {
            return false;
        }
        let Some((_weekday, rest)) = label.split_once(", ") else {
            return false;
        };
        let Some((_month_and_day, year)) = rest.rsplit_once(", ") else {
            return false;
        };
        if year.len() != 4 || !year.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        label.contains("st, ")
            || label.contains("nd, ")
            || label.contains("rd, ")
            || label.contains("th, ")
    }

    let day_buttons = snap
        .nodes
        .iter()
        .filter(|n| {
            n.role == SemanticsRole::Button
                && n.label
                    .as_deref()
                    .is_some_and(|label| is_calendar_day_label(label))
        })
        .count();
    assert_eq!(
        day_buttons, 35,
        "expected a 5-week (35-day) grid for January 2026 when week starts on Sunday"
    );

    let prev = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Go to the Previous Month"),
    )
    .expect("fret prev-month semantics node");
    assert_close_px(
        "calendar-demo prev button width",
        prev.bounds.size.width,
        web_prev.rect.w,
        1.0,
    );
    assert_close_px(
        "calendar-demo prev button height",
        prev.bounds.size.height,
        web_prev.rect.h,
        1.0,
    );

    let day = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Sunday, December 28th, 2025"),
    )
    .expect("fret day semantics node");
    assert_close_px(
        "calendar-demo day button width",
        day.bounds.size.width,
        web_day.rect.w,
        1.0,
    );
    assert_close_px(
        "calendar-demo day button height",
        day.bounds.size.height,
        web_day.rect.h,
        1.0,
    );

    let node_bounds = ui.debug_node_bounds(day.id).expect("fret day node bounds");
    assert_close_px(
        "calendar-demo day x",
        node_bounds.origin.x,
        web_day.rect.x,
        3.0,
    );
    assert_close_px(
        "calendar-demo day y",
        node_bounds.origin.y,
        web_day.rect.y,
        3.0,
    );
}

#[test]
fn web_vs_fret_calendar_hijri_day_grid_geometry_and_a11y_labels_match_web_targeted() {
    let web = read_web_golden("calendar-hijri");
    let theme = web_theme(&web);

    fn parse_css_px(s: &str) -> Option<f32> {
        s.strip_suffix("px")?.parse::<f32>().ok()
    }

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;
    let web_padding_left = web_rdp_root
        .computed_style
        .get("paddingLeft")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);
    let web_border_left = web_rdp_root
        .computed_style
        .get("borderLeftWidth")
        .and_then(|v| parse_css_px(v))
        .unwrap_or(0.0);

    let web_month_grid =
        web_find_by_class_token(&theme.root, "rdp-month_grid").expect("web month grid");
    let web_title = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label")
        .as_str();

    let web_prev = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Previous Month")
    })
    .expect("web prev-month button");
    let web_next = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Next Month")
    })
    .expect("web next-month button");

    const HIJRI_WEEKDAYS: [&str; 7] = [
        "شنبه",
        "یک\u{200c}شنبه",
        "دوشنبه",
        "سه\u{200c}شنبه",
        "چهارشنبه",
        "پنج\u{200c}شنبه",
        "جمعه",
    ];

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| HIJRI_WEEKDAYS.iter().any(|wd| label.starts_with(wd)))
    });
    assert_eq!(
        web_day_buttons.len(),
        42,
        "expected a 6-week (42-day) grid for calendar-hijri"
    );

    let cell_size = parse_calendar_cell_size_px(theme);

    let chrome_override = {
        let mut chrome = fret_ui_kit::ChromeRefinement::default();
        if (web_padding_left - 0.0).abs() < 0.5 {
            chrome = chrome.p(fret_ui_kit::Space::N0);
        } else if (web_padding_left - 12.0).abs() < 0.5 {
            chrome = chrome.p(fret_ui_kit::Space::N3);
        } else if (web_padding_left - 8.0).abs() < 0.5 {
            chrome = chrome.p(fret_ui_kit::Space::N2);
        }
        if web_border_left >= 0.5 {
            chrome = chrome.border_1();
        }
        chrome
    };

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let (ui, snap, _root) = run_fret_root_with_ui_and_services(bounds, &mut services, |cx| {
        use fret_ui_headless::calendar_solar_hijri::SolarHijriMonth;
        use time::{Date, Month};

        let selected_date = Date::from_calendar_date(2025, Month::June, 12).expect("valid date");
        let month = SolarHijriMonth::from_gregorian(selected_date);

        let month_model: Model<SolarHijriMonth> = cx.app.models_mut().insert(month);
        let selected: Model<Option<Date>> = cx.app.models_mut().insert(Some(selected_date));

        let mut cal = fret_ui_shadcn::CalendarHijri::new(month_model, selected)
            .show_outside_days(true)
            .refine_style(chrome_override);
        if let Some(cell_size) = cell_size {
            cal = cal.cell_size(cell_size);
        }

        vec![cx.container(
            fret_ui::element::ContainerProps {
                layout: {
                    let mut layout = fret_ui::element::LayoutStyle::default();
                    layout.size.width = fret_ui::element::Length::Fill;
                    layout.size.height = fret_ui::element::Length::Fill;
                    layout
                },
                padding: fret_core::Edges {
                    left: Px(web_origin_x),
                    top: Px(web_origin_y),
                    right: Px(0.0),
                    bottom: Px(0.0),
                },
                ..Default::default()
            },
            move |cx| vec![cal.into_element(cx)],
        )]
    });

    let prev = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Go to the Previous Month"),
    )
    .expect("fret prev-month semantics node");
    let next = find_semantics(&snap, SemanticsRole::Button, Some("Go to the Next Month"))
        .expect("fret next-month semantics node");

    let prev_bounds = ui.debug_node_bounds(prev.id).expect("prev bounds");
    let next_bounds = ui.debug_node_bounds(next.id).expect("next bounds");
    assert_close_px(
        "calendar-hijri prev x",
        prev_bounds.origin.x,
        web_prev.rect.x,
        3.0,
    );
    assert_close_px(
        "calendar-hijri prev y",
        prev_bounds.origin.y,
        web_prev.rect.y,
        3.0,
    );
    assert_close_px(
        "calendar-hijri next x",
        next_bounds.origin.x,
        web_next.rect.x,
        3.0,
    );
    assert_close_px(
        "calendar-hijri next y",
        next_bounds.origin.y,
        web_next.rect.y,
        3.0,
    );

    let title = find_semantics(&snap, SemanticsRole::Text, Some(web_title))
        .expect("fret calendar-hijri title semantics node");
    let web_title_node = find_first(&theme.root, &|n| n.text.as_deref() == Some(web_title))
        .expect("web calendar-hijri title node");

    let title_bounds = ui.debug_node_bounds(title.id).expect("title bounds");
    let title_center_x = title_bounds.origin.x.0 + title_bounds.size.width.0 / 2.0;
    let web_title_center_x = web_title_node.rect.x + web_title_node.rect.w / 2.0;
    assert_close_px(
        "calendar-hijri title center x",
        Px(title_center_x),
        web_title_center_x,
        3.0,
    );

    for web_day in web_day_buttons {
        let label = web_day.attrs.get("aria-label").expect("web day aria-label");
        let fret_day = find_semantics(&snap, SemanticsRole::Button, Some(label.as_str()))
            .unwrap_or_else(|| panic!("missing fret hijri day button label={label:?}"));
        let fret_bounds = ui.debug_node_bounds(fret_day.id).expect("fret day bounds");

        assert_close_px(
            "calendar-hijri day w",
            fret_bounds.size.width,
            web_day.rect.w,
            1.0,
        );
        assert_close_px(
            "calendar-hijri day h",
            fret_bounds.size.height,
            web_day.rect.h,
            1.0,
        );
        assert_close_px(
            "calendar-hijri day x",
            fret_bounds.origin.x,
            web_day.rect.x,
            3.0,
        );
        assert_close_px(
            "calendar-hijri day y",
            fret_bounds.origin.y,
            web_day.rect.y,
            3.0,
        );
    }
}

fn assert_calendar_11_disabled_navigation_semantics_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_month_grids = find_all(&theme.root, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(
        web_month_grids.len(),
        2,
        "expected two month grids for {web_name}"
    );

    let month_labels: Vec<(time::Month, i32)> = web_month_grids
        .iter()
        .map(|grid| {
            let label = grid
                .attrs
                .get("aria-label")
                .expect("web month grid aria-label");
            parse_calendar_title_label(label).expect("web month label (Month YYYY)")
        })
        .collect();
    let (month_a, year_a) = month_labels[0];
    let (month_b, year_b) = month_labels[1];

    let locale = web_month_grids
        .first()
        .and_then(|grid| grid.attrs.get("aria-label"))
        .and_then(|label| label.chars().next())
        .map(|c| {
            if c.is_ascii_uppercase() {
                fret_ui_shadcn::calendar::CalendarLocale::En
            } else {
                fret_ui_shadcn::calendar::CalendarLocale::Es
            }
        })
        .unwrap_or(fret_ui_shadcn::calendar::CalendarLocale::En);

    let in_view = |d: time::Date| {
        (d.month() == month_a && d.year() == year_a) || (d.month() == month_b && d.year() == year_b)
    };

    let web_prev = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Previous Month")
    })
    .expect("web prev-month button");
    let web_next = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Go to the Next Month")
    })
    .expect("web next-month button");

    let web_prev_icon = find_first(web_prev, &|n| n.tag == "svg")
        .unwrap_or_else(|| panic!("web prev-month icon svg for {web_name}"));
    let web_next_icon = find_first(web_next, &|n| n.tag == "svg")
        .unwrap_or_else(|| panic!("web next-month icon svg for {web_name}"));

    let web_prev_icon_color_css = web_prev_icon
        .computed_style
        .get("color")
        .or_else(|| web_prev.computed_style.get("color"))
        .map(String::as_str)
        .expect("web prev icon color");
    let web_next_icon_color_css = web_next_icon
        .computed_style
        .get("color")
        .or_else(|| web_next.computed_style.get("color"))
        .map(String::as_str)
        .expect("web next icon color");

    let web_prev_icon_color = parse_css_color(web_prev_icon_color_css)
        .unwrap_or_else(|| panic!("invalid css color: {web_prev_icon_color_css}"));
    let web_next_icon_color = parse_css_color(web_next_icon_color_css)
        .unwrap_or_else(|| panic!("invalid css color: {web_next_icon_color_css}"));

    let web_prev_icon_opacity = web_prev_icon
        .computed_style
        .get("opacity")
        .or_else(|| web_prev.computed_style.get("opacity"))
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(1.0);
    let web_next_icon_opacity = web_next_icon
        .computed_style
        .get("opacity")
        .or_else(|| web_next.computed_style.get("opacity"))
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(1.0);

    let expected_prev_icon = apply_opacity(web_prev_icon_color, web_prev_icon_opacity);
    let expected_next_icon = apply_opacity(web_next_icon_color, web_next_icon_opacity);

    let web_prev_disabled = web_prev
        .attrs
        .get("aria-disabled")
        .is_some_and(|v| v == "true");
    let web_next_disabled = web_next
        .attrs
        .get("aria-disabled")
        .is_some_and(|v| v == "true");

    let web_disable_navigation = web_prev
        .attrs
        .get("aria-disabled")
        .is_some_and(|v| v == "true")
        && web_next
            .attrs
            .get("aria-disabled")
            .is_some_and(|v| v == "true");

    let week_start = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    })
    .iter()
    .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
    .and_then(|n| n.attrs.get("aria-label"))
    .and_then(|label| parse_calendar_weekday_label(label))
    .unwrap_or(time::Weekday::Sunday);

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(
        !web_day_buttons.is_empty(),
        "expected calendar day buttons for {web_name}"
    );

    let web_today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let web_show_outside_days =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-outside")).is_some();

    let web_has_out_of_view_days = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).map(|(d, _)| d))
        .any(|d| !in_view(d));

    let web_month_bounds =
        if web_disable_navigation && web_show_outside_days && !web_has_out_of_view_days {
            Some(((month_a, year_a), (month_b, year_b)))
        } else {
            None
        };

    let web_disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if in_view(date) {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let outside_disabled = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label").map(|l| (*n, l)))
        .filter_map(|(n, label)| {
            let (date, _selected) = parse_calendar_day_aria_label(label)?;
            if in_view(date) {
                return None;
            }
            let disabled = n.attrs.contains_key("disabled")
                || n.attrs.get("aria-disabled").is_some_and(|v| v == "true");
            if !disabled {
                return None;
            }
            Some((n, label))
        })
        .next();

    let outside_expected_fg = outside_disabled.and_then(|(n, label)| {
        let web_color_css = n.computed_style.get("color")?.as_str();
        let web_color = parse_css_color(web_color_css)
            .unwrap_or_else(|| panic!("invalid css color: {web_color_css}"));
        let web_opacity = n
            .computed_style
            .get("opacity")
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(1.0);
        Some((label.to_string(), apply_opacity(web_color, web_opacity)))
    });

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let origin_x = web_rdp_root.rect.x;
    let origin_y = web_rdp_root.rect.y;

    let (snap, scene) = render_calendar_in_bounds_with_scene(
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
        ),
        move |cx| {
            use fret_ui_headless::calendar::CalendarMonth;

            let month_model: Model<CalendarMonth> = cx
                .app
                .models_mut()
                .insert(CalendarMonth::new(year_a, month_a));
            let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);

            let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
                .locale(locale)
                .week_start(week_start)
                .number_of_months(2)
                .disable_navigation(web_disable_navigation)
                .show_outside_days(web_show_outside_days)
                .disable_outside_days(web_disable_outside_days)
                .show_week_number(web_show_week_number);

            if let Some(today) = web_today {
                calendar = calendar.today(today);
            }
            if let Some(((start_month, start_year), (end_month, end_year))) = web_month_bounds {
                calendar = calendar.month_bounds(
                    CalendarMonth::new(start_year, start_month),
                    CalendarMonth::new(end_year, end_month),
                );
            }

            let calendar = calendar.into_element(cx);
            let calendar = cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        layout
                    },
                    padding: fret_core::Edges {
                        left: Px(origin_x),
                        top: Px(origin_y),
                        right: Px(0.0),
                        bottom: Px(0.0),
                    },
                    ..Default::default()
                },
                move |_cx| vec![calendar],
            );

            vec![calendar]
        },
    );

    let fret_prev = find_semantics(
        &snap,
        SemanticsRole::Button,
        Some("Go to the Previous Month"),
    )
    .expect("fret prev-month semantics node");
    assert!(
        fret_prev.flags.disabled == web_prev_disabled,
        "expected prev-month semantics flags.disabled={web_prev_disabled}"
    );

    let fret_next = find_semantics(&snap, SemanticsRole::Button, Some("Go to the Next Month"))
        .expect("fret next-month semantics node");
    assert!(
        fret_next.flags.disabled == web_next_disabled,
        "expected next-month semantics flags.disabled={web_next_disabled}"
    );

    let fret_prev_icon = find_best_icon_color_in_rect(&scene, fret_prev.bounds)
        .unwrap_or_else(|| panic!("painted prev icon for {web_name}"));
    assert_rgba_close(
        &format!("{web_name} prev icon color"),
        fret_prev_icon,
        expected_prev_icon,
        0.02,
    );

    let fret_next_icon = find_best_icon_color_in_rect(&scene, fret_next.bounds)
        .unwrap_or_else(|| panic!("painted next icon for {web_name}"));
    assert_rgba_close(
        &format!("{web_name} next icon color"),
        fret_next_icon,
        expected_next_icon,
        0.02,
    );

    if let Some((outside_label, expected)) = outside_expected_fg {
        let fret_outside =
            find_semantics(&snap, SemanticsRole::Button, Some(outside_label.as_str()))
                .unwrap_or_else(|| {
                    panic!("missing fret outside-day button label={outside_label:?} for {web_name}")
                });
        assert!(
            fret_outside.flags.disabled,
            "expected outside-day semantics flags.disabled=true for {outside_label:?} ({web_name})"
        );

        let fret_fg = find_best_text_color_in_rect(&scene, fret_outside.bounds)
            .unwrap_or_else(|| panic!("painted outside-day text for {web_name}"));
        assert_rgba_close(
            &format!("{web_name} outside-day disabled text color"),
            fret_fg,
            expected,
            0.02,
        );
    }
}

#[test]
fn web_vs_fret_calendar_11_disabled_navigation_semantics_matches_web() {
    assert_calendar_11_disabled_navigation_semantics_matches_web("calendar-11");
}

#[test]
fn web_vs_fret_calendar_11_vp375x320_disabled_navigation_semantics_matches_web() {
    assert_calendar_11_disabled_navigation_semantics_matches_web("calendar-11.vp375x320");
}

fn assert_calendar_08_disabled_day_semantics_matches_web(web_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_month_grid =
        web_find_by_class_token_in_theme(theme, "rdp-month_grid").expect("web month grid");
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let locale = web_month_label
        .chars()
        .next()
        .map(|c| {
            if c.is_ascii_uppercase() {
                fret_ui_shadcn::calendar::CalendarLocale::En
            } else {
                fret_ui_shadcn::calendar::CalendarLocale::Es
            }
        })
        .unwrap_or(fret_ui_shadcn::calendar::CalendarLocale::En);

    let web_weekday_headers = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = web_weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let web_show_outside_days =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-outside")).is_some();

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let origin_x = web_rdp_root.rect.x;
    let origin_y = web_rdp_root.rect.y;

    let web_disabled_cells: Vec<&WebNode> = find_all(&theme.root, &|n| {
        n.tag == "td"
            && (class_has_token(n, "rdp-disabled")
                || n.attrs.get("data-disabled").is_some_and(|v| v == "true"))
    });
    assert!(
        !web_disabled_cells.is_empty(),
        "expected at least one disabled day cell in calendar-08"
    );

    let mut disabled_dates = std::collections::HashSet::<time::Date>::new();
    let mut disabled_labels: Vec<String> = Vec::new();
    let mut disabled_button_node: Option<&WebNode> = None;
    for cell in web_disabled_cells {
        let Some(button) = find_first(cell, &|n| {
            n.tag == "button"
                && n.attrs
                    .get("aria-label")
                    .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
        }) else {
            continue;
        };
        let label = button
            .attrs
            .get("aria-label")
            .expect("web disabled day button aria-label");
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            continue;
        };
        if date.month() != month || date.year() != year {
            continue;
        }
        if disabled_dates.insert(date) {
            disabled_labels.push(label.clone());
            if disabled_button_node.is_none() {
                disabled_button_node = Some(button);
            }
        }
    }
    assert!(
        !disabled_labels.is_empty(),
        "expected disabled day labels within the primary month for calendar-08"
    );

    let disabled_label = disabled_labels
        .iter()
        .find(|l| !l.starts_with("Today, ") && !l.starts_with("Hoy, "))
        .cloned()
        .unwrap_or_else(|| disabled_labels[0].clone());

    let disabled_button_node = disabled_button_node.expect("web disabled day button node");
    let web_disabled_color_css = disabled_button_node
        .computed_style
        .get("color")
        .map(String::as_str)
        .expect("web disabled day color");
    let web_disabled_color = parse_css_color(web_disabled_color_css)
        .unwrap_or_else(|| panic!("invalid css color: {web_disabled_color_css}"));
    let web_disabled_opacity = disabled_button_node
        .computed_style
        .get("opacity")
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(1.0);
    let expected_disabled_fg = apply_opacity(web_disabled_color, web_disabled_opacity);

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(
        !web_day_buttons.is_empty(),
        "expected calendar day buttons for {web_name}"
    );

    let enabled_label = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter(|label| !label.starts_with("Today, ") && !label.starts_with("Hoy, "))
        .filter_map(|label| {
            let (date, _selected) = parse_calendar_day_aria_label(label)?;
            if date.month() != month || date.year() != year {
                return None;
            }
            if disabled_dates.contains(&date) {
                return None;
            }
            Some(label.clone())
        })
        .next()
        .expect("expected at least one enabled in-month day button in calendar-08");

    let disabled_dates = Arc::new(disabled_dates);
    let (snap, scene) = render_calendar_in_bounds_with_scene(
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
        ),
        move |cx| {
            use fret_ui_headless::calendar::CalendarMonth;

            let month_model: Model<CalendarMonth> =
                cx.app.models_mut().insert(CalendarMonth::new(year, month));
            let selected: Model<Option<time::Date>> = cx.app.models_mut().insert(None);

            let mut calendar = fret_ui_shadcn::Calendar::new(month_model, selected)
                .locale(locale)
                .week_start(week_start)
                .show_outside_days(web_show_outside_days)
                .show_week_number(web_show_week_number)
                .disabled_by({
                    let disabled_dates = Arc::clone(&disabled_dates);
                    move |d| disabled_dates.contains(&d)
                });

            let web_today = web_day_buttons
                .iter()
                .filter_map(|n| n.attrs.get("aria-label"))
                .find(|label| label.starts_with("Today, ") || label.starts_with("Hoy, "))
                .and_then(|label| parse_calendar_day_aria_label(label))
                .map(|(d, _)| d);
            if let Some(today) = web_today {
                calendar = calendar.today(today);
            }

            let calendar = calendar.into_element(cx);
            let calendar = cx.container(
                fret_ui::element::ContainerProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        layout
                    },
                    padding: fret_core::Edges {
                        left: Px(origin_x),
                        top: Px(origin_y),
                        right: Px(0.0),
                        bottom: Px(0.0),
                    },
                    ..Default::default()
                },
                move |_cx| vec![calendar],
            );

            vec![calendar]
        },
    );

    let fret_disabled = find_semantics(&snap, SemanticsRole::Button, Some(&disabled_label))
        .unwrap_or_else(|| panic!("missing fret disabled day button label={disabled_label:?}"));
    assert!(
        fret_disabled.flags.disabled,
        "expected semantics flags.disabled=true for {disabled_label:?}"
    );

    let fret_disabled_fg = find_best_text_color_in_rect(&scene, fret_disabled.bounds)
        .unwrap_or_else(|| panic!("painted disabled day text for {web_name}"));
    assert_rgba_close(
        &format!("{web_name} disabled day text color"),
        fret_disabled_fg,
        expected_disabled_fg,
        0.02,
    );

    let fret_enabled = find_semantics(&snap, SemanticsRole::Button, Some(&enabled_label))
        .unwrap_or_else(|| panic!("missing fret enabled day button label={enabled_label:?}"));
    assert!(
        !fret_enabled.flags.disabled,
        "expected semantics flags.disabled=false for {enabled_label:?}"
    );
}

#[test]
fn web_vs_fret_calendar_08_disabled_day_semantics_matches_web() {
    assert_calendar_08_disabled_day_semantics_matches_web("calendar-08");
}

#[test]
fn web_vs_fret_calendar_08_vp375x320_disabled_day_semantics_matches_web() {
    assert_calendar_08_disabled_day_semantics_matches_web("calendar-08.vp375x320");
}

macro_rules! calendar_single_month_geometry_test {
    ($test_name:ident, $web_name:expr) => {
        #[test]
        fn $test_name() {
            assert_calendar_single_month_variant_geometry_matches_web($web_name);
        }
    };
}

macro_rules! calendar_multi_month_geometry_test {
    ($test_name:ident, $web_name:expr) => {
        #[test]
        fn $test_name() {
            assert_calendar_multi_month_variant_geometry_matches_web($web_name);
        }
    };
}

calendar_single_month_geometry_test!(
    web_vs_fret_calendar_01_geometry_matches_web_targeted,
    "calendar-01"
);
calendar_multi_month_geometry_test!(
    web_vs_fret_calendar_02_geometry_matches_web_targeted,
    "calendar-02"
);
calendar_multi_month_geometry_test!(
    web_vs_fret_calendar_03_geometry_matches_web_targeted,
    "calendar-03"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_04_geometry_matches_web_targeted,
    "calendar-04"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_04_vp375x320_geometry_matches_web_targeted,
    "calendar-04.vp375x320"
);
calendar_multi_month_geometry_test!(
    web_vs_fret_calendar_05_geometry_matches_web_targeted,
    "calendar-05"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_06_geometry_matches_web_targeted,
    "calendar-06"
);
calendar_multi_month_geometry_test!(
    web_vs_fret_calendar_07_geometry_matches_web_targeted,
    "calendar-07"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_08_geometry_matches_web_targeted,
    "calendar-08"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_08_vp375x320_geometry_matches_web_targeted,
    "calendar-08.vp375x320"
);
calendar_multi_month_geometry_test!(
    web_vs_fret_calendar_09_geometry_matches_web_targeted,
    "calendar-09"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_10_geometry_matches_web_targeted,
    "calendar-10"
);
calendar_multi_month_geometry_test!(
    web_vs_fret_calendar_11_geometry_matches_web_targeted,
    "calendar-11"
);
calendar_multi_month_geometry_test!(
    web_vs_fret_calendar_11_vp375x320_geometry_matches_web_targeted,
    "calendar-11.vp375x320"
);
calendar_multi_month_geometry_test!(
    web_vs_fret_calendar_12_geometry_matches_web_targeted,
    "calendar-12"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_13_geometry_matches_web_targeted,
    "calendar-13"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_14_geometry_matches_web_targeted,
    "calendar-14"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_14_vp375x320_geometry_matches_web_targeted,
    "calendar-14.vp375x320"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_14_hover_day_13_geometry_matches_web_targeted,
    "calendar-14.hover-day-13"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_14_hover_day_13_vp375x320_geometry_matches_web_targeted,
    "calendar-14.hover-day-13-vp375x320"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_15_geometry_matches_web_targeted,
    "calendar-15"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_16_geometry_matches_web_targeted,
    "calendar-16"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_17_geometry_matches_web_targeted,
    "calendar-17"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_18_geometry_matches_web_targeted,
    "calendar-18"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_19_geometry_matches_web_targeted,
    "calendar-19"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_20_geometry_matches_web_targeted,
    "calendar-20"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_21_geometry_matches_web_targeted,
    "calendar-21"
);
calendar_single_month_geometry_test!(
    web_vs_fret_calendar_31_geometry_matches_web_targeted,
    "calendar-31"
);

fn assert_calendar_selected_day_background_matches_web(
    web_name: &str,
    quad_label: &str,
    bg_label: &str,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grid = find_first(&theme.root, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    })
    .expect("web month grid");
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let locale = web_month_label
        .chars()
        .next()
        .map(|c| {
            if c.is_ascii_uppercase() {
                fret_ui_shadcn::calendar::CalendarLocale::En
            } else {
                fret_ui_shadcn::calendar::CalendarLocale::Es
            }
        })
        .unwrap_or(fret_ui_shadcn::calendar::CalendarLocale::En);

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let web_selected_cell = find_first(&theme.root, &|n| {
        n.attrs.get("aria-selected").is_some_and(|v| v == "true")
    })
    .expect("web selected calendar cell (aria-selected=true)");
    let web_selected_button = find_first(web_selected_cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    })
    .expect("web selected day button");
    let web_selected_label = web_selected_button
        .attrs
        .get("aria-label")
        .expect("web selected day aria-label");
    let (selected_date, _selected_suffix) = parse_calendar_day_aria_label(web_selected_label)
        .unwrap_or_else(|| panic!("invalid web selected day aria-label: {web_selected_label}"));

    let web_bg_css = web_selected_button
        .computed_style
        .get("backgroundColor")
        .expect("web selected day backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let weekday_headers = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, ") || label.starts_with("Hoy, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let cell_size =
        parse_calendar_cell_size_px(theme).unwrap_or_else(|| Px(web_selected_button.rect.w));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );
    let config = CalendarChromeConfig {
        locale,
        month,
        year,
        origin_x: web_origin_x,
        origin_y: web_origin_y,
        cell_size,
        week_start,
        today,
        show_week_number,
        show_outside_days,
        disable_outside_days,
        selected: selected_date,
    };
    let (_snap, scene) = render_calendar_in_bounds_with_scene(bounds, move |cx| {
        render_calendar_chrome_from_config(cx, config)
    });

    let target = Rect::new(
        Point::new(
            Px(web_selected_button.rect.x),
            Px(web_selected_button.rect.y),
        ),
        CoreSize::new(
            Px(web_selected_button.rect.w),
            Px(web_selected_button.rect.h),
        ),
    );
    let quad = find_best_opaque_background_quad(&scene, target)
        .expect("painted opaque selected day background quad");

    assert_rect_xwh_close_px(quad_label, quad.rect, web_selected_button.rect, 3.0);
    assert_rgba_close(bg_label, color_to_rgba(quad.background), expected_bg, 0.02);
}

#[test]
fn web_vs_fret_calendar_14_selected_day_background_matches_web() {
    assert_calendar_selected_day_background_matches_web(
        "calendar-14",
        "calendar-14 selected day quad",
        "calendar-14 selected day background",
    );
}

#[test]
fn web_vs_fret_calendar_14_vp375x320_selected_day_background_matches_web() {
    assert_calendar_selected_day_background_matches_web(
        "calendar-14.vp375x320",
        "calendar-14.vp375x320 selected day quad",
        "calendar-14.vp375x320 selected day background",
    );
}

fn assert_calendar_hover_day_background_matches_web(
    web_name: &str,
    quad_label: &str,
    bg_label: &str,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grid = find_first(&theme.root, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    })
    .expect("web month grid");
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let locale = web_month_label
        .chars()
        .next()
        .map(|c| {
            if c.is_ascii_uppercase() {
                fret_ui_shadcn::calendar::CalendarLocale::En
            } else {
                fret_ui_shadcn::calendar::CalendarLocale::Es
            }
        })
        .unwrap_or(fret_ui_shadcn::calendar::CalendarLocale::En);

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let mut selected_dates: Vec<time::Date> = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .filter_map(|label| parse_calendar_day_aria_label(label).filter(|(_, sel)| *sel))
        .map(|(d, _)| d)
        .collect();
    selected_dates.sort();
    selected_dates.dedup();
    assert!(
        !selected_dates.is_empty(),
        "expected at least one selected day for hover gating"
    );

    let web_hovered_button = web_day_buttons
        .iter()
        .filter(|n| {
            n.computed_style
                .get("backgroundColor")
                .is_some_and(|v| v != "rgba(0, 0, 0, 0)")
        })
        .find(|n| {
            n.attrs.get("aria-label").is_some_and(|label| {
                parse_calendar_day_aria_label(label.as_str()).is_some_and(|(_, sel)| !sel)
            })
        })
        .copied()
        .expect("web hovered day button (non-transparent backgroundColor)");
    let web_hovered_label = web_hovered_button
        .attrs
        .get("aria-label")
        .expect("web hovered day aria-label")
        .to_string();

    let web_bg_css = web_hovered_button
        .computed_style
        .get("backgroundColor")
        .expect("web hovered day backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let weekday_headers = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, ") || label.starts_with("Hoy, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let cell_size =
        parse_calendar_cell_size_px(theme).unwrap_or_else(|| Px(web_day_buttons[0].rect.w));
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let web_is_range_mode = find_first_in_theme(theme, &|n| {
        class_has_token(n, "rdp-range_start")
            || class_has_token(n, "rdp-range_middle")
            || class_has_token(n, "rdp-range_end")
    })
    .is_some();

    let selection = if web_is_range_mode {
        assert!(
            selected_dates.len() >= 2,
            "expected at least 2 selected dates for range mode hover gating"
        );
        let (min, max) = selected_dates
            .iter()
            .copied()
            .fold((selected_dates[0], selected_dates[0]), |(min, max), d| {
                (min.min(d), max.max(d))
            });
        CalendarSelectionMode::Range { min, max }
    } else if selected_dates.len() > 1 {
        CalendarSelectionMode::Multiple(selected_dates.clone())
    } else {
        CalendarSelectionMode::Single(selected_dates[0])
    };

    let config = CalendarHoverChromeConfig {
        locale,
        month,
        year,
        origin_x: web_origin_x,
        origin_y: web_origin_y,
        cell_size,
        week_start,
        today,
        show_week_number,
        show_outside_days,
        disable_outside_days,
        selection,
        calendar_03_multiple_contract: web_name.starts_with("calendar-03"),
    };

    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let config2 = config.clone();
    let config1 = config;
    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-calendar-hover",
        move |cx| render_calendar_hover_chrome_from_config(cx, config1),
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap1 = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot (pre-hover)");
    let hover_button1 = find_semantics(&snap1, SemanticsRole::Button, Some(&web_hovered_label))
        .expect("fret hovered day button semantics node (pre-hover)");
    let hover_pos = Point::new(
        Px(hover_button1.bounds.origin.x.0 + hover_button1.bounds.size.width.0 * 0.5),
        Px(hover_button1.bounds.origin.y.0 + hover_button1.bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: hover_pos,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    let root_node = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-calendar-hover",
        move |cx| render_calendar_hover_chrome_from_config(cx, config2),
    );
    ui.set_root(root_node);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let target = Rect::new(
        Point::new(Px(web_hovered_button.rect.x), Px(web_hovered_button.rect.y)),
        CoreSize::new(Px(web_hovered_button.rect.w), Px(web_hovered_button.rect.h)),
    );
    let quad = find_best_opaque_background_quad(&scene, target)
        .expect("painted opaque hovered day background quad");

    assert_rect_xwh_close_px(quad_label, quad.rect, web_hovered_button.rect, 3.0);
    assert_rgba_close(bg_label, color_to_rgba(quad.background), expected_bg, 0.02);
}

#[test]
fn web_vs_fret_calendar_14_hover_day_background_matches_web() {
    assert_calendar_hover_day_background_matches_web(
        "calendar-14.hover-day-13",
        "calendar-14 hover day quad",
        "calendar-14 hover day background",
    );
}

#[test]
fn web_vs_fret_calendar_14_vp375x320_hover_day_background_matches_web() {
    assert_calendar_hover_day_background_matches_web(
        "calendar-14.hover-day-13-vp375x320",
        "calendar-14.vp375x320 hover day quad",
        "calendar-14.vp375x320 hover day background",
    );
}

#[test]
fn web_vs_fret_calendar_03_hover_day_background_matches_web() {
    assert_calendar_hover_day_background_matches_web(
        "calendar-03.hover-day-june-11",
        "calendar-03 hover day quad",
        "calendar-03 hover day background",
    );
}

#[test]
fn web_vs_fret_calendar_03_vp375x320_hover_day_background_matches_web() {
    assert_calendar_hover_day_background_matches_web(
        "calendar-03.hover-day-june-11-vp375x320",
        "calendar-03.vp375x320 hover day quad",
        "calendar-03.vp375x320 hover day background",
    );
}

#[test]
fn web_vs_fret_calendar_04_hover_day_background_matches_web() {
    assert_calendar_hover_day_background_matches_web(
        "calendar-04.hover-day-june-5",
        "calendar-04 hover day quad",
        "calendar-04 hover day background",
    );
}

#[test]
fn web_vs_fret_calendar_04_vp375x320_hover_day_background_matches_web() {
    assert_calendar_hover_day_background_matches_web(
        "calendar-04.hover-day-june-5-vp375x320",
        "calendar-04.vp375x320 hover day quad",
        "calendar-04.vp375x320 hover day background",
    );
}

fn assert_calendar_selected_day_text_centered_in_button(
    web_name: &str,
    x_label: &str,
    y_label: &str,
) {
    let web = read_web_golden(web_name);
    let theme = web_theme(&web);

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_origin_x = web_rdp_root.rect.x;
    let web_origin_y = web_rdp_root.rect.y;

    let web_month_grid = find_first(&theme.root, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    })
    .expect("web month grid");
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let locale = web_month_label
        .chars()
        .next()
        .map(|c| {
            if c.is_ascii_uppercase() {
                fret_ui_shadcn::calendar::CalendarLocale::En
            } else {
                fret_ui_shadcn::calendar::CalendarLocale::Es
            }
        })
        .unwrap_or(fret_ui_shadcn::calendar::CalendarLocale::En);

    let web_day_buttons = find_all(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let web_selected_cell = find_first(&theme.root, &|n| {
        n.attrs.get("aria-selected").is_some_and(|v| v == "true")
    })
    .expect("web selected calendar cell (aria-selected=true)");
    let web_selected_button = find_first(web_selected_cell, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    })
    .expect("web selected day button");
    let web_selected_label = web_selected_button
        .attrs
        .get("aria-label")
        .expect("web selected day aria-label");
    let (selected_date, _selected_suffix) = parse_calendar_day_aria_label(web_selected_label)
        .unwrap_or_else(|| panic!("invalid web selected day aria-label: {web_selected_label}"));

    let web_day_number = {
        let mut stack = vec![web_selected_button];
        let mut best: Option<&WebNode> = None;
        while let Some(node) = stack.pop() {
            for child in &node.children {
                stack.push(child);
            }

            let Some(text) = node.text.as_deref() else {
                continue;
            };
            let text = text.trim();
            if text.is_empty() || text.len() > 2 || !text.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            best = Some(node);
        }
        best.expect("web selected day number text node")
    };

    let weekday_headers = find_all(&theme.root, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let today = web_day_buttons
        .iter()
        .filter_map(|n| n.attrs.get("aria-label"))
        .find(|label| label.starts_with("Today, ") || label.starts_with("Hoy, "))
        .and_then(|label| parse_calendar_day_aria_label(label))
        .map(|(d, _)| d);

    let show_week_number =
        find_first(&theme.root, &|n| class_has_token(n, "rdp-week_number")).is_some();
    let show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let cell_size =
        parse_calendar_cell_size_px(theme).unwrap_or_else(|| Px(web_selected_button.rect.w));

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );
    let config = CalendarChromeConfig {
        locale,
        month,
        year,
        origin_x: web_origin_x,
        origin_y: web_origin_y,
        cell_size,
        week_start,
        today,
        show_week_number,
        show_outside_days,
        disable_outside_days,
        selected: selected_date,
    };

    let (_ui, snap, _root) = render_calendar_in_bounds(bounds, move |cx| {
        render_calendar_chrome_from_config(cx, config)
    });

    let fret_selected_button =
        find_semantics(&snap, SemanticsRole::Button, Some(web_selected_label))
            .expect("fret selected day button semantics node");

    let fret_day_number_text = {
        let want = web_day_number.text.as_deref().unwrap_or("").trim();
        assert!(!want.is_empty(), "expected web day number text");

        let mut candidates: Vec<&fret_core::SemanticsNode> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Text)
            .filter(|n| n.label.as_deref() == Some(want))
            .filter(|n| {
                let eps = 0.01;
                let outer = fret_selected_button.bounds;
                let inner = n.bounds;
                inner.origin.x.0 + eps >= outer.origin.x.0
                    && inner.origin.y.0 + eps >= outer.origin.y.0
                    && inner.origin.x.0 + inner.size.width.0
                        <= outer.origin.x.0 + outer.size.width.0 + eps
                    && inner.origin.y.0 + inner.size.height.0
                        <= outer.origin.y.0 + outer.size.height.0 + eps
            })
            .collect();

        candidates.sort_by(|a, b| {
            let aw = a.bounds.size.width.0;
            let bw = b.bounds.size.width.0;
            bw.total_cmp(&aw)
        });
        candidates
            .first()
            .copied()
            .unwrap_or_else(|| panic!("missing fret selected day number text node label={want:?}"))
    };

    let fret_button_cx =
        fret_selected_button.bounds.origin.x.0 + fret_selected_button.bounds.size.width.0 / 2.0;
    let fret_button_cy =
        fret_selected_button.bounds.origin.y.0 + fret_selected_button.bounds.size.height.0 / 2.0;
    let fret_text_cx =
        fret_day_number_text.bounds.origin.x.0 + fret_day_number_text.bounds.size.width.0 / 2.0;
    let fret_text_cy =
        fret_day_number_text.bounds.origin.y.0 + fret_day_number_text.bounds.size.height.0 / 2.0;

    assert_close_px(x_label, Px(fret_text_cx), fret_button_cx, 2.5);
    assert_close_px(y_label, Px(fret_text_cy), fret_button_cy, 2.5);
}

#[test]
fn web_vs_fret_calendar_14_selected_day_text_centered_in_button() {
    assert_calendar_selected_day_text_centered_in_button(
        "calendar-14",
        "calendar-14 day number center x ~= button center x",
        "calendar-14 day number center y ~= button center y",
    );
}

#[test]
fn web_vs_fret_calendar_14_vp375x320_selected_day_text_centered_in_button() {
    assert_calendar_selected_day_text_centered_in_button(
        "calendar-14.vp375x320",
        "calendar-14.vp375x320 day number center x ~= button center x",
        "calendar-14.vp375x320 day number center y ~= button center y",
    );
}

fn render_calendar_root_background_in_popover_scope(
    bounds: Rect,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    month: fret_ui_headless::calendar::CalendarMonth,
    week_start: time::Weekday,
    show_outside_days: bool,
    disable_outside_days: bool,
    cell_size: Option<Px>,
) -> fret_core::Color {
    use fret_ui::element::ElementKind;
    use fret_ui_kit::{ChromeRefinement, LayoutRefinement, LengthRefinement, Space};

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        scheme,
    );

    let open: Model<bool> = app.models_mut().insert(true);
    let month_model: Model<fret_ui_headless::calendar::CalendarMonth> =
        app.models_mut().insert(month);
    let selected: Model<Option<time::Date>> = app.models_mut().insert(None);

    let calendar_bg: std::rc::Rc<std::cell::Cell<Option<fret_core::Color>>> =
        std::rc::Rc::new(std::cell::Cell::new(None));
    let calendar_bg_for_render = calendar_bg.clone();

    let render = move |cx: &mut fret_ui::ElementContext<'_, App>| {
        let calendar_bg = calendar_bg_for_render.clone();
        let open = open.clone();
        let month_model = month_model.clone();
        let selected = selected.clone();

        let content = move |cx: &mut fret_ui::ElementContext<'_, App>| {
            let mut calendar = fret_ui_shadcn::Calendar::new(month_model.clone(), selected.clone())
                .week_start(week_start)
                .show_outside_days(show_outside_days)
                .disable_outside_days(disable_outside_days);
            if let Some(size) = cell_size {
                calendar = calendar.cell_size(size);
            }

            let calendar = calendar.into_element(cx);
            match &calendar.kind {
                ElementKind::Container(props) => {
                    let bg = props
                        .background
                        .expect("calendar root background (resolved)");
                    calendar_bg.set(Some(bg));
                }
                other => panic!("expected calendar root container, got {other:?}"),
            }

            fret_ui_shadcn::PopoverContent::new([calendar])
                // shadcn/ui DatePicker demo uses `PopoverContent` with `w-auto p-0`.
                .refine_style(ChromeRefinement::default().p(Space::N0))
                .refine_layout(LayoutRefinement::default().w(LengthRefinement::Auto))
                .into_element(cx)
        };

        vec![
            fret_ui_shadcn::Popover::new(open.clone())
                .side(fret_ui_shadcn::PopoverSide::Bottom)
                .align(fret_ui_shadcn::PopoverAlign::Start)
                .into_element(
                    cx,
                    move |cx| {
                        fret_ui_shadcn::Button::new("Open")
                            .variant(fret_ui_shadcn::ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    content,
                ),
        ]
    };

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    // Render twice to mirror other overlay-ish layout tests (ensures style resolution is stable).
    for frame in 1..=2 {
        app.set_frame_id(FrameId(frame));
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "web-vs-fret-calendar-popover-bg",
            &render,
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
    }

    calendar_bg
        .get()
        .expect("expected calendar root background to be captured")
}

fn assert_date_picker_calendar_root_background_matches_web(web_name: &str, web_theme_name: &str) {
    let web = read_web_golden(web_name);
    let theme = web
        .themes
        .get(web_theme_name)
        .unwrap_or_else(|| panic!("missing {web_theme_name} theme in web golden {web_name}"));

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let web_bg_css = web_rdp_root
        .computed_style
        .get("backgroundColor")
        .expect("web calendar root backgroundColor");
    let expected_bg =
        parse_css_color(web_bg_css).unwrap_or_else(|| panic!("invalid css color: {web_bg_css}"));

    let web_month_grid = find_first_in_theme(theme, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    })
    .expect("web month grid");
    let web_month_label = web_month_grid
        .attrs
        .get("aria-label")
        .expect("web month grid aria-label");
    let (month, year) =
        parse_calendar_title_label(web_month_label).expect("web month label (Month YYYY)");

    let weekday_headers = find_all_in_theme(theme, &|n| {
        class_has_token(n, "rdp-weekday")
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_weekday_label(label).is_some())
    });
    let week_start = weekday_headers
        .iter()
        .min_by(|a, b| a.rect.x.total_cmp(&b.rect.x))
        .and_then(|n| n.attrs.get("aria-label"))
        .and_then(|label| parse_calendar_weekday_label(label))
        .unwrap_or(time::Weekday::Sunday);

    let web_day_buttons = find_all_in_theme(theme, &|n| {
        n.tag == "button"
            && n.attrs
                .get("aria-label")
                .is_some_and(|label| parse_calendar_day_aria_label(label.as_str()).is_some())
    });
    assert!(!web_day_buttons.is_empty(), "expected calendar day buttons");

    let show_outside_days = web_day_buttons.len() != (days_in_month(year, month) as usize);
    let disable_outside_days = web_day_buttons.iter().any(|n| {
        let Some(label) = n.attrs.get("aria-label") else {
            return false;
        };
        let Some((date, _selected)) = parse_calendar_day_aria_label(label) else {
            return false;
        };
        if date.month() == month && date.year() == year {
            return false;
        }
        n.attrs.contains_key("disabled")
            || n.attrs.get("aria-disabled").is_some_and(|v| v == "true")
    });

    let cell_size = parse_calendar_cell_size_px(theme);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let scheme = match web_theme_name {
        "dark" => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        _ => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    };

    let actual_bg = render_calendar_root_background_in_popover_scope(
        bounds,
        scheme,
        fret_ui_headless::calendar::CalendarMonth::new(year, month),
        week_start,
        show_outside_days,
        disable_outside_days,
        cell_size,
    );

    assert_rgba_close(
        &format!("{web_name} {web_theme_name} calendar root background"),
        color_to_rgba(actual_bg),
        expected_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_date_picker_demo_open_calendar_root_background_matches_web_light() {
    assert_date_picker_calendar_root_background_matches_web("date-picker-demo.open", "light");
}

#[test]
fn web_vs_fret_date_picker_demo_open_calendar_root_background_matches_web_dark() {
    assert_date_picker_calendar_root_background_matches_web("date-picker-demo.open", "dark");
}

fn calendar_04_range_day_label(theme: &WebGoldenTheme, range_cell_class: &str) -> String {
    let cell = find_first_in_theme(theme, &|n| class_has_token(n, range_cell_class))
        .unwrap_or_else(|| panic!("web missing {range_cell_class} day cell"));
    let button = find_first(cell, &|n| {
        n.tag == "button" && n.attrs.contains_key("aria-label")
    })
    .unwrap_or_else(|| panic!("web missing {range_cell_class} day button"));
    button
        .attrs
        .get("aria-label")
        .expect("web range day aria-label")
        .clone()
}

#[test]
fn web_vs_fret_calendar_04_range_middle_day_background_matches_web_light() {
    let web = read_web_golden("calendar-04");
    let theme = web.themes.get("light").expect("missing light theme");
    let label = calendar_04_range_day_label(theme, "rdp-range_middle");
    assert_calendar_range_day_background_matches_web(
        "calendar-04",
        "rdp-range_middle",
        &label,
        "light",
    );
}

#[test]
fn web_vs_fret_calendar_04_range_middle_day_background_matches_web_dark() {
    let web = read_web_golden("calendar-04");
    let theme = web.themes.get("dark").expect("missing dark theme");
    let label = calendar_04_range_day_label(theme, "rdp-range_middle");
    assert_calendar_range_day_background_matches_web(
        "calendar-04",
        "rdp-range_middle",
        &label,
        "dark",
    );
}

#[test]
fn web_vs_fret_calendar_04_range_start_day_background_matches_web_light() {
    let web = read_web_golden("calendar-04");
    let theme = web.themes.get("light").expect("missing light theme");
    let label = calendar_04_range_day_label(theme, "rdp-range_start");
    assert_calendar_range_day_background_matches_web(
        "calendar-04",
        "rdp-range_start",
        &label,
        "light",
    );
}

#[test]
fn web_vs_fret_calendar_04_range_start_day_background_matches_web_dark() {
    let web = read_web_golden("calendar-04");
    let theme = web.themes.get("dark").expect("missing dark theme");
    let label = calendar_04_range_day_label(theme, "rdp-range_start");
    assert_calendar_range_day_background_matches_web(
        "calendar-04",
        "rdp-range_start",
        &label,
        "dark",
    );
}

#[test]
fn web_vs_fret_calendar_04_range_end_day_background_matches_web_light() {
    let web = read_web_golden("calendar-04");
    let theme = web.themes.get("light").expect("missing light theme");
    let label = calendar_04_range_day_label(theme, "rdp-range_end");
    assert_calendar_range_day_background_matches_web(
        "calendar-04",
        "rdp-range_end",
        &label,
        "light",
    );
}

#[test]
fn web_vs_fret_calendar_04_range_end_day_background_matches_web_dark() {
    let web = read_web_golden("calendar-04");
    let theme = web.themes.get("dark").expect("missing dark theme");
    let label = calendar_04_range_day_label(theme, "rdp-range_end");
    assert_calendar_range_day_background_matches_web(
        "calendar-04",
        "rdp-range_end",
        &label,
        "dark",
    );
}

#[test]
fn web_vs_fret_calendar_04_vp375x320_range_middle_day_background_matches_web_light() {
    let web = read_web_golden("calendar-04.vp375x320");
    let theme = web.themes.get("light").expect("missing light theme");
    let label = calendar_04_range_day_label(theme, "rdp-range_middle");
    assert_calendar_range_day_background_matches_web(
        "calendar-04.vp375x320",
        "rdp-range_middle",
        &label,
        "light",
    );
}

#[test]
fn web_vs_fret_calendar_04_vp375x320_range_middle_day_background_matches_web_dark() {
    let web = read_web_golden("calendar-04.vp375x320");
    let theme = web.themes.get("dark").expect("missing dark theme");
    let label = calendar_04_range_day_label(theme, "rdp-range_middle");
    assert_calendar_range_day_background_matches_web(
        "calendar-04.vp375x320",
        "rdp-range_middle",
        &label,
        "dark",
    );
}

#[test]
fn web_vs_fret_calendar_04_vp375x320_range_start_day_background_matches_web_light() {
    let web = read_web_golden("calendar-04.vp375x320");
    let theme = web.themes.get("light").expect("missing light theme");
    let label = calendar_04_range_day_label(theme, "rdp-range_start");
    assert_calendar_range_day_background_matches_web(
        "calendar-04.vp375x320",
        "rdp-range_start",
        &label,
        "light",
    );
}

#[test]
fn web_vs_fret_calendar_04_vp375x320_range_start_day_background_matches_web_dark() {
    let web = read_web_golden("calendar-04.vp375x320");
    let theme = web.themes.get("dark").expect("missing dark theme");
    let label = calendar_04_range_day_label(theme, "rdp-range_start");
    assert_calendar_range_day_background_matches_web(
        "calendar-04.vp375x320",
        "rdp-range_start",
        &label,
        "dark",
    );
}

#[test]
fn web_vs_fret_calendar_04_vp375x320_range_end_day_background_matches_web_light() {
    let web = read_web_golden("calendar-04.vp375x320");
    let theme = web.themes.get("light").expect("missing light theme");
    let label = calendar_04_range_day_label(theme, "rdp-range_end");
    assert_calendar_range_day_background_matches_web(
        "calendar-04.vp375x320",
        "rdp-range_end",
        &label,
        "light",
    );
}

#[test]
fn web_vs_fret_calendar_04_vp375x320_range_end_day_background_matches_web_dark() {
    let web = read_web_golden("calendar-04.vp375x320");
    let theme = web.themes.get("dark").expect("missing dark theme");
    let label = calendar_04_range_day_label(theme, "rdp-range_end");
    assert_calendar_range_day_background_matches_web(
        "calendar-04.vp375x320",
        "rdp-range_end",
        &label,
        "dark",
    );
}
