//! shadcn/ui `Collapsible` (headless).

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::{AnyElement, PressableProps, StackProps};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::{LayoutRefinement, MetricRef};

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
            let is_open = cx
                .watch_model(&self.open)
                .layout()
                .copied()
                .unwrap_or(false);

            let trigger = trigger(cx, is_open);
            let force_mount_content = self.force_mount_content;
            let disabled = self.disabled;
            let content = (is_open || force_mount_content).then(|| content(cx));
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
                        if is_open || !force_mount_content {
                            children.push(content);
                        } else {
                            let theme = fret_ui::Theme::global(&*cx.app);
                            let clipped_layout = fret_ui_kit::declarative::style::layout_style(
                                theme,
                                LayoutRefinement::default()
                                    .h_px(MetricRef::Px(fret_core::Px(0.0)))
                                    .overflow_hidden(),
                            );
                            children.push(cx.stack_props(
                                StackProps { layout: clipped_layout },
                                move |_cx| vec![content],
                            ));
                        }
                    }
                    children
                },
            );

            cx.semantics(
                fret_ui_kit::primitives::collapsible::collapsible_root_semantics(
                    disabled,
                    is_open,
                ),
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
                a11y: fret_ui_kit::primitives::collapsible::collapsible_trigger_a11y(
                    a11y_label,
                    is_open,
                ),
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

        let root = render(&mut ui, &mut app, &mut services, window, bounds, open.clone());

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

        let _ = render(&mut ui, &mut app, &mut services, window, bounds, open.clone());

        let is_open = app.models().get_copied(&open).unwrap_or(false);
        assert!(is_open);
    }
}
