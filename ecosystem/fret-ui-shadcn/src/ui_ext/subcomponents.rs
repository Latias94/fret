use crate::alert_dialog::{
    AlertDialogAction, AlertDialogCancel, AlertDialogDescription, AlertDialogFooter,
    AlertDialogHeader, AlertDialogTitle,
};
use crate::card::{CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
use crate::dialog::{DialogDescription, DialogFooter, DialogHeader, DialogTitle};
use crate::drawer::{DrawerFooter, DrawerHeader};
use crate::empty::{EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle};
use crate::popover::{PopoverDescription, PopoverHeader, PopoverTitle};
use crate::sheet::{SheetDescription, SheetFooter, SheetHeader, SheetTitle};
use crate::table::{TableBody, TableCaption, TableFooter, TableHead, TableHeader, TableRow};

impl_ui_patch_passthrough!(AlertDialogHeader);
impl_ui_patch_passthrough!(AlertDialogFooter);
impl_ui_patch_passthrough!(AlertDialogTitle);
impl_ui_patch_passthrough!(AlertDialogDescription);
impl_ui_patch_passthrough!(AlertDialogAction);
impl_ui_patch_passthrough!(AlertDialogCancel);

impl_ui_patch_passthrough!(DialogHeader);
impl_ui_patch_passthrough!(DialogFooter);
impl_ui_patch_passthrough!(DialogTitle);
impl_ui_patch_passthrough!(DialogDescription);

impl_ui_patch_passthrough!(SheetHeader);
impl_ui_patch_passthrough!(SheetFooter);
impl_ui_patch_passthrough!(SheetTitle);
impl_ui_patch_passthrough!(SheetDescription);

impl_ui_patch_passthrough!(DrawerHeader);
impl_ui_patch_passthrough!(DrawerFooter);

impl_ui_patch_passthrough!(PopoverHeader);
impl_ui_patch_passthrough!(PopoverTitle);
impl_ui_patch_passthrough!(PopoverDescription);

impl_ui_patch_passthrough!(CardHeader);
impl_ui_patch_passthrough!(CardAction);
impl_ui_patch_passthrough!(CardContent);
impl_ui_patch_passthrough!(CardFooter);
impl_ui_patch_passthrough!(CardTitle);
impl_ui_patch_passthrough!(CardDescription);

impl_ui_patch_layout_only!(EmptyHeader);
impl_ui_patch_chrome_layout!(EmptyMedia);
impl_ui_patch_passthrough!(EmptyTitle);
impl_ui_patch_passthrough!(EmptyDescription);
impl_ui_patch_layout_only!(EmptyContent);

impl_ui_patch_passthrough!(TableHeader);
impl_ui_patch_passthrough!(TableBody);
impl_ui_patch_passthrough!(TableFooter);
impl_ui_patch_passthrough!(TableRow);
impl_ui_patch_passthrough!(TableHead);
impl_ui_patch_passthrough!(TableCaption);
