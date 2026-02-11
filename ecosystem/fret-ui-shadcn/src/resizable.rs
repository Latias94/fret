use fret_core::{Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ContainerProps, ResizablePanelGroupProps};
use fret_ui::{ElementContext, ResizablePanelGroupStyle, Theme, UiHost};
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::resizable as resizable_recipe;

#[derive(Clone)]
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
        let theme = Theme::global(&*cx.app).clone();
        let layout = decl_style::layout_style(&theme, self.layout.relative().w_full().h_full());

        let props = ContainerProps {
            layout,
            padding: Edges::all(Px(0.0)),
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

#[derive(Debug, Clone)]
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

#[derive(Clone)]
pub struct ResizablePanelGroup {
    axis: fret_core::Axis,
    model: Model<Vec<f32>>,
    disabled: bool,
    layout: LayoutRefinement,
    style: Option<ResizablePanelGroupStyle>,
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
    entries: Vec<ResizableEntry>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let mut style =
        style.unwrap_or_else(|| resizable_recipe::default_resizable_panel_group_style(&theme));

    let mut panels: Vec<ResizablePanel> = Vec::new();
    let mut saw_handles = false;
    let mut with_handle = false;
    for e in entries {
        match e {
            ResizableEntry::Panel(p) => panels.push(p),
            ResizableEntry::Handle(h) => {
                saw_handles = true;
                with_handle |= h.with_handle;
                if h.disabled {
                    // Per-handle disabling is not supported yet; treat as a no-op marker.
                }
            }
        }
    }
    if saw_handles && panels.len() >= 2 {
        // We currently don't render per-handle elements; handles are painted by the runtime group.
        // This keeps the shadcn taxonomy surface while preserving a runtime-owned drag contract.
    }

    if with_handle {
        // shadcn/ui's `withHandle` adds a visible grip. Fret currently paints a uniform handle,
        // so we approximate by making it thicker.
        style.paint_device_px = style.paint_device_px.max(4.0);
    }

    let min_px: Vec<Px> = panels.iter().map(|p| p.min_px).collect();
    let children: Vec<AnyElement> = panels.into_iter().map(|p| p.into_element(cx)).collect();

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
