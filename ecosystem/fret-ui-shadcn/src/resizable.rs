use std::sync::Arc;

use fret_core::{Corners, Edges, Px};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, ResizablePanelGroupProps, SemanticsProps,
};
use fret_ui::{ElementContext, ResizablePanelGroupStyle, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::resizable as resizable_recipe;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius};

pub struct ResizablePanel {
    min_px: Px,
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for ResizablePanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResizablePanel")
            .field("min_px", &self.min_px)
            .field("layout", &self.layout)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl ResizablePanel {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            min_px: Px(120.0),
            layout: LayoutRefinement::default(),
            children,
        }
    }

    pub fn min_px(mut self, min_px: Px) -> Self {
        self.min_px = min_px;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let layout = decl_style::layout_style(&theme, self.layout.relative().w_full().h_full());

        let props = ContainerProps {
            layout,
            padding: Edges::all(Px(0.0)).into(),
            background: None,
            shadow: None,
            border: Edges::all(Px(0.0)),
            border_color: None,
            corner_radii: Corners::all(Px(0.0)),
            ..Default::default()
        };

        let children = self.children;
        cx.container(props, |_cx| children)
    }
}

#[derive(Clone)]
pub struct ResizableHandle {
    disabled: bool,
    with_handle: bool,
}

impl std::fmt::Debug for ResizableHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResizableHandle")
            .field("disabled", &self.disabled)
            .field("with_handle", &self.with_handle)
            .finish()
    }
}

impl ResizableHandle {
    pub fn new() -> Self {
        Self {
            disabled: false,
            with_handle: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Show a more prominent handle bar.
    ///
    /// Note: the handle is still runtime-owned; this hints the chrome used to paint it.
    pub fn with_handle(mut self, with_handle: bool) -> Self {
        self.with_handle = with_handle;
        self
    }
}

#[derive(Debug)]
pub enum ResizableEntry {
    Panel(ResizablePanel),
    Handle(ResizableHandle),
}

impl From<ResizablePanel> for ResizableEntry {
    fn from(value: ResizablePanel) -> Self {
        Self::Panel(value)
    }
}

impl From<ResizableHandle> for ResizableEntry {
    fn from(value: ResizableHandle) -> Self {
        Self::Handle(value)
    }
}

pub struct ResizablePanelGroup {
    axis: fret_core::Axis,
    model: Model<Vec<f32>>,
    disabled: bool,
    layout: LayoutRefinement,
    style: Option<ResizablePanelGroupStyle>,
    test_id_prefix: Option<Arc<str>>,
    entries: Vec<ResizableEntry>,
}

impl std::fmt::Debug for ResizablePanelGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResizablePanelGroup")
            .field("axis", &self.axis)
            .field("model", &"<model>")
            .field("disabled", &self.disabled)
            .field("layout", &self.layout)
            .field("entries_len", &self.entries.len())
            .finish()
    }
}

impl ResizablePanelGroup {
    pub fn new(model: Model<Vec<f32>>) -> Self {
        Self {
            axis: fret_core::Axis::Horizontal,
            model,
            disabled: false,
            layout: LayoutRefinement::default(),
            style: None,
            test_id_prefix: None,
            entries: Vec::new(),
        }
    }

    pub fn axis(mut self, axis: fret_core::Axis) -> Self {
        self.axis = axis;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn style(mut self, style: ResizablePanelGroupStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Prefix used to stamp deterministic automation ids onto the group's splitter semantics.
    ///
    /// When set, handles are assigned `"{prefix}.splitter.{ix}"` test ids; otherwise the default
    /// `resizable.splitter.{ix}` ids are used.
    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    pub fn entries(mut self, entries: impl IntoIterator<Item = ResizableEntry>) -> Self {
        self.entries = entries.into_iter().collect();
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        resizable_panel_group_with_entries(
            cx,
            self.axis,
            self.model,
            self.disabled,
            self.layout,
            self.style,
            self.test_id_prefix,
            self.entries,
        )
    }
}

fn resizable_panel_group_with_entries<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    axis: fret_core::Axis,
    model: Model<Vec<f32>>,
    disabled: bool,
    layout: LayoutRefinement,
    style: Option<ResizablePanelGroupStyle>,
    test_id_prefix: Option<Arc<str>>,
    entries: Vec<ResizableEntry>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    let mut style = style.unwrap_or_else(|| {
        resizable_recipe::default_resizable_panel_group_style(Theme::global(&*cx.app))
    });

    let mut panels: Vec<ResizablePanel> = Vec::new();
    let mut saw_handles = false;
    let mut handle_grips: Vec<bool> = Vec::new();
    let mut any_grip = false;
    for e in entries {
        match e {
            ResizableEntry::Panel(p) => panels.push(p),
            ResizableEntry::Handle(h) => {
                saw_handles = true;
                handle_grips.push(h.with_handle);
                any_grip |= h.with_handle;
                if h.disabled {
                    // Per-handle disabling is not supported yet; treat as a no-op marker.
                }
            }
        }
    }
    let panels_len = panels.len();

    if any_grip {
        // shadcn/ui's `withHandle` adds a visible grip on top of the handle line.
        // We approximate the DOM behavior by ensuring the runtime hit region is wide enough to
        // host the grip without constraining it to a 1px gap.
        style.hit_thickness = style.hit_thickness.max(Px(16.0));
    }

    let min_px: Vec<Px> = panels.iter().map(|p| p.min_px).collect();

    let weights = cx.app.models().get_cloned(&model).unwrap_or_default();
    let total_weight: f32 = weights.iter().copied().filter(|v| v.is_finite()).sum();

    let handle_orientation = match axis {
        // Horizontal panel layout => vertical splitter handle line.
        fret_core::Axis::Horizontal => fret_core::SemanticsOrientation::Vertical,
        // Vertical panel layout => horizontal splitter handle line.
        fret_core::Axis::Vertical => fret_core::SemanticsOrientation::Horizontal,
    };

    let mut children: Vec<AnyElement> = Vec::new();
    for (panel_ix, panel) in panels.into_iter().enumerate() {
        children.push(panel.into_element(cx));

        if saw_handles && panels_len >= 2 && panel_ix + 1 < panels_len {
            let handle_ix = panel_ix;
            let with_handle = handle_grips.get(handle_ix).copied().unwrap_or(false);

            let value = if total_weight.is_finite() && total_weight > 0.0 {
                let mut prefix = 0.0f32;
                for w in weights.iter().take(handle_ix + 1) {
                    if w.is_finite() {
                        prefix += *w;
                    }
                }
                Some((prefix / total_weight) as f64)
            } else {
                None
            };

            let mut semantics_layout = LayoutStyle::default();
            semantics_layout.size.width = Length::Fill;
            semantics_layout.size.height = Length::Fill;

            let test_id: Arc<str> = match &test_id_prefix {
                Some(prefix) => Arc::from(format!("{prefix}.splitter.{handle_ix}")),
                None => Arc::from(format!("resizable.splitter.{handle_ix}")),
            };
            let mut props = SemanticsProps {
                layout: semantics_layout,
                role: fret_core::SemanticsRole::Splitter,
                label: Some(Arc::from("Resize")),
                test_id: Some(test_id),
                orientation: Some(handle_orientation),
                numeric_value: value,
                min_numeric_value: Some(0.0),
                max_numeric_value: Some(1.0),
                numeric_value_step: Some(0.01),
                numeric_value_jump: Some(0.1),
                focusable: true,
                value_editable: Some(true),
                disabled,
                ..Default::default()
            };

            // Prefer omitting a numeric value surface when the fractions model is missing or
            // degenerate; this keeps `SetValue` gated off until we can compute a stable value.
            if value.is_none() {
                props.numeric_value_step = None;
                props.numeric_value_jump = None;
            }

            let theme = theme.clone();
            let handle = cx.semantics(props, move |cx| {
                vec![cx.hit_test_gate(false, move |cx| {
                    if !with_handle {
                        return Vec::<AnyElement>::new();
                    }

                    let (w, h, icon) = match axis {
                        fret_core::Axis::Horizontal => (
                            Px(12.0),
                            Px(16.0),
                            IconId::new_static("lucide.grip-vertical"),
                        ),
                        fret_core::Axis::Vertical => (
                            Px(16.0),
                            Px(12.0),
                            IconId::new_static("lucide.grip-horizontal"),
                        ),
                    };

                    let bg = theme.color_token("border");
                    let fg = theme.color_token("foreground");

                    let grip = cx.container(
                        decl_style::container_props(
                            &theme,
                            ChromeRefinement::default()
                                .bg(ColorRef::Color(bg))
                                .border_1()
                                .rounded(Radius::Sm),
                            LayoutRefinement::default().w_px(w).h_px(h),
                        ),
                        move |cx| {
                            [crate::icon::icon_with(
                                cx,
                                icon.clone(),
                                Some(Px(10.0)),
                                Some(ColorRef::Color(fg)),
                            )]
                        },
                    );

                    let centered = crate::stack::hstack(
                        cx,
                        crate::stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full().h_full())
                            .items_center()
                            .justify_center(),
                        |_cx| vec![grip],
                    );

                    vec![centered]
                })]
            });
            children.push(handle);
        }
    }

    let root_layout = {
        let mut root_layout = layout;

        // Default sizing should be Fill, but MUST NOT override caller-provided constraints.
        // In particular, a Fill height behaves like a percentage height and can resolve to 0 when
        // the parent height is indefinite (a common pattern in the gallery demos).
        if root_layout.position.is_none() {
            root_layout = root_layout.relative();
        }

        let has_width = root_layout
            .size
            .as_ref()
            .and_then(|s| s.width.as_ref())
            .is_some();
        let has_height = root_layout
            .size
            .as_ref()
            .and_then(|s| s.height.as_ref())
            .is_some();

        if !has_width {
            root_layout = root_layout.w_full();
        }
        if !has_height {
            root_layout = root_layout.h_full();
        }

        decl_style::layout_style(&theme, root_layout)
    };

    let mut props = ResizablePanelGroupProps::new(axis, model);
    props.enabled = !disabled;
    props.min_px = min_px;
    props.chrome = style;
    props.layout = root_layout;

    cx.resizable_panel_group(props, |_cx| children)
}

pub fn resizable_panel_group<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Vec<f32>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = ResizableEntry>,
{
    ResizablePanelGroup::new(model)
        .entries(f(cx))
        .into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, SemanticsOrientation, Size};
    use fret_ui::element::ElementKind;

    fn apply_theme(app: &mut App) {
        crate::shadcn_themes::apply_shadcn_new_york(
            app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );
    }

    fn bounds_320x240() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        )
    }

    fn build_group(app: &mut App, window: AppWindowId, group: ResizablePanelGroup) -> AnyElement {
        fret_ui::elements::with_element_cx(app, window, bounds_320x240(), "test", |cx| {
            group.into_element(cx)
        })
    }

    fn default_entries() -> Vec<ResizableEntry> {
        vec![
            ResizablePanel::new(std::iter::empty()).into(),
            ResizableHandle::new().into(),
            ResizablePanel::new(std::iter::empty()).into(),
        ]
    }

    #[test]
    fn resizable_panel_group_defaults_use_theme_gap_and_hit_thickness() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let model = app.models_mut().insert(vec![1.0, 1.0]);
        let group = ResizablePanelGroup::new(model).entries(default_entries());

        let element = build_group(&mut app, window, group);
        let ElementKind::ResizablePanelGroup(props) = &element.kind else {
            panic!("expected ResizablePanelGroup element kind");
        };

        assert_eq!(props.chrome.gap, Px(1.0));
        assert_eq!(props.chrome.hit_thickness, Px(6.0));

        let theme = Theme::global(&app);
        assert_eq!(props.chrome.handle_color, theme.color_token("border"));
        assert_eq!(props.chrome.handle_alpha, 1.0);
        assert_eq!(props.chrome.handle_hover_alpha, 1.0);
        assert_eq!(props.chrome.handle_drag_alpha, 1.0);
    }

    #[test]
    fn resizable_handle_semantics_stamps_splitter_role_and_orientation() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let model = app.models_mut().insert(vec![1.0, 1.0]);
        let group = ResizablePanelGroup::new(model)
            .test_id_prefix("demo")
            .entries(default_entries());

        let element = build_group(&mut app, window, group);
        assert_eq!(element.children.len(), 3);

        let handle = &element.children[1];
        let ElementKind::Semantics(props) = &handle.kind else {
            panic!("expected handle child to be a Semantics element");
        };

        assert_eq!(props.role, fret_core::SemanticsRole::Splitter);
        assert_eq!(props.test_id.as_deref(), Some("demo.splitter.0"));
        assert_eq!(props.orientation, Some(SemanticsOrientation::Vertical));
        assert_eq!(props.numeric_value, Some(0.5));
        assert_eq!(props.min_numeric_value, Some(0.0));
        assert_eq!(props.max_numeric_value, Some(1.0));
        assert_eq!(props.numeric_value_step, Some(0.01));
        assert_eq!(props.numeric_value_jump, Some(0.1));
        assert!(props.focusable);
        assert_eq!(props.value_editable, Some(true));
    }

    #[test]
    fn resizable_handle_semantics_omits_value_steps_when_value_is_unknown() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let model = app.models_mut().insert(vec![0.0, 0.0]);
        let group = ResizablePanelGroup::new(model).entries(default_entries());

        let element = build_group(&mut app, window, group);
        let handle = &element.children[1];
        let ElementKind::Semantics(props) = &handle.kind else {
            panic!("expected handle child to be a Semantics element");
        };

        assert_eq!(props.numeric_value, None);
        assert_eq!(props.numeric_value_step, None);
        assert_eq!(props.numeric_value_jump, None);
    }

    #[test]
    fn resizable_handle_with_handle_increases_hit_thickness() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let model = app.models_mut().insert(vec![1.0, 1.0]);
        let group = ResizablePanelGroup::new(model).entries(vec![
            ResizablePanel::new(std::iter::empty()).into(),
            ResizableHandle::new().with_handle(true).into(),
            ResizablePanel::new(std::iter::empty()).into(),
        ]);

        let element = build_group(&mut app, window, group);
        let ElementKind::ResizablePanelGroup(props) = &element.kind else {
            panic!("expected ResizablePanelGroup element kind");
        };

        assert!(
            props.chrome.hit_thickness.0 >= 16.0,
            "expected hit_thickness >= 16px when with_handle=true, got {:?}",
            props.chrome.hit_thickness
        );
    }

    #[test]
    fn resizable_handle_grip_is_paint_only() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let model = app.models_mut().insert(vec![1.0, 1.0]);
        let group = ResizablePanelGroup::new(model).entries(vec![
            ResizablePanel::new(std::iter::empty()).into(),
            ResizableHandle::new().with_handle(true).into(),
            ResizablePanel::new(std::iter::empty()).into(),
        ]);

        let element = build_group(&mut app, window, group);
        let handle = &element.children[1];
        assert_eq!(handle.children.len(), 1);

        let hit_gate = &handle.children[0];
        let ElementKind::HitTestGate(props) = &hit_gate.kind else {
            panic!("expected resizable grip wrapper to be a HitTestGate element");
        };
        assert!(!props.hit_test);
        assert!(
            !hit_gate.children.is_empty(),
            "expected grip subtree to be present when with_handle=true"
        );
    }

    #[test]
    fn resizable_vertical_axis_flips_handle_orientation() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_theme(&mut app);

        let model = app.models_mut().insert(vec![1.0, 1.0]);
        let group = ResizablePanelGroup::new(model)
            .axis(fret_core::Axis::Vertical)
            .entries(default_entries());

        let element = build_group(&mut app, window, group);
        let handle = &element.children[1];
        let ElementKind::Semantics(props) = &handle.kind else {
            panic!("expected handle child to be a Semantics element");
        };

        assert_eq!(props.orientation, Some(SemanticsOrientation::Horizontal));
    }
}
