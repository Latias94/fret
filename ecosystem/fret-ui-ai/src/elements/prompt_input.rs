use std::sync::Arc;

use fret_core::{Axis, Edges, ExternalDragKind, Px};
use fret_icons::IconId;
use fret_runtime::{Effect, Model};
use fret_ui::action::{ActivateReason, OnActivate, OnExternalDrag, OnKeyDown};
use fret_ui::element::{AnyElement, ContainerProps, CrossAlign, FlexProps, MainAlign};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{LayoutRefinement, MetricRef, Space};

use fret_ui_shadcn::{
    Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
    DropdownMenuItem, DropdownMenuSide, InputGroup,
};

use crate::elements::attachments::{
    Attachment, AttachmentData, AttachmentFileData, AttachmentSourceDocumentData,
    AttachmentVariant, Attachments,
};
use crate::model::item_key_from_external_id;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptInputErrorCode {
    Accept,
    MaxFiles,
    MaxFileSize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptInputError {
    pub code: PromptInputErrorCode,
    pub message: Arc<str>,
}

impl PromptInputError {
    pub fn new(code: PromptInputErrorCode, message: impl Into<Arc<str>>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub type OnPromptInputError = Arc<
    dyn Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx, PromptInputError)
        + 'static,
>;

#[derive(Debug, Clone)]
pub struct PromptInputController {
    pub text: Model<String>,
    pub attachments: Option<Model<Vec<AttachmentData>>>,
}

#[derive(Debug, Default, Clone)]
struct PromptInputProviderState {
    controller: Option<PromptInputController>,
}

#[derive(Debug, Default, Clone)]
struct PromptInputLocalState {
    controller: Option<PromptInputController>,
}

#[derive(Clone)]
pub struct PromptInputConfig {
    pub disabled: bool,
    pub loading: bool,
    pub clear_on_send: bool,
    pub clear_attachments_on_send: bool,
    pub on_send: Option<OnActivate>,
    pub on_stop: Option<OnActivate>,
    pub on_add_attachments: Option<OnActivate>,
    pub accept: Option<Arc<str>>,
    pub multiple: bool,
    pub max_files: Option<usize>,
    pub max_file_size_bytes: Option<u64>,
    pub on_error: Option<OnPromptInputError>,
    pub test_id_root: Option<Arc<str>>,
    pub test_id_textarea: Option<Arc<str>>,
    pub test_id_send: Option<Arc<str>>,
    pub test_id_stop: Option<Arc<str>>,
    pub test_id_attachments: Option<Arc<str>>,
    pub test_id_referenced_sources: Option<Arc<str>>,
    pub test_id_add_attachments: Option<Arc<str>>,
}

#[derive(Default, Clone)]
struct PromptInputConfigState {
    config: Option<PromptInputConfig>,
}

pub fn use_prompt_input_config<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<PromptInputConfig> {
    cx.inherited_state::<PromptInputConfigState>()
        .and_then(|st| st.config.clone())
}

#[derive(Debug, Clone)]
pub struct PromptInputReferencedSourcesController {
    pub sources: Model<Vec<AttachmentSourceDocumentData>>,
}

#[derive(Debug, Default, Clone)]
struct PromptInputReferencedSourcesState {
    controller: Option<PromptInputReferencedSourcesController>,
}

pub fn use_prompt_input_referenced_sources<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<PromptInputReferencedSourcesController> {
    cx.inherited_state::<PromptInputReferencedSourcesState>()
        .and_then(|st| st.controller.clone())
}

/// Returns the nearest prompt input controller in scope.
///
/// This mirrors AI Elements `PromptInputProvider` behavior: prefer a local controller (inside a
/// `PromptInput`), falling back to a provider controller when present.
pub fn use_prompt_input_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<PromptInputController> {
    if let Some(local) = cx.inherited_state::<PromptInputLocalState>() {
        if let Some(controller) = local.controller.clone() {
            return Some(controller);
        }
    }
    cx.inherited_state::<PromptInputProviderState>()
        .and_then(|st| st.controller.clone())
}

#[derive(Debug, Clone)]
pub struct PromptInputProvider {
    text: Option<Model<String>>,
    attachments: Option<Model<Vec<AttachmentData>>>,
    initial_input: Arc<str>,
}

impl PromptInputProvider {
    pub fn new() -> Self {
        Self {
            text: None,
            attachments: None,
            initial_input: Arc::<str>::from(""),
        }
    }

    pub fn initial_input(mut self, input: impl Into<Arc<str>>) -> Self {
        self.initial_input = input.into();
        self
    }

    pub fn text_model(mut self, model: Model<String>) -> Self {
        self.text = Some(model);
        self
    }

    pub fn attachments_model(mut self, model: Model<Vec<AttachmentData>>) -> Self {
        self.attachments = Some(model);
        self
    }

    pub fn into_element_with_children<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>, PromptInputController) -> Vec<AnyElement>,
    ) -> AnyElement {
        let controlled_text = self.text.clone();
        let controlled_attachments = self.attachments.clone();
        let initial_input = self.initial_input.to_string();

        cx.container(Default::default(), move |cx| {
            let text =
                controllable_state::use_controllable_model(cx, controlled_text, || initial_input)
                    .model();
            let attachments = controllable_state::use_controllable_model(
                cx,
                controlled_attachments,
                Vec::<AttachmentData>::new,
            )
            .model();

            let controller = PromptInputController {
                text,
                attachments: Some(attachments),
            };
            cx.with_state(PromptInputProviderState::default, |st| {
                st.controller = Some(controller.clone());
            });

            children(cx, controller)
        })
    }
}

fn prompt_input_send_activate(
    text: Model<String>,
    attachments: Option<Model<Vec<AttachmentData>>>,
    clear_on_send: bool,
    clear_attachments_on_send: bool,
    on_send: Option<OnActivate>,
) -> OnActivate {
    Arc::new(move |host, action_cx, reason| {
        let text_value = host.models_mut().read(&text, Clone::clone).ok();
        let is_empty = text_value
            .as_deref()
            .map(|v| v.trim().is_empty())
            .unwrap_or(true);

        let attachments_len = attachments
            .as_ref()
            .and_then(|m| host.models_mut().read(m, |v| v.len()).ok())
            .unwrap_or(0);

        if is_empty && attachments_len == 0 {
            return;
        }

        let Some(on_send) = on_send.as_ref() else {
            return;
        };
        on_send(host, action_cx, reason);

        if clear_on_send {
            let _ = host.models_mut().update(&text, |v| v.clear());
        }
        if clear_attachments_on_send {
            if let Some(attachments) = attachments.as_ref() {
                let _ = host.models_mut().update(attachments, |v| v.clear());
            }
        }
    })
}

fn prompt_input_control_key_handler(
    text: Model<String>,
    attachments: Option<Model<Vec<AttachmentData>>>,
    disabled: bool,
    loading: bool,
    send_activate: OnActivate,
) -> OnKeyDown {
    Arc::new(move |host, action_cx, down| {
        if disabled {
            return false;
        }

        match down.key {
            fret_core::KeyCode::Enter => {
                if loading || down.repeat {
                    return false;
                }
                if down.modifiers.shift {
                    return false;
                }

                let text_value = host.models_mut().read(&text, Clone::clone).ok();
                let is_empty = text_value
                    .as_deref()
                    .map(|v| v.trim().is_empty())
                    .unwrap_or(true);
                let attachments_len = attachments
                    .as_ref()
                    .and_then(|m| host.models_mut().read(m, |v| v.len()).ok())
                    .unwrap_or(0);
                if is_empty && attachments_len == 0 {
                    return false;
                }

                send_activate(host, action_cx, ActivateReason::Keyboard);
                host.notify(action_cx);
                true
            }
            fret_core::KeyCode::Backspace => {
                let Some(attachments) = attachments.as_ref() else {
                    return false;
                };
                let attachments_len = host
                    .models_mut()
                    .read(attachments, |v| v.len())
                    .ok()
                    .unwrap_or(0);
                if attachments_len == 0 {
                    return false;
                }

                let text_value = host.models_mut().read(&text, Clone::clone).ok();
                let is_empty = text_value
                    .as_deref()
                    .map(|v| v.trim().is_empty())
                    .unwrap_or(true);
                if !is_empty {
                    return false;
                }

                let _ = host.models_mut().update(attachments, |v| {
                    let _ = v.pop();
                });
                host.notify(action_cx);
                true
            }
            _ => false,
        }
    })
}

fn prompt_input_file_extension_lower(name: &str) -> Option<String> {
    let (_, ext) = name.rsplit_once('.')?;
    let ext = ext.trim();
    if ext.is_empty() {
        return None;
    }
    Some(ext.to_ascii_lowercase())
}

fn prompt_input_accept_matches(accept: &str, file_name: &str, media_type: Option<&str>) -> bool {
    let accept = accept.trim();
    if accept.is_empty() {
        return true;
    }

    let ext_lower = prompt_input_file_extension_lower(file_name);
    let media_type = media_type.map(str::trim).filter(|s| !s.is_empty());

    // Minimal best-effort matcher:
    // - exact MIME (`image/png`)
    // - wildcard MIME (`image/*`) via `media_type` prefix, or common extension sets when MIME is absent
    // - extension patterns (`.png`)
    let patterns = accept
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    for pattern in patterns {
        if pattern.starts_with('.') {
            if let Some(ext_lower) = ext_lower.as_deref() {
                if pattern[1..].eq_ignore_ascii_case(ext_lower) {
                    return true;
                }
            }
            continue;
        }

        if let Some(media_type) = media_type {
            if pattern.ends_with("/*") {
                let prefix = &pattern[..pattern.len().saturating_sub(1)];
                if media_type.starts_with(prefix) {
                    return true;
                }
            } else if pattern.contains('/') && media_type.eq_ignore_ascii_case(pattern) {
                return true;
            }
            continue;
        }

        if pattern.eq_ignore_ascii_case("image/*") {
            if let Some(ext_lower) = ext_lower.as_deref() {
                if matches!(
                    ext_lower,
                    "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tif" | "tiff" | "svg"
                ) {
                    return true;
                }
            }
        }
    }

    false
}

fn prompt_input_emit_error(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    on_error: Option<&OnPromptInputError>,
    code: PromptInputErrorCode,
    message: &'static str,
) {
    if let Some(on_error) = on_error {
        on_error(host, action_cx, PromptInputError::new(code, message));
    }
}

fn prompt_input_handle_drop_files(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    attachments_model: &Model<Vec<AttachmentData>>,
    drop_files: &fret_core::ExternalDragFiles,
    accept: Option<&str>,
    max_files: Option<usize>,
    max_file_size_bytes: Option<u64>,
    on_error: Option<&OnPromptInputError>,
) -> bool {
    if drop_files.files.is_empty() {
        host.push_effect(Effect::ExternalDropRelease {
            token: drop_files.token,
        });
        return true;
    }

    let accepted: Vec<(usize, &fret_core::ExternalDragFile)> = drop_files
        .files
        .iter()
        .enumerate()
        .filter(|(_ix, f)| {
            accept
                .map(|accept| prompt_input_accept_matches(accept, &f.name, f.media_type.as_deref()))
                .unwrap_or(true)
        })
        .collect();

    if !drop_files.files.is_empty() && accepted.is_empty() {
        prompt_input_emit_error(
            host,
            action_cx,
            on_error,
            PromptInputErrorCode::Accept,
            "No files match the accepted types.",
        );
        host.push_effect(Effect::ExternalDropRelease {
            token: drop_files.token,
        });
        host.notify(action_cx);
        return true;
    }

    let accepted_len = accepted.len();
    let sized: Vec<(usize, &fret_core::ExternalDragFile)> = accepted
        .into_iter()
        .filter(|(_ix, f)| match max_file_size_bytes {
            Some(max) => f.size_bytes.map(|s| s <= max).unwrap_or(true),
            None => true,
        })
        .collect();

    if accepted_len > 0 && max_file_size_bytes.is_some() && sized.is_empty() {
        // If `max_file_size_bytes` was set and we filtered everything out, mirror upstream.
        prompt_input_emit_error(
            host,
            action_cx,
            on_error,
            PromptInputErrorCode::MaxFileSize,
            "All files exceed the maximum size.",
        );
        host.push_effect(Effect::ExternalDropRelease {
            token: drop_files.token,
        });
        host.notify(action_cx);
        return true;
    }

    let existing_len = host
        .models_mut()
        .read(attachments_model, |v| v.len())
        .ok()
        .unwrap_or(0);
    let capacity = max_files
        .map(|m| m.saturating_sub(existing_len))
        .unwrap_or(usize::MAX);

    if capacity == 0 && !sized.is_empty() {
        prompt_input_emit_error(
            host,
            action_cx,
            on_error,
            PromptInputErrorCode::MaxFiles,
            "Too many files. Some were not added.",
        );
        host.push_effect(Effect::ExternalDropRelease {
            token: drop_files.token,
        });
        host.notify(action_cx);
        return true;
    }

    let token_id = drop_files.token.0;
    let dropped: Vec<AttachmentData> = sized
        .iter()
        .take(capacity)
        .map(|(ix, f)| {
            let ix = *ix;
            let id = Arc::<str>::from(format!("drop-{token_id}-{ix}"));
            let mut file = AttachmentFileData::new(id).filename(Arc::<str>::from(f.name.clone()));
            if let Some(media_type) = f.media_type.as_deref() {
                file = file.media_type(Arc::<str>::from(media_type.to_owned()));
            }
            if let Some(size_bytes) = f.size_bytes {
                file = file.size_bytes(size_bytes);
            }
            AttachmentData::File(file)
        })
        .collect();

    let _ = host.models_mut().update(attachments_model, |v| {
        for item in &dropped {
            let id = item.id();
            if v.iter()
                .any(|existing| existing.id().as_ref() == id.as_ref())
            {
                continue;
            }
            v.push(item.clone());
        }
    });

    if max_files.is_some() && sized.len() > capacity {
        prompt_input_emit_error(
            host,
            action_cx,
            on_error,
            PromptInputErrorCode::MaxFiles,
            "Too many files. Some were not added.",
        );
    }

    host.push_effect(Effect::ExternalDropRelease {
        token: drop_files.token,
    });
    host.notify(action_cx);
    true
}

#[derive(Debug, Default, Clone)]
pub struct PromptInputSlots {
    pub block_start: Vec<AnyElement>,
    pub block_end: Vec<AnyElement>,
}

fn prompt_input_referenced_sources_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<Vec<AttachmentSourceDocumentData>>>,
) -> Model<Vec<AttachmentSourceDocumentData>> {
    if let Some(model) = controlled {
        cx.with_state(PromptInputReferencedSourcesState::default, |st| {
            st.controller = Some(PromptInputReferencedSourcesController {
                sources: model.clone(),
            });
        });
        return model;
    }

    #[derive(Default)]
    struct LocalState {
        model: Option<Model<Vec<AttachmentSourceDocumentData>>>,
    }

    let existing = cx.with_state(LocalState::default, |st| st.model.clone());
    let model = match existing {
        Some(m) => m,
        None => {
            let m = cx
                .app
                .models_mut()
                .insert(Vec::<AttachmentSourceDocumentData>::new());
            cx.with_state(LocalState::default, |st| st.model = Some(m.clone()));
            m
        }
    };

    cx.with_state(PromptInputReferencedSourcesState::default, |st| {
        st.controller = Some(PromptInputReferencedSourcesController {
            sources: model.clone(),
        });
    });

    model
}

#[derive(Clone)]
/// Parts-first prompt input root aligned with AI Elements `PromptInput` composition.
///
/// This root provides:
///
/// - a `PromptInputController` in scope (see `use_prompt_input_controller`)
/// - a prompt input config in scope (see `use_prompt_input_config`)
/// - textarea keyboard behaviors (Enter submit, Backspace remove-attachment, IME-safe)
/// - external file drop fallback (metadata-only attachments; bytes remain app-owned)
///
/// Use `PromptInputRoot::into_element_with_slots` to inject block-start/block-end content.
pub struct PromptInputRoot {
    model: Option<Model<String>>,
    textarea_min_height: Px,
    textarea_max_height: Option<Px>,
    disabled: bool,
    loading: bool,
    clear_on_send: bool,
    clear_attachments_on_send: bool,
    on_send: Option<OnActivate>,
    on_stop: Option<OnActivate>,
    on_add_attachments: Option<OnActivate>,
    accept: Option<Arc<str>>,
    multiple: bool,
    max_files: Option<usize>,
    max_file_size_bytes: Option<u64>,
    on_error: Option<OnPromptInputError>,
    attachments: Option<Model<Vec<AttachmentData>>>,
    referenced_sources: Option<Model<Vec<AttachmentSourceDocumentData>>>,
    test_id_root: Option<Arc<str>>,
    test_id_textarea: Option<Arc<str>>,
    test_id_send: Option<Arc<str>>,
    test_id_stop: Option<Arc<str>>,
    test_id_attachments: Option<Arc<str>>,
    test_id_referenced_sources: Option<Arc<str>>,
    test_id_add_attachments: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl PromptInputRoot {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model: Some(model),
            textarea_min_height: Px(96.0),
            textarea_max_height: None,
            disabled: false,
            loading: false,
            clear_on_send: true,
            clear_attachments_on_send: true,
            on_send: None,
            on_stop: None,
            on_add_attachments: None,
            accept: None,
            multiple: false,
            max_files: None,
            max_file_size_bytes: None,
            on_error: None,
            attachments: None,
            referenced_sources: None,
            test_id_root: None,
            test_id_textarea: None,
            test_id_send: None,
            test_id_stop: None,
            test_id_attachments: None,
            test_id_referenced_sources: None,
            test_id_add_attachments: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn new_uncontrolled() -> Self {
        Self {
            model: None,
            textarea_min_height: Px(96.0),
            textarea_max_height: None,
            disabled: false,
            loading: false,
            clear_on_send: true,
            clear_attachments_on_send: true,
            on_send: None,
            on_stop: None,
            on_add_attachments: None,
            accept: None,
            multiple: false,
            max_files: None,
            max_file_size_bytes: None,
            on_error: None,
            attachments: None,
            referenced_sources: None,
            test_id_root: None,
            test_id_textarea: None,
            test_id_send: None,
            test_id_stop: None,
            test_id_attachments: None,
            test_id_referenced_sources: None,
            test_id_add_attachments: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn textarea_min_height(mut self, min_height: Px) -> Self {
        self.textarea_min_height = min_height;
        self
    }

    pub fn textarea_max_height(mut self, max_height: Px) -> Self {
        self.textarea_max_height = Some(max_height);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn clear_on_send(mut self, clear_on_send: bool) -> Self {
        self.clear_on_send = clear_on_send;
        self
    }

    pub fn clear_attachments_on_send(mut self, clear_attachments_on_send: bool) -> Self {
        self.clear_attachments_on_send = clear_attachments_on_send;
        self
    }

    pub fn on_send(mut self, on_send: OnActivate) -> Self {
        self.on_send = Some(on_send);
        self
    }

    pub fn on_stop(mut self, on_stop: OnActivate) -> Self {
        self.on_stop = Some(on_stop);
        self
    }

    pub fn on_add_attachments(mut self, on_add_attachments: OnActivate) -> Self {
        self.on_add_attachments = Some(on_add_attachments);
        self
    }

    pub fn accept(mut self, accept: impl Into<Arc<str>>) -> Self {
        self.accept = Some(accept.into());
        self
    }

    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    pub fn max_files(mut self, max_files: usize) -> Self {
        self.max_files = Some(max_files);
        self
    }

    pub fn max_file_size_bytes(mut self, max_file_size_bytes: u64) -> Self {
        self.max_file_size_bytes = Some(max_file_size_bytes);
        self
    }

    pub fn on_error(mut self, on_error: OnPromptInputError) -> Self {
        self.on_error = Some(on_error);
        self
    }

    pub fn attachments(mut self, model: Model<Vec<AttachmentData>>) -> Self {
        self.attachments = Some(model);
        self
    }

    pub fn referenced_sources_model(
        mut self,
        model: Model<Vec<AttachmentSourceDocumentData>>,
    ) -> Self {
        self.referenced_sources = Some(model);
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn test_id_textarea(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_textarea = Some(id.into());
        self
    }

    pub fn test_id_send(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_send = Some(id.into());
        self
    }

    pub fn test_id_stop(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_stop = Some(id.into());
        self
    }

    pub fn test_id_attachments(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_attachments = Some(id.into());
        self
    }

    pub fn test_id_referenced_sources(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_referenced_sources = Some(id.into());
        self
    }

    pub fn test_id_add_attachments(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_add_attachments = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element_with_slots<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        slots: impl FnOnce(&mut ElementContext<'_, H>) -> PromptInputSlots,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let provider = use_prompt_input_controller(cx);
        let text_model = controllable_state::use_controllable_model(
            cx,
            self.model
                .clone()
                .or_else(|| provider.as_ref().map(|c| c.text.clone())),
            String::new,
        )
        .model();

        let attachments_model = self
            .attachments
            .clone()
            .or_else(|| provider.as_ref().and_then(|c| c.attachments.clone()));

        let _referenced_sources_model =
            prompt_input_referenced_sources_model(cx, self.referenced_sources.clone());

        cx.with_state(PromptInputLocalState::default, |st| {
            st.controller = Some(PromptInputController {
                text: text_model.clone(),
                attachments: attachments_model.clone(),
            });
        });

        cx.with_state(PromptInputConfigState::default, |st| {
            st.config = Some(PromptInputConfig {
                disabled: self.disabled,
                loading: self.loading,
                clear_on_send: self.clear_on_send,
                clear_attachments_on_send: self.clear_attachments_on_send,
                on_send: self.on_send.clone(),
                on_stop: self.on_stop.clone(),
                on_add_attachments: self.on_add_attachments.clone(),
                accept: self.accept.clone(),
                multiple: self.multiple,
                max_files: self.max_files,
                max_file_size_bytes: self.max_file_size_bytes,
                on_error: self.on_error.clone(),
                test_id_root: self.test_id_root.clone(),
                test_id_textarea: self.test_id_textarea.clone(),
                test_id_send: self.test_id_send.clone(),
                test_id_stop: self.test_id_stop.clone(),
                test_id_attachments: self.test_id_attachments.clone(),
                test_id_referenced_sources: self.test_id_referenced_sources.clone(),
                test_id_add_attachments: self.test_id_add_attachments.clone(),
            });
        });

        let textarea_min_height = if self.textarea_min_height == Px(96.0) {
            theme
                .metric_by_key("fret.ai.prompt_input.min_height")
                .unwrap_or(self.textarea_min_height)
        } else {
            self.textarea_min_height
        };

        let textarea_max_height = self
            .textarea_max_height
            .or_else(|| theme.metric_by_key("fret.ai.prompt_input.max_height"));

        let send_activate = prompt_input_send_activate(
            text_model.clone(),
            attachments_model.clone(),
            self.clear_on_send,
            self.clear_attachments_on_send,
            self.on_send.clone(),
        );

        let control_key_handler = prompt_input_control_key_handler(
            text_model.clone(),
            attachments_model.clone(),
            self.disabled,
            self.loading,
            send_activate,
        );

        let current = cx
            .get_model_cloned(&text_model, Invalidation::Layout)
            .unwrap_or_default();
        let is_empty = current.trim().is_empty();

        let prompt_empty_state_marker = self.test_id_root.clone().map(|root| {
            let suffix = if is_empty {
                "prompt-empty"
            } else {
                "prompt-nonempty"
            };
            let id = Arc::<str>::from(format!("{root}-{suffix}"));
            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Text,
                    test_id: Some(id),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.container(
                        fret_ui::element::ContainerProps {
                            layout: fret_ui::element::LayoutStyle {
                                size: fret_ui::element::SizeStyle {
                                    width: fret_ui::element::Length::Px(Px(0.0)),
                                    height: fret_ui::element::Length::Px(Px(0.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    )]
                },
            )
        });

        let mut slots = slots(cx);
        if let Some(marker) = prompt_empty_state_marker {
            slots.block_end.push(marker);
        }

        let mut group = InputGroup::new(text_model.clone())
            .textarea()
            .textarea_min_height(textarea_min_height)
            .control_on_key_down(control_key_handler)
            .refine_layout(self.layout.w_full());

        if let Some(max_h) = textarea_max_height {
            group = group.textarea_max_height(max_h);
        }

        if !slots.block_end.is_empty() {
            group = group.block_end(slots.block_end).block_end_border_top(true);
        }

        if !slots.block_start.is_empty() {
            group = group
                .block_start(slots.block_start)
                .block_start_border_bottom(true);
        }

        if let Some(id) = self.test_id_root {
            group = group.test_id(id);
        }
        if let Some(id) = self.test_id_textarea {
            group = group.control_test_id(id);
        }

        let content = group.into_element(cx);

        let drop_handler: Option<OnExternalDrag> =
            attachments_model
                .clone()
                .map(|attachments| -> OnExternalDrag {
                    let disabled = self.disabled;
                    let loading = self.loading;
                    let accept = self.accept.clone();
                    let max_files = self.max_files;
                    let max_file_size_bytes = self.max_file_size_bytes;
                    let on_error = self.on_error.clone();
                    Arc::new(
                        move |host: &mut dyn fret_ui::action::UiActionHost,
                              action_cx: fret_ui::action::ActionCx,
                              e: &fret_core::ExternalDragEvent| {
                            if disabled || loading {
                                return false;
                            }

                            let ExternalDragKind::DropFiles(files) = &e.kind else {
                                return false;
                            };
                            prompt_input_handle_drop_files(
                                host,
                                action_cx,
                                &attachments,
                                files,
                                accept.as_deref(),
                                max_files,
                                max_file_size_bytes,
                                on_error.as_ref(),
                            )
                        },
                    )
                });

        if let Some(on_drop) = drop_handler {
            cx.external_drag_region(fret_ui::element::ExternalDragRegionProps::default(), |cx| {
                cx.external_drag_region_on_external_drag(on_drop);
                vec![content]
            })
        } else {
            content
        }
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_with_slots(cx, |_cx| PromptInputSlots::default())
    }
}

/// Alias for upstream naming parity: in AI Elements the textarea is a child part; in Fret the
/// shadcn `InputGroup` owns the textarea control, so `PromptInputTextarea` maps to the root.
pub type PromptInputTextarea = PromptInputRoot;

#[derive(Clone)]
pub struct PromptInput {
    model: Option<Model<String>>,
    textarea_min_height: Px,
    textarea_max_height: Option<Px>,
    disabled: bool,
    loading: bool,
    clear_on_send: bool,
    clear_attachments_on_send: bool,
    on_send: Option<OnActivate>,
    on_stop: Option<OnActivate>,
    on_add_attachments: Option<OnActivate>,
    accept: Option<Arc<str>>,
    multiple: bool,
    max_files: Option<usize>,
    max_file_size_bytes: Option<u64>,
    on_error: Option<OnPromptInputError>,
    attachments: Option<Model<Vec<AttachmentData>>>,
    test_id_root: Option<Arc<str>>,
    test_id_textarea: Option<Arc<str>>,
    test_id_send: Option<Arc<str>>,
    test_id_stop: Option<Arc<str>>,
    test_id_attachments: Option<Arc<str>>,
    test_id_add_attachments: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for PromptInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PromptInput")
            .field("model", &self.model.as_ref().map(|_| "<model>"))
            .field("textarea_min_height", &self.textarea_min_height)
            .field("textarea_max_height", &self.textarea_max_height)
            .field("disabled", &self.disabled)
            .field("loading", &self.loading)
            .field("clear_on_send", &self.clear_on_send)
            .field("clear_attachments_on_send", &self.clear_attachments_on_send)
            .field("on_send", &self.on_send.as_ref().map(|_| "<on_send>"))
            .field("on_stop", &self.on_stop.as_ref().map(|_| "<on_stop>"))
            .field(
                "on_add_attachments",
                &self
                    .on_add_attachments
                    .as_ref()
                    .map(|_| "<on_add_attachments>"),
            )
            .field(
                "attachments",
                &self.attachments.as_ref().map(|_| "<attachments>"),
            )
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("test_id_textarea", &self.test_id_textarea.as_deref())
            .field("test_id_send", &self.test_id_send.as_deref())
            .field("test_id_stop", &self.test_id_stop.as_deref())
            .field("test_id_attachments", &self.test_id_attachments.as_deref())
            .field(
                "test_id_add_attachments",
                &self.test_id_add_attachments.as_deref(),
            )
            .field("layout", &self.layout)
            .finish()
    }
}

impl PromptInput {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model: Some(model),
            textarea_min_height: Px(96.0),
            textarea_max_height: None,
            disabled: false,
            loading: false,
            clear_on_send: true,
            clear_attachments_on_send: true,
            on_send: None,
            on_stop: None,
            on_add_attachments: None,
            accept: None,
            multiple: false,
            max_files: None,
            max_file_size_bytes: None,
            on_error: None,
            attachments: None,
            test_id_root: None,
            test_id_textarea: None,
            test_id_send: None,
            test_id_stop: None,
            test_id_attachments: None,
            test_id_add_attachments: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn new_uncontrolled() -> Self {
        Self {
            model: None,
            textarea_min_height: Px(96.0),
            textarea_max_height: None,
            disabled: false,
            loading: false,
            clear_on_send: true,
            clear_attachments_on_send: true,
            on_send: None,
            on_stop: None,
            on_add_attachments: None,
            accept: None,
            multiple: false,
            max_files: None,
            max_file_size_bytes: None,
            on_error: None,
            attachments: None,
            test_id_root: None,
            test_id_textarea: None,
            test_id_send: None,
            test_id_stop: None,
            test_id_attachments: None,
            test_id_add_attachments: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn textarea_min_height(mut self, min_height: Px) -> Self {
        self.textarea_min_height = min_height;
        self
    }

    pub fn textarea_max_height(mut self, max_height: Px) -> Self {
        self.textarea_max_height = Some(max_height);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn clear_on_send(mut self, clear_on_send: bool) -> Self {
        self.clear_on_send = clear_on_send;
        self
    }

    pub fn clear_attachments_on_send(mut self, clear_attachments_on_send: bool) -> Self {
        self.clear_attachments_on_send = clear_attachments_on_send;
        self
    }

    pub fn on_send(mut self, on_send: OnActivate) -> Self {
        self.on_send = Some(on_send);
        self
    }

    pub fn on_stop(mut self, on_stop: OnActivate) -> Self {
        self.on_stop = Some(on_stop);
        self
    }

    /// Add an "add attachments" affordance aligned with AI Elements `PromptInputActionAddAttachments`.
    ///
    /// Effects (file dialog, file IO, clipboard files) remain app-owned; this action hook only
    /// emits an intent.
    pub fn on_add_attachments(mut self, on_add_attachments: OnActivate) -> Self {
        self.on_add_attachments = Some(on_add_attachments);
        self
    }

    pub fn accept(mut self, accept: impl Into<Arc<str>>) -> Self {
        self.accept = Some(accept.into());
        self
    }

    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    pub fn max_files(mut self, max_files: usize) -> Self {
        self.max_files = Some(max_files);
        self
    }

    pub fn max_file_size_bytes(mut self, max_file_size_bytes: u64) -> Self {
        self.max_file_size_bytes = Some(max_file_size_bytes);
        self
    }

    pub fn on_error(mut self, on_error: OnPromptInputError) -> Self {
        self.on_error = Some(on_error);
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn test_id_textarea(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_textarea = Some(id.into());
        self
    }

    pub fn test_id_send(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_send = Some(id.into());
        self
    }

    pub fn test_id_stop(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_stop = Some(id.into());
        self
    }

    pub fn attachments(mut self, model: Model<Vec<AttachmentData>>) -> Self {
        self.attachments = Some(model);
        self
    }

    pub fn test_id_attachments(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_attachments = Some(id.into());
        self
    }

    pub fn test_id_add_attachments(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_add_attachments = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let provider = use_prompt_input_controller(cx);
        let text_model = controllable_state::use_controllable_model(
            cx,
            self.model
                .clone()
                .or_else(|| provider.as_ref().map(|c| c.text.clone())),
            String::new,
        )
        .model();

        let attachments_model = self
            .attachments
            .clone()
            .or_else(|| provider.as_ref().and_then(|c| c.attachments.clone()));

        cx.with_state(PromptInputLocalState::default, |st| {
            st.controller = Some(PromptInputController {
                text: text_model.clone(),
                attachments: attachments_model.clone(),
            });
        });

        let current = cx
            .get_model_cloned(&text_model, Invalidation::Layout)
            .unwrap_or_default();
        let is_empty = current.trim().is_empty();

        let attachments = attachments_model.as_ref().and_then(|m| {
            cx.get_model_cloned(m, Invalidation::Layout)
                .or_else(|| Some(Vec::new()))
        });
        let attachments_len = attachments.as_ref().map(|v| v.len()).unwrap_or(0);

        let text_model_for_handlers = text_model.clone();
        let clear_on_send = self.clear_on_send;
        let on_send = self.on_send.clone();
        let attachments_model_for_send = attachments_model.clone();
        let clear_attachments_on_send = self.clear_attachments_on_send;
        let send_activate: OnActivate = Arc::new(move |host, action_cx, reason| {
            let text = host
                .models_mut()
                .read(&text_model_for_handlers, Clone::clone)
                .ok();
            let is_empty = text.as_deref().map(|v| v.trim().is_empty()).unwrap_or(true);

            let attachments_len = attachments_model_for_send
                .as_ref()
                .and_then(|m| host.models_mut().read(m, |v| v.len()).ok())
                .unwrap_or(0);

            if is_empty && attachments_len == 0 {
                return;
            }

            if let Some(on_send) = on_send.as_ref() {
                on_send(host, action_cx, reason);
            }

            if clear_on_send {
                let _ = host
                    .models_mut()
                    .update(&text_model_for_handlers, |v| v.clear());
            }
            if clear_attachments_on_send {
                if let Some(attachments_model) = attachments_model_for_send.as_ref() {
                    let _ = host.models_mut().update(attachments_model, |v| v.clear());
                }
            }
        });

        let stop_activate = self.on_stop.clone();

        let send_disabled = self.disabled || self.loading || (is_empty && attachments_len == 0);
        let stop_disabled = self.disabled || !self.loading;

        let textarea_min_height = if self.textarea_min_height == Px(96.0) {
            theme
                .metric_by_key("fret.ai.prompt_input.min_height")
                .unwrap_or(self.textarea_min_height)
        } else {
            self.textarea_min_height
        };

        let textarea_max_height = self
            .textarea_max_height
            .or_else(|| theme.metric_by_key("fret.ai.prompt_input.max_height"));

        let send_activate_for_button = send_activate.clone();
        let send_button = (!self.loading).then(|| {
            let mut btn = Button::new("Send")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .disabled(send_disabled)
                .on_activate(send_activate_for_button);
            if let Some(id) = self.test_id_send.clone() {
                btn = btn.test_id(id);
            }
            btn.into_element(cx)
        });

        let stop_button = (self.loading).then(|| {
            let mut btn = Button::new("Stop")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .disabled(stop_disabled);

            if let Some(on_stop) = stop_activate {
                btn = btn.on_activate(on_stop);
            }
            if let Some(id) = self.test_id_stop.clone() {
                btn = btn.test_id(id);
            }
            btn.into_element(cx)
        });

        let add_attachments_button = self.on_add_attachments.clone().map(|on_add| {
            let add_disabled = self.disabled || self.loading;
            let mut btn = Button::new("Add attachments")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .disabled(add_disabled)
                .children([decl_icon::icon(cx, IconId::new("lucide.plus"))])
                .on_activate(on_add);

            let test_id = self.test_id_add_attachments.clone().or_else(|| {
                self.test_id_root
                    .clone()
                    .map(|id| Arc::<str>::from(format!("{id}-add-attachments")))
            });
            if let Some(id) = test_id {
                btn = btn.test_id(id);
            }

            btn.into_element(cx)
        });

        let gap = MetricRef::space(Space::N2).resolve(&theme);
        let actions = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(&theme, LayoutRefinement::default().w_full()),
                direction: Axis::Horizontal,
                gap,
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::End,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| {
                let mut out = Vec::new();
                if let Some(add_button) = add_attachments_button {
                    out.push(cx.container(
                        ContainerProps {
                            layout: decl_style::layout_style(
                                &theme,
                                LayoutRefinement::default().mr_auto(),
                            ),
                            ..Default::default()
                        },
                        move |_cx| vec![add_button],
                    ));
                }
                if let Some(stop_button) = stop_button {
                    out.push(stop_button);
                }
                if let Some(send_button) = send_button {
                    out.push(send_button);
                }
                out
            },
        );

        let attachments_el = attachments.and_then(|items| {
            if items.is_empty() {
                return None;
            }

            let attachments_model = attachments_model.clone();
            let on_remove = attachments_model.map(|attachments_model| {
                let model = attachments_model.clone();
                Arc::new(
                    move |host: &mut dyn fret_ui::action::UiActionHost,
                          _action_cx: fret_ui::action::ActionCx,
                          id: Arc<str>| {
                        let _ = host.models_mut().update(&model, |v| {
                            v.retain(|item| item.id().as_ref() != id.as_ref());
                        });
                    },
                )
            });

            let row_test_id = self.test_id_attachments.clone().or_else(|| {
                self.test_id_root
                    .clone()
                    .map(|id| Arc::<str>::from(format!("{id}-attachments")))
            });

            let mut children = Vec::new();
            for item in items {
                let item_id = item.id().clone();
                let key = item_key_from_external_id(item_id.as_ref());

                let item_test_id = row_test_id
                    .as_deref()
                    .map(|root| Arc::<str>::from(format!("{root}-item-{item_id}")));
                let remove_test_id = item_test_id
                    .as_deref()
                    .map(|root| Arc::<str>::from(format!("{root}-remove")));

                let on_remove = on_remove.clone();
                let el = cx.keyed(key, move |cx| {
                    let mut chip = Attachment::new(item.clone()).variant(AttachmentVariant::Inline);
                    if let Some(on_remove) = on_remove.clone() {
                        chip = chip.on_remove(on_remove);
                    }
                    if let Some(id) = item_test_id.clone() {
                        chip = chip.test_id(id);
                    }
                    if let Some(id) = remove_test_id.clone() {
                        chip = chip.remove_test_id(id);
                    }
                    chip.into_element(cx)
                });
                children.push(el);
            }

            let mut row = Attachments::new(children).variant(AttachmentVariant::Inline);
            if let Some(id) = row_test_id {
                row = row.test_id(id);
            }
            Some(row.into_element(cx))
        });

        let control_key_handler: OnKeyDown = {
            let text_model = text_model.clone();
            let attachments_model = attachments_model.clone();
            let disabled = self.disabled;
            let loading = self.loading;
            let send_activate = send_activate.clone();

            Arc::new(move |host, action_cx, down| {
                if disabled {
                    return false;
                }

                match down.key {
                    fret_core::KeyCode::Enter => {
                        if loading || down.repeat {
                            return false;
                        }
                        if down.modifiers.shift {
                            return false;
                        }

                        let text = host.models_mut().read(&text_model, Clone::clone).ok();
                        let is_empty = text.as_deref().map(|v| v.trim().is_empty()).unwrap_or(true);
                        let attachments_len = attachments_model
                            .as_ref()
                            .and_then(|m| host.models_mut().read(m, |v| v.len()).ok())
                            .unwrap_or(0);
                        if is_empty && attachments_len == 0 {
                            return false;
                        }

                        send_activate(host, action_cx, ActivateReason::Keyboard);
                        host.notify(action_cx);
                        true
                    }
                    fret_core::KeyCode::Backspace => {
                        let Some(attachments_model) = attachments_model.as_ref() else {
                            return false;
                        };
                        let attachments_len = host
                            .models_mut()
                            .read(attachments_model, |v| v.len())
                            .ok()
                            .unwrap_or(0);
                        if attachments_len == 0 {
                            return false;
                        }

                        let text = host.models_mut().read(&text_model, Clone::clone).ok();
                        let is_empty = text.as_deref().map(|v| v.trim().is_empty()).unwrap_or(true);
                        if !is_empty {
                            return false;
                        }

                        let _ = host.models_mut().update(attachments_model, |v| {
                            let _ = v.pop();
                        });
                        host.notify(action_cx);
                        true
                    }
                    _ => false,
                }
            })
        };

        let prompt_empty_state_marker = self.test_id_root.clone().map(|root| {
            let suffix = if is_empty {
                "prompt-empty"
            } else {
                "prompt-nonempty"
            };
            let id = Arc::<str>::from(format!("{root}-{suffix}"));
            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Text,
                    test_id: Some(id),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.container(
                        fret_ui::element::ContainerProps {
                            layout: fret_ui::element::LayoutStyle {
                                size: fret_ui::element::SizeStyle {
                                    width: fret_ui::element::Length::Px(Px(0.0)),
                                    height: fret_ui::element::Length::Px(Px(0.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    )]
                },
            )
        });

        let mut group = InputGroup::new(text_model.clone())
            .textarea()
            .textarea_min_height(textarea_min_height)
            .control_on_key_down(control_key_handler)
            .block_end({
                let mut out = vec![actions];
                if let Some(marker) = prompt_empty_state_marker {
                    out.push(marker);
                }
                out
            })
            .block_end_border_top(true)
            .refine_layout(self.layout.w_full());

        if let Some(max_h) = textarea_max_height {
            group = group.textarea_max_height(max_h);
        }

        if let Some(attachments_el) = attachments_el {
            group = group
                .block_start(vec![attachments_el])
                .block_start_border_bottom(true);
        }

        if let Some(id) = self.test_id_root {
            group = group.test_id(id);
        }
        if let Some(id) = self.test_id_textarea {
            group = group.control_test_id(id);
        }

        let content = group.into_element(cx);

        let drop_handler: Option<OnExternalDrag> =
            attachments_model
                .clone()
                .map(|attachments| -> OnExternalDrag {
                    let disabled = self.disabled;
                    let loading = self.loading;
                    let accept = self.accept.clone();
                    let max_files = self.max_files;
                    let max_file_size_bytes = self.max_file_size_bytes;
                    let on_error = self.on_error.clone();
                    Arc::new(
                        move |host: &mut dyn fret_ui::action::UiActionHost,
                              action_cx: fret_ui::action::ActionCx,
                              e: &fret_core::ExternalDragEvent| {
                            if disabled || loading {
                                return false;
                            }

                            let ExternalDragKind::DropFiles(files) = &e.kind else {
                                return false;
                            };
                            prompt_input_handle_drop_files(
                                host,
                                action_cx,
                                &attachments,
                                files,
                                accept.as_deref(),
                                max_files,
                                max_file_size_bytes,
                                on_error.as_ref(),
                            )
                        },
                    )
                });

        if let Some(on_drop) = drop_handler {
            cx.external_drag_region(fret_ui::element::ExternalDragRegionProps::default(), |cx| {
                cx.external_drag_region_on_external_drag(on_drop);
                vec![content]
            })
        } else {
            content
        }
    }
}

#[derive(Clone)]
/// Block-start header row aligned with AI Elements `PromptInputHeader`.
pub struct PromptInputHeader {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl PromptInputHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N1).resolve(&theme);

        cx.flex(
            FlexProps {
                layout: decl_style::layout_style(&theme, self.layout),
                direction: Axis::Horizontal,
                gap,
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: true,
            },
            move |_cx| self.children,
        )
    }
}

#[derive(Clone)]
/// Block-end footer row aligned with AI Elements `PromptInputFooter`.
pub struct PromptInputFooter {
    leading: Vec<AnyElement>,
    trailing: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl PromptInputFooter {
    pub fn new(
        leading: impl IntoIterator<Item = AnyElement>,
        trailing: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        Self {
            leading: leading.into_iter().collect(),
            trailing: trailing.into_iter().collect(),
            layout: LayoutRefinement::default().w_full(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N2).resolve(&theme);

        let leading = self.leading;
        let trailing = self.trailing;

        cx.flex(
            FlexProps {
                layout: decl_style::layout_style(&theme, self.layout),
                direction: Axis::Horizontal,
                gap,
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| {
                let mut out = Vec::new();
                if !leading.is_empty() {
                    out.push(cx.container(
                        ContainerProps {
                            layout: decl_style::layout_style(
                                &theme,
                                LayoutRefinement::default().mr_auto(),
                            ),
                            ..Default::default()
                        },
                        move |_cx| leading,
                    ));
                }
                out.extend(trailing);
                out
            },
        )
    }
}

#[derive(Clone)]
/// Left-aligned tools container aligned with AI Elements `PromptInputTools`.
pub struct PromptInputTools {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl PromptInputTools {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().min_w_0(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N1).resolve(&theme);

        cx.flex(
            FlexProps {
                layout: decl_style::layout_style(&theme, self.layout),
                direction: Axis::Horizontal,
                gap,
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| self.children,
        )
    }
}

#[derive(Clone)]
/// Generic prompt input button aligned with AI Elements `PromptInputButton` (ghost by default).
pub struct PromptInputButton {
    label: Arc<str>,
    children: Vec<AnyElement>,
    disabled: bool,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl PromptInputButton {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
            disabled: false,
            on_activate: None,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut btn = Button::new(self.label)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .disabled(self.disabled)
            .children(self.children)
            .refine_layout(self.layout);

        if let Some(on_activate) = self.on_activate {
            btn = btn.on_activate(on_activate);
        }
        if let Some(id) = self.test_id {
            btn = btn.test_id(id);
        }
        btn.into_element(cx)
    }
}

#[derive(Clone)]
/// Action menu trigger aligned with AI Elements `PromptInputActionMenuTrigger`.
///
/// This is a small “+” button that toggles a dropdown menu open model.
pub struct PromptInputActionMenuTrigger {
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl PromptInputActionMenuTrigger {
    pub fn new() -> Self {
        Self {
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element_with_open<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        open: Model<bool>,
    ) -> AnyElement {
        let cfg = use_prompt_input_config(cx);
        let disabled = cfg
            .as_ref()
            .map(|c| c.disabled || c.loading)
            .unwrap_or(false);

        let on_toggle: OnActivate = Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = !*v);
            host.notify(action_cx);
        });

        let mut btn = PromptInputButton::new("Prompt actions")
            .children([decl_icon::icon(cx, IconId::new("lucide.plus"))])
            .disabled(disabled)
            .on_activate(on_toggle)
            .refine_layout(self.layout);

        if let Some(id) = self.test_id {
            btn = btn.test_id(id);
        }
        btn.into_element(cx)
    }
}

#[derive(Clone)]
/// Action menu item aligned with AI Elements `PromptInputActionMenuItem`.
pub struct PromptInputActionMenuItem {
    label: Arc<str>,
    value: Option<Arc<str>>,
    leading: Option<AnyElement>,
    disabled: bool,
    close_on_select: bool,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
}

impl PromptInputActionMenuItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            value: None,
            leading: None,
            disabled: false,
            close_on_select: true,
            on_activate: None,
            test_id: None,
        }
    }

    pub fn value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn leading(mut self, leading: AnyElement) -> Self {
        self.leading = Some(leading);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_entry(self) -> DropdownMenuEntry {
        let mut item = DropdownMenuItem::new(self.label.clone());
        if let Some(value) = self.value.or_else(|| Some(self.label.clone())) {
            item.value = value;
        }
        item.leading = self.leading;
        item.disabled = self.disabled;
        item.close_on_select = self.close_on_select;
        item.on_activate = self.on_activate;
        item.test_id = self.test_id;
        DropdownMenuEntry::Item(item)
    }
}

#[derive(Clone)]
/// Action menu content wrapper aligned with AI Elements `PromptInputActionMenuContent`.
pub struct PromptInputActionMenuContent {
    entries: Vec<DropdownMenuEntry>,
}

impl PromptInputActionMenuContent {
    pub fn new(entries: impl IntoIterator<Item = DropdownMenuEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }

    pub fn into_entries(self) -> Vec<DropdownMenuEntry> {
        self.entries
    }
}

#[derive(Default)]
struct PromptInputActionMenuState {
    open: Option<Model<bool>>,
}

fn prompt_input_action_menu_open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let open = cx.with_state(PromptInputActionMenuState::default, |st| st.open.clone());
    match open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(PromptInputActionMenuState::default, |st| {
                st.open = Some(model.clone())
            });
            model
        }
    }
}

#[derive(Clone)]
/// Action menu root aligned with AI Elements `PromptInputActionMenu`.
pub struct PromptInputActionMenu {
    align: DropdownMenuAlign,
    side: DropdownMenuSide,
    side_offset: Px,
    modal: bool,
    trigger: PromptInputActionMenuTrigger,
    content: PromptInputActionMenuContent,
}

impl PromptInputActionMenu {
    pub fn new(content: PromptInputActionMenuContent) -> Self {
        Self {
            align: DropdownMenuAlign::Start,
            side: DropdownMenuSide::Top,
            side_offset: Px(4.0),
            modal: false,
            trigger: PromptInputActionMenuTrigger::new(),
            content,
        }
    }

    pub fn align(mut self, align: DropdownMenuAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side(mut self, side: DropdownMenuSide) -> Self {
        self.side = side;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    pub fn trigger(mut self, trigger: PromptInputActionMenuTrigger) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let open = prompt_input_action_menu_open_model(cx);
        let modal = self.modal;
        let align = self.align;
        let side = self.side;
        let side_offset = self.side_offset;
        let trigger = self.trigger;
        let entries = self.content.into_entries();

        DropdownMenu::new(open.clone())
            .modal(modal)
            .align(align)
            .side(side)
            .side_offset(side_offset)
            .into_element(
                cx,
                move |cx| trigger.into_element_with_open(cx, open.clone()),
                move |_cx| entries,
            )
    }
}

#[derive(Clone)]
/// Menu item aligned with AI Elements `PromptInputActionAddAttachments` (intent-driven).
///
/// Upstream reference: `prompt-input.tsx` (`DropdownMenuItem` that opens the file dialog).
pub struct PromptInputActionAddAttachments {
    label: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl PromptInputActionAddAttachments {
    pub fn new() -> Self {
        Self {
            label: Arc::<str>::from("Add photos or files"),
            test_id: None,
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = label.into();
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_entry<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> DropdownMenuEntry {
        let cfg = use_prompt_input_config(cx);
        let disabled = cfg
            .as_ref()
            .map(|c| c.disabled || c.loading || c.on_add_attachments.is_none())
            .unwrap_or(true);
        let on_activate = cfg.as_ref().and_then(|c| c.on_add_attachments.clone());

        let mut item = PromptInputActionMenuItem::new(self.label)
            .leading(decl_icon::icon(cx, IconId::new("lucide.image")))
            .disabled(disabled);

        if let Some(on_activate) = on_activate {
            item = item.on_activate(on_activate);
        }
        if let Some(id) = self.test_id {
            item = item.test_id(id);
        }
        item.into_entry()
    }
}

#[deprecated(note = "Renamed to PromptInputActionAddAttachments (AI Elements-aligned menu item).")]
pub type PromptInputActionAddAttachmentsMenuItem = PromptInputActionAddAttachments;

#[derive(Clone)]
/// Attachments chips row aligned with upstream prompt input attachment outcomes.
pub struct PromptInputAttachmentsRow {
    variant: AttachmentVariant,
}

impl PromptInputAttachmentsRow {
    pub fn new() -> Self {
        Self {
            variant: AttachmentVariant::Inline,
        }
    }

    pub fn variant(mut self, variant: AttachmentVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_prompt_input_controller(cx) else {
            return cx.text("");
        };
        let Some(attachments_model) = controller.attachments else {
            return cx.text("");
        };

        let items = cx
            .get_model_cloned(&attachments_model, Invalidation::Layout)
            .unwrap_or_default();
        if items.is_empty() {
            return cx.text("");
        }

        let cfg = use_prompt_input_config(cx);
        let row_test_id = cfg
            .as_ref()
            .and_then(|c| c.test_id_attachments.clone())
            .or_else(|| {
                cfg.as_ref().and_then(|c| {
                    c.test_id_root
                        .clone()
                        .map(|id| Arc::<str>::from(format!("{id}-attachments")))
                })
            });

        let on_remove: crate::elements::attachments::OnAttachmentRemove = {
            let model = attachments_model.clone();
            Arc::new(move |host, _action_cx, id| {
                let _ = host.models_mut().update(&model, |v| {
                    v.retain(|item| item.id().as_ref() != id.as_ref());
                });
            })
        };

        let mut children = Vec::new();
        for item in items {
            let item_id = item.id().clone();
            let key = item_key_from_external_id(item_id.as_ref());

            let item_test_id = row_test_id
                .as_deref()
                .map(|root| Arc::<str>::from(format!("{root}-item-{item_id}")));
            let remove_test_id = item_test_id
                .as_deref()
                .map(|root| Arc::<str>::from(format!("{root}-remove")));

            let on_remove = on_remove.clone();
            let variant = self.variant;
            let el = cx.keyed(key, move |cx| {
                let mut chip = Attachment::new(item.clone()).variant(variant);
                chip = chip.on_remove(on_remove);
                if let Some(id) = item_test_id.clone() {
                    chip = chip.test_id(id);
                }
                if let Some(id) = remove_test_id.clone() {
                    chip = chip.remove_test_id(id);
                }
                chip.into_element(cx)
            });
            children.push(el);
        }

        let mut row = Attachments::new(children).variant(self.variant);
        if let Some(id) = row_test_id {
            row = row.test_id(id);
        }
        row.into_element(cx)
    }
}

#[derive(Clone)]
/// Referenced sources chips row aligned with upstream prompt input referenced sources outcomes.
///
/// Upstream reference: `prompt-input.tsx` (`ReferencedSourcesContext`, local to `PromptInput`).
pub struct PromptInputReferencedSourcesRow {
    variant: AttachmentVariant,
}

impl PromptInputReferencedSourcesRow {
    pub fn new() -> Self {
        Self {
            variant: AttachmentVariant::Inline,
        }
    }

    pub fn variant(mut self, variant: AttachmentVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_prompt_input_referenced_sources(cx) else {
            return cx.text("");
        };

        let items = cx
            .get_model_cloned(&controller.sources, Invalidation::Layout)
            .unwrap_or_default();
        if items.is_empty() {
            return cx.text("");
        }

        let cfg = use_prompt_input_config(cx);
        let row_test_id = cfg
            .as_ref()
            .and_then(|c| c.test_id_referenced_sources.clone())
            .or_else(|| {
                cfg.as_ref().and_then(|c| {
                    c.test_id_root
                        .clone()
                        .map(|id| Arc::<str>::from(format!("{id}-referenced-sources")))
                })
            });

        let on_remove: crate::elements::attachments::OnAttachmentRemove = {
            let model = controller.sources.clone();
            Arc::new(move |host, _action_cx, id| {
                let _ = host.models_mut().update(&model, |v| {
                    v.retain(|item| item.id.as_ref() != id.as_ref());
                });
            })
        };

        let mut children = Vec::new();
        for item in items {
            let item_id = item.id.clone();
            let key = item_key_from_external_id(item_id.as_ref());

            let item_test_id = row_test_id
                .as_deref()
                .map(|root| Arc::<str>::from(format!("{root}-item-{item_id}")));
            let remove_test_id = item_test_id
                .as_deref()
                .map(|root| Arc::<str>::from(format!("{root}-remove")));

            let on_remove = on_remove.clone();
            let variant = self.variant;
            let el = cx.keyed(key, move |cx| {
                let data = AttachmentData::SourceDocument(item.clone());
                let mut chip = Attachment::new(data).variant(variant);
                chip = chip.on_remove(on_remove);
                if let Some(id) = item_test_id.clone() {
                    chip = chip.test_id(id);
                }
                if let Some(id) = remove_test_id.clone() {
                    chip = chip.remove_test_id(id);
                }
                chip.into_element(cx)
            });
            children.push(el);
        }

        let mut row = Attachments::new(children).variant(self.variant);
        if let Some(id) = row_test_id {
            row = row.test_id(id);
        }
        row.into_element(cx)
    }
}

#[derive(Clone)]
/// Intent-driven “add attachments” button (extra recipe; not in upstream AI Elements taxonomy).
pub struct PromptInputActionAddAttachmentsButton {
    layout: LayoutRefinement,
}

impl PromptInputActionAddAttachmentsButton {
    pub fn new() -> Self {
        Self {
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let cfg = use_prompt_input_config(cx);
        let on_add = cfg.as_ref().and_then(|c| c.on_add_attachments.clone());
        let disabled = cfg.as_ref().map(|c| c.disabled).unwrap_or(true);
        let loading = cfg.as_ref().map(|c| c.loading).unwrap_or(false);

        let test_id = cfg
            .as_ref()
            .and_then(|c| c.test_id_add_attachments.clone())
            .or_else(|| {
                cfg.as_ref().and_then(|c| {
                    c.test_id_root
                        .clone()
                        .map(|id| Arc::<str>::from(format!("{id}-add-attachments")))
                })
            });

        let mut btn = PromptInputButton::new("Add attachments")
            .children([decl_icon::icon(cx, IconId::new("lucide.plus"))])
            .disabled(disabled || loading)
            .refine_layout(self.layout);

        if let Some(on_add) = on_add {
            btn = btn.on_activate(on_add);
        }
        if let Some(id) = test_id {
            btn = btn.test_id(id);
        }
        btn.into_element(cx)
    }
}

#[derive(Clone)]
/// Send/stop button aligned with AI Elements `PromptInputSubmit` outcomes.
pub struct PromptInputSubmit {
    layout: LayoutRefinement,
}

impl PromptInputSubmit {
    pub fn new() -> Self {
        Self {
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let cfg = use_prompt_input_config(cx);
        let controller = use_prompt_input_controller(cx);

        let disabled = cfg.as_ref().map(|c| c.disabled).unwrap_or(true);
        let loading = cfg.as_ref().map(|c| c.loading).unwrap_or(false);
        let on_send = cfg.as_ref().and_then(|c| c.on_send.clone());
        let on_stop = cfg.as_ref().and_then(|c| c.on_stop.clone());

        let text_model = controller.as_ref().map(|c| c.text.clone());
        let attachments_model = controller.as_ref().and_then(|c| c.attachments.clone());

        let is_empty = text_model
            .as_ref()
            .and_then(|m| cx.get_model_cloned(m, Invalidation::Layout))
            .map(|v| v.trim().is_empty())
            .unwrap_or(true);
        let attachments_len = attachments_model
            .as_ref()
            .and_then(|m| cx.get_model_cloned(m, Invalidation::Layout))
            .map(|v| v.len())
            .unwrap_or(0);

        if loading {
            let mut btn = Button::new("Stop")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .disabled(disabled || on_stop.is_none())
                .refine_layout(self.layout);

            if let Some(on_stop) = on_stop {
                btn = btn.on_activate(on_stop);
            }
            if let Some(id) = cfg.as_ref().and_then(|c| c.test_id_stop.clone()) {
                btn = btn.test_id(id);
            }
            return btn.into_element(cx);
        }

        let send_disabled = disabled || on_send.is_none() || (is_empty && attachments_len == 0);

        let mut btn = Button::new("Send")
            .variant(ButtonVariant::Default)
            .size(ButtonSize::Sm)
            .disabled(send_disabled)
            .refine_layout(self.layout);

        if let (Some(text_model), Some(on_send)) = (text_model, on_send) {
            let send_activate = prompt_input_send_activate(
                text_model,
                attachments_model,
                cfg.as_ref().map(|c| c.clear_on_send).unwrap_or(true),
                cfg.as_ref()
                    .map(|c| c.clear_attachments_on_send)
                    .unwrap_or(true),
                Some(on_send),
            );
            btn = btn.on_activate(send_activate);
        }

        if let Some(id) = cfg.as_ref().and_then(|c| c.test_id_send.clone()) {
            btn = btn.test_id(id);
        }
        btn.into_element(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Event, ExternalDragEvent, ExternalDragFile, ExternalDragFiles,
        ExternalDragKind, ExternalDropToken, Modifiers, MouseButton, PathCommand, PathConstraints,
        PathId, PathMetrics, PathService, PathStyle, Point, PointerEvent, PointerId, PointerType,
        Px, Rect, Size, SvgId, SvgService, TextBlobId, TextConstraints, TextInput, TextMetrics,
        TextService,
    };
    use fret_ui::UiTree;
    use fret_ui::declarative::render_root;
    use std::sync::Mutex;

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
                    size: Size::new(Px(0.0), Px(0.0)),
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

    #[test]
    fn prompt_input_provider_text_model_receives_text_input() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let controlled_text = app.models_mut().insert(String::new());
        let controlled_attachments = app.models_mut().insert(Vec::<AttachmentData>::new());

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(180.0)),
        );
        let mut services = FakeServices::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "prompt-input-provider-test",
            |cx| {
                vec![
                    PromptInputProvider::new()
                        .text_model(controlled_text.clone())
                        .attachments_model(controlled_attachments.clone())
                        .into_element_with_children(cx, |cx, _controller| {
                            vec![
                                PromptInput::new_uncontrolled()
                                    .test_id_root("pi-root")
                                    .test_id_textarea("pi-textarea")
                                    .refine_layout(LayoutRefinement::default().w_full())
                                    .into_element(cx),
                            ]
                        }),
                ]
            },
        );

        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let click_pos = Point::new(Px(20.0), Px(20.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                pointer_id: PointerId(0),
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                pointer_id: PointerId(0),
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(&mut app, &mut services, &Event::TextInput("a".to_string()));

        let value = app
            .models_mut()
            .read(&controlled_text, Clone::clone)
            .unwrap();
        assert_eq!(value, "a");
    }

    #[test]
    fn prompt_input_drop_respects_max_files_and_emits_error() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let attachments = app.models_mut().insert(Vec::<AttachmentData>::new());
        let errors: Arc<Mutex<Vec<PromptInputErrorCode>>> = Arc::new(Mutex::new(Vec::new()));
        let on_error: OnPromptInputError = {
            let errors = errors.clone();
            Arc::new(move |_host, _action_cx, err| {
                errors.lock().unwrap().push(err.code);
            })
        };

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(180.0)),
        );
        let mut services = FakeServices::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "prompt-input-drop-max-files-test",
            |cx| {
                vec![
                    PromptInputRoot::new_uncontrolled()
                        .attachments(attachments.clone())
                        .max_files(1)
                        .on_error(on_error.clone())
                        .test_id_root("pi-root")
                        .test_id_textarea("pi-textarea")
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                ]
            },
        );

        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::ExternalDrag(ExternalDragEvent {
                position: Point::new(Px(10.0), Px(10.0)),
                kind: ExternalDragKind::DropFiles(ExternalDragFiles {
                    token: ExternalDropToken(123),
                    files: vec![
                        ExternalDragFile {
                            name: "a.txt".to_string(),
                            size_bytes: Some(1),
                            media_type: Some("text/plain".to_string()),
                        },
                        ExternalDragFile {
                            name: "b.txt".to_string(),
                            size_bytes: Some(1),
                            media_type: Some("text/plain".to_string()),
                        },
                    ],
                }),
            }),
        );

        let len = app
            .models_mut()
            .read(&attachments, |v| v.len())
            .unwrap_or(0);
        assert_eq!(len, 1);

        let codes = errors.lock().unwrap().clone();
        assert!(codes.contains(&PromptInputErrorCode::MaxFiles));
    }

    #[test]
    fn prompt_input_drop_accept_and_size_errors_do_not_add_attachments() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let attachments = app.models_mut().insert(Vec::<AttachmentData>::new());
        let errors: Arc<Mutex<Vec<PromptInputErrorCode>>> = Arc::new(Mutex::new(Vec::new()));
        let on_error: OnPromptInputError = {
            let errors = errors.clone();
            Arc::new(move |_host, _action_cx, err| {
                errors.lock().unwrap().push(err.code);
            })
        };

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(180.0)),
        );
        let mut services = FakeServices::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "prompt-input-drop-accept-size-test",
            |cx| {
                vec![
                    PromptInputRoot::new_uncontrolled()
                        .attachments(attachments.clone())
                        .accept(Arc::<str>::from("image/*"))
                        .max_file_size_bytes(1)
                        .on_error(on_error.clone())
                        .test_id_root("pi-root")
                        .test_id_textarea("pi-textarea")
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                ]
            },
        );

        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Accept error: media type is known and does not match `image/*`.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::ExternalDrag(ExternalDragEvent {
                position: Point::new(Px(10.0), Px(10.0)),
                kind: ExternalDragKind::DropFiles(ExternalDragFiles {
                    token: ExternalDropToken(200),
                    files: vec![ExternalDragFile {
                        name: "note.txt".to_string(),
                        size_bytes: Some(1),
                        media_type: Some("text/plain".to_string()),
                    }],
                }),
            }),
        );

        // Size error: accepted, but exceeds max file size.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::ExternalDrag(ExternalDragEvent {
                position: Point::new(Px(10.0), Px(10.0)),
                kind: ExternalDragKind::DropFiles(ExternalDragFiles {
                    token: ExternalDropToken(201),
                    files: vec![ExternalDragFile {
                        name: "image.png".to_string(),
                        size_bytes: Some(10),
                        media_type: Some("image/png".to_string()),
                    }],
                }),
            }),
        );

        let len = app
            .models_mut()
            .read(&attachments, |v| v.len())
            .unwrap_or(0);
        assert_eq!(len, 0);

        let codes = errors.lock().unwrap().clone();
        assert!(codes.contains(&PromptInputErrorCode::Accept));
        assert!(codes.contains(&PromptInputErrorCode::MaxFileSize));
    }
}
