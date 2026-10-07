// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
    accelerator::{Accelerator, KeyAccelerator, MenuAccelerator},
    icon::{Icon, NativeIcon},
    CheckMenuItem, MenuId, TextStyle,
};

/// A builder type for [`CheckMenuItem`]
#[derive(Clone, Debug)]
pub struct CheckMenuItemBuilder {
    text: String,
    enabled: bool,
    checked: bool,
    accelerator: Option<MenuAccelerator>,
    id: Option<MenuId>,
    styled_text: Option<Vec<(String, TextStyle)>>,
    icon: Option<Icon>,
    native_icon: Option<NativeIcon>,
    icon_size: Option<u32>,
}

impl Default for CheckMenuItemBuilder {
    fn default() -> Self {
        Self {
            text: String::new(),
            enabled: true,
            checked: false,
            accelerator: None,
            id: None,
            styled_text: None,
            icon: None,
            native_icon: None,
            icon_size: None,
        }
    }
}

impl CheckMenuItemBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the id this check menu item.
    pub fn id(mut self, id: MenuId) -> Self {
        self.id.replace(id);
        self
    }

    /// Set the text for this check menu item.
    ///
    /// See [`CheckMenuItem::set_text`] for more info.
    pub fn text<S: Into<String>>(mut self, text: S) -> Self {
        self.text = text.into();
        self
    }

    /// Enable or disable this menu item.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Check or uncheck this menu item.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set this check menu item icon.
    ///
    /// See [`CheckMenuItem::set_icon`] for more info.
    ///
    /// (Note that setting an icon will override any existing [.native_icon()](Self::native_icon))
    pub fn icon(mut self, icon: Option<Icon>) -> Self {
        self.icon = icon;
        self.native_icon = None;
        self
    }

    /// Set this check menu item native icon.
    ///
    /// See [`CheckMenuItem::set_native_icon`] for more info.
    ///
    /// (Note that setting a native icon will override any existing [.icon()](Self::icon))
    pub fn native_icon(mut self, icon: Option<NativeIcon>) -> Self {
        self.native_icon = icon;
        self.icon = None;
        self
    }

    /// Set the logical size, in pixels, at which this check menu item's icon is drawn.
    ///
    /// See [`CheckMenuItem::set_icon_size`] for more info.
    pub fn icon_size(mut self, size: Option<u32>) -> Self {
        self.icon_size = size;
        self
    }

    /// Set this check menu item accelerator.
    ///
    /// (Note that setting an accelerator will override any existing [.key_accelerator()](Self::key_accelerator))
    pub fn accelerator<A: TryInto<Accelerator>>(
        mut self,
        accelerator: Option<A>,
    ) -> crate::Result<Self>
    where
        crate::Error: From<<A as TryInto<Accelerator>>::Error>,
    {
        self.accelerator = accelerator
            .map(|a| a.try_into().map(MenuAccelerator::Physical))
            .transpose()?;
        Ok(self)
    }

    /// Set this check menu item accelerator using a [`KeyAccelerator`].
    ///
    /// (Note that setting a key_accelerator will override any existing [.accelerator()](Self::accelerator))
    pub fn key_accelerator<A: TryInto<KeyAccelerator>>(
        mut self,
        accelerator: Option<A>,
    ) -> crate::Result<Self>
    where
        crate::Error: From<<A as TryInto<KeyAccelerator>>::Error>,
    {
        self.accelerator = accelerator
            .map(|a| a.try_into().map(MenuAccelerator::Logical))
            .transpose()?;
        Ok(self)
    }

    /// Set the text for this menu item as a sequence of styled text, so one part of the
    /// label can be de-emphasized relative to the rest.
    ///
    /// Overrides any text set with [`.text()`](Self::text).
    ///
    /// See [`CheckMenuItem::set_styled_text`] for more info.
    pub fn styled_text<S: Into<String>>(
        mut self,
        parts: impl IntoIterator<Item = (S, TextStyle)>,
    ) -> Self {
        self.styled_text = Some(
            parts
                .into_iter()
                .map(|(text, style)| (text.into(), style))
                .collect(),
        );
        self
    }

    /// Build this check menu item.
    pub fn build(self) -> CheckMenuItem {
        let item = if let Some(id) = self.id {
            CheckMenuItem::with_id(id, self.text, self.enabled, self.checked, None)
        } else {
            CheckMenuItem::new(self.text, self.enabled, self.checked, None)
        };

        if let Some(accelerator) = self.accelerator {
            let _ = match accelerator {
                MenuAccelerator::Physical(accelerator) => item.set_accelerator(Some(accelerator)),
                MenuAccelerator::Logical(accelerator) => {
                    item.set_key_accelerator(Some(accelerator))
                }
            };
        }

        if let Some(parts) = self.styled_text {
            item.set_styled_text(parts);
        }

        // The size goes first so the icon is only drawn once.
        if self.icon_size.is_some() {
            item.set_icon_size(self.icon_size);
        }

        if self.icon.is_some() {
            item.set_icon(self.icon);
        } else if self.native_icon.is_some() {
            item.set_native_icon(self.native_icon);
        }

        item
    }
}
