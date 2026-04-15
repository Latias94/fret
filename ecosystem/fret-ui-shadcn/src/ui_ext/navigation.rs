use crate::navigation_menu::{NavigationMenu, NavigationMenuLink};
use crate::sidebar::{
    Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupContent, SidebarGroupLabel,
    SidebarHeader, SidebarInset, SidebarMenu, SidebarMenuButton, SidebarMenuItem,
};

impl_ui_patch_layout_only!(NavigationMenu);
impl_ui_patch_passthrough!(NavigationMenuLink);
impl_ui_patch_chrome_layout!(Sidebar);
impl_ui_patch_passthrough!(SidebarHeader);
impl_ui_patch_passthrough!(SidebarFooter);
impl_ui_patch_passthrough!(SidebarContent);
impl_ui_patch_chrome_layout!(SidebarInset);
impl_ui_patch_passthrough!(SidebarGroup);
impl_ui_patch_chrome_layout!(SidebarGroupContent);
impl_ui_patch_passthrough!(SidebarGroupLabel);
impl_ui_patch_passthrough!(SidebarMenu);
impl_ui_patch_passthrough!(SidebarMenuItem);
impl_ui_patch_passthrough!(SidebarMenuButton);
