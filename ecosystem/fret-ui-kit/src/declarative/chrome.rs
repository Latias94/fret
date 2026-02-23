use fret_core::Axis;
use fret_ui::ElementContext;
use fret_ui::UiHost;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow, PressableProps,
    PressableState, SpacingLength,
};
use fret_ui::elements::GlobalElementId;
use std::sync::Arc;

fn normalize_control_chrome_sizing(
    pressable_props: &PressableProps,
    chrome_props: &mut ContainerProps,
) {
    // Normalize chrome sizing so control min/max constraints behave like `box-sizing:
    // border-box` (Tailwind preflight default): the pressable drives the *outer* box size,
    // while the chrome node's constraints apply to its inner content box after
    // padding/border.
    //
    // Without this, shadcn-style controls that apply `min-height` + `py-*` would inflate:
    // the chrome node enforces the min-height, then adds padding on top (e.g. `h-9` becomes
    // 52px).
    let spacing_px = |v: SpacingLength| match v {
        SpacingLength::Px(px) => px.0.max(0.0),
        SpacingLength::Fill | SpacingLength::Fraction(_) => 0.0,
    };
    let pad_x = spacing_px(chrome_props.padding.left) + spacing_px(chrome_props.padding.right);
    let pad_y = spacing_px(chrome_props.padding.top) + spacing_px(chrome_props.padding.bottom);
    let border_x = chrome_props.border.left.0 + chrome_props.border.right.0;
    let border_y = chrome_props.border.top.0 + chrome_props.border.bottom.0;
    let inset_x = pad_x + border_x;
    let inset_y = pad_y + border_y;
    let shrink_px = |v: fret_core::Px, inset: f32| fret_core::Px((v.0 - inset).max(0.0));

    let parent_size = pressable_props.layout.size;
    let child_size = &mut chrome_props.layout.size;

    // If the pressable is expected to resolve to a definite box in an axis, the chrome should
    // opt into percent fill sizing for that same axis.
    //
    // Note: Setting `Fill` on an otherwise shrink-wrapped node can promote wrapper chains into
    // percent-sized containing blocks (see `build_flow_subtree_impl`), so keep this constrained to
    // cases where the pressable is already intended to fill a parent-provided box.
    let chrome_fill_w = matches!(parent_size.width, Length::Px(_) | Length::Fill)
        || pressable_props.layout.flex.grow > 0.0;
    let chrome_fill_h = matches!(parent_size.height, Length::Px(_) | Length::Fill);
    if chrome_fill_w {
        child_size.width = Length::Fill;
    }
    if chrome_fill_h {
        child_size.height = Length::Fill;
    }

    if let Some(Length::Px(min_h)) = parent_size.min_height {
        child_size.min_height = Some(Length::Px(shrink_px(min_h, inset_y)));
    }
    if let Some(Length::Px(max_h)) = parent_size.max_height {
        child_size.max_height = Some(Length::Px(shrink_px(max_h, inset_y)));
    }

    if let Some(Length::Px(min_w)) = parent_size.min_width {
        child_size.min_width = Some(Length::Px(shrink_px(min_w, inset_x)));
    }
    if let Some(Length::Px(max_w)) = parent_size.max_width {
        child_size.max_width = Some(Length::Px(shrink_px(max_w, inset_x)));
    }
}

/// Composes the recommended "control chrome" structure:
///
/// - outer `Pressable` remains `Overflow::Visible` so focus rings can extend outward
/// - inner chrome `Container` is forced to `Overflow::Clip` so rounded corners/borders mask content
///
/// This matches the common shadcn/Radix mental model of:
/// `Pressable (focus ring) -> SurfaceChrome (overflow-hidden) -> content`.
#[track_caller]
pub fn control_chrome_pressable_with_id_props<'a, H, F, C, I>(
    cx: &mut ElementContext<'a, H>,
    f: F,
) -> AnyElement
where
    H: UiHost + 'a,
    F: FnOnce(
        &mut ElementContext<'a, H>,
        PressableState,
        GlobalElementId,
    ) -> (PressableProps, ContainerProps, C),
    C: for<'b> FnOnce(&'b mut ElementContext<'a, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    cx.pressable_with_id_props(|cx, st, id| {
        let (mut pressable_props, mut chrome_props, children) = f(cx, st, id);

        pressable_props.layout.overflow = Overflow::Visible;
        chrome_props.layout.overflow = Overflow::Clip;
        normalize_control_chrome_sizing(&pressable_props, &mut chrome_props);

        let chrome_test_id = pressable_props
            .a11y
            .test_id
            .as_ref()
            .map(|id| Arc::<str>::from(format!("{id}.chrome")));

        let mut content = cx.container(chrome_props, children);
        if let Some(test_id) = chrome_test_id {
            content = content.test_id(test_id);
        }
        (pressable_props, vec![content])
    })
}

/// Composes a centered "fixed chrome" structure:
///
/// - outer `Pressable` remains `Overflow::Visible` so focus rings can extend outward
/// - chrome `Container` is forced to `Overflow::Clip` so rounded corners/borders mask content
/// - chrome is centered inside a `Flex` wrapper that fills the pressable box
///
/// This is useful when the interactive hit box may stretch (flex/grid/min touch target), but the
/// visual chrome should remain token-sized and centered (Material-style).
#[track_caller]
pub fn centered_fixed_chrome_pressable_with_id_props<'a, H, F, C, I>(
    cx: &mut ElementContext<'a, H>,
    f: F,
) -> AnyElement
where
    H: UiHost + 'a,
    F: FnOnce(
        &mut ElementContext<'a, H>,
        PressableState,
        GlobalElementId,
    ) -> (PressableProps, ContainerProps, C),
    C: for<'b> FnOnce(&'b mut ElementContext<'a, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    cx.pressable_with_id_props(|cx, st, id| {
        let (mut pressable_props, mut chrome_props, children) = f(cx, st, id);

        pressable_props.layout.overflow = Overflow::Visible;
        chrome_props.layout.overflow = Overflow::Clip;

        let chrome_test_id = pressable_props
            .a11y
            .test_id
            .as_ref()
            .map(|id| Arc::<str>::from(format!("{id}.chrome")));

        let mut chrome = cx.container(chrome_props, children);
        if let Some(test_id) = chrome_test_id {
            chrome = chrome.test_id(test_id);
        }

        let mut center = FlexProps::default();
        center.direction = Axis::Horizontal;
        center.layout.size.width = Length::Fill;
        center.layout.size.height = Length::Fill;
        center.justify = MainAlign::Center;
        center.align = CrossAlign::Center;
        let centered = cx.flex(center, move |_cx| vec![chrome]);

        (pressable_props, vec![centered])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_ui::Theme;
    use fret_ui::element::ElementKind;

    #[test]
    fn control_chrome_fills_when_pressable_flex_grows() {
        let mut pressable = PressableProps::default();
        pressable.layout.flex.grow = 1.0;

        let mut chrome = ContainerProps::default();
        assert_eq!(chrome.layout.size.width, Length::Auto);
        assert_eq!(chrome.layout.size.height, Length::Auto);

        normalize_control_chrome_sizing(&pressable, &mut chrome);
        assert_eq!(chrome.layout.size.width, Length::Fill);
        assert_eq!(chrome.layout.size.height, Length::Auto);
    }

    #[test]
    fn control_chrome_does_not_force_fill_by_default() {
        let pressable = PressableProps::default();
        let mut chrome = ContainerProps::default();
        normalize_control_chrome_sizing(&pressable, &mut chrome);
        assert_eq!(chrome.layout.size.width, Length::Auto);
        assert_eq!(chrome.layout.size.height, Length::Auto);
    }

    #[test]
    fn control_chrome_fills_when_pressable_width_is_fill() {
        let mut pressable = PressableProps::default();
        pressable.layout.size.width = Length::Fill;

        let mut chrome = ContainerProps::default();
        normalize_control_chrome_sizing(&pressable, &mut chrome);
        assert_eq!(chrome.layout.size.width, Length::Fill);
        assert_eq!(chrome.layout.size.height, Length::Auto);
    }

    #[test]
    fn control_chrome_fills_when_pressable_height_is_fill() {
        let mut pressable = PressableProps::default();
        pressable.layout.size.height = Length::Fill;

        let mut chrome = ContainerProps::default();
        normalize_control_chrome_sizing(&pressable, &mut chrome);
        assert_eq!(chrome.layout.size.width, Length::Auto);
        assert_eq!(chrome.layout.size.height, Length::Fill);
    }

    #[test]
    fn control_chrome_fills_when_pressable_width_is_px() {
        let mut pressable = PressableProps::default();
        pressable.layout.size.width = Length::Px(fret_core::Px(123.0));

        let mut chrome = ContainerProps::default();
        normalize_control_chrome_sizing(&pressable, &mut chrome);
        assert_eq!(chrome.layout.size.width, Length::Fill);
    }

    #[test]
    fn control_chrome_fills_when_pressable_height_is_px() {
        let mut pressable = PressableProps::default();
        pressable.layout.size.height = Length::Px(fret_core::Px(45.0));

        let mut chrome = ContainerProps::default();
        normalize_control_chrome_sizing(&pressable, &mut chrome);
        assert_eq!(chrome.layout.size.height, Length::Fill);
    }

    #[test]
    fn control_chrome_shrinks_min_max_constraints_by_padding_and_border() {
        let mut pressable = PressableProps::default();
        pressable.layout.size.min_width = Some(fret_core::Px(40.0));
        pressable.layout.size.max_width = Some(fret_core::Px(50.0));
        pressable.layout.size.min_height = Some(fret_core::Px(20.0));
        pressable.layout.size.max_height = Some(fret_core::Px(30.0));

        let mut chrome = ContainerProps::default();
        chrome.padding = fret_core::Edges::all(fret_core::Px(2.0));
        chrome.border = fret_core::Edges::all(fret_core::Px(1.0));

        normalize_control_chrome_sizing(&pressable, &mut chrome);

        // inset = (pad_left + pad_right) + (border_left + border_right) = (2+2) + (1+1) = 6
        assert_eq!(chrome.layout.size.min_width, Some(fret_core::Px(34.0)));
        assert_eq!(chrome.layout.size.max_width, Some(fret_core::Px(44.0)));
        assert_eq!(chrome.layout.size.min_height, Some(fret_core::Px(14.0)));
        assert_eq!(chrome.layout.size.max_height, Some(fret_core::Px(24.0)));
    }

    #[test]
    fn centered_fixed_chrome_enforces_overflow_and_center_wrapper_fill() {
        use fret_ui::element::SizeStyle;

        let mut app = fret_app::App::new();
        let window = fret_core::AppWindowId::default();
        let bounds = fret_core::Rect::new(
            fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            fret_core::Size::new(fret_core::Px(200.0), fret_core::Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = centered_fixed_chrome_pressable_with_id_props(cx, |_cx, _st, _id| {
                let mut pressable = PressableProps::default();
                pressable.enabled = true;
                pressable.focusable = true;
                pressable.layout.size.width = Length::Fill;
                pressable.layout.size.height = Length::Fill;

                let mut chrome = ContainerProps::default();
                chrome.layout.size = SizeStyle {
                    width: Length::Px(fret_core::Px(28.0)),
                    height: Length::Px(fret_core::Px(28.0)),
                    ..Default::default()
                };

                (pressable, chrome, |_cx| Vec::<AnyElement>::new())
            });

            let ElementKind::Pressable(PressableProps { layout, .. }) = &el.kind else {
                panic!("expected pressable root");
            };
            assert_eq!(layout.overflow, Overflow::Visible);

            let child = el.children.first().expect("pressable child");
            let ElementKind::Flex(FlexProps {
                layout,
                justify,
                align,
                ..
            }) = &child.kind
            else {
                panic!("expected centering flex wrapper");
            };
            assert_eq!(layout.size.width, Length::Fill);
            assert_eq!(layout.size.height, Length::Fill);
            assert_eq!(*justify, MainAlign::Center);
            assert_eq!(*align, CrossAlign::Center);

            let chrome = child.children.first().expect("chrome child");
            let ElementKind::Container(ContainerProps { layout, .. }) = &chrome.kind else {
                panic!("expected chrome container");
            };
            assert_eq!(layout.overflow, Overflow::Clip);
        });
    }

    #[derive(Default)]
    struct FakeServices;

    impl fret_core::TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: fret_core::TextConstraints,
        ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                fret_core::TextMetrics {
                    size: fret_core::Size::new(fret_core::Px(0.0), fret_core::Px(0.0)),
                    baseline: fret_core::Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for FakeServices {
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

    impl fret_core::SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    fn bounds() -> fret_core::Rect {
        fret_core::Rect::new(
            fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            fret_core::Size::new(fret_core::Px(200.0), fret_core::Px(80.0)),
        )
    }

    fn rect_eq_eps(a: fret_core::Rect, b: fret_core::Rect, eps: f32) -> bool {
        let d = |x: f32, y: f32| (x - y).abs() <= eps;
        d(a.origin.x.0, b.origin.x.0)
            && d(a.origin.y.0, b.origin.y.0)
            && d(a.size.width.0, b.size.width.0)
            && d(a.size.height.0, b.size.height.0)
    }

    #[test]
    fn control_chrome_layout_bounds_match_pressable_when_flex_grown() {
        use std::cell::Cell;

        let window = fret_core::AppWindowId::default();
        let mut app = fret_app::App::new();
        let mut ui: fret_ui::UiTree<fret_app::App> = fret_ui::UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&fret_ui::ThemeConfig {
                name: "Test".to_string(),
                ..fret_ui::ThemeConfig::default()
            });
        });

        let b = bounds();
        let mut services = FakeServices;

        let pressable_id = Cell::new(None);
        let chrome_id = Cell::new(None);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "control-chrome-layout-bounds-test",
            |cx| {
                vec![cx.flex(
                    FlexProps {
                        layout: {
                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        direction: fret_core::Axis::Horizontal,
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                        ..Default::default()
                    },
                    |cx| {
                        let el = control_chrome_pressable_with_id_props(cx, |_cx, _st, _id| {
                            let mut pressable = PressableProps::default();
                            pressable.enabled = true;
                            pressable.focusable = true;
                            pressable.layout.flex.grow = 1.0;
                            pressable.layout.size.height = Length::Fill;

                            let mut chrome = ContainerProps::default();
                            chrome.padding = fret_core::Edges::all(fret_core::Px(6.0));
                            chrome.corner_radii = fret_core::Corners::all(fret_core::Px(8.0));
                            chrome.background = Some(fret_core::Color::TRANSPARENT);

                            (pressable, chrome, |_cx| Vec::<AnyElement>::new())
                        });

                        pressable_id.set(Some(el.id));
                        chrome_id.set(el.children.first().map(|c| c.id));
                        vec![el]
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let pid = pressable_id.get().expect("pressable id");
        let cid = chrome_id.get().expect("chrome id");
        let pressable_bounds =
            fret_ui::elements::bounds_for_element(&mut app, window, pid).expect("pressable bounds");
        let chrome_bounds =
            fret_ui::elements::bounds_for_element(&mut app, window, cid).expect("chrome bounds");

        assert!(
            rect_eq_eps(pressable_bounds, chrome_bounds, 0.5),
            "expected chrome bounds to match pressable bounds; pressable={pressable_bounds:?} chrome={chrome_bounds:?}"
        );
    }

    #[test]
    fn centered_fixed_chrome_layout_keeps_chrome_fixed_and_centered() {
        use std::cell::Cell;

        let window = fret_core::AppWindowId::default();
        let mut app = fret_app::App::new();
        let mut ui: fret_ui::UiTree<fret_app::App> = fret_ui::UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&fret_ui::ThemeConfig {
                name: "Test".to_string(),
                ..fret_ui::ThemeConfig::default()
            });
        });

        let b = bounds();
        let mut services = FakeServices;

        let pressable_id = Cell::new(None);
        let chrome_id = Cell::new(None);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "centered-fixed-chrome-layout-bounds-test",
            |cx| {
                vec![cx.flex(
                    FlexProps {
                        layout: {
                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        direction: fret_core::Axis::Horizontal,
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                        ..Default::default()
                    },
                    |cx| {
                        let el =
                            centered_fixed_chrome_pressable_with_id_props(cx, |_cx, _st, _id| {
                                let mut pressable = PressableProps::default();
                                pressable.enabled = true;
                                pressable.focusable = true;
                                pressable.layout.flex.grow = 1.0;
                                pressable.layout.size.height = Length::Fill;

                                let mut chrome = ContainerProps::default();
                                chrome.layout.size.width = Length::Px(fret_core::Px(28.0));
                                chrome.layout.size.height = Length::Px(fret_core::Px(28.0));
                                chrome.corner_radii = fret_core::Corners::all(fret_core::Px(8.0));
                                chrome.background = Some(fret_core::Color::TRANSPARENT);

                                (pressable, chrome, |_cx| Vec::<AnyElement>::new())
                            });

                        pressable_id.set(Some(el.id));
                        chrome_id.set(
                            el.children
                                .first()
                                .and_then(|c| c.children.first())
                                .map(|c| c.id),
                        );
                        vec![el]
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        let pid = pressable_id.get().expect("pressable id");
        let cid = chrome_id.get().expect("chrome id");
        let pressable_bounds =
            fret_ui::elements::bounds_for_element(&mut app, window, pid).expect("pressable bounds");
        let chrome_bounds =
            fret_ui::elements::bounds_for_element(&mut app, window, cid).expect("chrome bounds");

        assert!(
            rect_eq_eps(
                chrome_bounds,
                fret_core::Rect::new(
                    fret_core::Point::new(
                        fret_core::Px((pressable_bounds.size.width.0 - 28.0) * 0.5),
                        fret_core::Px((pressable_bounds.size.height.0 - 28.0) * 0.5)
                    ),
                    fret_core::Size::new(fret_core::Px(28.0), fret_core::Px(28.0)),
                ),
                0.75
            ),
            "expected chrome bounds to be fixed 28x28 and centered; pressable={pressable_bounds:?} chrome={chrome_bounds:?}"
        );
    }
}
