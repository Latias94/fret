use fret_app::App;
use fret_core::{AppWindowId, NodeId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::tree::UiTree;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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

#[test]
fn web_vs_fret_calendar_11_disabled_navigation_semantics_matches_web() {
    let web = read_web_golden("calendar-11");
    let theme = web_theme(&web);

    let web_month_grids = find_all(&theme.root, &|n| {
        n.tag == "table" && class_has_token(n, "rdp-month_grid")
    });
    assert_eq!(
        web_month_grids.len(),
        2,
        "expected two month grids for calendar-11"
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
        "expected calendar day buttons for calendar-11"
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

    let web_rdp_root = web_find_by_class_token_in_theme(theme, "rdp-root").expect("web rdp-root");
    let origin_x = web_rdp_root.rect.x;
    let origin_y = web_rdp_root.rect.y;

    let (_ui, snap, _root) = render_calendar_in_bounds(
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
