//! AI Elements-aligned attachment surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/attachments.tsx`.

use std::sync::Arc;

use fret_assets::{AssetCapabilities, AssetKindHint, AssetLocator, AssetRequest};
use fret_core::{Color, Corners, ImageId, Px, SemanticsRole, TextOverflow, TextWrap, ViewportFit};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, HoverRegionProps, ImageProps, InteractivityGateProps, LayoutStyle,
    Length, MarginEdge, PressableA11y, PressableProps, SemanticsDecoration, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_assets::ui::ImageSourceElementContextExt as _;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, Items, LayoutRefinement, MetricRef, Radius, Space};
use fret_ui_kit::{WidgetStateProperty, WidgetStates};
use fret_ui_shadcn::facade::{self as shadcn, Button, ButtonSize, ButtonVariant};
use fret_ui_shadcn::raw::button::ButtonStyle;

pub type OnAttachmentActivate = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, Arc<str>) + 'static>;
pub type OnAttachmentRemove = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, Arc<str>) + 'static>;

fn use_attachment_parts<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<AttachmentChildParts> {
    cx.provided::<AttachmentChildParts>().cloned()
}

fn alpha(color: Color, a: f32) -> Color {
    Color {
        a: a.clamp(0.0, 1.0),
        ..color
    }
}

fn resolve_muted(theme: &Theme) -> Color {
    theme
        .color_by_key("muted")
        .unwrap_or_else(|| theme.color_token("muted.background"))
}

fn resolve_muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"))
}

fn resolve_border(theme: &Theme) -> Color {
    theme
        .color_by_key("border")
        .unwrap_or_else(|| theme.color_token("border"))
}

fn resolve_accent_hover(theme: &Theme) -> Color {
    theme.color_token("color.menu.item.hover")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttachmentVariant {
    #[default]
    Grid,
    Inline,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentMediaCategory {
    Image,
    Video,
    Audio,
    Document,
    Source,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum AttachmentData {
    File(AttachmentFileData),
    SourceDocument(AttachmentSourceDocumentData),
}

impl AttachmentData {
    pub fn id(&self) -> &Arc<str> {
        match self {
            AttachmentData::File(f) => &f.id,
            AttachmentData::SourceDocument(s) => &s.id,
        }
    }

    pub fn media_type(&self) -> Option<&Arc<str>> {
        match self {
            AttachmentData::File(f) => f.media_type.as_ref(),
            AttachmentData::SourceDocument(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AttachmentFileData {
    pub id: Arc<str>,
    pub filename: Option<Arc<str>>,
    pub media_type: Option<Arc<str>>,
    pub size_bytes: Option<u64>,
    pub preview_image: Option<ImageId>,
    pub url: Option<Arc<str>>,
}

impl AttachmentFileData {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            filename: None,
            media_type: None,
            size_bytes: None,
            preview_image: None,
            url: None,
        }
    }

    pub fn filename(mut self, filename: impl Into<Arc<str>>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn media_type(mut self, media_type: impl Into<Arc<str>>) -> Self {
        self.media_type = Some(media_type.into());
        self
    }

    pub fn size_bytes(mut self, size_bytes: u64) -> Self {
        self.size_bytes = Some(size_bytes);
        self
    }

    pub fn preview_image(mut self, image: ImageId) -> Self {
        self.preview_image = Some(image);
        self
    }

    pub fn url(mut self, url: impl Into<Arc<str>>) -> Self {
        self.url = Some(url.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct AttachmentSourceDocumentData {
    pub id: Arc<str>,
    pub title: Option<Arc<str>>,
    pub filename: Option<Arc<str>>,
    pub url: Option<Arc<str>>,
}

impl AttachmentSourceDocumentData {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            title: None,
            filename: None,
            url: None,
        }
    }

    pub fn title(mut self, title: impl Into<Arc<str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn filename(mut self, filename: impl Into<Arc<str>>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn url(mut self, url: impl Into<Arc<str>>) -> Self {
        self.url = Some(url.into());
        self
    }
}

pub fn media_category_for(data: &AttachmentData) -> AttachmentMediaCategory {
    match data {
        AttachmentData::SourceDocument(_) => AttachmentMediaCategory::Source,
        AttachmentData::File(file) => {
            let media_type = file.media_type.as_deref().unwrap_or("");
            if media_type.starts_with("image/") {
                return AttachmentMediaCategory::Image;
            }
            if media_type.starts_with("video/") {
                return AttachmentMediaCategory::Video;
            }
            if media_type.starts_with("audio/") {
                return AttachmentMediaCategory::Audio;
            }
            if media_type.starts_with("application/") || media_type.starts_with("text/") {
                return AttachmentMediaCategory::Document;
            }
            AttachmentMediaCategory::Unknown
        }
    }
}

/// Upstream parity alias for AI Elements `getMediaCategory`.
#[inline]
pub fn get_media_category(data: &AttachmentData) -> AttachmentMediaCategory {
    media_category_for(data)
}

pub fn attachment_label_for(data: &AttachmentData) -> Arc<str> {
    match data {
        AttachmentData::SourceDocument(src) => src
            .title
            .clone()
            .or_else(|| src.filename.clone())
            .unwrap_or_else(|| Arc::<str>::from("Source")),
        AttachmentData::File(file) => {
            if let Some(filename) = file.filename.clone() {
                return filename;
            }
            match media_category_for(data) {
                AttachmentMediaCategory::Image => Arc::<str>::from("Image"),
                _ => Arc::<str>::from("Attachment"),
            }
        }
    }
}

/// Upstream parity alias for AI Elements `getAttachmentLabel`.
#[inline]
pub fn get_attachment_label(data: &AttachmentData) -> Arc<str> {
    attachment_label_for(data)
}

fn attachment_preview_request_for(data: &AttachmentData) -> Option<AssetRequest> {
    match data {
        AttachmentData::File(file)
            if media_category_for(data) == AttachmentMediaCategory::Image =>
        {
            file.url.as_ref().map(|url| {
                AssetRequest::new(AssetLocator::url(url.as_ref()))
                    .with_kind_hint(AssetKindHint::Image)
            })
        }
        _ => None,
    }
}

fn capability_gated_attachment_preview_request_for(
    data: &AttachmentData,
    capabilities: Option<AssetCapabilities>,
) -> Option<AssetRequest> {
    let request = attachment_preview_request_for(data)?;
    let supports_request = capabilities
        .map(|caps| caps.supports(&request.locator))
        .unwrap_or(false);
    supports_request.then_some(request)
}

fn resolve_attachment_preview_source_from_host(
    host: &impl fret_runtime::GlobalsHost,
    data: &AttachmentData,
) -> Option<fret_ui_assets::ImageSource> {
    let request = capability_gated_attachment_preview_request_for(
        data,
        fret_runtime::asset_capabilities(host),
    )?;
    fret_ui_assets::resolve_image_source_from_host(host, &request).ok()
}

fn resolve_attachment_preview_image<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    data: &AttachmentData,
) -> Option<ImageId> {
    match data {
        AttachmentData::File(file) => {
            if let Some(image) = file.preview_image {
                return Some(image);
            }

            let source = resolve_attachment_preview_source_from_host(&*cx.app, data)?;
            cx.use_image_source_state(&source).image
        }
        AttachmentData::SourceDocument(_) => None,
    }
}

fn media_category_icon(category: AttachmentMediaCategory) -> IconId {
    match category {
        AttachmentMediaCategory::Audio => IconId::new("lucide.music-2"),
        AttachmentMediaCategory::Document => IconId::new("lucide.file-text"),
        AttachmentMediaCategory::Image => IconId::new("lucide.image"),
        AttachmentMediaCategory::Source => IconId::new("lucide.globe"),
        AttachmentMediaCategory::Video => IconId::new("lucide.video"),
        AttachmentMediaCategory::Unknown => IconId::new("lucide.paperclip"),
    }
}

#[derive(Debug)]
pub struct Attachments {
    variant: AttachmentVariant,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl Attachments {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            variant: AttachmentVariant::Grid,
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn variant(mut self, variant: AttachmentVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let gap = MetricRef::space(Space::N2).resolve(&theme);
        let mut flex_layout = LayoutStyle::default();
        flex_layout.size.width = match self.variant {
            // Upstream shadcn: `variant=grid` => `ml-auto w-fit`.
            AttachmentVariant::Grid => Length::Auto,
            AttachmentVariant::Inline | AttachmentVariant::List => Length::Fill,
        };

        if self.variant == AttachmentVariant::Grid {
            flex_layout.margin.left = MarginEdge::Auto;
        }

        let mut props = fret_ui::element::FlexProps::default();
        props.layout = flex_layout;
        props.gap = gap.into();
        props.wrap = self.variant != AttachmentVariant::List;
        props.direction = match self.variant {
            AttachmentVariant::List => fret_core::Axis::Vertical,
            AttachmentVariant::Grid | AttachmentVariant::Inline => fret_core::Axis::Horizontal,
        };
        props.align = match self.variant {
            AttachmentVariant::List => fret_ui::element::CrossAlign::Start,
            AttachmentVariant::Grid | AttachmentVariant::Inline => {
                fret_ui::element::CrossAlign::Start
            }
        };

        let mut el = cx.flex(props, move |_cx| self.children);
        if let Some(test_id) = self.test_id {
            el = el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(test_id),
            );
        }
        el
    }
}

#[derive(Clone)]
pub struct Attachment {
    data: AttachmentData,
    variant: AttachmentVariant,
    on_activate: Option<OnAttachmentActivate>,
    on_remove: Option<OnAttachmentRemove>,
    show_media_type: bool,
    test_id: Option<Arc<str>>,
    preview_test_id: Option<Arc<str>>,
    info_test_id: Option<Arc<str>>,
    remove_test_id: Option<Arc<str>>,
    hovered_model: Option<Model<bool>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Attachment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Attachment")
            .field("data_id", &self.data.id().as_ref())
            .field("variant", &self.variant)
            .field("has_on_activate", &self.on_activate.is_some())
            .field("has_on_remove", &self.on_remove.is_some())
            .field("show_media_type", &self.show_media_type)
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl Attachment {
    pub fn new(data: AttachmentData) -> Self {
        Self {
            data,
            variant: AttachmentVariant::Grid,
            on_activate: None,
            on_remove: None,
            show_media_type: false,
            test_id: None,
            preview_test_id: None,
            info_test_id: None,
            remove_test_id: None,
            hovered_model: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn variant(mut self, variant: AttachmentVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn on_activate(mut self, on_activate: OnAttachmentActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn on_remove(mut self, on_remove: OnAttachmentRemove) -> Self {
        self.on_remove = Some(on_remove);
        self
    }

    pub fn show_media_type(mut self, show_media_type: bool) -> Self {
        self.show_media_type = show_media_type;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn preview_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.preview_test_id = Some(id.into());
        self
    }

    pub fn info_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.info_test_id = Some(id.into());
        self
    }

    pub fn remove_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.remove_test_id = Some(id.into());
        self
    }

    pub fn hovered_model(mut self, model: Model<bool>) -> Self {
        self.hovered_model = Some(model);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element_with_children<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl for<'a> Fn(&mut ElementContext<'a, H>, AttachmentChildParts) -> Vec<AnyElement>
        + 'static,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let children: Arc<
            dyn for<'a> Fn(&mut ElementContext<'a, H>, AttachmentChildParts) -> Vec<AnyElement>,
        > = Arc::new(children);

        let data = self.data;
        let variant = self.variant;

        let label = attachment_label_for(&data);
        let hover_bg_inline = resolve_accent_hover(&theme);
        let hover_bg_list = alpha(theme.color_token("accent"), 0.5);
        let hover_fg_inline = theme.color_token("accent-foreground");

        let base_chrome = match variant {
            AttachmentVariant::Grid => ChromeRefinement::default().rounded(Radius::Lg),
            AttachmentVariant::Inline => ChromeRefinement::default()
                .rounded(Radius::Md)
                .border_1()
                .border_color(ColorRef::Color(resolve_border(&theme)))
                .p(Space::N1),
            AttachmentVariant::List => ChromeRefinement::default()
                .rounded(Radius::Lg)
                .border_1()
                .border_color(ColorRef::Color(resolve_border(&theme)))
                .p(Space::N3),
        }
        .merge(self.chrome);

        let item_layout = match variant {
            AttachmentVariant::Grid => LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(96.0)))
                .h_px(MetricRef::Px(Px(96.0))),
            AttachmentVariant::Inline => LayoutRefinement::default()
                .h_px(MetricRef::Px(Px(32.0)))
                .min_w_0()
                .flex_shrink_0(),
            AttachmentVariant::List => LayoutRefinement::default().w_full().min_w_0(),
        }
        .merge(self.layout);

        let on_activate = self.on_activate;
        let on_remove = self.on_remove;
        let show_media_type = self.show_media_type;
        let test_id = self.test_id;
        let preview_test_id = self.preview_test_id;
        let info_test_id = self.info_test_id;
        let remove_test_id = self.remove_test_id;
        let hovered_model = self.hovered_model;

        let mut hover = HoverRegionProps::default();
        hover.layout = decl_style::layout_style(&theme, item_layout);

        let el = cx.hover_region(hover, move |cx, hovered| {
            let children = children.clone();
            if let Some(model) = hovered_model.clone() {
                let current = cx.watch_model(&model).layout().copied().unwrap_or(!hovered);
                if current != hovered {
                    let _ = cx.app.models_mut().update(&model, |v| *v = hovered);
                }
            }
            let row = control_chrome_pressable_with_id_props(cx, move |cx, _st, _id| {
                if let Some(on_activate) = on_activate.clone() {
                    cx.pressable_on_activate({
                        let id = data.id().clone();
                        Arc::new(move |host, action_cx, _reason| {
                            on_activate(host, action_cx, id.clone());
                            host.notify(action_cx);
                            host.request_redraw(action_cx.window);
                        })
                    });
                }

                let mut pressable = PressableProps::default();
                pressable.enabled = true;
                pressable.focusable = on_activate.is_some();
                match variant {
                    AttachmentVariant::Grid => {
                        pressable.layout.size.width = Length::Fill;
                        pressable.layout.size.height = Length::Fill;
                    }
                    AttachmentVariant::Inline => {
                        pressable.layout.size.height = Length::Fill;
                    }
                    AttachmentVariant::List => {
                        pressable.layout.size.width = Length::Fill;
                    }
                }
                pressable.a11y = PressableA11y {
                    role: Some(if on_activate.is_some() {
                        SemanticsRole::Button
                    } else {
                        SemanticsRole::Generic
                    }),
                    label: Some(label.clone()),
                    test_id: None,
                    ..Default::default()
                };

                let bg = match variant {
                    AttachmentVariant::Grid => None,
                    AttachmentVariant::Inline => hovered.then_some(hover_bg_inline),
                    AttachmentVariant::List => hovered.then_some(hover_bg_list),
                };

                let chrome_layout = match variant {
                    AttachmentVariant::Grid => LayoutRefinement::default().w_full().h_full(),
                    AttachmentVariant::Inline => LayoutRefinement::default().h_full().min_w_0(),
                    AttachmentVariant::List => LayoutRefinement::default().w_full().min_w_0(),
                };
                let mut chrome =
                    decl_style::container_props(&theme, base_chrome.clone(), chrome_layout);
                chrome.background = bg.or(chrome.background);

                let parts = AttachmentChildParts {
                    data: data.clone(),
                    variant,
                    hovered,
                    show_media_type,
                    label_color: (variant == AttachmentVariant::Inline && hovered)
                        .then_some(hover_fg_inline),
                    on_remove: on_remove.clone(),
                    preview_test_id: preview_test_id.clone(),
                    info_test_id: info_test_id.clone(),
                    remove_test_id: remove_test_id.clone(),
                };
                let content = cx.provide(parts.clone(), |cx| (children.as_ref())(cx, parts));

                (pressable, chrome, move |_cx| content)
            });

            vec![row]
        });

        if let Some(test_id) = test_id {
            return el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::ListItem)
                    .test_id(test_id),
            );
        }

        el
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_with_children(cx, move |cx, parts| {
            let preview = parts.preview().into_element(cx);
            let info = parts.info().into_element(cx);
            let remove = parts.remove().into_element(cx);

            match parts.variant() {
                AttachmentVariant::Grid => {
                    let mut overlay = ContainerProps::default();
                    overlay.layout = decl_style::layout_style(
                        &Theme::global(&*cx.app).clone(),
                        LayoutRefinement::default().relative().w_full().h_full(),
                    );

                    let abs_layout = decl_style::layout_style(
                        &Theme::global(&*cx.app).clone(),
                        LayoutRefinement::default()
                            .absolute()
                            .top_px(Px(8.0))
                            .right_px(Px(8.0)),
                    );
                    let remove = cx.interactivity_gate_props(
                        InteractivityGateProps {
                            present: true,
                            interactive: true,
                            layout: abs_layout,
                        },
                        move |_cx| vec![remove],
                    );
                    let overlay = cx.container(overlay, move |_cx| vec![preview, remove]);
                    vec![overlay]
                }
                AttachmentVariant::Inline => {
                    let theme = Theme::global(&*cx.app).clone();
                    let mut affordance_props = ContainerProps::default();
                    affordance_props.layout = decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default()
                            .relative()
                            .w_px(MetricRef::Px(Px(20.0)))
                            .h_px(MetricRef::Px(Px(20.0)))
                            .min_w(MetricRef::Px(Px(20.0)))
                            .min_h(MetricRef::Px(Px(20.0)))
                            .flex_shrink_0(),
                    );

                    let remove = cx.interactivity_gate_props(
                        InteractivityGateProps {
                            present: true,
                            interactive: parts.hovered(),
                            layout: decl_style::layout_style(
                                &theme,
                                LayoutRefinement::default()
                                    .absolute()
                                    .top_px(Px(0.0))
                                    .left_px(Px(0.0))
                                    .w_px(MetricRef::Px(Px(20.0)))
                                    .h_px(MetricRef::Px(Px(20.0))),
                            ),
                        },
                        move |_cx| vec![remove],
                    );

                    let hovered = parts.hovered();
                    let affordance = cx.container(affordance_props, move |cx| {
                        vec![
                            cx.opacity(if hovered { 0.0 } else { 1.0 }, move |_cx| vec![preview]),
                            remove,
                        ]
                    });

                    let row = ui::h_row(move |_cx| vec![affordance, info])
                        .layout(LayoutRefinement::default().min_w_0())
                        .gap(Space::N2)
                        .items(Items::Center)
                        .into_element(cx);
                    vec![row]
                }
                AttachmentVariant::List => {
                    let row = ui::h_row(move |_cx| vec![preview, info, remove])
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .gap(Space::N3)
                        .items(Items::Center)
                        .into_element(cx);
                    vec![row]
                }
            }
        })
    }
}

#[derive(Clone)]
pub struct AttachmentChildParts {
    data: AttachmentData,
    variant: AttachmentVariant,
    hovered: bool,
    show_media_type: bool,
    label_color: Option<Color>,
    on_remove: Option<OnAttachmentRemove>,
    preview_test_id: Option<Arc<str>>,
    info_test_id: Option<Arc<str>>,
    remove_test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for AttachmentChildParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachmentChildParts")
            .field("data_id", &self.data.id().as_ref())
            .field("variant", &self.variant)
            .field("hovered", &self.hovered)
            .field("show_media_type", &self.show_media_type)
            .finish()
    }
}

impl AttachmentChildParts {
    pub fn data(&self) -> &AttachmentData {
        &self.data
    }

    pub fn variant(&self) -> AttachmentVariant {
        self.variant
    }

    pub fn hovered(&self) -> bool {
        self.hovered
    }

    pub fn label(&self) -> Arc<str> {
        attachment_label_for(&self.data)
    }

    pub fn preview(&self) -> AttachmentPreview {
        AttachmentPreview::new(self.data.clone())
            .variant(self.variant)
            .test_id_opt(self.preview_test_id.clone())
    }

    pub fn info(&self) -> AttachmentInfo {
        AttachmentInfo::new(self.data.clone())
            .variant(self.variant)
            .show_media_type(self.show_media_type)
            .label_color_opt(self.label_color)
            .test_id_opt(self.info_test_id.clone())
    }

    pub fn remove(&self) -> AttachmentRemove {
        AttachmentRemove::new(self.data.id().clone())
            .variant(self.variant)
            .visible(self.hovered)
            .test_id_opt(self.remove_test_id.clone())
            .on_remove_opt(self.on_remove.clone())
    }
}

#[derive(Debug, Clone)]
pub struct AttachmentPreview {
    data: Option<AttachmentData>,
    variant: Option<AttachmentVariant>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl AttachmentPreview {
    pub fn new(data: AttachmentData) -> Self {
        Self {
            data: Some(data),
            variant: Some(AttachmentVariant::Grid),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn from_context() -> Self {
        Self {
            data: None,
            variant: None,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn variant(mut self, variant: AttachmentVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn test_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.test_id = id;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(parts) = use_attachment_parts(cx).or_else(|| {
            self.data.clone().map(|data| AttachmentChildParts {
                data,
                variant: self.variant.unwrap_or(AttachmentVariant::Grid),
                hovered: false,
                show_media_type: false,
                label_color: None,
                on_remove: None,
                preview_test_id: None,
                info_test_id: None,
                remove_test_id: None,
            })
        }) else {
            return cx.text("");
        };

        let theme = Theme::global(&*cx.app).clone();

        let muted = resolve_muted(&theme);
        let fg = resolve_muted_fg(&theme);

        let category = media_category_for(parts.data());
        let variant = self.variant.unwrap_or(parts.variant());

        let size = match variant {
            AttachmentVariant::Inline => Px(20.0),
            AttachmentVariant::Grid => Px(96.0),
            AttachmentVariant::List => Px(48.0),
        };

        let corner = match variant {
            AttachmentVariant::Inline => MetricRef::radius(Radius::Sm).resolve(&theme),
            AttachmentVariant::Grid => Px(0.0),
            AttachmentVariant::List => MetricRef::radius(Radius::Md).resolve(&theme),
        };

        let bg = match variant {
            AttachmentVariant::Grid | AttachmentVariant::List => Some(muted),
            AttachmentVariant::Inline => Some(theme.color_token("background")),
        };

        let mut wrapper = ContainerProps::default();
        wrapper.layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .w_px(MetricRef::Px(size))
                .h_px(MetricRef::Px(size))
                .flex_shrink_0()
                .merge(self.layout),
        );
        wrapper.background = bg;
        wrapper.corner_radii = Corners::all(corner);
        wrapper.snap_to_device_pixels = true;

        let preview_image = resolve_attachment_preview_image(cx, parts.data());

        let content = match preview_image {
            Some(image) if category == AttachmentMediaCategory::Image => {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Fill;
                vec![cx.image_props(ImageProps {
                    layout,
                    fit: ViewportFit::Cover,
                    opacity: 1.0,
                    uv: None,
                    ..ImageProps::new(image)
                })]
            }
            _ => vec![decl_icon::icon_with(
                cx,
                media_category_icon(category),
                Some(if variant == AttachmentVariant::Inline {
                    Px(12.0)
                } else {
                    Px(16.0)
                }),
                Some(ColorRef::Color(fg)),
            )],
        };

        let mut el = cx.container(wrapper, move |_cx| content);
        if let Some(test_id) = self.test_id {
            el = el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(test_id),
            );
        }
        el
    }
}

#[derive(Debug, Clone)]
pub struct AttachmentInfo {
    data: Option<AttachmentData>,
    variant: Option<AttachmentVariant>,
    show_media_type: bool,
    label_color: Option<Color>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl AttachmentInfo {
    pub fn new(data: AttachmentData) -> Self {
        Self {
            data: Some(data),
            variant: Some(AttachmentVariant::Grid),
            show_media_type: false,
            label_color: None,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn from_context() -> Self {
        Self {
            data: None,
            variant: None,
            show_media_type: false,
            label_color: None,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn variant(mut self, variant: AttachmentVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    pub fn show_media_type(mut self, show: bool) -> Self {
        self.show_media_type = show;
        self
    }

    pub fn label_color(mut self, color: Color) -> Self {
        self.label_color = Some(color);
        self
    }

    fn label_color_opt(mut self, color: Option<Color>) -> Self {
        self.label_color = color;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn test_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.test_id = id;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(parts) = use_attachment_parts(cx).or_else(|| {
            self.data.clone().map(|data| AttachmentChildParts {
                data,
                variant: self.variant.unwrap_or(AttachmentVariant::Grid),
                hovered: false,
                show_media_type: self.show_media_type,
                label_color: self.label_color,
                on_remove: None,
                preview_test_id: None,
                info_test_id: None,
                remove_test_id: None,
            })
        }) else {
            return cx.text("");
        };

        let variant = self.variant.unwrap_or(parts.variant());
        if variant == AttachmentVariant::Grid {
            return cx.text("");
        }

        let theme = Theme::global(&*cx.app).clone();
        let muted_fg = resolve_muted_fg(&theme);
        let label_fg = self
            .label_color
            .unwrap_or_else(|| theme.color_token("foreground"));

        let label = attachment_label_for(parts.data());
        let label_el = cx.text_props(TextProps {
            layout: decl_style::layout_style(&theme, LayoutRefinement::default().min_w_0()),
            text: label,
            style: Some(
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm)
                    .resolve(&theme),
            ),
            color: Some(label_fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        });

        let media_type_row = self
            .show_media_type
            .then_some(())
            .and_then(|_| parts.data().media_type().cloned())
            .map(|media_type| {
                cx.text_props(TextProps {
                    layout: decl_style::layout_style(&theme, LayoutRefinement::default().min_w_0()),
                    text: media_type,
                    style: Some(
                        typography::TypographyPreset::control_ui(typography::UiTextSize::Xs)
                            .resolve(&theme),
                    ),
                    color: Some(muted_fg),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Ellipsis,
                    align: fret_core::TextAlign::Start,
                    ink_overflow: Default::default(),
                })
            });

        let mut rows = vec![label_el];
        if let Some(mt) = media_type_row {
            rows.push(mt);
        }

        let mut props = ContainerProps::default();
        props.layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .min_w_0()
                .flex_grow(1.0)
                .merge(self.layout),
        );

        let mut el = cx.container(props, move |cx| {
            vec![
                ui::v_stack(move |_cx| rows)
                    .layout(LayoutRefinement::default().min_w_0())
                    .gap(Space::N0)
                    .items(Items::Start)
                    .into_element(cx),
            ]
        });

        if let Some(test_id) = self.test_id {
            el = el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(test_id),
            );
        }
        el
    }
}

#[derive(Clone)]
pub struct AttachmentRemove {
    id: Option<Arc<str>>,
    label: Arc<str>,
    on_remove: Option<OnAttachmentRemove>,
    visible: bool,
    test_id: Option<Arc<str>>,
    variant: Option<AttachmentVariant>,
}

impl std::fmt::Debug for AttachmentRemove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachmentRemove")
            .field("id", &self.id.as_deref())
            .field("has_on_remove", &self.on_remove.is_some())
            .field("visible", &self.visible)
            .field("test_id", &self.test_id.as_deref())
            .field("variant", &self.variant)
            .finish()
    }
}

impl AttachmentRemove {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self {
            id: Some(id.into()),
            label: Arc::<str>::from("Remove"),
            on_remove: None,
            visible: true,
            test_id: None,
            variant: Some(AttachmentVariant::Grid),
        }
    }

    pub fn from_context() -> Self {
        Self {
            id: None,
            label: Arc::<str>::from("Remove"),
            on_remove: None,
            visible: true,
            test_id: None,
            variant: None,
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = label.into();
        self
    }

    pub fn on_remove(mut self, on_remove: OnAttachmentRemove) -> Self {
        self.on_remove = Some(on_remove);
        self
    }

    fn on_remove_opt(mut self, on_remove: Option<OnAttachmentRemove>) -> Self {
        self.on_remove = on_remove;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn test_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.test_id = id;
        self
    }

    pub fn variant(mut self, variant: AttachmentVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    fn resolved_variant(&self, parts: Option<&AttachmentChildParts>) -> AttachmentVariant {
        self.variant
            .or_else(|| parts.map(|parts| parts.variant()))
            .unwrap_or(AttachmentVariant::Grid)
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let parts = use_attachment_parts(cx);
        let variant = self.resolved_variant(parts.as_ref());
        let on_remove = self
            .on_remove
            .or_else(|| parts.as_ref().and_then(|p| p.on_remove.clone()));
        let Some(on_remove) = on_remove else {
            return cx.text("");
        };

        let theme = Theme::global(&*cx.app).clone();

        let id = self
            .id
            .clone()
            .or_else(|| parts.as_ref().map(|p| p.data().id().clone()));
        let Some(id) = id else {
            return cx.text("");
        };
        let label = self.label.clone();
        let mut btn = Button::new("")
            .a11y_label(label.clone())
            .variant(ButtonVariant::Ghost)
            .size(match variant {
                AttachmentVariant::Grid => ButtonSize::IconXs,
                AttachmentVariant::Inline => ButtonSize::IconXs,
                AttachmentVariant::List => ButtonSize::Icon,
            })
            .icon(IconId::new("lucide.x"))
            .on_activate(Arc::new(move |host, action_cx, _reason| {
                on_remove(host, action_cx, id.clone());
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            }));

        match variant {
            AttachmentVariant::Grid => {
                // Upstream:
                // - `size-6` (24px),
                // - `rounded-full`,
                // - `bg-background/80` and `hover:bg-background`.
                let bg = theme.color_token("background");
                btn = btn
                    .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                    .style(
                        ButtonStyle::default().background(
                            WidgetStateProperty::new(Some(ColorRef::Color(alpha(bg, 0.8))))
                                .when(WidgetStates::HOVERED, Some(ColorRef::Color(bg))),
                        ),
                    );
            }
            AttachmentVariant::Inline => {
                // Upstream: `size-5` (20px) with a slightly smaller icon.
                btn = btn.leading_icon_size(Px(10.0)).refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(20.0)))
                        .h_px(MetricRef::Px(Px(20.0)))
                        .min_w(MetricRef::Px(Px(20.0)))
                        .min_h(MetricRef::Px(Px(20.0))),
                );
            }
            AttachmentVariant::List => {}
        }

        if let Some(test_id) = self.test_id {
            btn = btn.test_id(test_id);
        }

        let btn = btn.into_element(cx);
        let visible = parts.as_ref().map(|p| p.hovered()).unwrap_or(self.visible);
        let opacity = if visible { 1.0 } else { 0.0 };
        let interactive = visible;

        cx.interactivity_gate(true, interactive, move |cx| {
            vec![cx.opacity(opacity, move |_cx| vec![btn])]
        })
    }
}

#[derive(Debug)]
pub enum AttachmentHoverCardContentArg {
    Element(AnyElement),
    Builder(AttachmentHoverCardContent),
}

impl From<AnyElement> for AttachmentHoverCardContentArg {
    fn from(value: AnyElement) -> Self {
        Self::Element(value)
    }
}

impl From<AttachmentHoverCardContent> for AttachmentHoverCardContentArg {
    fn from(value: AttachmentHoverCardContent) -> Self {
        Self::Builder(value)
    }
}

#[derive(Debug)]
pub struct AttachmentHoverCard {
    trigger: AnyElement,
    content: AttachmentHoverCardContentArg,
    open_delay_frames: u32,
    close_delay_frames: u32,
    open_model: Option<Model<bool>>,
}

impl AttachmentHoverCard {
    pub fn new(trigger: AnyElement, content: impl Into<AttachmentHoverCardContentArg>) -> Self {
        Self {
            trigger,
            content: content.into(),
            open_delay_frames: 0,
            close_delay_frames: 0,
            open_model: None,
        }
    }

    pub fn open_delay_frames(mut self, frames: u32) -> Self {
        self.open_delay_frames = frames;
        self
    }

    pub fn close_delay_frames(mut self, frames: u32) -> Self {
        self.close_delay_frames = frames;
        self
    }

    pub fn open_model(mut self, model: Model<bool>) -> Self {
        self.open_model = Some(model);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let content = match self.content {
            AttachmentHoverCardContentArg::Element(content) => content,
            AttachmentHoverCardContentArg::Builder(content) => content.into_element(cx),
        };

        let mut hover = shadcn::HoverCard::new(cx, self.trigger, content)
            .open_delay_frames(self.open_delay_frames)
            .close_delay_frames(self.close_delay_frames);

        if let Some(open_model) = self.open_model {
            hover = hover.open(Some(open_model));
        }

        hover.into_element(cx)
    }
}

#[derive(Debug)]
pub struct AttachmentHoverCardTrigger {
    child: AnyElement,
}

impl AttachmentHoverCardTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

#[derive(Debug)]
pub struct AttachmentHoverCardContent {
    children: Vec<AnyElement>,
    align: shadcn::HoverCardAlign,
    side: Option<shadcn::HoverCardSide>,
    side_offset: Option<Px>,
    align_offset: Option<Px>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

impl AttachmentHoverCardContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            align: shadcn::HoverCardAlign::Start,
            side: None,
            side_offset: None,
            align_offset: None,
            chrome: ChromeRefinement::default().p(Space::N2),
            layout: LayoutRefinement::default().w_auto().min_w_0(),
            test_id: None,
        }
    }

    pub fn align(mut self, align: shadcn::HoverCardAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side(mut self, side: shadcn::HoverCardSide) -> Self {
        self.side = Some(side);
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = Some(offset);
        self
    }

    pub fn align_offset(mut self, offset: Px) -> Self {
        self.align_offset = Some(offset);
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    fn into_builder(self) -> shadcn::HoverCardContent {
        let mut content = shadcn::HoverCardContent::new(self.children)
            .align(self.align)
            .refine_style(self.chrome)
            .refine_layout(self.layout);

        if let Some(side) = self.side {
            content = content.side(side);
        }
        if let Some(side_offset) = self.side_offset {
            content = content.side_offset(side_offset);
        }
        if let Some(align_offset) = self.align_offset {
            content = content.align_offset(align_offset);
        }
        if let Some(test_id) = self.test_id {
            content = content.test_id(test_id);
        }

        content
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_builder().into_element(cx)
    }
}

#[derive(Debug)]
pub struct AttachmentEmpty {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl AttachmentEmpty {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = resolve_muted_fg(&theme);

        let chrome = ChromeRefinement::default().p(Space::N4).merge(self.chrome);
        let mut props = decl_style::container_props(
            &theme,
            chrome,
            LayoutRefinement::default().merge(self.layout),
        );

        props.background = None;

        let content = if self.children.is_empty() {
            vec![
                cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: Arc::<str>::from("No attachments"),
                    style: Some(
                        typography::TypographyPreset::control_ui(typography::UiTextSize::Sm)
                            .resolve(&theme),
                    ),
                    color: Some(fg),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Ellipsis,
                    align: fret_core::TextAlign::Start,
                    ink_overflow: Default::default(),
                }),
            ]
        } else {
            self.children
        };

        let centered = ui::h_flex(move |_cx| content)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .items_center()
            .justify_center()
            .into_element(cx);

        let mut el = cx.container(props, move |_cx| vec![centered]);
        if let Some(test_id) = self.test_id {
            el = el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(test_id),
            );
        }
        el
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_parts(variant: AttachmentVariant) -> AttachmentChildParts {
        AttachmentChildParts {
            data: AttachmentData::File(
                AttachmentFileData::new("att-test")
                    .filename("sample.txt")
                    .media_type("text/plain"),
            ),
            variant,
            hovered: false,
            show_media_type: false,
            label_color: None,
            on_remove: None,
            preview_test_id: None,
            info_test_id: None,
            remove_test_id: None,
        }
    }

    #[test]
    fn attachment_remove_from_context_uses_parent_variant() {
        let remove = AttachmentRemove::from_context();

        assert_eq!(
            remove.resolved_variant(Some(&sample_parts(AttachmentVariant::Inline))),
            AttachmentVariant::Inline
        );
        assert_eq!(
            remove.resolved_variant(Some(&sample_parts(AttachmentVariant::List))),
            AttachmentVariant::List
        );
    }

    #[test]
    fn attachment_hover_card_content_defaults_match_ai_elements_docs() {
        let content = AttachmentHoverCardContent::new(Vec::<AnyElement>::new());

        assert_eq!(content.align, shadcn::HoverCardAlign::Start);
        assert!(content.side.is_none());
        assert!(content.side_offset.is_none());
        assert!(content.align_offset.is_none());
        assert!(content.test_id.is_none());
    }

    #[test]
    fn attachment_hover_card_trigger_is_layout_transparent() {
        use fret_app::App;
        use fret_core::{AppWindowId, Point, Px, Rect, Size};

        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let child = cx.text("trigger");
            let child_id = child.id;

            let trigger = AttachmentHoverCardTrigger::new(child).into_element(cx);

            assert_eq!(trigger.id, child_id);
        });
    }

    #[test]
    fn attachment_preview_request_uses_url_for_image_files() {
        let data = AttachmentData::File(
            AttachmentFileData::new("att-image")
                .filename("preview.png")
                .media_type("image/png")
                .url("https://example.com/preview.png"),
        );

        let request = attachment_preview_request_for(&data).expect("image URL request");

        assert_eq!(
            request,
            AssetRequest::new(AssetLocator::url("https://example.com/preview.png"))
                .with_kind_hint(AssetKindHint::Image)
        );
    }

    #[test]
    fn attachment_preview_request_skips_non_image_urls() {
        let data = AttachmentData::File(
            AttachmentFileData::new("att-video")
                .filename("demo.mp4")
                .media_type("video/mp4")
                .url("https://example.com/demo.mp4"),
        );

        assert!(attachment_preview_request_for(&data).is_none());
    }

    #[test]
    fn capability_gated_attachment_preview_request_requires_url_capability() {
        let data = AttachmentData::File(
            AttachmentFileData::new("att-image")
                .filename("preview.png")
                .media_type("image/png")
                .url("https://example.com/preview.png"),
        );

        assert!(capability_gated_attachment_preview_request_for(&data, None).is_none());
        assert!(
            capability_gated_attachment_preview_request_for(
                &data,
                Some(AssetCapabilities {
                    url: false,
                    ..Default::default()
                }),
            )
            .is_none()
        );
        assert_eq!(
            capability_gated_attachment_preview_request_for(
                &data,
                Some(AssetCapabilities {
                    url: true,
                    ..Default::default()
                }),
            ),
            Some(
                AssetRequest::new(AssetLocator::url("https://example.com/preview.png"))
                    .with_kind_hint(AssetKindHint::Image)
            )
        );
    }

    #[test]
    fn attachment_preview_source_accepts_byte_backed_url_resolvers() {
        use std::sync::Arc;

        use fret_app::App;
        use fret_assets::{
            AssetLoadError, AssetResolver, AssetRevision, ResolvedAssetBytes,
            ResolvedAssetReference,
        };

        struct ByteUrlResolver;

        impl AssetResolver for ByteUrlResolver {
            fn capabilities(&self) -> AssetCapabilities {
                AssetCapabilities {
                    url: true,
                    ..Default::default()
                }
            }

            fn resolve_bytes(
                &self,
                request: &AssetRequest,
            ) -> Result<ResolvedAssetBytes, AssetLoadError> {
                Ok(ResolvedAssetBytes::new(
                    request.locator.clone(),
                    AssetRevision(7),
                    &b"fake-image-bytes"[..],
                ))
            }

            fn resolve_reference(
                &self,
                _request: &AssetRequest,
            ) -> Result<ResolvedAssetReference, AssetLoadError> {
                Err(AssetLoadError::ExternalReferenceUnavailable {
                    kind: fret_assets::AssetLocatorKind::Url,
                })
            }
        }

        let data = AttachmentData::File(
            AttachmentFileData::new("att-image")
                .filename("preview.png")
                .media_type("image/png")
                .url("https://example.com/preview.png"),
        );

        let mut app = App::new();
        fret_runtime::set_asset_resolver(&mut app, Arc::new(ByteUrlResolver));

        assert!(resolve_attachment_preview_source_from_host(&app, &data).is_some());
    }

    #[test]
    fn attachment_preview_source_accepts_reference_only_url_resolvers() {
        use std::sync::Arc;

        use fret_app::App;
        use fret_assets::{
            AssetExternalReference, AssetLoadError, AssetResolver, AssetRevision,
            ResolvedAssetBytes, ResolvedAssetReference,
        };

        struct ReferenceOnlyUrlResolver;

        impl AssetResolver for ReferenceOnlyUrlResolver {
            fn capabilities(&self) -> AssetCapabilities {
                AssetCapabilities {
                    url: true,
                    ..Default::default()
                }
            }

            fn resolve_bytes(
                &self,
                _request: &AssetRequest,
            ) -> Result<ResolvedAssetBytes, AssetLoadError> {
                Err(AssetLoadError::ReferenceOnlyLocator {
                    kind: fret_assets::AssetLocatorKind::Url,
                })
            }

            fn resolve_reference(
                &self,
                request: &AssetRequest,
            ) -> Result<ResolvedAssetReference, AssetLoadError> {
                Ok(ResolvedAssetReference::new(
                    request.locator.clone(),
                    AssetRevision(11),
                    AssetExternalReference::url("https://example.com/preview.png"),
                ))
            }
        }

        let data = AttachmentData::File(
            AttachmentFileData::new("att-image")
                .filename("preview.png")
                .media_type("image/png")
                .url("https://example.com/preview.png"),
        );

        let mut app = App::new();
        fret_runtime::set_asset_resolver(&mut app, Arc::new(ReferenceOnlyUrlResolver));

        assert!(resolve_attachment_preview_source_from_host(&app, &data).is_some());
    }

    #[test]
    fn attachment_preview_source_accepts_first_party_url_passthrough_resolver() {
        use std::sync::Arc;

        use fret_app::App;
        use fret_assets::UrlPassthroughAssetResolver;

        let data = AttachmentData::File(
            AttachmentFileData::new("att-image")
                .filename("preview.png")
                .media_type("image/png")
                .url("https://example.com/preview.png"),
        );

        let mut app = App::new();
        fret_runtime::set_asset_resolver(&mut app, Arc::new(UrlPassthroughAssetResolver::new()));

        assert!(resolve_attachment_preview_source_from_host(&app, &data).is_some());
    }
}
