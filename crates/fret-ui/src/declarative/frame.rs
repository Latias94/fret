use super::prelude::*;

#[derive(Default)]
pub(crate) struct ElementFrame {
    pub(super) windows: HashMap<AppWindowId, WindowFrame>,
}

pub(crate) struct WindowFrame {
    pub(super) frame_id: FrameId,
    pub(crate) instances: HashMap<NodeId, ElementRecord>,
    pub(crate) children: HashMap<NodeId, Vec<NodeId>>,
}

impl Default for WindowFrame {
    fn default() -> Self {
        Self {
            frame_id: FrameId(0),
            instances: HashMap::new(),
            children: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ElementInstance {
    Container(ContainerProps),
    Semantics(crate::element::SemanticsProps),
    SemanticFlex(crate::element::SemanticFlexProps),
    FocusScope(FocusScopeProps),
    InteractivityGate(crate::element::InteractivityGateProps),
    Opacity(crate::element::OpacityProps),
    EffectLayer(crate::element::EffectLayerProps),
    ViewCache(crate::element::ViewCacheProps),
    VisualTransform(VisualTransformProps),
    RenderTransform(crate::element::RenderTransformProps),
    FractionalRenderTransform(crate::element::FractionalRenderTransformProps),
    Anchored(crate::element::AnchoredProps),
    Pressable(PressableProps),
    PointerRegion(PointerRegionProps),
    InternalDragRegion(crate::element::InternalDragRegionProps),
    DismissibleLayer(DismissibleLayerProps),
    RovingFlex(crate::element::RovingFlexProps),
    Stack(StackProps),
    Spacer(SpacerProps),
    Text(TextProps),
    StyledText(crate::element::StyledTextProps),
    SelectableText(crate::element::SelectableTextProps),
    TextInput(crate::element::TextInputProps),
    TextArea(crate::element::TextAreaProps),
    ResizablePanelGroup(crate::element::ResizablePanelGroupProps),
    VirtualList(crate::element::VirtualListProps),
    Flex(FlexProps),
    Grid(crate::element::GridProps),
    Image(crate::element::ImageProps),
    Canvas(crate::element::CanvasProps),
    ViewportSurface(crate::element::ViewportSurfaceProps),
    SvgIcon(crate::element::SvgIconProps),
    Spinner(SpinnerProps),
    HoverRegion(HoverRegionProps),
    WheelRegion(crate::element::WheelRegionProps),
    Scroll(crate::element::ScrollProps),
    Scrollbar(crate::element::ScrollbarProps),
}

impl ElementInstance {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Container(_) => "Container",
            Self::Semantics(_) => "Semantics",
            Self::SemanticFlex(_) => "SemanticFlex",
            Self::FocusScope(_) => "FocusScope",
            Self::InteractivityGate(_) => "InteractivityGate",
            Self::Opacity(_) => "Opacity",
            Self::EffectLayer(_) => "EffectLayer",
            Self::ViewCache(_) => "ViewCache",
            Self::VisualTransform(_) => "VisualTransform",
            Self::RenderTransform(_) => "RenderTransform",
            Self::FractionalRenderTransform(_) => "FractionalRenderTransform",
            Self::Anchored(_) => "Anchored",
            Self::Pressable(_) => "Pressable",
            Self::PointerRegion(_) => "PointerRegion",
            Self::InternalDragRegion(_) => "InternalDragRegion",
            Self::DismissibleLayer(_) => "DismissibleLayer",
            Self::RovingFlex(_) => "RovingFlex",
            Self::Stack(_) => "Stack",
            Self::Spacer(_) => "Spacer",
            Self::Text(_) => "Text",
            Self::StyledText(_) => "StyledText",
            Self::SelectableText(_) => "SelectableText",
            Self::TextInput(_) => "TextInput",
            Self::TextArea(_) => "TextArea",
            Self::ResizablePanelGroup(_) => "ResizablePanelGroup",
            Self::VirtualList(_) => "VirtualList",
            Self::Flex(_) => "Flex",
            Self::Grid(_) => "Grid",
            Self::Image(_) => "Image",
            Self::Canvas(_) => "Canvas",
            Self::ViewportSurface(_) => "ViewportSurface",
            Self::SvgIcon(_) => "SvgIcon",
            Self::Spinner(_) => "Spinner",
            Self::HoverRegion(_) => "HoverRegion",
            Self::WheelRegion(_) => "WheelRegion",
            Self::Scroll(_) => "Scroll",
            Self::Scrollbar(_) => "Scrollbar",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ElementRecord {
    pub element: GlobalElementId,
    pub instance: ElementInstance,
}

#[derive(Clone)]
pub(crate) struct DismissibleLayerProps {
    pub layout: LayoutStyle,
    pub enabled: bool,
}

impl std::fmt::Debug for DismissibleLayerProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = f.debug_struct("DismissibleLayerProps");
        out.field("layout", &self.layout)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl Default for DismissibleLayerProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
        Self {
            layout,
            enabled: true,
        }
    }
}

pub(crate) fn element_record_for_node<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    node: NodeId,
) -> Option<ElementRecord> {
    app.with_global_mut_untracked(ElementFrame::default, |frame, _app| {
        frame
            .windows
            .get(&window)
            .and_then(|w| w.instances.get(&node))
            .cloned()
    })
}

pub(crate) fn with_window_frame_mut<H: UiHost, R>(
    app: &mut H,
    window: AppWindowId,
    f: impl FnOnce(&mut WindowFrame) -> R,
) -> R {
    app.with_global_mut_untracked(ElementFrame::default, |frame, _app| {
        let window_frame = frame.windows.entry(window).or_default();
        f(window_frame)
    })
}

#[derive(Debug, Clone)]
pub(crate) struct ScrollHandleBinding {
    pub handle_key: usize,
    pub element: GlobalElementId,
    pub handle: crate::scroll::ScrollHandle,
}

#[derive(Default)]
pub(crate) struct ScrollHandleRegistry {
    pub(super) windows: HashMap<AppWindowId, WindowScrollHandleRegistry>,
}

pub(crate) struct WindowScrollHandleRegistry {
    pub(super) frame_id: FrameId,
    pub(super) by_handle: HashMap<usize, Vec<GlobalElementId>>,
    pub(super) handles: HashMap<usize, crate::scroll::ScrollHandle>,
    pub(super) last_revision: HashMap<usize, u64>,
    pub(super) last_offset: HashMap<usize, fret_core::Point>,
    pub(super) last_viewport: HashMap<usize, fret_core::Size>,
    pub(super) last_content: HashMap<usize, fret_core::Size>,
}

impl Default for WindowScrollHandleRegistry {
    fn default() -> Self {
        Self {
            frame_id: FrameId(0),
            by_handle: HashMap::new(),
            handles: HashMap::new(),
            last_revision: HashMap::new(),
            last_offset: HashMap::new(),
            last_viewport: HashMap::new(),
            last_content: HashMap::new(),
        }
    }
}

fn prepare_window_scroll_registry_for_frame(
    registry: &mut WindowScrollHandleRegistry,
    frame_id: FrameId,
) {
    if registry.frame_id != frame_id {
        registry.frame_id = frame_id;
        registry.by_handle.clear();
        registry.handles.clear();
    }
}

pub(crate) fn register_scroll_handle_bindings_batch<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    frame_id: FrameId,
    bindings: impl IntoIterator<Item = ScrollHandleBinding>,
) {
    app.with_global_mut_untracked(ScrollHandleRegistry::default, |registry, _app| {
        let window_registry = registry.windows.entry(window).or_default();
        prepare_window_scroll_registry_for_frame(window_registry, frame_id);

        for binding in bindings {
            let handle_key = binding.handle_key;
            let element = binding.element;
            let handle = binding.handle;
            window_registry
                .by_handle
                .entry(handle_key)
                .or_default()
                .push(element);
            window_registry.handles.entry(handle_key).or_insert(handle);
        }
    });
}

pub(crate) fn bound_elements_for_scroll_handle<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    handle_key: usize,
) -> Vec<GlobalElementId> {
    app.with_global_mut_untracked(ScrollHandleRegistry::default, |registry, _app| {
        registry
            .windows
            .get(&window)
            .and_then(|window_registry| window_registry.by_handle.get(&handle_key))
            .cloned()
            .unwrap_or_default()
    })
}

pub(crate) fn take_changed_scroll_handle_keys<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
) -> Vec<ScrollHandleChange> {
    app.with_global_mut_untracked(ScrollHandleRegistry::default, |registry, _app| {
        let Some(window_registry) = registry.windows.get_mut(&window) else {
            return Vec::new();
        };

        let mut changed: Vec<ScrollHandleChange> = Vec::new();
        for (&handle_key, handle) in window_registry.handles.iter() {
            let revision = handle.revision();
            let offset = handle.offset();
            let viewport = handle.viewport_size();
            let content = handle.content_size();
            let prev = window_registry.last_revision.get(&handle_key).copied();
            if prev != Some(revision) {
                let prev_offset = window_registry.last_offset.get(&handle_key).copied();
                let prev_viewport = window_registry.last_viewport.get(&handle_key).copied();
                let prev_content = window_registry.last_content.get(&handle_key).copied();

                let offset_changed = prev_offset != Some(offset);
                let viewport_changed = prev_viewport != Some(viewport);
                let content_changed = prev_content != Some(content);

                // If the revision changed but none of the observable values changed, treat it as
                // layout-affecting (e.g. deferred scroll-to-item requests that are consumed during
                // layout). Otherwise, treat it as "transform-only": recompute hit-testing and
                // repaint without forcing a layout pass.
                let kind = if !offset_changed && !viewport_changed && !content_changed {
                    ScrollHandleChangeKind::Layout
                } else {
                    ScrollHandleChangeKind::HitTestOnly
                };

                changed.push(ScrollHandleChange {
                    handle_key,
                    kind,
                    revision,
                    prev_revision: prev,
                    offset,
                    prev_offset,
                    viewport,
                    prev_viewport,
                    content,
                    prev_content,
                    offset_changed,
                    viewport_changed,
                    content_changed,
                });
            }
            window_registry.last_revision.insert(handle_key, revision);
            window_registry.last_offset.insert(handle_key, offset);
            window_registry.last_viewport.insert(handle_key, viewport);
            window_registry.last_content.insert(handle_key, content);
        }
        changed
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollHandleChangeKind {
    Layout,
    HitTestOnly,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ScrollHandleChange {
    pub handle_key: usize,
    pub kind: ScrollHandleChangeKind,
    pub revision: u64,
    pub prev_revision: Option<u64>,
    pub offset: Point,
    pub prev_offset: Option<Point>,
    pub viewport: Size,
    pub prev_viewport: Option<Size>,
    pub content: Size,
    pub prev_content: Option<Size>,
    pub offset_changed: bool,
    pub viewport_changed: bool,
    pub content_changed: bool,
}

pub(crate) fn element_id_map_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
) -> HashMap<u64, NodeId> {
    app.with_global_mut_untracked(ElementFrame::default, |frame, _app| {
        frame
            .windows
            .get(&window)
            .map(|w| {
                let mut out = HashMap::with_capacity(w.instances.len());
                for (node, record) in w.instances.iter() {
                    out.insert(record.element.0, *node);
                }
                out
            })
            .unwrap_or_default()
    })
}

pub(crate) fn layout_style_for_instance(instance: &ElementInstance) -> LayoutStyle {
    match instance {
        ElementInstance::Container(p) => p.layout,
        ElementInstance::Semantics(p) => p.layout,
        ElementInstance::SemanticFlex(p) => p.flex.layout,
        ElementInstance::FocusScope(p) => p.layout,
        ElementInstance::InteractivityGate(p) => p.layout,
        ElementInstance::Opacity(p) => p.layout,
        ElementInstance::EffectLayer(p) => p.layout,
        ElementInstance::ViewCache(p) => p.layout,
        ElementInstance::VisualTransform(p) => p.layout,
        ElementInstance::RenderTransform(p) => p.layout,
        ElementInstance::FractionalRenderTransform(p) => p.layout,
        ElementInstance::Anchored(p) => p.layout,
        ElementInstance::Pressable(p) => p.layout,
        ElementInstance::PointerRegion(p) => p.layout,
        ElementInstance::InternalDragRegion(p) => p.layout,
        ElementInstance::DismissibleLayer(p) => p.layout,
        ElementInstance::RovingFlex(p) => p.flex.layout,
        ElementInstance::Stack(p) => p.layout,
        ElementInstance::Spacer(p) => p.layout,
        ElementInstance::Text(p) => p.layout,
        ElementInstance::StyledText(p) => p.layout,
        ElementInstance::SelectableText(p) => p.layout,
        ElementInstance::TextInput(p) => p.layout,
        ElementInstance::TextArea(p) => p.layout,
        ElementInstance::ResizablePanelGroup(p) => p.layout,
        ElementInstance::VirtualList(p) => p.layout,
        ElementInstance::Flex(p) => p.layout,
        ElementInstance::Grid(p) => p.layout,
        ElementInstance::Image(p) => p.layout,
        ElementInstance::Canvas(p) => p.layout,
        ElementInstance::ViewportSurface(p) => p.layout,
        ElementInstance::SvgIcon(p) => p.layout,
        ElementInstance::Spinner(p) => p.layout,
        ElementInstance::HoverRegion(p) => p.layout,
        ElementInstance::WheelRegion(p) => p.layout,
        ElementInstance::Scroll(p) => p.layout,
        ElementInstance::Scrollbar(p) => p.layout,
    }
}

pub(crate) fn layout_style_for_node<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    node: NodeId,
) -> LayoutStyle {
    element_record_for_node(app, window, node)
        .map(|r| layout_style_for_instance(&r.instance))
        .unwrap_or_default()
}
