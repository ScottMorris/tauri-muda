// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Lets the host application theme the menus muda builds on GTK 3 with its own CSS.

use std::{cell::RefCell, sync::Mutex};

use gtk::{gdk, prelude::*};

static MENU_CSS_CLASS: Mutex<Option<String>> = Mutex::new(None);

thread_local! {
    // GTK is only usable from one thread, so the installed provider can live beside it.
    static PROVIDER: RefCell<Option<gtk::CssProvider>> = const { RefCell::new(None) };
}

/// Installs `css` for the default screen at application priority, replacing any CSS installed by
/// a previous call. See [`crate::gtk_style`] for details.
pub fn set_css(css: &str) -> crate::Result<()> {
    let screen = gdk::Screen::default().ok_or(crate::Error::GtkStyleUnavailable)?;

    let provider = gtk::CssProvider::new();
    provider
        .load_from_data(css.as_bytes())
        .map_err(|error| crate::Error::InvalidGtkCss(error.to_string()))?;

    PROVIDER.with(|current| {
        let mut current = current.borrow_mut();
        if let Some(previous) = current.take() {
            gtk::StyleContext::remove_provider_for_screen(&screen, &previous);
        }
        gtk::StyleContext::add_provider_for_screen(
            &screen,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        *current = Some(provider);
    });

    Ok(())
}

/// Removes the CSS installed by [`set_css`], if any.
pub fn clear_css() {
    let Some(screen) = gdk::Screen::default() else {
        return;
    };
    PROVIDER.with(|current| {
        if let Some(previous) = current.borrow_mut().take() {
            gtk::StyleContext::remove_provider_for_screen(&screen, &previous);
        }
    });
}

/// Sets the style class added to every menu (popup menu and submenu) muda creates afterwards.
/// See [`crate::gtk_style`] for details.
pub fn set_menu_css_class(class: Option<&str>) -> crate::Result<()> {
    let class = match class {
        None => None,
        Some(class) => {
            let valid = !class.is_empty()
                && !class.starts_with(|c: char| c.is_ascii_digit() || c == '-')
                && class
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
            if !valid {
                return Err(crate::Error::InvalidGtkCssClass(class.to_string()));
            }
            Some(class.to_string())
        }
    };

    *MENU_CSS_CLASS.lock().unwrap_or_else(|e| e.into_inner()) = class;
    Ok(())
}

/// Returns the class set by [`set_menu_css_class`].
pub fn menu_css_class() -> Option<String> {
    MENU_CSS_CLASS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

/// Creates a menu carrying the configured style class.
pub(super) fn new_menu() -> gtk::Menu {
    let menu = gtk::Menu::new();
    if let Some(class) = menu_css_class() {
        menu.style_context().add_class(&class);
    }
    menu
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_css_class_is_validated_and_stored() {
        for invalid in ["", "has space", "-leading", "1leading", "dot.ted", "a>b"] {
            assert!(
                matches!(
                    set_menu_css_class(Some(invalid)),
                    Err(crate::Error::InvalidGtkCssClass(_))
                ),
                "{invalid:?} should be rejected"
            );
        }

        set_menu_css_class(Some("my-app_menu2")).unwrap();
        assert_eq!(menu_css_class().as_deref(), Some("my-app_menu2"));
        set_menu_css_class(None).unwrap();
        assert_eq!(menu_css_class(), None);
    }
}
