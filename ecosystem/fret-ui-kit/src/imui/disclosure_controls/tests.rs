use super::*;

use std::sync::Arc;

use super::super::{CollapsingHeaderOptions, TreeNodeOptions, UiWriterImUiFacadeExt};
use fret_app::App;
use fret_authoring::UiWriter;
use fret_core::{AppWindowId, Color, Point, Px, Rect, SemanticsRole, Size, TextOverflow, TextWrap};
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, ElementKind, Length, PressableProps, PressableState};
use fret_ui::elements;
use fret_ui::{ElementContext, Theme, ThemeConfig};

struct TestWriter<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    out: &'cx mut Vec<AnyElement>,
}

impl<'cx, 'a, H: UiHost> UiWriter<H> for TestWriter<'cx, 'a, H> {
    fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {
        f(self.cx)
    }

    fn add(&mut self, element: AnyElement) {
        self.out.push(element);
    }
}

fn contains_text(root: &AnyElement, expected: &str) -> bool {
    match &root.kind {
        ElementKind::Text(props) if props.text.as_ref() == expected => true,
        _ => root
            .children
            .iter()
            .any(|child| contains_text(child, expected)),
    }
}

fn first_pressable(root: &AnyElement) -> Option<&PressableProps> {
    match &root.kind {
        ElementKind::Pressable(props) => Some(props),
        _ => root.children.iter().find_map(first_pressable),
    }
}

fn first_text<'a>(root: &'a AnyElement, expected: &str) -> Option<&'a AnyElement> {
    match &root.kind {
        ElementKind::Text(props) if props.text.as_ref() == expected => Some(root),
        _ => root
            .children
            .iter()
            .find_map(|child| first_text(child, expected)),
    }
}

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    )
}

#[test]
fn collapsing_header_default_open_mounts_body() {
    let mut app = App::new();
    fret_ui::elements::with_element_cx(
        &mut app,
        Default::default(),
        Default::default(),
        "test",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };
            let response = collapsing_header_with_options(
                &mut ui,
                "header",
                Arc::from("Section"),
                CollapsingHeaderOptions {
                    default_open: true,
                    ..Default::default()
                },
                |ui| {
                    ui.text("Body");
                },
            );

            assert!(response.open());
            assert_eq!(out.len(), 1);
            assert!(contains_text(&out[0], "Section"));
            assert!(contains_text(&out[0], "Body"));
        },
    );
}

#[test]
fn tree_node_leaf_uses_tree_item_semantics() {
    let mut app = App::new();
    fret_ui::elements::with_element_cx(
        &mut app,
        Default::default(),
        Default::default(),
        "test",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };
            let response = tree_node_with_options(
                &mut ui,
                "leaf",
                Arc::from("Leaf"),
                TreeNodeOptions {
                    leaf: true,
                    level: 3,
                    selected: true,
                    ..Default::default()
                },
                |_ui| {},
            );

            assert!(!response.open());
            let pressable = first_pressable(&out[0]).expect("expected pressable row");
            assert_eq!(pressable.a11y.role, Some(SemanticsRole::TreeItem));
            assert_eq!(pressable.a11y.level, Some(3));
            assert!(pressable.a11y.selected);
            assert_eq!(pressable.a11y.expanded, None);
        },
    );
}

#[test]
fn tree_node_default_options_start_at_level_one() {
    let options = TreeNodeOptions::default();
    assert_eq!(options.level, 1);
    assert!(!options.selected);
    assert!(!options.leaf);
}

#[test]
fn tree_node_hover_palette_prefers_accent_chrome_over_popover_fill() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors
            .insert("list.active.background".to_string(), "#224466".to_string());
        cfg.colors
            .insert("accent".to_string(), "#335577".to_string());
        cfg.colors
            .insert("accent-foreground".to_string(), "#fefefe".to_string());
        cfg.colors
            .insert("foreground".to_string(), "#f5f6f7".to_string());
        cfg.colors.insert("card".to_string(), "#101418".to_string());
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    let spec = DisclosureSpec::tree_node(Arc::from("Scene"), TreeNodeOptions::default());

    let hovered = resolve_disclosure_palette(
        theme,
        &spec,
        PressableState {
            hovered: true,
            ..Default::default()
        },
    );
    assert_eq!(
        hovered.background,
        Some(Color::from_srgb_hex_rgb(0x33_55_77))
    );
    assert_eq!(hovered.foreground, Color::from_srgb_hex_rgb(0xfe_fe_fe));

    let idle = resolve_disclosure_palette(theme, &spec, PressableState::default());
    assert_eq!(idle.background, None);
    assert_eq!(idle.foreground, Color::from_srgb_hex_rgb(0xf5_f6_f7));
}

#[test]
fn tree_row_label_uses_shared_list_row_text_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        let spec = DisclosureSpec::tree_node(
            Arc::from("Very long tree node"),
            TreeNodeOptions {
                leaf: true,
                ..Default::default()
            },
        );
        header_row(
            cx,
            &spec,
            spec.label.clone(),
            false,
            PressableState::default(),
        )
    });
    let expected_palette = resolve_disclosure_palette(
        Theme::global(&app),
        &DisclosureSpec::tree_node(
            Arc::from("Very long tree node"),
            TreeNodeOptions {
                leaf: true,
                ..Default::default()
            },
        ),
        PressableState::default(),
    );

    let text = first_text(&el, "Very long tree node").expect("expected tree row label text");
    let ElementKind::Text(props) = &text.kind else {
        panic!("expected tree row label to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.size.width, Length::Fill);
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Ellipsis);
    assert!(text.inherited_text_style.is_some());
    assert_eq!(text.inherited_foreground, Some(expected_palette.foreground));
}

#[test]
fn disclosure_indicator_uses_shared_chrome_glyph_text_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        let spec = DisclosureSpec::tree_node(
            Arc::from("Expandable tree node"),
            TreeNodeOptions::default(),
        );
        header_row(
            cx,
            &spec,
            spec.label.clone(),
            false,
            PressableState::default(),
        )
    });
    let expected_palette = resolve_disclosure_palette(
        Theme::global(&app),
        &DisclosureSpec::tree_node(
            Arc::from("Expandable tree node"),
            TreeNodeOptions::default(),
        ),
        PressableState::default(),
    );

    let text = first_text(&el, ">").expect("expected disclosure indicator text");
    let ElementKind::Text(props) = &text.kind else {
        panic!("expected disclosure indicator to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Clip);
    assert!(text.inherited_text_style.is_some());
    assert_eq!(text.inherited_foreground, Some(expected_palette.foreground));
}
