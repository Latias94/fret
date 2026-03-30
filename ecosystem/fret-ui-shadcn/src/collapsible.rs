//! shadcn/ui `Collapsible` (headless).

use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use fret_core::SemanticsRole;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, ElementKind, InteractivityGateProps, LayoutStyle,
    OpacityProps, PressableProps, SemanticsDecoration,
};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;
use fret_ui_kit::primitives::collapsible as radix_collapsible;
use fret_ui_kit::{
    ChromeRefinement, IntoUiElement, LayoutRefinement, UiPatch, UiPatchTarget, UiSupportsChrome,
    UiSupportsLayout,
};

use crate::bool_model::IntoBoolModel;
use crate::overlay_motion;

fn apply_disabled_to_trigger(mut trigger: AnyElement, disabled: bool) -> AnyElement {
    if !disabled {
        return trigger;
    }

    trigger.children = trigger
        .children
        .into_iter()
        .map(|child| apply_disabled_to_trigger(child, disabled))
        .collect();

    match &mut trigger.kind {
        ElementKind::Pressable(props) => {
            props.enabled = false;
            props.focusable = false;
        }
        ElementKind::Semantics(props) => {
            props.disabled = true;
            props.focusable = false;
        }
        _ => {}
    }

    trigger
}

#[derive(Clone)]
pub struct Collapsible {
    open: Option<Model<bool>>,
    default_open: bool,
    disabled: bool,
    chrome: ChromeRefinement,
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
    pub fn new(open: impl IntoBoolModel) -> Self {
        let open = open.into_bool_model();
        Self {
            open: Some(open),
            default_open: false,
            disabled: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            force_mount_content: false,
        }
    }

    /// Creates an uncontrolled collapsible with the given initial open value (Radix `defaultOpen`).
    pub fn uncontrolled(default_open: bool) -> Self {
        Self {
            open: None,
            default_open,
            disabled: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            force_mount_content: false,
        }
    }

    /// Sets the uncontrolled initial open value (Radix `defaultOpen`).
    ///
    /// Note: If a controlled `open` model is provided, this value is ignored.
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
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

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>, bool) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        self.into_element_with_open_model(cx, |cx, _open, is_open| trigger(cx, is_open), content)
    }

    #[track_caller]
    pub fn into_element_with_open_model<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>, Model<bool>, bool) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        let open_root = radix_collapsible::CollapsibleRoot::new()
            .open(self.open)
            .default_open(self.default_open);
        let disabled = self.disabled;
        let chrome = self.chrome;
        let layout = self.layout;
        let force_mount_content = self.force_mount_content;

        cx.scope(move |cx| {
            let open = open_root.use_open_model(cx).model();
            let is_open = cx.watch_model(&open).layout().copied().unwrap_or(false);

            let trigger = apply_disabled_to_trigger(trigger(cx, open.clone(), is_open), disabled);

            let theme = fret_ui::Theme::global(&*cx.app).snapshot();

            let toggle_duration = {
                let theme_full = fret_ui::Theme::global(&*cx.app);
                theme_full
                    .duration_ms_by_key("duration.shadcn.motion.collapsible.toggle")
                    .or_else(|| theme_full.duration_ms_by_key("duration.motion.collapsible.toggle"))
                    .or_else(|| theme_full.duration_ms_by_key("duration.shadcn.motion.200"))
            }
            .map(|ms| Duration::from_millis(ms as u64))
            .unwrap_or(Duration::from_millis(200));
            let toggle_ticks = ticks_60hz_for_duration(toggle_duration);
            let toggle_easing = {
                let theme_full = fret_ui::Theme::global(&*cx.app);
                theme_full
                    .easing_by_key("easing.shadcn.motion.collapsible.toggle")
                    .or_else(|| theme_full.easing_by_key("easing.motion.collapsible.toggle"))
            }
            .unwrap_or_else(|| overlay_motion::shadcn_motion_ease_bezier(cx));

            let motion = radix_collapsible::measured_height_motion_for_root_with_cubic_bezier(
                cx,
                is_open,
                force_mount_content,
                true,
                toggle_ticks,
                toggle_ticks,
                toggle_easing,
            );

            let content = motion.should_render.then(|| content(cx));

            let stack = cx.column(
                ColumnProps {
                    layout: decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default().min_w_0().merge(layout),
                    ),
                    ..Default::default()
                },
                move |cx| {
                    let mut children = Vec::new();
                    let motion_for_wrapper = motion.clone();
                    let motion_for_update = motion.clone();

                    let (content_id, wrapper_el) = cx.keyed("collapsible-content", move |cx| {
                        cx.scope(|cx| {
                            let content_id = cx.root_id();
                            let Some(content) = content else {
                                return (content_id, None);
                            };

                            let theme = fret_ui::Theme::global(&*cx.app);
                            let wrapper_refinement = motion_for_wrapper.wrapper_refinement.clone();
                            let wrapper_layout = fret_ui_kit::declarative::style::layout_style(
                                theme,
                                wrapper_refinement,
                            );

                            let wrapper_child = cx.opacity_props(
                                OpacityProps {
                                    layout: LayoutStyle::default(),
                                    opacity: motion_for_wrapper.wrapper_opacity,
                                },
                                move |_cx| vec![content],
                            );
                            let children = vec![wrapper_child];

                            let wrapper_kind = if motion_for_wrapper.wants_measurement {
                                ElementKind::InteractivityGate(InteractivityGateProps {
                                    layout: wrapper_layout,
                                    present: true,
                                    interactive: false,
                                })
                            } else {
                                ElementKind::Container(ContainerProps {
                                    layout: wrapper_layout,
                                    ..Default::default()
                                })
                            };

                            let wrapper_el = AnyElement::new(content_id, wrapper_kind, children);

                            (content_id, Some(wrapper_el))
                        })
                    });

                    let trigger = radix_collapsible::apply_collapsible_trigger_controls_expanded(
                        trigger, content_id, is_open,
                    );
                    let trigger = if disabled {
                        cx.interactivity_gate(true, false, move |_cx| vec![trigger])
                    } else {
                        trigger
                    };
                    children.push(trigger);

                    if let Some(wrapper_el) = wrapper_el {
                        let _ = radix_collapsible::update_measured_for_motion(
                            cx,
                            motion_for_update,
                            wrapper_el.id,
                        );
                        children.push(wrapper_el);
                    }
                    children
                },
            );

            let wrapper = decl_style::container_props(
                &theme,
                chrome,
                LayoutRefinement::default().w_full().min_w_0(),
            );
            let root = cx.container(wrapper, move |_cx| vec![stack]);

            root.attach_semantics(SemanticsDecoration {
                role: Some(SemanticsRole::Generic),
                disabled: Some(disabled),
                expanded: Some(is_open),
                ..Default::default()
            })
        })
    }
}

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
    pub fn new(open: Model<bool>, children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            open,
            disabled: false,
            a11y_label: None,
            children: children.into_iter().collect(),
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

    #[track_caller]
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

pub struct CollapsibleContent {
    chrome: ChromeRefinement,
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
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            children,
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = fret_ui::Theme::global(&*cx.app).snapshot();
        let wrapper = decl_style::container_props(&theme, self.chrome, self.layout);
        let children = self.children;

        cx.container(wrapper, move |cx| {
            vec![cx.column(
                ColumnProps {
                    layout: decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default().w_full().min_w_0(),
                    ),
                    ..Default::default()
                },
                move |_cx| children,
            )]
        })
    }
}

pub struct CollapsibleBuild<H, Trigger, Content> {
    root: Collapsible,
    trigger: Option<Trigger>,
    content: Option<Content>,
    _phantom: PhantomData<fn() -> H>,
}

impl<H, Trigger, Content> CollapsibleBuild<H, Trigger, Content> {
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.root = self.root.disabled(disabled);
        self
    }

    pub fn force_mount_content(mut self, force_mount_content: bool) -> Self {
        self.root = self.root.force_mount_content(force_mount_content);
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.root = self.root.refine_style(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.root = self.root.refine_layout(layout);
        self
    }
}

impl<H, Trigger, Content, TriggerEl, ContentEl> CollapsibleBuild<H, Trigger, Content>
where
    H: UiHost,
    Trigger: FnOnce(&mut ElementContext<'_, H>, bool) -> TriggerEl,
    Content: FnOnce(&mut ElementContext<'_, H>) -> ContentEl,
    TriggerEl: IntoUiElement<H>,
    ContentEl: IntoUiElement<H>,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let trigger = self
            .trigger
            .expect("expected collapsible trigger builder closure");
        let content = self
            .content
            .expect("expected collapsible content builder closure");

        self.root.into_element(
            cx,
            move |cx, is_open| trigger(cx, is_open).into_element(cx),
            move |cx| content(cx).into_element(cx),
        )
    }
}

impl<H, Trigger, Content> UiPatchTarget for CollapsibleBuild<H, Trigger, Content> {
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl<H, Trigger, Content> UiSupportsChrome for CollapsibleBuild<H, Trigger, Content> {}

impl<H, Trigger, Content> UiSupportsLayout for CollapsibleBuild<H, Trigger, Content> {}

impl<H, Trigger, Content, TriggerEl, ContentEl> IntoUiElement<H>
    for CollapsibleBuild<H, Trigger, Content>
where
    H: UiHost,
    Trigger: FnOnce(&mut ElementContext<'_, H>, bool) -> TriggerEl,
    Content: FnOnce(&mut ElementContext<'_, H>) -> ContentEl,
    TriggerEl: IntoUiElement<H>,
    ContentEl: IntoUiElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        CollapsibleBuild::into_element(self, cx)
    }
}

pub struct CollapsibleUncontrolledBuild<H, Trigger, Content> {
    root: Collapsible,
    trigger: Option<Trigger>,
    content: Option<Content>,
    _phantom: PhantomData<fn() -> H>,
}

impl<H, Trigger, Content> CollapsibleUncontrolledBuild<H, Trigger, Content> {
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.root = self.root.disabled(disabled);
        self
    }

    pub fn force_mount_content(mut self, force_mount_content: bool) -> Self {
        self.root = self.root.force_mount_content(force_mount_content);
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.root = self.root.refine_style(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.root = self.root.refine_layout(layout);
        self
    }
}

impl<H, Trigger, Content, TriggerEl, ContentEl> CollapsibleUncontrolledBuild<H, Trigger, Content>
where
    H: UiHost,
    Trigger: FnOnce(&mut ElementContext<'_, H>, Model<bool>, bool) -> TriggerEl,
    Content: FnOnce(&mut ElementContext<'_, H>) -> ContentEl,
    TriggerEl: IntoUiElement<H>,
    ContentEl: IntoUiElement<H>,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let trigger = self
            .trigger
            .expect("expected uncontrolled collapsible trigger builder closure");
        let content = self
            .content
            .expect("expected uncontrolled collapsible content builder closure");

        self.root.into_element_with_open_model(
            cx,
            move |cx, open, is_open| trigger(cx, open, is_open).into_element(cx),
            move |cx| content(cx).into_element(cx),
        )
    }
}

impl<H, Trigger, Content> UiPatchTarget for CollapsibleUncontrolledBuild<H, Trigger, Content> {
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl<H, Trigger, Content> UiSupportsChrome for CollapsibleUncontrolledBuild<H, Trigger, Content> {}

impl<H, Trigger, Content> UiSupportsLayout for CollapsibleUncontrolledBuild<H, Trigger, Content> {}

impl<H, Trigger, Content, TriggerEl, ContentEl> IntoUiElement<H>
    for CollapsibleUncontrolledBuild<H, Trigger, Content>
where
    H: UiHost,
    Trigger: FnOnce(&mut ElementContext<'_, H>, Model<bool>, bool) -> TriggerEl,
    Content: FnOnce(&mut ElementContext<'_, H>) -> ContentEl,
    TriggerEl: IntoUiElement<H>,
    ContentEl: IntoUiElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        CollapsibleUncontrolledBuild::into_element(self, cx)
    }
}

pub fn collapsible<H: UiHost, Trigger, Content, TriggerEl, ContentEl>(
    open: Model<bool>,
    trigger: Trigger,
    content: Content,
) -> CollapsibleBuild<H, Trigger, Content>
where
    Trigger: FnOnce(&mut ElementContext<'_, H>, bool) -> TriggerEl,
    Content: FnOnce(&mut ElementContext<'_, H>) -> ContentEl,
    TriggerEl: IntoUiElement<H>,
    ContentEl: IntoUiElement<H>,
{
    CollapsibleBuild {
        root: Collapsible::new(open),
        trigger: Some(trigger),
        content: Some(content),
        _phantom: PhantomData,
    }
}

pub fn collapsible_uncontrolled<H: UiHost, Trigger, Content, TriggerEl, ContentEl>(
    default_open: bool,
    trigger: Trigger,
    content: Content,
) -> CollapsibleUncontrolledBuild<H, Trigger, Content>
where
    Trigger: FnOnce(&mut ElementContext<'_, H>, Model<bool>, bool) -> TriggerEl,
    Content: FnOnce(&mut ElementContext<'_, H>) -> ContentEl,
    TriggerEl: IntoUiElement<H>,
    ContentEl: IntoUiElement<H>,
{
    CollapsibleUncontrolledBuild {
        root: Collapsible::uncontrolled(default_open),
        trigger: Some(trigger),
        content: Some(content),
        _phantom: PhantomData,
    }
}

/// Upstream-shaped Collapsible primitives (`Collapsible`/`CollapsibleTrigger`/`CollapsibleContent`).
///
/// We keep these in a nested module to preserve the existing ergonomic builder surface at
/// `fret_ui_shadcn::Collapsible` while also exposing a shadcn/Radix-style children API via
/// `fret_ui_shadcn::collapsible::primitives`.
pub mod primitives {
    pub use crate::collapsible_primitives::{Collapsible, CollapsibleContent, CollapsibleTrigger};
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{AppWindowId, Modifiers, Point, Px, Rect, Size, SvgId, SvgService};
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::tree::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
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

    impl fret_core::MaterialService for FakeServices {
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

    fn render(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Option<Model<bool>>,
        default_open: bool,
        disabled: bool,
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
                vec![cx.keyed("collapsible-root", |cx| {
                    let collapsible = if let Some(open) = open.clone() {
                        Collapsible::new(open)
                    } else {
                        Collapsible::uncontrolled(default_open)
                    }
                    .disabled(disabled);

                    collapsible.into_element_with_open_model(
                        cx,
                        |cx, open, is_open| {
                            CollapsibleTrigger::new(open, vec![cx.text("Trigger")])
                                .into_element(cx, is_open)
                        },
                        |cx| CollapsibleContent::new(vec![cx.text("Content")]).into_element(cx),
                    )
                })]
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
        let mut services = FakeServices;

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(open.clone()),
            false,
            false,
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
            Some(open.clone()),
            false,
            false,
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
        let mut services = FakeServices;

        let _ = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(open.clone()),
            false,
            false,
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
                Some(open.clone()),
                false,
                false,
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
            Some(open.clone()),
            false,
            false,
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
                Some(open.clone()),
                false,
                false,
            );
        }
        assert!(!snapshot_has_label(&ui, "Content"));
    }

    #[test]
    fn collapsible_uncontrolled_applies_default_open_once_and_allows_toggle() {
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

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices;

        // First render: default_open=true should mount the content subtree.
        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            None,
            true,
            false,
        );
        assert!(snapshot_has_label(&ui, "Content"));

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

        // After enough frames for close presence to finish, content should unmount and not reopen
        // even though default_open stays true on each render.
        for _ in 0..24 {
            let _ = render(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                None,
                true,
                false,
            );
        }
        assert!(!snapshot_has_label(&ui, "Content"));
    }

    #[test]
    fn collapsible_trigger_controls_resolves_to_content_when_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices;

        // Render enough frames for measurement/presence to settle.
        for _ in 0..4 {
            let _ = render(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                Some(open.clone()),
                false,
                false,
            );
        }

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.role == fret_core::SemanticsRole::Button)
            .expect("trigger node");

        assert!(
            !trigger_node.controls.is_empty(),
            "expected trigger controls relationship to resolve when content is mounted"
        );
    }

    #[test]
    fn collapsible_root_disabled_marks_trigger_disabled_and_prevents_space_toggle() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(open.clone()),
            false,
            true,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.role == fret_core::SemanticsRole::Button)
            .expect("trigger node");
        assert!(
            trigger_node.flags.disabled,
            "expected trigger to be disabled when root is disabled"
        );

        let focusable = ui.first_focusable_descendant_including_declarative(&mut app, window, root);
        assert!(
            focusable.is_none(),
            "disabled root should make trigger non-focusable"
        );

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
            Some(open.clone()),
            false,
            true,
        );

        let is_open = app.models().get_copied(&open).unwrap_or(false);
        assert!(
            !is_open,
            "disabled root should prevent trigger activation from toggling open state"
        );
    }

    #[test]
    fn collapsible_custom_trigger_receives_expanded_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let render_custom =
            |ui: &mut UiTree<App>, app: &mut App, services: &mut dyn fret_core::UiServices| {
                app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
                app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
                let root = fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "collapsible-custom-trigger",
                    |cx| {
                        vec![Collapsible::new(open.clone()).into_element_with_open_model(
                            cx,
                            |cx, open, _is_open| {
                                crate::button::Button::new("Toggle")
                                    .toggle_model(open)
                                    .into_element(cx)
                            },
                            |cx| CollapsibleContent::new(vec![cx.text("Content")]).into_element(cx),
                        )]
                    },
                );
                ui.set_root(root);
                ui.request_semantics_snapshot();
                ui.layout_all(app, services, bounds, 1.0);
            };

        render_custom(&mut ui, &mut app, &mut services);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.role == fret_core::SemanticsRole::Button)
            .expect("custom trigger node");
        assert!(
            !trigger_node.flags.expanded,
            "custom trigger should expose expanded=false while closed"
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        for _ in 0..4 {
            render_custom(&mut ui, &mut app, &mut services);
        }

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| n.role == fret_core::SemanticsRole::Button)
            .expect("custom trigger node");
        assert!(
            trigger_node.flags.expanded,
            "custom trigger should expose expanded=true while open"
        );
    }

    #[test]
    fn collapsible_nested_primitives_surface_is_exposed() {
        let _ = crate::collapsible::primitives::Collapsible::new();
        let _ = crate::collapsible::primitives::CollapsibleTrigger::new(Vec::<
            fret_ui::element::AnyElement,
        >::new());
        let _ = crate::collapsible::primitives::CollapsibleContent::new(Vec::<
            fret_ui::element::AnyElement,
        >::new());
    }

    #[test]
    fn curated_facade_exposes_collapsible_parts_aliases() {
        let _ = crate::facade::CollapsibleRoot::new();
        let _ =
            crate::facade::CollapsibleTriggerPart::new(Vec::<fret_ui::element::AnyElement>::new());
        let _ =
            crate::facade::CollapsibleContentPart::new(Vec::<fret_ui::element::AnyElement>::new());
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

    #[test]
    fn collapsible_helper_accepts_typed_trigger_and_content() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(true);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(240.0), Px(160.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            collapsible(
                open.clone(),
                |_cx, is_open| {
                    let label = if is_open { "Open" } else { "Closed" };
                    fret_ui_kit::ui::text(label)
                },
                |_cx| fret_ui_kit::ui::text("Body"),
            )
            .into_element(cx)
        });

        assert!(contains_text(&element, "Open"));
        assert!(contains_text(&element, "Body"));
    }

    #[test]
    fn collapsible_uncontrolled_helper_accepts_typed_trigger_and_content() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(240.0), Px(160.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            collapsible_uncontrolled(
                true,
                |_cx, _open, is_open| {
                    let label = if is_open {
                        "Uncontrolled Open"
                    } else {
                        "Uncontrolled Closed"
                    };
                    fret_ui_kit::ui::text(label)
                },
                |_cx| fret_ui_kit::ui::text("Body"),
            )
            .into_element(cx)
        });

        assert!(contains_text(&element, "Uncontrolled Open"));
        assert!(contains_text(&element, "Body"));
    }
}
