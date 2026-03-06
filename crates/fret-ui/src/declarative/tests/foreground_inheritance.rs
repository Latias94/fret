use super::*;

use crate::SvgSource;
use crate::element::{Length, SvgIconProps, TextProps};
use fret_core::{Color, Paint, Px, SvgId};

#[test]
fn foreground_scope_late_binds_foreground_for_text_and_icons() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let expected = Color {
        r: 0.25,
        g: 0.5,
        b: 0.75,
        a: 1.0,
    };

    let node = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "foreground-scope-late-binds-foreground",
        |cx| {
            vec![cx.foreground_scope(expected, |cx| {
                let mut icon = SvgIconProps::new(SvgSource::Id(SvgId::default()));
                icon.layout.size.width = Length::Px(Px(16.0));
                icon.layout.size.height = Length::Px(Px(16.0));
                icon.color = Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                };
                icon.inherit_color = true;

                let mut text = TextProps::new("hello");
                text.layout.size.width = Length::Px(Px(40.0));
                text.layout.size.height = Length::Px(Px(16.0));
                text.color = None;

                vec![cx.svg_icon_props(icon), cx.text_props(text)]
            })]
        },
    );
    ui.set_root(node);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let icon_color = scene.ops().iter().find_map(|op| match op {
        SceneOp::SvgMaskIcon { color, .. } => Some(*color),
        _ => None,
    });
    assert_eq!(
        icon_color,
        Some(expected),
        "expected SvgIcon(inherit_color=true) to use the ForegroundScope foreground during paint"
    );

    let text_color = scene.ops().iter().find_map(|op| match op {
        SceneOp::Text { paint, .. } => match paint.paint {
            Paint::Solid(color) => Some(color),
            _ => None,
        },
        _ => None,
    });
    assert_eq!(
        text_color,
        Some(expected),
        "expected Text(color=None) to use the ForegroundScope foreground during paint"
    );
}

#[test]
fn inherited_foreground_on_existing_root_late_binds_for_text_icon_and_spinner() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(160.0), Px(100.0)),
    );
    let mut services = FakeTextService::default();

    let expected = Color {
        r: 0.8,
        g: 0.3,
        b: 0.2,
        a: 1.0,
    };

    let node = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "inherited-foreground-existing-root",
        |cx| {
            let root = cx
                .container(crate::element::ContainerProps::default(), |cx| {
                    let mut icon = SvgIconProps::new(SvgSource::Id(SvgId::default()));
                    icon.layout.size.width = Length::Px(Px(16.0));
                    icon.layout.size.height = Length::Px(Px(16.0));
                    icon.color = Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    };
                    icon.inherit_color = true;

                    let mut text = TextProps::new("hello");
                    text.layout.size.width = Length::Px(Px(40.0));
                    text.layout.size.height = Length::Px(Px(16.0));
                    text.color = None;

                    let mut spinner = crate::element::SpinnerProps::default();
                    spinner.speed = 0.0;
                    spinner.dot_count = 1;

                    vec![
                        cx.svg_icon_props(icon),
                        cx.text_props(text),
                        cx.spinner_props(spinner),
                    ]
                })
                .inherit_foreground(expected);
            vec![root]
        },
    );
    ui.set_root(node);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let icon_color = scene.ops().iter().find_map(|op| match op {
        SceneOp::SvgMaskIcon { color, .. } => Some(*color),
        _ => None,
    });
    assert_eq!(
        icon_color,
        Some(expected),
        "expected SvgIcon(inherit_color=true) to use inherited foreground stamped on the existing root"
    );

    let text_color = scene.ops().iter().find_map(|op| match op {
        SceneOp::Text { paint, .. } => match paint.paint {
            Paint::Solid(color) => Some(color),
            _ => None,
        },
        _ => None,
    });
    assert_eq!(
        text_color,
        Some(expected),
        "expected Text(color=None) to use inherited foreground stamped on the existing root"
    );

    let spinner_color = scene.ops().iter().find_map(|op| match op {
        SceneOp::Quad { background, .. } => match background.paint {
            Paint::Solid(color)
                if color.r == expected.r && color.g == expected.g && color.b == expected.b =>
            {
                Some(color)
            }
            _ => None,
        },
        _ => None,
    });
    assert_eq!(
        spinner_color,
        Some(expected),
        "expected Spinner(color=None) to use inherited foreground stamped on the existing root"
    );
}
