use crate::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
use crate::field::{
    Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend,
    FieldSeparator, FieldSet, FieldTitle,
};
use crate::form::FormField;
use crate::input_group::InputGroup;
use crate::input_otp::InputOtp;
use crate::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia,
    ItemSeparator, ItemTitle,
};
use crate::pagination::{
    Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink,
    PaginationNext, PaginationPrevious,
};

impl_ui_patch_chrome_layout!(InputGroup);
impl_ui_patch_chrome_layout!(InputOtp);

impl_ui_patch_chrome_layout_patch_only!(Collapsible);
impl_ui_patch_passthrough_patch_only!(CollapsibleTrigger);
impl_ui_patch_chrome_layout!(CollapsibleContent);

impl_ui_patch_chrome_layout!(Field);
impl_ui_patch_passthrough!(FieldSet);
impl_ui_patch_passthrough!(FieldLegend);
impl_ui_patch_passthrough!(FieldLabel);
impl_ui_patch_passthrough!(FieldTitle);
impl_ui_patch_passthrough!(FieldDescription);
impl_ui_patch_passthrough!(FieldError);
impl_ui_patch_passthrough!(FieldSeparator);
impl_ui_patch_passthrough!(FieldGroup);
impl_ui_patch_passthrough!(FieldContent);

impl_ui_patch_chrome_layout!(Item);
impl_ui_patch_passthrough!(ItemGroup);
impl_ui_patch_passthrough!(ItemHeader);
impl_ui_patch_passthrough!(ItemContent);
impl_ui_patch_passthrough!(ItemTitle);
impl_ui_patch_passthrough!(ItemDescription);
impl_ui_patch_passthrough!(ItemMedia);
impl_ui_patch_passthrough!(ItemActions);
impl_ui_patch_passthrough!(ItemFooter);
impl_ui_patch_passthrough!(ItemSeparator);

impl_ui_patch_layout_only!(Pagination);
impl_ui_patch_passthrough!(PaginationContent);
impl_ui_patch_passthrough!(PaginationItem);
impl_ui_patch_passthrough!(PaginationLink);
impl_ui_patch_passthrough!(PaginationPrevious);
impl_ui_patch_passthrough!(PaginationNext);
impl_ui_patch_passthrough!(PaginationEllipsis);

impl_ui_patch_passthrough!(FormField);
