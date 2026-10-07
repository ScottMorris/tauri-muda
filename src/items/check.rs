// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{cell::RefCell, mem, rc::Rc, sync::Arc};

use crate::{
    accelerator::{Accelerator, KeyAccelerator, MenuAccelerator},
    icon::{Icon, NativeIcon},
    platform_impl::PlatformMenuItem,
    util, CheckMenuItemBuilder, IconType, IsMenuItem, MenuId, MenuItemAction, MenuItemKind,
    StateCell, TextStyle,
};

/// A check menu item inside a [`Menu`] or [`Submenu`]
/// and usually contains a text and a check mark or a similar toggle
/// that corresponds to a checked and unchecked states.
///
/// [`Menu`]: crate::Menu
/// [`Submenu`]: crate::Submenu
#[derive(Clone)]
pub struct CheckMenuItem {
    pub(crate) id: Arc<MenuId>,
    pub(crate) state: StateCell<CheckMenuItemState>,
    pub(crate) platform: Rc<RefCell<PlatformMenuItem>>,
}

/// Shared state of a [`CheckMenuItem`].
#[derive(Debug, Clone)]
pub(crate) struct CheckMenuItemState {
    pub text: String,
    pub enabled: bool,
    pub checked: bool,
    pub accelerator: Option<MenuAccelerator>,
    pub styled_text: Option<Vec<(String, TextStyle)>>,
    pub icon: Option<IconType>,
    pub icon_size: Option<u32>,
}

impl crate::sealed::Sealed for CheckMenuItem {}
impl IsMenuItem for CheckMenuItem {
    fn kind(&self) -> MenuItemKind {
        MenuItemKind::Check(self.clone())
    }

    fn id(&self) -> &MenuId {
        self.id()
    }

    fn into_id(self) -> MenuId {
        self.into_id()
    }
}

impl CheckMenuItem {
    /// Returns a new [`CheckMenuItemBuilder`].
    pub fn builder() -> CheckMenuItemBuilder {
        CheckMenuItemBuilder::new()
    }

    /// Create a new check menu item.
    ///
    /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
    ///   for this check menu item. To display a `&` without assigning a mnemenonic, use `&&`.
    pub fn new<S: AsRef<str>>(
        text: S,
        enabled: bool,
        checked: bool,
        accelerator: Option<Accelerator>,
    ) -> Self {
        Self::new_inner(
            None,
            text.as_ref(),
            enabled,
            checked,
            accelerator.map(MenuAccelerator::Physical),
        )
    }

    /// Create a new check menu item with the specified id.
    ///
    /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
    ///   for this check menu item. To display a `&` without assigning a mnemenonic, use `&&`.
    pub fn with_id<I: Into<MenuId>, S: AsRef<str>>(
        id: I,
        text: S,
        enabled: bool,
        checked: bool,
        accelerator: Option<Accelerator>,
    ) -> Self {
        Self::new_inner(
            Some(id.into()),
            text.as_ref(),
            enabled,
            checked,
            accelerator.map(MenuAccelerator::Physical),
        )
    }

    fn new_inner(
        id: Option<MenuId>,
        text: &str,
        enabled: bool,
        checked: bool,
        accelerator: Option<MenuAccelerator>,
    ) -> Self {
        let id = util::next_id(id);
        let state = StateCell::new(CheckMenuItemState {
            text: text.to_string(),
            enabled,
            checked,
            accelerator,
            styled_text: None,
            icon: None,
            icon_size: None,
        });
        // The click path flips `checked` through this handle rather than through the wrapper,
        // which it has no way to reach. It is weak so that state does not own the platform that
        // owns it back (O4).
        let click = MenuItemAction::Toggle(id.clone(), state.downgrade());
        let platform = PlatformMenuItem::new(click);

        Self {
            id: Arc::new(id),
            state,
            platform: Rc::new(RefCell::new(platform)),
        }
    }

    /// Returns a unique identifier associated with this submenu.
    pub fn id(&self) -> &MenuId {
        &self.id
    }

    /// Get the text for this check menu item.
    pub fn text(&self) -> String {
        let text = self.platform.borrow().text();
        text.unwrap_or_else(|| self.state.borrow().text.clone())
    }

    /// Set the text for this check menu item. `text` could optionally contain
    /// an `&` before a character to assign this character as the mnemonic
    /// for this check menu item. To display a `&` without assigning a mnemenonic, use `&&`.
    pub fn set_text<S: AsRef<str>>(&self, text: S) {
        let accelerator = {
            let mut state = self.state.borrow_mut();
            state.text = text.as_ref().to_string();
            state.styled_text = None;
            state.accelerator.clone()
        };

        self.platform
            .borrow_mut()
            .set_text(text.as_ref(), accelerator.as_ref())
    }

    /// Set the item's label as styled parts. On Windows and Linux the parts render as plain text.
    pub fn set_styled_text<S: AsRef<str>>(&self, parts: impl IntoIterator<Item = (S, TextStyle)>) {
        let parts = parts
            .into_iter()
            .map(|(text, style)| (text.as_ref().to_string(), style))
            .collect::<Vec<_>>();
        let (text, accelerator) = {
            let mut state = self.state.borrow_mut();
            state.text = parts.iter().map(|(text, _)| text.as_str()).collect();
            state.styled_text = Some(parts.clone());
            (state.text.clone(), state.accelerator.clone())
        };
        self.platform
            .borrow_mut()
            .set_styled_text(&text, &parts, accelerator.as_ref())
    }

    /// Get whether this check menu item is enabled or not.
    pub fn is_enabled(&self) -> bool {
        let enabled = self.platform.borrow().is_enabled();
        enabled.unwrap_or_else(|| self.state.borrow().enabled)
    }

    /// Enable or disable this check menu item.
    pub fn set_enabled(&self, enabled: bool) {
        self.state.borrow_mut().enabled = enabled;
        self.platform.borrow_mut().set_enabled(enabled)
    }

    /// Set this check menu item accelerator.
    ///
    /// (Note that setting an accelerator will override any existing [.set_key_accelerator()](Self::set_key_accelerator))
    pub fn set_accelerator(&self, accelerator: Option<Accelerator>) -> crate::Result<()> {
        self.set_accelerator_inner(accelerator.map(MenuAccelerator::Physical))
    }

    /// Set this check menu item accelerator using a [`KeyAccelerator`].
    ///
    /// (Note that setting a key_accelerator will override any existing [.set_accelerator()](Self::set_accelerator))
    pub fn set_key_accelerator(&self, accelerator: Option<KeyAccelerator>) -> crate::Result<()> {
        self.set_accelerator_inner(accelerator.map(MenuAccelerator::Logical))
    }

    fn set_accelerator_inner(&self, accelerator: Option<MenuAccelerator>) -> crate::Result<()> {
        let text = {
            let mut state = self.state.borrow_mut();
            state.accelerator = accelerator.clone();
            state.text.clone()
        };

        self.platform
            .borrow_mut()
            .set_accelerator(&text, accelerator.as_ref())
    }

    /// Get whether this check menu item is checked or not.
    pub fn is_checked(&self) -> bool {
        let checked = self.platform.borrow().is_checked();
        checked.unwrap_or_else(|| self.state.borrow().checked)
    }

    /// Check or Uncheck this check menu item.
    pub fn set_checked(&self, checked: bool) {
        self.state.borrow_mut().checked = checked;
        self.platform.borrow_mut().set_checked(checked)
    }

    /// Change this check menu item's icon or remove it.
    ///
    /// ## Platform-specific
    ///
    /// - **Windows**: The icon is drawn in the item's bitmap slot, which is also where the system
    ///   draws the checked state: a checked item with a bitmap is shown with its bitmap framed
    ///   rather than with a separate check mark. Whether an item is checked is not affected.
    /// - **GTK 3**: The icon is drawn between the check indicator and the text.
    /// - **macOS**: The icon is drawn next to the text, after the check mark.
    /// - **GTK 4**: Unsupported.
    ///
    /// (Note that setting an icon will override any existing [.set_native_icon()](Self::set_native_icon))
    pub fn set_icon(&self, icon: Option<Icon>) {
        let icon = {
            let mut state = self.state.borrow_mut();
            state.icon = icon.map(IconType::Custom);
            state.icon.clone()
        };
        self.platform.borrow_mut().set_icon(icon.as_ref())
    }

    /// Change this check menu item's icon to a native image or remove it.
    ///
    /// See [`CheckMenuItem::set_icon`] for how an icon is drawn next to the checked state.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS**: Known variants map to AppKit image names. Use [`NativeIcon::Raw`] or
    ///   `NativeIcon::from_name` to pass an AppKit [`NSImage.Name`] string.
    /// - **Windows**: Known variants map to stock shell icons where an equivalent exists. Use
    ///   [`NativeIcon::Raw`] or `NativeIcon::from_id` to pass a raw [`SHSTOCKICONID`] value.
    /// - **GTK 3**: Known variants map to freedesktop-style icon theme names. Use
    ///   [`NativeIcon::Raw`] or `NativeIcon::from_name` to pass an icon theme name resolved by
    ///   [`GtkIconTheme`].
    /// - **GTK 4**: Unsupported.
    ///
    /// [`NSImage.Name`]: https://developer.apple.com/documentation/appkit/nsimage/name-swift.typealias
    /// [`SHSTOCKICONID`]: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ne-shellapi-shstockiconid
    /// [`GtkIconTheme`]: https://docs.gtk.org/gtk3/class.IconTheme.html
    ///
    /// (Note that setting a native icon will override any existing [.set_icon()](Self::set_icon))
    pub fn set_native_icon(&self, icon: Option<NativeIcon>) {
        let icon = {
            let mut state = self.state.borrow_mut();
            state.icon = icon.map(IconType::Native);
            state.icon.clone()
        };
        self.platform.borrow_mut().set_icon(icon.as_ref())
    }

    /// Get the logical size, in pixels, this item's icon is drawn at, if one was set with
    /// [`set_icon_size`](Self::set_icon_size).
    pub fn icon_size(&self) -> Option<u32> {
        self.state.borrow().icon_size
    }

    /// Set the logical size, in pixels, at which this item's icon is drawn in a menu, or `None`
    /// to use the crate-wide default ([`set_default_menu_icon_size`](crate::set_default_menu_icon_size))
    /// and then the platform default.
    ///
    /// See [`IconMenuItem::set_icon_size`](crate::IconMenuItem::set_icon_size) for how sizes are
    /// normalised and for the platform-specific behaviour.
    pub fn set_icon_size(&self, size: Option<u32>) {
        let size = crate::menu_icon::normalize_icon_size(size);
        let icon = {
            let mut state = self.state.borrow_mut();
            state.icon_size = size;
            state.icon.clone()
        };
        self.platform
            .borrow_mut()
            .set_icon_size(size, icon.as_ref())
    }

    /// Convert this menu item into its menu ID.
    pub fn into_id(mut self) -> MenuId {
        if let Some(id) = Arc::get_mut(&mut self.id) {
            mem::take(id)
        } else {
            self.id().clone()
        }
    }
}
