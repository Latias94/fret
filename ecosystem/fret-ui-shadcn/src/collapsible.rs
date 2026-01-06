//! shadcn/ui `Collapsible` (headless).

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, OpacityProps, PressableProps, StackProps,
};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::transition;
use fret_ui_kit::primitives::collapsible as radix_collapsible;

use crate::overlay_motion;

#[derive(Clone)]
pub struct Collapsible {
    open: Model<bool>,
    disabled: bool,
    layout: LayoutRefinement,
    force_mount_content: bool,
}

impl std::fmt::Debug for Collapsible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Collapsible")
            .field("open", &"<model>")
            .field("disabled", &self.disabled)
            .field("layout", &self.layout)
            .field("force_mount_content", &self.force_mount_content)
            .finish()
    }
}

impl Collapsible {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            disabled: false,
            layout: LayoutRefinement::default(),
            force_mount_content: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// When `true`, the content subtree is always mounted (but clipped to zero height when closed).
    ///
    /// This is a partial parity knob for Radix's `forceMount` on `CollapsibleContent`.
    pub fn force_mount_content(mut self, force_mount_content: bool) -> Self {
        self.force_mount_content = force_mount_content;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>, bool) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        cx.scope(|cx| {
            let state_id = cx.root_id();

            let is_open = cx
                .watch_model(&self.open)
                .layout()
                .copied()
                .unwrap_or(false);

            let trigger = trigger(cx, is_open);
            let force_mount_content = self.force_mount_content;
            let disabled = self.disabled;

            // Radix/shadcn-like behavior keeps content mounted during close animations. We
            // approximate the height transition by mapping transition progress to a clipped height.
            let last_height = radix_collapsible::last_measured_height_for(cx, state_id);
            let wants_measurement = is_open && last_height.0 <= 0.0;

            // Radix measures content size to drive open/close animations. In Fret we first mount a
            // hidden, off-flow wrapper to populate the cached height, then start the transition.
            let motion_open = is_open && !wants_measurement;
            let motion = transition::drive_transition_with_durations_and_easing(
                cx,
                motion_open,
                8,
                8,
                overlay_motion::shadcn_ease,
            );

            let (should_render_content, wrapper) =
                radix_collapsible::collapsible_height_wrapper_refinement(
                    motion_open,
                    force_mount_content,
                    true,
                    motion,
                    last_height,
                );
            let should_build_content = wants_measurement || should_render_content;
            let content = should_build_content.then(|| content(cx));
            let layout = self.layout;

            let stack = cx.stack_props(
                StackProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        fret_ui::Theme::global(&*cx.app),
                        layout,
                    ),
                },
                move |cx| {
                    let mut children = Vec::new();
                    children.push(trigger);
                    if let Some(content) = content {
                        let theme = fret_ui::Theme::global(&*cx.app);

                        let (wrapper_refinement, opacity) = if wants_measurement {
                            (
                                radix_collapsible::collapsible_measurement_wrapper_refinement(),
                                0.0,
                            )
                        } else {
                            (wrapper, 1.0)
                        };

                        let wrapper_layout = fret_ui_kit::declarative::style::layout_style(
                            theme,
                            wrapper_refinement,
                        );

                        let wrapper_el = cx.keyed("collapsible-content", |cx| {
                            cx.container(
                                ContainerProps {
                                    layout: wrapper_layout,
                                    ..Default::default()
                                },
                                move |cx| {
                                    vec![cx.opacity_props(
                                        OpacityProps {
                                            layout: LayoutStyle::default(),
                                            opacity,
                                        },
                                        move |_cx| vec![content],
                                    )]
                                },
                            )
                        });
                        let wrapper_id = wrapper_el.id;

                        if wants_measurement {
                            let _ =
                                radix_collapsible::update_measured_size_from_element_if_open_for(
                                    cx, state_id, wrapper_id, is_open,
                                );
                        } else {
                            // Update the cached content height once the collapsible is fully open and
                            // not animating. This gives subsequent close/open transitions a stable
                            // target (and supports content changes while open).
                            let _ = radix_collapsible::update_measured_height_if_open_for(
                                cx,
                                state_id,
                                wrapper_id,
                                is_open,
                                motion.animating,
                            );
                        }

                        children.push(wrapper_el);
                    }
                    children
                },
            );

            cx.semantics(
                radix_collapsible::collapsible_root_semantics(disabled, is_open),
                move |_cx| vec![stack],
            )
        })
    }
}

#[derive(Clone)]
pub struct CollapsibleTrigger {
    open: Model<bool>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for CollapsibleTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollapsibleTrigger")
            .field("open", &"<model>")
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl CollapsibleTrigger {
    pub fn new(open: Model<bool>, children: Vec<AnyElement>) -> Self {
        Self {
            open,
            disabled: false,
            a11y_label: None,
            children,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        is_open: bool,
    ) -> AnyElement {
        let open = self.open;
        let disabled = self.disabled;
        let children = self.children;
        let a11y_label = self.a11y_label;

        cx.pressable(
            PressableProps {
                enabled: !disabled,
                a11y: radix_collapsible::collapsible_trigger_a11y(a11y_label, is_open),
                ..Default::default()
            },
            move |cx, _state| {
                cx.pressable_toggle_bool(&open);
                children
            },
        )
    }
}

#[derive(Clone)]
pub struct CollapsibleContent {
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for CollapsibleContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollapsibleContent")
            .field("layout", &self.layout)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl CollapsibleContent {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            layout: LayoutRefinement::default(),
            children,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let layout = self.layout;
        let children = self.children;

        cx.stack_props(
            StackProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    fret_ui::Theme::global(&*cx.app),
                    layout,
                ),
            },
            move |_cx| children,
        )
    }
}

pub fn collapsible<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    trigger: impl FnOnce(&mut ElementContext<'_, H>, bool) -> AnyElement,
    content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    Collapsible::new(open).into_element(cx, trigger, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{AppWindowId, Modifiers, Point, Px, Rect, Size, SvgId, SvgService};
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::tree::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &TextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
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

    fn render(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
    ) -> fret_core::NodeId {
        app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));

        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "collapsible",
            |cx| {
                vec![Collapsible::new(open.clone()).into_element(
                    cx,
                    |cx, is_open| {
                        CollapsibleTrigger::new(open.clone(), vec![cx.text("Trigger")])
                            .into_element(cx, is_open)
                    },
                    |cx| CollapsibleContent::new(vec![cx.text("Content")]).into_element(cx),
                )]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn collapsible_trigger_toggles_open_model_on_space() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyUp {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
            },
        );

        let _ = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let is_open = app.models().get_copied(&open).unwrap_or(false);
        assert!(is_open);
    }

    #[test]
    fn collapsible_content_remains_mounted_for_close_animation_when_measured() {
        fn snapshot_has_label(ui: &UiTree<App>, label: &str) -> bool {
            ui.semantics_snapshot()
                .expect("semantics snapshot")
                .nodes
                .iter()
                .any(|n| n.label.as_deref() == Some(label))
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let _ = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );
        assert!(!snapshot_has_label(&ui, "Content"));

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Render enough frames for presence to settle and for height to be measured.
        for _ in 0..12 {
            let _ = render(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
            );
        }
        assert!(snapshot_has_label(&ui, "Content"));

        let _ = app.models_mut().update(&open, |v| *v = false);

        // First close frame: content should still be mounted (present=true) for the transition.
        let _ = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );
        assert!(snapshot_has_label(&ui, "Content"));

        // After enough frames, presence completes and content unmounts.
        for _ in 0..16 {
            let _ = render(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
            );
        }
        assert!(!snapshot_has_label(&ui, "Content"));
    }
}
