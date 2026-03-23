use std::any::Any;
use std::sync::Arc;

use fret_core::{Axis, Color, Edges, ExternalDragKind, Px};
use fret_icons::IconId;
use fret_runtime::{ActionId, Effect, Model};
use fret_ui::action::{ActivateReason, OnActivate, OnExternalDrag, OnKeyDown};
use fret_ui::element::{AnyElement, ContainerProps, CrossAlign, FlexProps, MainAlign};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, IntoUiElement, LayoutRefinement, MetricRef, Space,
    WidgetStateProperty, WidgetStates,
};

use fret_ui_shadcn::facade::{
    Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
    DropdownMenuItem, DropdownMenuSide, InputGroup, Kbd, Select as ShadcnSelect,
    SelectContent as ShadcnSelectContent, SelectItem as ShadcnSelectItem,
    SelectTrigger as ShadcnSelectTrigger, SelectTriggerSize, SelectValue as ShadcnSelectValue,
    Spinner, Tooltip, TooltipContent, TooltipSide,
};
use fret_ui_shadcn::raw::button::ButtonStyle;

type ActionPayloadFactory = Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>;

const DEFAULT_PROMPT_INPUT_PLACEHOLDER: &str = "What would you like to know?";

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

/// Upstream parity data for AI Elements `PromptInput` submission.
///
/// Reference: `repo-ref/ai-elements/packages/elements/src/prompt-input.tsx` (`PromptInputMessage`).
#[derive(Debug, Clone)]
pub struct PromptInputMessage {
    pub text: Arc<str>,
    pub files: Vec<AttachmentData>,
}

pub type OnPromptInputSubmit = Arc<
    dyn Fn(
            &mut dyn fret_ui::action::UiActionHost,
            fret_ui::action::ActionCx,
            PromptInputMessage,
            ActivateReason,
        ) + 'static,
>;

#[derive(Debug, Clone)]
pub struct PromptInputController {
    pub text: Model<String>,
    pub attachments: Option<Model<Vec<AttachmentData>>>,
}

#[derive(Debug, Clone)]
struct PromptInputProviderController {
    controller: PromptInputController,
}

#[derive(Debug, Clone)]
struct PromptInputLocalController {
    controller: PromptInputController,
}

#[derive(Clone)]
pub struct PromptInputConfig {
    pub disabled: bool,
    pub loading: bool,
    pub status: Option<PromptInputStatus>,
    pub clear_on_send: bool,
    pub clear_attachments_on_send: bool,
    pub on_submit: Option<OnPromptInputSubmit>,
    pub on_send: Option<OnActivate>,
    pub on_stop: Option<OnActivate>,
    pub on_add_attachments: Option<OnActivate>,
    pub on_add_screenshot: Option<OnActivate>,
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

/// PromptInput submission state aligned with AI Elements `ChatStatus`-driven outcomes.
///
/// Reference: `repo-ref/ai-elements/packages/elements/src/prompt-input.tsx` (`PromptInputSubmit`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PromptInputStatus {
    #[default]
    Idle,
    Submitted,
    Streaming,
    Error,
}

impl PromptInputStatus {
    fn is_generating(self) -> bool {
        matches!(
            self,
            PromptInputStatus::Submitted | PromptInputStatus::Streaming
        )
    }
}

pub fn use_prompt_input_config<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<PromptInputConfig> {
    cx.provided::<PromptInputConfig>().cloned()
}

#[derive(Debug, Clone)]
pub struct PromptInputReferencedSourcesController {
    pub sources: Model<Vec<AttachmentSourceDocumentData>>,
}

pub fn use_prompt_input_referenced_sources<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<PromptInputReferencedSourcesController> {
    cx.provided::<PromptInputReferencedSourcesController>()
        .cloned()
}

/// Returns the nearest prompt input controller in scope.
///
/// This mirrors AI Elements `PromptInputProvider` behavior: prefer a local controller (inside a
/// `PromptInput`), falling back to a provider controller when present.
pub fn use_prompt_input_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<PromptInputController> {
    cx.provided::<PromptInputLocalController>()
        .map(|local| local.controller.clone())
        .or_else(|| {
            cx.provided::<PromptInputProviderController>()
                .map(|provider| provider.controller.clone())
        })
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

    pub fn into_element_with_children<H: UiHost>(
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
            cx.provide(
                PromptInputProviderController {
                    controller: controller.clone(),
                },
                |cx| children(cx, controller),
            )
        })
    }
}

fn prompt_input_send_activate(
    text: Model<String>,
    attachments: Option<Model<Vec<AttachmentData>>>,
    clear_on_send: bool,
    clear_attachments_on_send: bool,
    on_submit: Option<OnPromptInputSubmit>,
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

        let message = PromptInputMessage {
            text: Arc::<str>::from(text_value.unwrap_or_default()),
            files: attachments
                .as_ref()
                .and_then(|m| host.models_mut().read(m, Clone::clone).ok())
                .unwrap_or_default(),
        };

        if let Some(on_submit) = on_submit.as_ref() {
            on_submit(host, action_cx, message, reason);
        } else if let Some(on_send) = on_send.as_ref() {
            on_send(host, action_cx, reason);
        } else {
            return;
        }

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

#[derive(Debug, Default)]
pub struct PromptInputSlots {
    pub block_start: Vec<AnyElement>,
    pub block_end: Vec<AnyElement>,
}

#[track_caller]
fn prompt_input_referenced_sources_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<Vec<AttachmentSourceDocumentData>>>,
) -> Model<Vec<AttachmentSourceDocumentData>> {
    if let Some(model) = controlled {
        return model;
    }
    cx.local_model(Vec::<AttachmentSourceDocumentData>::new)
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
    placeholder: Option<Arc<str>>,
    textarea_min_height: Px,
    textarea_max_height: Option<Px>,
    disabled: bool,
    loading: bool,
    status: Option<PromptInputStatus>,
    clear_on_send: bool,
    clear_attachments_on_send: bool,
    on_submit: Option<OnPromptInputSubmit>,
    on_send: Option<OnActivate>,
    on_stop: Option<OnActivate>,
    on_add_attachments: Option<OnActivate>,
    on_add_screenshot: Option<OnActivate>,
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
            placeholder: None,
            textarea_min_height: Px(96.0),
            textarea_max_height: None,
            disabled: false,
            loading: false,
            status: None,
            clear_on_send: true,
            clear_attachments_on_send: true,
            on_submit: None,
            on_send: None,
            on_stop: None,
            on_add_attachments: None,
            on_add_screenshot: None,
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
            placeholder: None,
            textarea_min_height: Px(96.0),
            textarea_max_height: None,
            disabled: false,
            loading: false,
            status: None,
            clear_on_send: true,
            clear_attachments_on_send: true,
            on_submit: None,
            on_send: None,
            on_stop: None,
            on_add_attachments: None,
            on_add_screenshot: None,
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

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
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

    /// Overrides the prompt input submission state (icon + stop behavior).
    ///
    /// Notes:
    /// - When unset, `loading=true` is treated as `Streaming` for backwards compatibility.
    /// - When set to a generating state (`Submitted`/`Streaming`), Enter-to-send is suppressed.
    pub fn status(mut self, status: PromptInputStatus) -> Self {
        self.status = Some(status);
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

    pub fn on_submit(mut self, on_submit: OnPromptInputSubmit) -> Self {
        self.on_submit = Some(on_submit);
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

    pub fn on_add_screenshot(mut self, on_add_screenshot: OnActivate) -> Self {
        self.on_add_screenshot = Some(on_add_screenshot);
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

    pub fn children<I, P>(self, parts: I) -> PromptInputChildren
    where
        I: IntoIterator<Item = P>,
        P: Into<PromptInputPart>,
    {
        PromptInputChildren::new(self, parts)
    }

    pub fn into_element_with_slots<H: UiHost>(
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

        let referenced_sources_model =
            prompt_input_referenced_sources_model(cx, self.referenced_sources.clone());
        let local_controller = PromptInputLocalController {
            controller: PromptInputController {
                text: text_model.clone(),
                attachments: attachments_model.clone(),
            },
        };
        let config = PromptInputConfig {
            disabled: self.disabled,
            loading: self.loading,
            status: self.status,
            clear_on_send: self.clear_on_send,
            clear_attachments_on_send: self.clear_attachments_on_send,
            on_submit: self.on_submit.clone(),
            on_send: self.on_send.clone(),
            on_stop: self.on_stop.clone(),
            on_add_attachments: self.on_add_attachments.clone(),
            on_add_screenshot: self.on_add_screenshot.clone(),
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
        };
        let referenced_sources_controller = PromptInputReferencedSourcesController {
            sources: referenced_sources_model.clone(),
        };

        cx.provide(local_controller, |cx| {
            cx.provide(config.clone(), |cx| {
                cx.provide(referenced_sources_controller.clone(), |cx| {
                    let status = self.status.unwrap_or(if self.loading {
                        PromptInputStatus::Streaming
                    } else {
                        PromptInputStatus::Idle
                    });
                    let generating = status.is_generating();

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
                        self.on_submit.clone(),
                        self.on_send.clone(),
                    );

                    let control_key_handler = prompt_input_control_key_handler(
                        text_model.clone(),
                        attachments_model.clone(),
                        self.disabled,
                        self.loading || generating,
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

                    let mut group =
                        InputGroup::new(text_model.clone())
                            .textarea()
                            .placeholder(self.placeholder.clone().unwrap_or_else(|| {
                                Arc::<str>::from(DEFAULT_PROMPT_INPUT_PLACEHOLDER)
                            }))
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
                        cx.external_drag_region(
                            fret_ui::element::ExternalDragRegionProps::default(),
                            |cx| {
                                cx.external_drag_region_on_external_drag(on_drop);
                                vec![content]
                            },
                        )
                    } else {
                        content
                    }
                })
            })
        })
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_with_slots(cx, |_cx| PromptInputSlots::default())
    }
}

/// Child control part aligned with AI Elements `PromptInputTextarea`.
///
/// Fret's `InputGroup` still owns the actual textarea widget; this type captures the control-side
/// authoring defaults so the root can offer a composable `children([...])` lane.
#[derive(Debug, Clone, Default)]
pub struct PromptInputTextarea {
    placeholder: Option<Arc<str>>,
    disabled: Option<bool>,
    test_id: Option<Arc<str>>,
    min_height: Option<Px>,
    max_height: Option<Px>,
}

impl PromptInputTextarea {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn min_height(mut self, min_height: Px) -> Self {
        self.min_height = Some(min_height);
        self
    }

    pub fn max_height(mut self, max_height: Px) -> Self {
        self.max_height = Some(max_height);
        self
    }

    fn apply_to_root(self, mut root: PromptInputRoot) -> PromptInputRoot {
        if let Some(placeholder) = self.placeholder {
            root = root.placeholder(placeholder);
        }
        if let Some(disabled) = self.disabled {
            root = root.disabled(disabled);
        }
        if let Some(test_id) = self.test_id {
            root = root.test_id_textarea(test_id);
        }
        if let Some(min_height) = self.min_height {
            root = root.textarea_min_height(min_height);
        }
        if let Some(max_height) = self.max_height {
            root = root.textarea_max_height(max_height);
        }
        root
    }
}

impl<H: UiHost> IntoUiElement<H> for PromptInputTextarea {
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.apply_to_root(PromptInputRoot::new_uncontrolled())
            .into_element(cx)
    }
}

/// Grouping wrapper aligned with AI Elements `PromptInputBody`.
///
/// The upstream DOM version is effectively a `contents` wrapper; in Fret we use it to keep the
/// docs-shaped children lane explicit while still lowering into the existing input-group control.
#[derive(Debug, Clone)]
pub struct PromptInputBody {
    textarea: PromptInputTextarea,
}

impl PromptInputBody {
    pub fn new(children: impl IntoIterator<Item = PromptInputTextarea>) -> Self {
        let mut children = children.into_iter();
        let textarea = children
            .next()
            .expect("PromptInputBody::new(...) requires one PromptInputTextarea");
        assert!(
            children.next().is_none(),
            "PromptInputBody::new(...) accepts at most one PromptInputTextarea"
        );
        Self { textarea }
    }

    fn into_textarea(self) -> PromptInputTextarea {
        self.textarea
    }
}

pub enum PromptInputPart {
    Header(PromptInputHeader),
    Body(PromptInputBody),
    Footer(PromptInputFooter),
}

impl std::fmt::Debug for PromptInputPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Header(_) => f.write_str("PromptInputPart::Header(<deferred>)"),
            Self::Body(body) => f.debug_tuple("PromptInputPart::Body").field(body).finish(),
            Self::Footer(_) => f.write_str("PromptInputPart::Footer(<deferred>)"),
        }
    }
}

impl From<PromptInputHeader> for PromptInputPart {
    fn from(value: PromptInputHeader) -> Self {
        Self::Header(value)
    }
}

impl From<PromptInputBody> for PromptInputPart {
    fn from(value: PromptInputBody) -> Self {
        Self::Body(value)
    }
}

impl From<PromptInputFooter> for PromptInputPart {
    fn from(value: PromptInputFooter) -> Self {
        Self::Footer(value)
    }
}

/// Root-level children adapter for docs-shaped prompt-input composition.
///
/// This stays in the component/policy layer and only lowers authored parts into the existing
/// `PromptInputRoot` slot surface.
pub struct PromptInputChildren {
    root: PromptInputRoot,
    parts: Vec<PromptInputPart>,
}

impl std::fmt::Debug for PromptInputChildren {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut header = 0usize;
        let mut body = 0usize;
        let mut footer = 0usize;

        for part in &self.parts {
            match part {
                PromptInputPart::Header(_) => header += 1,
                PromptInputPart::Body(_) => body += 1,
                PromptInputPart::Footer(_) => footer += 1,
            }
        }

        f.debug_struct("PromptInputChildren")
            .field("header_parts", &header)
            .field("body_parts", &body)
            .field("footer_parts", &footer)
            .finish()
    }
}

impl PromptInputChildren {
    pub fn new<I, P>(root: PromptInputRoot, parts: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PromptInputPart>,
    {
        Self {
            root,
            parts: parts.into_iter().map(Into::into).collect(),
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut header: Option<PromptInputHeader> = None;
        let mut body: Option<PromptInputBody> = None;
        let mut footer: Option<PromptInputFooter> = None;

        for part in self.parts {
            match part {
                PromptInputPart::Header(next) => {
                    assert!(
                        header.replace(next).is_none(),
                        "PromptInputRoot::children(...) accepts at most one PromptInputHeader"
                    );
                }
                PromptInputPart::Body(next) => {
                    assert!(
                        body.replace(next).is_none(),
                        "PromptInputRoot::children(...) accepts at most one PromptInputBody"
                    );
                }
                PromptInputPart::Footer(next) => {
                    assert!(
                        footer.replace(next).is_none(),
                        "PromptInputRoot::children(...) accepts at most one PromptInputFooter"
                    );
                }
            }
        }

        let body = body
            .expect("PromptInputRoot::children(...) requires one PromptInputBody-compatible part");
        let root = body.into_textarea().apply_to_root(self.root);

        root.into_element_with_slots(cx, move |cx| {
            let mut slots = PromptInputSlots::default();
            if let Some(header) = header {
                slots.block_start.push(header.into_element(cx));
            }
            if let Some(footer) = footer {
                slots.block_end.push(footer.into_element(cx));
            }
            slots
        })
    }
}

impl<H: UiHost> IntoUiElement<H> for PromptInputChildren {
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        PromptInputChildren::into_element(self, cx)
    }
}

#[derive(Clone)]
pub struct PromptInput {
    model: Option<Model<String>>,
    placeholder: Option<Arc<str>>,
    textarea_min_height: Px,
    textarea_max_height: Option<Px>,
    disabled: bool,
    loading: bool,
    status: Option<PromptInputStatus>,
    clear_on_send: bool,
    clear_attachments_on_send: bool,
    on_submit: Option<OnPromptInputSubmit>,
    on_send: Option<OnActivate>,
    on_stop: Option<OnActivate>,
    on_add_attachments: Option<OnActivate>,
    on_add_screenshot: Option<OnActivate>,
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

impl std::fmt::Debug for PromptInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PromptInput")
            .field("model", &self.model.as_ref().map(|_| "<model>"))
            .field("placeholder", &self.placeholder.as_deref())
            .field("textarea_min_height", &self.textarea_min_height)
            .field("textarea_max_height", &self.textarea_max_height)
            .field("disabled", &self.disabled)
            .field("loading", &self.loading)
            .field("status", &self.status)
            .field("clear_on_send", &self.clear_on_send)
            .field("clear_attachments_on_send", &self.clear_attachments_on_send)
            .field("on_submit", &self.on_submit.as_ref().map(|_| "<on_submit>"))
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
                "on_add_screenshot",
                &self
                    .on_add_screenshot
                    .as_ref()
                    .map(|_| "<on_add_screenshot>"),
            )
            .field(
                "attachments",
                &self.attachments.as_ref().map(|_| "<attachments>"),
            )
            .field(
                "referenced_sources",
                &self
                    .referenced_sources
                    .as_ref()
                    .map(|_| "<referenced_sources>"),
            )
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("test_id_textarea", &self.test_id_textarea.as_deref())
            .field("test_id_send", &self.test_id_send.as_deref())
            .field("test_id_stop", &self.test_id_stop.as_deref())
            .field("test_id_attachments", &self.test_id_attachments.as_deref())
            .field(
                "test_id_referenced_sources",
                &self.test_id_referenced_sources.as_deref(),
            )
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
            placeholder: None,
            textarea_min_height: Px(96.0),
            textarea_max_height: None,
            disabled: false,
            loading: false,
            status: None,
            clear_on_send: true,
            clear_attachments_on_send: true,
            on_submit: None,
            on_send: None,
            on_stop: None,
            on_add_attachments: None,
            on_add_screenshot: None,
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
            placeholder: None,
            textarea_min_height: Px(96.0),
            textarea_max_height: None,
            disabled: false,
            loading: false,
            status: None,
            clear_on_send: true,
            clear_attachments_on_send: true,
            on_submit: None,
            on_send: None,
            on_stop: None,
            on_add_attachments: None,
            on_add_screenshot: None,
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

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
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

    pub fn status(mut self, status: PromptInputStatus) -> Self {
        self.status = Some(status);
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

    /// Prefer this when app code needs upstream-like `onSubmit(message)` data.
    ///
    /// When both `on_submit(...)` and `on_send(...)` are provided, `on_submit(...)` wins and the
    /// legacy `on_send(...)` compatibility hook is skipped for that activation.
    pub fn on_submit(mut self, on_submit: OnPromptInputSubmit) -> Self {
        self.on_submit = Some(on_submit);
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

    /// Add a screenshot-capture affordance aligned with AI Elements `PromptInputActionAddScreenshot`.
    ///
    /// Screen-capture acquisition remains app-owned so the component layer does not grow
    /// platform-specific capture policy.
    pub fn on_add_screenshot(mut self, on_add_screenshot: OnActivate) -> Self {
        self.on_add_screenshot = Some(on_add_screenshot);
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

    pub fn attachments(mut self, model: Model<Vec<AttachmentData>>) -> Self {
        self.attachments = Some(model);
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

    fn into_root(self) -> PromptInputRoot {
        let mut root = match self.model {
            Some(model) => PromptInputRoot::new(model),
            None => PromptInputRoot::new_uncontrolled(),
        }
        .textarea_min_height(self.textarea_min_height)
        .disabled(self.disabled)
        .loading(self.loading)
        .clear_on_send(self.clear_on_send)
        .clear_attachments_on_send(self.clear_attachments_on_send)
        .multiple(self.multiple)
        .refine_layout(self.layout);

        if let Some(placeholder) = self.placeholder {
            root = root.placeholder(placeholder);
        }
        if let Some(max_h) = self.textarea_max_height {
            root = root.textarea_max_height(max_h);
        }
        if let Some(status) = self.status {
            root = root.status(status);
        }
        if let Some(on_submit) = self.on_submit {
            root = root.on_submit(on_submit);
        }
        if let Some(on_send) = self.on_send {
            root = root.on_send(on_send);
        }
        if let Some(on_stop) = self.on_stop {
            root = root.on_stop(on_stop);
        }
        if let Some(on_add_attachments) = self.on_add_attachments {
            root = root.on_add_attachments(on_add_attachments);
        }
        if let Some(on_add_screenshot) = self.on_add_screenshot {
            root = root.on_add_screenshot(on_add_screenshot);
        }
        if let Some(accept) = self.accept {
            root = root.accept(accept);
        }
        if let Some(max_files) = self.max_files {
            root = root.max_files(max_files);
        }
        if let Some(max_file_size_bytes) = self.max_file_size_bytes {
            root = root.max_file_size_bytes(max_file_size_bytes);
        }
        if let Some(on_error) = self.on_error {
            root = root.on_error(on_error);
        }
        if let Some(attachments) = self.attachments {
            root = root.attachments(attachments);
        }
        if let Some(referenced_sources) = self.referenced_sources {
            root = root.referenced_sources_model(referenced_sources);
        }
        if let Some(id) = self.test_id_root {
            root = root.test_id_root(id);
        }
        if let Some(id) = self.test_id_textarea {
            root = root.test_id_textarea(id);
        }
        if let Some(id) = self.test_id_send {
            root = root.test_id_send(id);
        }
        if let Some(id) = self.test_id_stop {
            root = root.test_id_stop(id);
        }
        if let Some(id) = self.test_id_attachments {
            root = root.test_id_attachments(id);
        }
        if let Some(id) = self.test_id_referenced_sources {
            root = root.test_id_referenced_sources(id);
        }
        if let Some(id) = self.test_id_add_attachments {
            root = root.test_id_add_attachments(id);
        }

        root
    }

    pub fn children<I, P>(self, parts: I) -> PromptInputChildren
    where
        I: IntoIterator<Item = P>,
        P: Into<PromptInputPart>,
    {
        self.into_root().children(parts)
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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

        let local_controller = PromptInputLocalController {
            controller: PromptInputController {
                text: text_model.clone(),
                attachments: attachments_model.clone(),
            },
        };

        cx.provide(local_controller, |cx| {
            let current = cx
                .get_model_cloned(&text_model, Invalidation::Layout)
                .unwrap_or_default();
            let is_empty = current.trim().is_empty();

            let attachments = attachments_model.as_ref().and_then(|m| {
                cx.get_model_cloned(m, Invalidation::Layout)
                    .or_else(|| Some(Vec::new()))
            });
            let attachments_len = attachments.as_ref().map(|v| v.len()).unwrap_or(0);

            let send_activate = prompt_input_send_activate(
                text_model.clone(),
                attachments_model.clone(),
                self.clear_on_send,
                self.clear_attachments_on_send,
                self.on_submit.clone(),
                self.on_send.clone(),
            );

            let stop_activate = self.on_stop.clone();
            let status = self.status.unwrap_or(if self.loading {
                PromptInputStatus::Streaming
            } else {
                PromptInputStatus::Idle
            });
            let generating = status.is_generating();

            let send_disabled = self.disabled
                || generating
                || (self.on_submit.is_none() && self.on_send.is_none())
                || (is_empty && attachments_len == 0);
            let stop_disabled = self.disabled || !generating;

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
            let send_button = (!generating).then(|| {
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

            let stop_button = generating.then(|| {
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
                let mut btn = Button::new("")
                    .a11y_label("Add attachments")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .disabled(add_disabled)
                    .icon(IconId::new("lucide.plus"))
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
                    gap: gap.into(),
                    padding: Edges::all(Px(0.0)).into(),
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
                        let mut chip =
                            Attachment::new(item.clone()).variant(AttachmentVariant::Inline);
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
                            let is_empty =
                                text.as_deref().map(|v| v.trim().is_empty()).unwrap_or(true);
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
                            let is_empty =
                                text.as_deref().map(|v| v.trim().is_empty()).unwrap_or(true);
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
                .placeholder(
                    self.placeholder
                        .clone()
                        .unwrap_or_else(|| Arc::<str>::from(DEFAULT_PROMPT_INPUT_PLACEHOLDER)),
                )
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
                cx.external_drag_region(
                    fret_ui::element::ExternalDragRegionProps::default(),
                    |cx| {
                        cx.external_drag_region_on_external_drag(on_drop);
                        vec![content]
                    },
                )
            } else {
                content
            }
        })
    }
}

pub type PromptInputSelect = ShadcnSelect;
pub type PromptInputSelectContent = ShadcnSelectContent;
pub type PromptInputSelectItem = ShadcnSelectItem;
pub type PromptInputSelectValue = ShadcnSelectValue;

/// Prompt-input-scoped Select trigger aligned with AI Elements toolbar chrome.
#[derive(Debug, Clone)]
pub struct PromptInputSelectTrigger {
    inner: ShadcnSelectTrigger,
}

impl Default for PromptInputSelectTrigger {
    fn default() -> Self {
        let transparent = ColorRef::Color(Color::TRANSPARENT);
        let chrome = ChromeRefinement::default()
            .shadow_none()
            .bg(transparent.clone())
            .border_color(transparent);

        Self {
            inner: ShadcnSelectTrigger::new()
                .size(SelectTriggerSize::Sm)
                .refine_style(chrome),
        }
    }
}

impl PromptInputSelectTrigger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn size(mut self, size: SelectTriggerSize) -> Self {
        self.inner = self.inner.size(size);
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.inner = self.inner.refine_style(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.inner = self.inner.refine_layout(layout);
        self
    }

    pub fn into_inner(self) -> ShadcnSelectTrigger {
        self.inner
    }
}

impl From<PromptInputSelectTrigger> for ShadcnSelectTrigger {
    fn from(value: PromptInputSelectTrigger) -> Self {
        value.into_inner()
    }
}

/// Children accepted by [`PromptInputHeader`].
///
/// This enum exists so docs-shaped prompt-input composition can include prompt-scoped parts (like
/// attachments chips) without forcing the caller to eagerly materialize `AnyElement` outside of the
/// prompt-input config/controller scope.
pub enum PromptInputHeaderChild {
    Element(AnyElement),
    AttachmentsRow(PromptInputAttachmentsRow),
    ReferencedSourcesRow(PromptInputReferencedSourcesRow),
}

impl From<AnyElement> for PromptInputHeaderChild {
    fn from(value: AnyElement) -> Self {
        Self::Element(value)
    }
}

impl From<PromptInputAttachmentsRow> for PromptInputHeaderChild {
    fn from(value: PromptInputAttachmentsRow) -> Self {
        Self::AttachmentsRow(value)
    }
}

impl From<PromptInputReferencedSourcesRow> for PromptInputHeaderChild {
    fn from(value: PromptInputReferencedSourcesRow) -> Self {
        Self::ReferencedSourcesRow(value)
    }
}

impl PromptInputHeaderChild {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        match self {
            Self::Element(el) => el,
            Self::AttachmentsRow(row) => row.into_element(cx),
            Self::ReferencedSourcesRow(row) => row.into_element(cx),
        }
    }
}

/// Block-start header row aligned with AI Elements `PromptInputHeader`.
pub struct PromptInputHeader {
    children: Vec<PromptInputHeaderChild>,
    layout: LayoutRefinement,
}

impl PromptInputHeader {
    pub fn new<I, T>(children: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<PromptInputHeaderChild>,
    {
        Self {
            children: children.into_iter().map(Into::into).collect(),
            layout: LayoutRefinement::default().w_full(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N1).resolve(&theme);
        let children = self.children;

        cx.flex(
            FlexProps {
                layout: decl_style::layout_style(&theme, self.layout),
                direction: Axis::Horizontal,
                gap: gap.into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: true,
            },
            move |cx| {
                children
                    .into_iter()
                    .map(|child| child.into_element(cx))
                    .collect::<Vec<_>>()
            },
        )
    }
}

/// Children accepted by [`PromptInputFooter`].
///
/// Similar to [`PromptInputHeaderChild`], this keeps scope-sensitive prompt parts (like the submit
/// affordance) inside the prompt-input scope so test ids and activation callbacks resolve
/// correctly.
pub enum PromptInputFooterChild {
    Element(AnyElement),
    Submit(PromptInputSubmit),
    Tools(PromptInputTools),
}

impl From<AnyElement> for PromptInputFooterChild {
    fn from(value: AnyElement) -> Self {
        Self::Element(value)
    }
}

impl From<PromptInputSubmit> for PromptInputFooterChild {
    fn from(value: PromptInputSubmit) -> Self {
        Self::Submit(value)
    }
}

impl From<PromptInputTools> for PromptInputFooterChild {
    fn from(value: PromptInputTools) -> Self {
        Self::Tools(value)
    }
}

impl PromptInputFooterChild {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        match self {
            Self::Element(el) => el,
            Self::Submit(submit) => submit.into_element(cx),
            Self::Tools(tools) => tools.into_element(cx),
        }
    }
}

/// Block-end footer row aligned with AI Elements `PromptInputFooter`.
pub struct PromptInputFooter {
    leading: Vec<PromptInputFooterChild>,
    trailing: Vec<PromptInputFooterChild>,
    layout: LayoutRefinement,
}

impl PromptInputFooter {
    pub fn new<L, LT, T, TT>(leading: L, trailing: T) -> Self
    where
        L: IntoIterator<Item = LT>,
        LT: Into<PromptInputFooterChild>,
        T: IntoIterator<Item = TT>,
        TT: Into<PromptInputFooterChild>,
    {
        Self {
            leading: leading.into_iter().map(Into::into).collect(),
            trailing: trailing.into_iter().map(Into::into).collect(),
            layout: LayoutRefinement::default().w_full(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N2).resolve(&theme);

        let leading = self.leading;
        let trailing = self.trailing;

        cx.flex(
            FlexProps {
                layout: decl_style::layout_style(&theme, self.layout),
                direction: Axis::Horizontal,
                gap: gap.into(),
                padding: Edges::all(Px(0.0)).into(),
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
                        move |cx| {
                            leading
                                .into_iter()
                                .map(|child| child.into_element(cx))
                                .collect::<Vec<_>>()
                        },
                    ));
                }
                out.extend(trailing.into_iter().map(|child| child.into_element(cx)));
                out
            },
        )
    }
}

/// Children accepted by [`PromptInputTools`].
///
/// This keeps prompt-scoped composite affordances like [`PromptInputActionMenu`] on a deferred lane
/// so they can resolve prompt callbacks before dropdown content is portalled.
pub enum PromptInputToolsChild {
    Element(AnyElement),
    ActionMenu(PromptInputActionMenu),
}

impl From<AnyElement> for PromptInputToolsChild {
    fn from(value: AnyElement) -> Self {
        Self::Element(value)
    }
}

impl From<PromptInputActionMenu> for PromptInputToolsChild {
    fn from(value: PromptInputActionMenu) -> Self {
        Self::ActionMenu(value)
    }
}

impl PromptInputToolsChild {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        match self {
            Self::Element(el) => el,
            Self::ActionMenu(menu) => menu.into_element(cx),
        }
    }
}

/// Left-aligned tools container aligned with AI Elements `PromptInputTools`.
pub struct PromptInputTools {
    children: Vec<PromptInputToolsChild>,
    layout: LayoutRefinement,
}

impl PromptInputTools {
    pub fn empty() -> Self {
        Self {
            children: Vec::new(),
            layout: LayoutRefinement::default().min_w_0(),
        }
    }

    pub fn new<I, T>(children: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<PromptInputToolsChild>,
    {
        Self::empty().children(children)
    }

    pub fn child<T>(mut self, child: T) -> Self
    where
        T: Into<PromptInputToolsChild>,
    {
        self.children.push(child.into());
        self
    }

    pub fn children<I, T>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<PromptInputToolsChild>,
    {
        self.children.extend(children.into_iter().map(Into::into));
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N1).resolve(&theme);

        cx.flex(
            FlexProps {
                layout: decl_style::layout_style(&theme, self.layout),
                direction: Axis::Horizontal,
                gap: gap.into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| {
                self.children
                    .into_iter()
                    .map(|child| child.into_element(cx))
                    .collect::<Vec<_>>()
            },
        )
    }
}

/// Generic prompt input button aligned with AI Elements `PromptInputButton` (ghost by default).
pub struct PromptInputButton {
    label: Arc<str>,
    icon: Option<IconId>,
    tooltip: Option<PromptInputButtonTooltip>,
    children: Vec<AnyElement>,
    disabled: bool,
    action: Option<ActionId>,
    action_payload: Option<ActionPayloadFactory>,
    on_activate: Option<OnActivate>,
    variant: ButtonVariant,
    expanded: bool,
    size: Option<ButtonSize>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl PromptInputButton {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            tooltip: None,
            children: Vec::new(),
            disabled: false,
            action: None,
            action_payload: None,
            on_activate: None,
            variant: ButtonVariant::Ghost,
            expanded: false,
            size: None,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Adds a tooltip aligned with AI Elements `PromptInputButton` `tooltip` prop.
    ///
    /// This is intended for compact toolbar actions (e.g. "Search the web", "New chat"), where a
    /// short hint + optional shortcut improves discoverability without adding chrome.
    pub fn tooltip(mut self, tooltip: PromptInputButtonTooltip) -> Self {
        self.tooltip = Some(tooltip);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Bind a stable action ID to this prompt-input button (action-first authoring).
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Attach a payload for parameterized prompt-input button actions (ADR 0312).
    pub fn action_payload<T>(mut self, payload: T) -> Self
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        let payload = Arc::new(payload);
        self.action_payload = Some(Arc::new(move || Box::new(payload.as_ref().clone())));
        self
    }

    /// Like [`PromptInputButton::action_payload`], but computes the payload lazily.
    pub fn action_payload_factory(mut self, payload: ActionPayloadFactory) -> Self {
        self.action_payload = Some(payload);
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = Some(size);
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
        let icon_only = self.icon.is_some() && self.children.is_empty();
        let size = self.size.unwrap_or_else(|| {
            if icon_only {
                ButtonSize::IconSm
            } else if self.children.len() > 1 {
                ButtonSize::Sm
            } else {
                ButtonSize::IconSm
            }
        });

        let expanded = self.expanded;
        let mut style = ButtonStyle::default();
        if self.variant == ButtonVariant::Ghost {
            // Upstream AI Elements `PromptInputButton`:
            // - base: `text-muted-foreground`, `bg-transparent`, `border-none`, `shadow-none`
            // - hover: `bg-accent`, `text-foreground`
            // - expanded: `bg-accent`, `text-foreground`
            //
            // Reference: `repo-ref/ai-elements/packages/elements/src/prompt-input.tsx`
            let transparent = ColorRef::Color(Color::TRANSPARENT);
            let bg_base = if expanded {
                ColorRef::Token {
                    key: "accent",
                    fallback: ColorFallback::ThemeHoverBackground,
                }
            } else {
                transparent.clone()
            };
            let fg_base = if expanded {
                ColorRef::Token {
                    key: "foreground",
                    fallback: ColorFallback::ThemeTextPrimary,
                }
            } else {
                ColorRef::Token {
                    key: "muted-foreground",
                    fallback: ColorFallback::ThemeTextMuted,
                }
            };

            style = style
                .background(
                    WidgetStateProperty::new(Some(bg_base))
                        .when(
                            WidgetStates::HOVERED,
                            Some(ColorRef::Token {
                                key: "accent",
                                fallback: ColorFallback::ThemeHoverBackground,
                            }),
                        )
                        .when(
                            WidgetStates::ACTIVE,
                            Some(ColorRef::Token {
                                key: "accent",
                                fallback: ColorFallback::ThemeHoverBackground,
                            }),
                        ),
                )
                .foreground(
                    WidgetStateProperty::new(Some(fg_base))
                        .when(
                            WidgetStates::HOVERED,
                            Some(ColorRef::Token {
                                key: "foreground",
                                fallback: ColorFallback::ThemeTextPrimary,
                            }),
                        )
                        .when(
                            WidgetStates::ACTIVE,
                            Some(ColorRef::Token {
                                key: "foreground",
                                fallback: ColorFallback::ThemeTextPrimary,
                            }),
                        ),
                )
                .border_color(WidgetStateProperty::new(Some(transparent)));
        }

        let mut btn = if icon_only {
            Button::new("")
                .a11y_label(self.label)
                .variant(self.variant)
                .style(style)
                .size(size)
                .disabled(self.disabled)
                .icon(self.icon.expect("icon_only implies icon is Some"))
                .refine_layout(self.layout)
        } else {
            Button::new(self.label)
                .variant(self.variant)
                .style(style)
                .size(size)
                .disabled(self.disabled)
                .children(self.children)
                .refine_layout(self.layout)
        };

        if let Some(action) = self.action {
            btn = btn.action(action);
        }
        if let Some(payload) = self.action_payload {
            btn = btn.action_payload_factory(payload);
        }
        if let Some(on_activate) = self.on_activate {
            btn = btn.on_activate(on_activate);
        }
        let button_test_id = self.test_id.clone();
        if let Some(id) = self.test_id {
            btn = btn.test_id(id);
        }

        let trigger = btn.into_element(cx);
        let Some(tooltip) = self.tooltip else {
            return trigger;
        };

        let content = TooltipContent::build(cx, |cx| {
            let text = TooltipContent::text(tooltip.content.clone()).into_element(cx);
            if let Some(shortcut) = tooltip.shortcut.clone() {
                let kbd = Kbd::new(shortcut).into_element(cx);
                vec![
                    ui::h_flex(|_cx| vec![text, kbd])
                        .gap(Space::N2)
                        .items_center()
                        .into_element(cx),
                ]
            } else {
                vec![text]
            }
        });

        let mut tooltip_el = Tooltip::new(cx, trigger, content);
        if let Some(side) = tooltip.side {
            tooltip_el = tooltip_el.side(side);
        }
        if let Some(panel_test_id) = tooltip
            .panel_test_id
            .or_else(|| button_test_id.map(|id| Arc::<str>::from(format!("{id}-tooltip-panel"))))
        {
            tooltip_el = tooltip_el.panel_test_id(panel_test_id);
        }
        tooltip_el.into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct PromptInputButtonTooltip {
    content: Arc<str>,
    shortcut: Option<Arc<str>>,
    side: Option<TooltipSide>,
    panel_test_id: Option<Arc<str>>,
}

impl PromptInputButtonTooltip {
    pub fn new(content: impl Into<Arc<str>>) -> Self {
        Self {
            content: content.into(),
            shortcut: None,
            side: None,
            panel_test_id: None,
        }
    }

    pub fn shortcut(mut self, shortcut: impl Into<Arc<str>>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn side(mut self, side: TooltipSide) -> Self {
        self.side = Some(side);
        self
    }

    pub fn panel_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.panel_test_id = Some(id.into());
        self
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

    pub fn into_element_with_open<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        open: Model<bool>,
    ) -> AnyElement {
        let cfg = use_prompt_input_config(cx);
        let disabled = cfg
            .as_ref()
            .map(|c| c.disabled || c.loading)
            .unwrap_or(false);
        let expanded = cx
            .get_model_cloned(&open, Invalidation::Paint)
            .unwrap_or(false);

        let mut btn = PromptInputButton::new("Prompt actions")
            .icon(IconId::new("lucide.plus"))
            .disabled(disabled)
            .expanded(expanded)
            .refine_layout(self.layout);

        if let Some(id) = self.test_id {
            btn = btn.test_id(id);
        }
        btn.into_element(cx)
    }
}

/// Action menu item aligned with AI Elements `PromptInputActionMenuItem`.
pub struct PromptInputActionMenuItem {
    label: Arc<str>,
    value: Option<Arc<str>>,
    leading_icon: Option<IconId>,
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
            leading_icon: None,
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

    pub fn leading_icon(mut self, icon: IconId) -> Self {
        self.leading_icon = Some(icon);
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
        if let Some(icon) = self.leading_icon {
            item = item.leading_icon(icon);
        }
        if let Some(value) = self.value.or_else(|| Some(self.label.clone())) {
            item.value = value;
        }
        item.disabled = self.disabled;
        item.close_on_select = self.close_on_select;
        item.on_activate = self.on_activate;
        item.test_id = self.test_id;
        DropdownMenuEntry::Item(item)
    }
}

/// Action menu content wrapper aligned with AI Elements `PromptInputActionMenuContent`.
enum PromptInputActionMenuContentEntry {
    Entry(DropdownMenuEntry),
    Item(PromptInputActionMenuItem),
    AddAttachments(PromptInputActionAddAttachments),
    AddScreenshot(PromptInputActionAddScreenshot),
}

impl PromptInputActionMenuContentEntry {
    fn into_entry<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> DropdownMenuEntry {
        match self {
            Self::Entry(entry) => entry,
            Self::Item(item) => item.into_entry(),
            Self::AddAttachments(item) => item.into_entry(cx),
            Self::AddScreenshot(item) => item.into_entry(cx),
        }
    }
}

pub struct PromptInputActionMenuContent {
    entries: Vec<PromptInputActionMenuContentEntry>,
}

impl PromptInputActionMenuContent {
    pub fn new(entries: impl IntoIterator<Item = DropdownMenuEntry>) -> Self {
        Self {
            entries: entries
                .into_iter()
                .map(PromptInputActionMenuContentEntry::Entry)
                .collect(),
        }
    }

    /// Add a generic prompt-input action-menu item on the docs-shaped builder lane.
    pub fn item(mut self, item: PromptInputActionMenuItem) -> Self {
        self.entries
            .push(PromptInputActionMenuContentEntry::Item(item));
        self
    }

    /// Add the upstream-like attachments action without eagerly resolving prompt scope.
    pub fn add_attachments(mut self, item: PromptInputActionAddAttachments) -> Self {
        self.entries
            .push(PromptInputActionMenuContentEntry::AddAttachments(item));
        self
    }

    /// Add the upstream-like screenshot action without eagerly resolving prompt scope.
    pub fn add_screenshot(mut self, item: PromptInputActionAddScreenshot) -> Self {
        self.entries
            .push(PromptInputActionMenuContentEntry::AddScreenshot(item));
        self
    }

    pub fn into_entries<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> Vec<DropdownMenuEntry> {
        self.entries
            .into_iter()
            .map(|entry| entry.into_entry(cx))
            .collect()
    }
}

#[track_caller]
fn prompt_input_action_menu_open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    cx.local_model(|| false)
}

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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let open = prompt_input_action_menu_open_model(cx);
        let modal = self.modal;
        let align = self.align;
        let side = self.side;
        let side_offset = self.side_offset;
        let trigger = self.trigger;
        let entries = self.content.into_entries(cx);

        DropdownMenu::from_open(open.clone())
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
    on_activate: Option<OnActivate>,
}

impl PromptInputActionAddAttachments {
    pub fn new() -> Self {
        Self {
            label: Arc::<str>::from("Add photos or files"),
            test_id: None,
            on_activate: None,
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

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn into_entry<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> DropdownMenuEntry {
        let cfg = use_prompt_input_config(cx);
        let on_activate = self
            .on_activate
            .clone()
            .or_else(|| cfg.as_ref().and_then(|c| c.on_add_attachments.clone()));
        let disabled = cfg
            .as_ref()
            .map(|c| c.disabled || c.loading)
            .unwrap_or(false)
            || on_activate.is_none();

        let mut item = PromptInputActionMenuItem::new(self.label)
            .leading_icon(IconId::new("lucide.image"))
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

#[derive(Clone)]
/// Menu item aligned with AI Elements `PromptInputActionAddScreenshot` (intent-driven).
///
/// The actual capture mechanism remains app-owned; this surface only exposes the upstream menu
/// taxonomy and routes activation to the app callback.
pub struct PromptInputActionAddScreenshot {
    label: Arc<str>,
    test_id: Option<Arc<str>>,
    on_activate: Option<OnActivate>,
}

impl PromptInputActionAddScreenshot {
    pub fn new() -> Self {
        Self {
            label: Arc::<str>::from("Take screenshot"),
            test_id: None,
            on_activate: None,
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

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn into_entry<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> DropdownMenuEntry {
        let cfg = use_prompt_input_config(cx);
        let on_activate = self
            .on_activate
            .clone()
            .or_else(|| cfg.as_ref().and_then(|c| c.on_add_screenshot.clone()));
        let disabled = cfg
            .as_ref()
            .map(|c| c.disabled || c.loading)
            .unwrap_or(false)
            || on_activate.is_none();

        let mut item = PromptInputActionMenuItem::new(self.label)
            .leading_icon(IconId::new("lucide.monitor"))
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

#[derive(Clone)]
/// Attachments chips row aligned with upstream prompt input attachment outcomes.
pub struct PromptInputAttachmentsRow {
    variant: AttachmentVariant,
    attachments: Option<Model<Vec<AttachmentData>>>,
    test_id_root: Option<Arc<str>>,
}

impl PromptInputAttachmentsRow {
    pub fn new() -> Self {
        Self {
            variant: AttachmentVariant::Inline,
            attachments: None,
            test_id_root: None,
        }
    }

    pub fn variant(mut self, variant: AttachmentVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Override the attachments model instead of resolving it from the nearest prompt input controller.
    pub fn attachments_model(mut self, model: Model<Vec<AttachmentData>>) -> Self {
        self.attachments = Some(model);
        self
    }

    /// Provide a stable `test_id_root` when the prompt input config is not in scope.
    ///
    /// When a config is in scope, its `test_id_*` settings take precedence.
    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let attachments_model = self
            .attachments
            .or_else(|| use_prompt_input_controller(cx).and_then(|c| c.attachments));
        let Some(attachments_model) = attachments_model else {
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
        let row_test_id = row_test_id.or_else(|| {
            self.test_id_root
                .map(|id| Arc::<str>::from(format!("{id}-attachments")))
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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
            .icon(IconId::new("lucide.plus"))
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let cfg = use_prompt_input_config(cx);
        let controller = use_prompt_input_controller(cx);

        let disabled = cfg.as_ref().map(|c| c.disabled).unwrap_or(true);
        let loading = cfg.as_ref().map(|c| c.loading).unwrap_or(false);
        let status = cfg.as_ref().and_then(|c| c.status).unwrap_or(if loading {
            PromptInputStatus::Streaming
        } else {
            PromptInputStatus::Idle
        });
        let on_submit = cfg.as_ref().and_then(|c| c.on_submit.clone());
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

        let (label, icon, activate, test_id) = if status.is_generating() {
            let icon = match status {
                PromptInputStatus::Submitted => Spinner::new().into_element(cx),
                PromptInputStatus::Streaming => decl_icon::icon(cx, IconId::new("lucide.square")),
                _ => Spinner::new().into_element(cx),
            };
            (
                Arc::<str>::from("Stop"),
                icon,
                on_stop,
                cfg.as_ref().and_then(|c| c.test_id_stop.clone()),
            )
        } else {
            let send_disabled = disabled
                || (on_submit.is_none() && on_send.is_none())
                || (is_empty && attachments_len == 0);
            let icon = match status {
                PromptInputStatus::Error => decl_icon::icon(cx, IconId::new("lucide.x")),
                _ => decl_icon::icon(cx, IconId::new("lucide.corner-down-left")),
            };
            let activate = if send_disabled {
                None
            } else {
                match text_model {
                    Some(text_model) => Some(prompt_input_send_activate(
                        text_model,
                        attachments_model,
                        cfg.as_ref().map(|c| c.clear_on_send).unwrap_or(true),
                        cfg.as_ref()
                            .map(|c| c.clear_attachments_on_send)
                            .unwrap_or(true),
                        on_submit,
                        on_send,
                    )),
                    None => None,
                }
            };
            (
                Arc::<str>::from("Submit"),
                icon,
                activate,
                cfg.as_ref().and_then(|c| c.test_id_send.clone()),
            )
        };

        let mut btn = Button::new(label)
            .variant(ButtonVariant::Default)
            .size(ButtonSize::IconSm)
            .disabled(disabled || activate.is_none())
            .children([icon])
            .refine_layout(self.layout);

        if let Some(activate) = activate {
            btn = btn.on_activate(activate);
        }
        if let Some(id) = test_id {
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
        ExternalDragKind, ExternalDropToken, KeyCode, MaterialDescriptor, MaterialId,
        MaterialRegistrationError, MaterialService, Modifiers, MouseButton, PathCommand,
        PathConstraints, PathId, PathMetrics, PathService, PathStyle, Point, PointerEvent,
        PointerId, PointerType, Px, Rect, Size, SvgId, SvgService, TextBlobId, TextConstraints,
        TextInput, TextMetrics, TextService,
    };
    use fret_ui::UiTree;
    use fret_ui::declarative::render_root;
    use std::sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    fn has_test_id(element: &AnyElement, test_id: &str) -> bool {
        if element
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
            == Some(test_id)
        {
            return true;
        }

        if matches!(
            &element.kind,
            fret_ui::element::ElementKind::Semantics(props) if props.test_id.as_deref() == Some(test_id)
        ) {
            return true;
        }

        element
            .children
            .iter()
            .any(|child| has_test_id(child, test_id))
    }

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

    impl MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: MaterialDescriptor,
        ) -> Result<MaterialId, MaterialRegistrationError> {
            Err(MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: MaterialId) -> bool {
            false
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
    fn prompt_input_children_lane_accepts_scoped_builtin_parts() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let attachments = app.models_mut().insert(vec![AttachmentData::File(
            AttachmentFileData::new("att-1")
                .filename("design.png")
                .media_type("image/png"),
        )]);
        let on_submit: OnPromptInputSubmit = Arc::new(|_host, _action_cx, _message, _reason| {});
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(180.0)),
        );

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds, "prompt-input", |cx| {
                PromptInput::new_uncontrolled()
                    .attachments(attachments.clone())
                    .on_submit(on_submit.clone())
                    .test_id_root("pi-root")
                    .test_id_send("pi-send")
                    .children([
                        PromptInputPart::from(PromptInputHeader::new([
                            PromptInputAttachmentsRow::new(),
                        ])),
                        PromptInputPart::from(PromptInputBody::new([
                            PromptInputTextarea::new().test_id("pi-textarea")
                        ])),
                        PromptInputPart::from(PromptInputFooter::new(
                            std::iter::empty::<AnyElement>(),
                            [PromptInputSubmit::new()],
                        )),
                    ])
                    .into_element(cx)
            });

        assert!(has_test_id(&element, "pi-root-attachments-item-att-1"));
    }

    #[test]
    fn prompt_input_tools_child_builder_accepts_deferred_action_menu() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let attachments = app.models_mut().insert(Vec::<AttachmentData>::new());
        let on_submit: OnPromptInputSubmit = Arc::new(|_host, _action_cx, _message, _reason| {});
        let on_add_attachments: OnActivate = Arc::new(|_host, _action_cx, _reason| {});
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(180.0)),
        );

        let _element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds, "prompt-input", |cx| {
                let menu = PromptInputActionMenu::new(
                    PromptInputActionMenuContent::new([]).add_attachments(
                        PromptInputActionAddAttachments::new().test_id("pi-add-attachments-item"),
                    ),
                )
                .trigger(PromptInputActionMenuTrigger::new().test_id("pi-action-menu-trigger"));
                let search = PromptInputButton::new("Search").test_id("pi-search-btn");
                let tools = PromptInputTools::empty()
                    .child(menu)
                    .child(search.into_element(cx));
                assert_eq!(tools.children.len(), 2);
                assert!(matches!(
                    tools.children[0],
                    PromptInputToolsChild::ActionMenu(_)
                ));
                assert!(matches!(
                    tools.children[1],
                    PromptInputToolsChild::Element(_)
                ));

                PromptInput::new_uncontrolled()
                    .attachments(attachments.clone())
                    .on_submit(on_submit.clone())
                    .on_add_attachments(on_add_attachments.clone())
                    .test_id_root("pi-root")
                    .children([
                        PromptInputPart::from(PromptInputBody::new([
                            PromptInputTextarea::new().test_id("pi-textarea")
                        ])),
                        PromptInputPart::from(PromptInputFooter::new(
                            [tools],
                            [PromptInputSubmit::new()],
                        )),
                    ])
                    .into_element(cx)
            });
    }

    #[test]
    fn prompt_input_on_submit_receives_message_and_supersedes_legacy_on_send() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let text = app.models_mut().insert(String::new());
        let attachments = app.models_mut().insert(Vec::<AttachmentData>::new());
        let submitted: Arc<Mutex<Vec<PromptInputMessage>>> = Arc::new(Mutex::new(Vec::new()));
        let legacy_send_count = Arc::new(AtomicUsize::new(0));

        let on_submit: OnPromptInputSubmit = {
            let submitted = submitted.clone();
            Arc::new(move |_host, _action_cx, message, _reason| {
                submitted.lock().unwrap().push(message);
            })
        };
        let on_send: OnActivate = {
            let legacy_send_count = legacy_send_count.clone();
            Arc::new(move |_host, _action_cx, _reason| {
                legacy_send_count.fetch_add(1, Ordering::SeqCst);
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
            "prompt-input-submit-message-test",
            |cx| {
                vec![
                    PromptInputRoot::new(text.clone())
                        .attachments(attachments.clone())
                        .on_submit(on_submit.clone())
                        .on_send(on_send.clone())
                        .test_id_root("pi-root")
                        .test_id_textarea("pi-textarea")
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
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

        let _ = app.models_mut().update(&text, |value| {
            *value = String::from("Hello Fret");
        });
        let _ = app.models_mut().update(&attachments, |items| {
            items.push(AttachmentData::File(
                AttachmentFileData::new("att-1")
                    .filename("design.png")
                    .media_type("image/png"),
            ));
        });

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let submissions = submitted.lock().unwrap().clone();
        assert_eq!(submissions.len(), 1);
        assert_eq!(submissions[0].text.as_ref(), "Hello Fret");
        assert_eq!(submissions[0].files.len(), 1);
        assert_eq!(
            crate::elements::attachments::get_attachment_label(&submissions[0].files[0]).as_ref(),
            "design.png"
        );
        assert_eq!(legacy_send_count.load(Ordering::SeqCst), 0);

        let cleared_text = app.models_mut().read(&text, Clone::clone).unwrap();
        assert!(cleared_text.is_empty());
        let cleared_attachments_len = app
            .models_mut()
            .read(&attachments, |items| items.len())
            .unwrap_or(usize::MAX);
        assert_eq!(cleared_attachments_len, 0);
    }

    #[test]
    fn prompt_input_action_menu_content_deferred_builder_resolves_prompt_callbacks() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(180.0)),
        );

        let mut app = App::new();
        let on_add_attachments: OnActivate = Arc::new(|_host, _action_cx, _reason| {});
        let entries = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "prompt-input-action-menu-content-test",
            |cx| {
                let config = PromptInputConfig {
                    disabled: false,
                    loading: false,
                    status: None,
                    clear_on_send: true,
                    clear_attachments_on_send: true,
                    on_submit: None,
                    on_send: None,
                    on_stop: None,
                    on_add_attachments: Some(on_add_attachments.clone()),
                    on_add_screenshot: None,
                    accept: None,
                    multiple: false,
                    max_files: None,
                    max_file_size_bytes: None,
                    on_error: None,
                    test_id_root: None,
                    test_id_textarea: None,
                    test_id_send: None,
                    test_id_stop: None,
                    test_id_attachments: None,
                    test_id_referenced_sources: None,
                    test_id_add_attachments: None,
                };
                cx.provide(config, |cx| {
                    PromptInputActionMenuContent::new([])
                        .add_attachments(
                            PromptInputActionAddAttachments::new()
                                .test_id("pi-add-attachments-item"),
                        )
                        .into_entries(cx)
                })
            },
        );

        assert_eq!(entries.len(), 1);
        match &entries[0] {
            DropdownMenuEntry::Item(item) => {
                assert!(!item.disabled);
                assert!(item.on_activate.is_some());
                assert_eq!(item.test_id.as_deref(), Some("pi-add-attachments-item"));
            }
            other => panic!("expected dropdown item entry, got {other:?}"),
        }
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
