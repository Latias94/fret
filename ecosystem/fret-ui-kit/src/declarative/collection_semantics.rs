use fret_ui::element::PressableA11y;

pub trait CollectionSemanticsExt {
    fn with_collection_position(self, index: usize, count: usize) -> Self;
}

impl CollectionSemanticsExt for PressableA11y {
    fn with_collection_position(mut self, index: usize, count: usize) -> Self {
        if count == 0 || index >= count {
            self.pos_in_set = None;
            self.set_size = None;
            return self;
        }

        let pos_in_set = index.saturating_add(1);
        self.pos_in_set = u32::try_from(pos_in_set).ok();
        self.set_size = u32::try_from(count).ok();

        if let (Some(pos), Some(size)) = (self.pos_in_set, self.set_size)
            && pos > size
        {
            self.pos_in_set = None;
            self.set_size = None;
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, SvgId, SvgService, TextBlobId, TextConstraints, TextInput,
        TextMetrics, TextService,
    };
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Point, Px, Rect};
    use fret_ui::element::{ContainerProps, LayoutStyle, Length, PressableProps};
    use fret_ui::{Theme, ThemeConfig, UiTree};

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
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

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
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

    #[test]
    fn pressable_a11y_can_stamp_collection_position() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(200.0), Px(80.0)),
        );

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                vec![cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Px(Px(32.0));
                            layout
                        },
                        a11y: PressableA11y::default().with_collection_position(56, 1200),
                        ..Default::default()
                    },
                    |cx, _st| vec![cx.container(ContainerProps::default(), |_| Vec::new())],
                )]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.role == fret_core::SemanticsRole::Button)
            .expect("pressable semantics node");
        assert_eq!(node.pos_in_set, Some(57));
        assert_eq!(node.set_size, Some(1200));
    }
}
