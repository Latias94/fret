//! AI Elements-aligned attachment surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/attachments.tsx`.

use std::sync::Arc;

use fret_core::{
    Color, Corners, FontId, FontWeight, ImageId, Px, SemanticsRole, TextOverflow, TextStyle,
    TextWrap,
};
use fret_icons::IconId;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, HoverRegionProps, ImageProps, InteractivityGateProps, LayoutStyle,
    Length, MarginEdge, PressableA11y, PressableProps, SemanticsDecoration, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::theme_tokens;
use fret_ui_kit::{ChromeRefinement, ColorRef, Items, LayoutRefinement, MetricRef, Radius, Space};
use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

pub type OnAttachmentActivate = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, Arc<str>) + 'static>;
pub type OnAttachmentRemove = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, Arc<str>) + 'static>;

fn alpha(color: Color, a: f32) -> Color {
    Color {
        a: a.clamp(0.0, 1.0),
        ..color
    }
}

fn resolve_muted(theme: &Theme) -> Color {
    theme
        .color_by_key("muted")
        .unwrap_or_else(|| theme.color_required("muted.background"))
}

fn resolve_muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn resolve_border(theme: &Theme) -> Color {
    theme
        .color_by_key("border")
        .unwrap_or_else(|| theme.color_required("border"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentVariant {
    Grid,
    Inline,
    List,
}

impl Default for AttachmentVariant {
    fn default() -> Self {
        Self::Grid
    }
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
    pub preview_image: Option<ImageId>,
    pub url: Option<Arc<str>>,
}

impl AttachmentFileData {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            filename: None,
            media_type: None,
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

#[derive(Debug, Clone)]
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
        flex_layout.size.width = Length::Fill;

        if self.variant == AttachmentVariant::Grid {
            flex_layout.margin.left = MarginEdge::Auto;
        }

        let mut props = fret_ui::element::FlexProps::default();
        props.layout = flex_layout;
        props.gap = gap;
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

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let data = self.data;
        let variant = self.variant;

        let label = attachment_label_for(&data);
        let hover_bg = alpha(resolve_muted(&theme), 0.5);

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

        let mut hover = HoverRegionProps::default();
        hover.layout = decl_style::layout_style(&theme, item_layout);

        let el = cx.hover_region(hover, move |cx, hovered| {
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
                // Even without an activation handler, keep the row present so nested controls
                // (e.g. the remove button) remain hittable.
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
                    // Stamp `test_id` on the hover region (outer wrapper) to avoid duplicate IDs
                    // when the pressable also contributes semantics.
                    test_id: None,
                    ..Default::default()
                };

                let bg = match variant {
                    AttachmentVariant::Grid => None,
                    AttachmentVariant::Inline | AttachmentVariant::List => {
                        hovered.then_some(hover_bg)
                    }
                };

                let chrome_layout = match variant {
                    AttachmentVariant::Grid => LayoutRefinement::default().w_full().h_full(),
                    AttachmentVariant::Inline => LayoutRefinement::default().h_full().min_w_0(),
                    AttachmentVariant::List => LayoutRefinement::default().w_full().min_w_0(),
                };
                let mut chrome =
                    decl_style::container_props(&theme, base_chrome.clone(), chrome_layout);
                chrome.background = bg.or(chrome.background);

                let preview = AttachmentPreview::new(data.clone())
                    .variant(variant)
                    .test_id_opt(preview_test_id.clone())
                    .into_element(cx);
                let info = AttachmentInfo::new(data.clone())
                    .variant(variant)
                    .show_media_type(show_media_type)
                    .test_id_opt(info_test_id.clone())
                    .into_element(cx);

                let remove = AttachmentRemove::new(data.id().clone())
                    .variant(variant)
                    .visible(hovered)
                    .test_id_opt(remove_test_id.clone())
                    .on_remove_opt(on_remove.clone())
                    .into_element(cx);

                let content = match variant {
                    AttachmentVariant::Grid => {
                        let mut overlay = ContainerProps::default();
                        overlay.layout = decl_style::layout_style(
                            &theme,
                            LayoutRefinement::default().relative().w_full().h_full(),
                        );

                        let abs_layout = decl_style::layout_style(
                            &theme,
                            LayoutRefinement::default()
                                .absolute()
                                .top_px(Px(4.0))
                                .right_px(Px(4.0)),
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
                        let row = stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().min_w_0())
                                .gap(Space::N2)
                                .items(Items::Center),
                            move |_cx| vec![preview, info, remove],
                        );
                        vec![row]
                    }
                    AttachmentVariant::List => {
                        let row = stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full().min_w_0())
                                .gap(Space::N3)
                                .items(Items::Center),
                            move |_cx| vec![preview, info, remove],
                        );
                        vec![row]
                    }
                };

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
}

#[derive(Debug, Clone)]
pub struct AttachmentPreview {
    data: AttachmentData,
    variant: AttachmentVariant,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl AttachmentPreview {
    pub fn new(data: AttachmentData) -> Self {
        Self {
            data,
            variant: AttachmentVariant::Grid,
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

    fn test_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.test_id = id;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let muted = resolve_muted(&theme);
        let fg = resolve_muted_fg(&theme);

        let category = media_category_for(&self.data);

        let size = match self.variant {
            AttachmentVariant::Inline => Px(20.0),
            AttachmentVariant::Grid => Px(96.0),
            AttachmentVariant::List => Px(48.0),
        };

        let corner = match self.variant {
            AttachmentVariant::Inline => MetricRef::radius(Radius::Sm).resolve(&theme),
            AttachmentVariant::Grid => Px(0.0),
            AttachmentVariant::List => MetricRef::radius(Radius::Md).resolve(&theme),
        };

        let bg = match self.variant {
            AttachmentVariant::Grid | AttachmentVariant::List => Some(muted),
            AttachmentVariant::Inline => Some(theme.color_required("background")),
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

        let content = match &self.data {
            AttachmentData::File(file)
                if category == AttachmentMediaCategory::Image && file.preview_image.is_some() =>
            {
                let image = file.preview_image.unwrap();
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Fill;
                vec![cx.image_props(ImageProps {
                    layout,
                    image,
                    opacity: 1.0,
                    uv: None,
                })]
            }
            _ => vec![decl_icon::icon_with(
                cx,
                media_category_icon(category),
                Some(if self.variant == AttachmentVariant::Inline {
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
    data: AttachmentData,
    variant: AttachmentVariant,
    show_media_type: bool,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl AttachmentInfo {
    pub fn new(data: AttachmentData) -> Self {
        Self {
            data,
            variant: AttachmentVariant::Grid,
            show_media_type: false,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn variant(mut self, variant: AttachmentVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn show_media_type(mut self, show: bool) -> Self {
        self.show_media_type = show;
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
        if self.variant == AttachmentVariant::Grid {
            return cx.text("");
        }

        let theme = Theme::global(&*cx.app).clone();
        let muted_fg = resolve_muted_fg(&theme);

        let label = attachment_label_for(&self.data);
        let label_el = cx.text_props(TextProps {
            layout: decl_style::layout_style(&theme, LayoutRefinement::default().min_w_0()),
            text: label,
            style: Some(TextStyle {
                font: FontId::default(),
                size: theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_SM_PX),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(
                    theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT),
                ),
                letter_spacing_em: None,
            }),
            color: Some(theme.color_required("foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
        });

        let media_type_row = self
            .show_media_type
            .then_some(())
            .and_then(|_| self.data.media_type().cloned())
            .map(|media_type| {
                cx.text_props(TextProps {
                    layout: decl_style::layout_style(&theme, LayoutRefinement::default().min_w_0()),
                    text: media_type,
                    style: Some(TextStyle {
                        font: FontId::default(),
                        size: theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_XS_PX),
                        weight: FontWeight::NORMAL,
                        slant: Default::default(),
                        line_height: Some(
                            theme.metric_required(
                                theme_tokens::metric::COMPONENT_TEXT_XS_LINE_HEIGHT,
                            ),
                        ),
                        letter_spacing_em: None,
                    }),
                    color: Some(muted_fg),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Ellipsis,
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
            vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().min_w_0())
                    .gap(Space::N0)
                    .items(Items::Start),
                move |_cx| rows,
            )]
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
    id: Arc<str>,
    on_remove: Option<OnAttachmentRemove>,
    visible: bool,
    test_id: Option<Arc<str>>,
    variant: AttachmentVariant,
}

impl std::fmt::Debug for AttachmentRemove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachmentRemove")
            .field("id", &self.id.as_ref())
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
            id: id.into(),
            on_remove: None,
            visible: true,
            test_id: None,
            variant: AttachmentVariant::Grid,
        }
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
        self.variant = variant;
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(on_remove) = self.on_remove else {
            return cx.text("");
        };

        let id = self.id.clone();
        let mut btn = Button::new("")
            .variant(ButtonVariant::Ghost)
            .size(match self.variant {
                AttachmentVariant::List => ButtonSize::Icon,
                AttachmentVariant::Grid | AttachmentVariant::Inline => ButtonSize::IconSm,
            })
            .children([decl_icon::icon(cx, IconId::new("lucide.x"))])
            .on_activate(Arc::new(move |host, action_cx, _reason| {
                on_remove(host, action_cx, id.clone());
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            }));

        if let Some(test_id) = self.test_id {
            btn = btn.test_id(test_id);
        }

        let btn = btn.into_element(cx);
        let opacity = if self.visible { 1.0 } else { 0.0 };
        let interactive = self.visible;

        cx.interactivity_gate(true, interactive, move |cx| {
            vec![cx.opacity(opacity, move |_cx| vec![btn])]
        })
    }
}

#[derive(Debug, Clone)]
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
            vec![cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: Arc::<str>::from("No attachments"),
                style: Some(TextStyle {
                    font: FontId::default(),
                    size: theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_SM_PX),
                    weight: FontWeight::NORMAL,
                    slant: Default::default(),
                    line_height: Some(
                        theme.metric_required(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT),
                    ),
                    letter_spacing_em: None,
                }),
                color: Some(fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
            })]
        } else {
            self.children
        };

        let mut el = cx.container(props, move |_cx| content);
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
