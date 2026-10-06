// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(clippy::uninlined_format_args)]

//! muda is a Menu Utilities library for Desktop Applications.
//!
//! # Platforms supported:
//!
//! - Windows
//! - macOS
//! - Linux/BSD with GTK 3 or GTK 4
//!
//! # Platform-specific notes:
//!
//! - On macOS, menus can only be used from the main thread, and most
//!   functionality will panic if you try to use it from any other thread.
//!
//! - On Windows, accelerators don't work unless the win32 message loop calls
//!   [`TranslateAcceleratorW`](https://docs.rs/windows-sys/latest/windows_sys/Win32/UI/WindowsAndMessaging/fn.TranslateAcceleratorW.html).
//!   See [`Menu::init_for_hwnd`](https://docs.rs/muda/latest/x86_64-pc-windows-msvc/muda/struct.Menu.html#method.init_for_hwnd) for more details
//!
//! # Cargo features
//!
//! The Win32 and AppKit backends are always enabled on Windows and macOS, respectively.
//!
//! - `gtk3`: Enables the GTK 3 backend on Linux and BSD platforms. This is enabled by default.
//! - `gtk4`: Enables the GTK 4 backend on Linux and BSD platforms. Disable default features when
//!   enabling this feature because the defaults include `gtk3`.
//! - `libxdo`: Enables linking to `libxdo` for the GTK 3 backend. This is used by the predefined
//!   `Copy`, `Cut`, `Paste` and `SelectAll` menu items, and is enabled by default. It is not used
//!   by GTK 4.
//! - `snapshot`: Enables thread-safe menu snapshot types and methods, switching shared menu state
//!   to thread-safe synchronization.
//!
//! When both `gtk3` and `gtk4` features are enabled, Muda uses the GTK 4 backend.
//!
//! # Dependencies (Linux/BSD)
//!
//! The `gtk3` feature uses GTK 3 for menus. The `gtk4` feature uses GTK 4 for menus. `libxdo` is
//! only used by the GTK 3 backend to make the predefined `Copy`, `Cut`, `Paste` and `SelectAll`
//! menu items work when the `libxdo` feature is enabled. Be sure to install the packages for the
//! GTK backend you enabled before building:
//!
//! #### Arch Linux / Manjaro:
//!
//! ```sh
//! # GTK 3 backend
//! pacman -S gtk3 xdotool
//!
//! # GTK 4 backend
//! pacman -S gtk4
//! ```
//!
//! #### Debian / Ubuntu:
//!
//! ```sh
//! # GTK 3 backend
//! sudo apt install libgtk-3-dev libxdo-dev
//!
//! # GTK 4 backend
//! sudo apt install libgtk-4-dev
//! ```
//!
//! #### FreeBSD:
//!
//! ```sh
//! # GTK 3 backend
//! pkg install -y rust glib pkgconf gtk3 xdotool
//!
//! # GTK 4 backend
//! pkg install -y rust glib pkgconf gtk4
//! ```
//!
//! # Example
//!
//! Create the menu and add your items
//!
//! ```no_run
//! # use muda::{Menu, Submenu, MenuItem, accelerator::{Code, Modifiers, Accelerator}, PredefinedMenuItem};
//! let menu = Menu::new();
//! let menu_item2 = MenuItem::new("Menu item #2", false, None);
//! let submenu = Submenu::with_items(
//!     "Submenu Outer",
//!     true,
//!     &[
//!         &MenuItem::new(
//!             "Menu item #1",
//!             true,
//!             Some(Accelerator::new(Modifiers::ALT, Code::KeyD)),
//!         ),
//!         &PredefinedMenuItem::separator(),
//!         &menu_item2,
//!         &MenuItem::new("Menu item #3", true, None),
//!         &PredefinedMenuItem::separator(),
//!         &Submenu::with_items(
//!             "Submenu Inner",
//!             true,
//!             &[
//!                 &MenuItem::new("Submenu item #1", true, None),
//!                 &PredefinedMenuItem::separator(),
//!                 &menu_item2,
//!             ],
//!         ).unwrap(),
//!     ],
//! );
//! ```
//!
//! Then add your root menu to a window on Windows, GTK 3, or GTK 4
//! or use it as your global app menu on macOS
//!
//! ```no_run
//! # #[cfg(feature = "gtk4")]
//! # use gtk4 as gtk;
//! # let menu = muda::Menu::new();
//! # let window_hwnd = 0;
//! # #[cfg(any(
//!     target_os = "linux",
//!         target_os = "dragonfly",
//!         target_os = "freebsd",
//!         target_os = "netbsd",
//!         target_os = "openbsd"
//! ))]
//! # let gtk_window = gtk::Window::builder().build();
//! # #[cfg(any(
//!     target_os = "linux",
//!     target_os = "dragonfly",
//!     target_os = "freebsd",
//!     target_os = "netbsd",
//!     target_os = "openbsd"
//! ))]
//! # let vertical_gtk_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
//! // --snip--
//! #[cfg(target_os = "windows")]
//! unsafe { menu.init_for_hwnd(window_hwnd) };
//! #[cfg(any(
//!     target_os = "linux",
//!     target_os = "dragonfly",
//!     target_os = "freebsd",
//!     target_os = "netbsd",
//!     target_os = "openbsd"
//! ))]
//! menu.init_for_gtk_window(&gtk_window, Some(&vertical_gtk_box));
//! #[cfg(target_os = "macos")]
//! menu.init_for_nsapp();
//! ```
//!
//! # Context menus (Popup menus)
//!
//! You can also use a [`Menu`] or a [`Submenu`] to show a context menu.
//!
//! ```no_run
//! # #[cfg(feature = "gtk4")]
//! # use gtk4 as gtk;
//! use muda::ContextMenu;
//! # let menu = muda::Menu::new();
//! # let window_hwnd = 0;
//! # #[cfg(any(
//!     target_os = "linux",
//!     target_os = "dragonfly",
//!     target_os = "freebsd",
//!     target_os = "netbsd",
//!     target_os = "openbsd"
//! ))]
//! # let gtk_window = gtk::Window::builder().build();
//! # #[cfg(target_os = "macos")]
//! # let nsview = std::ptr::null();
//! // --snip--
//! let position = muda::dpi::PhysicalPosition { x: 100., y: 120. };
//! #[cfg(target_os = "windows")]
//! unsafe { menu.show_context_menu_for_hwnd(window_hwnd, Some(position.into())) };
//! #[cfg(any(
//!     target_os = "linux",
//!     target_os = "dragonfly",
//!     target_os = "freebsd",
//!     target_os = "netbsd",
//!     target_os = "openbsd"
//! ))]
//! menu.show_context_menu_for_gtk_window(&gtk_window, Some(position.into()));
//! #[cfg(target_os = "macos")]
//! unsafe { menu.show_context_menu_for_nsview(nsview, Some(position.into())) };
//! ```
//! # Processing menu events
//!
//! You can use [`MenuEvent::receiver`] to get a reference to the [`MenuEventReceiver`]
//! which you can use to listen to events when a menu item is activated
//! ```no_run
//! # use muda::MenuEvent;
//! #
//! # let save_item: muda::MenuItem = unsafe { std::mem::zeroed() };
//! if let Ok(event) = MenuEvent::receiver().try_recv() {
//!     match event.id {
//!         id if id == save_item.id() => {
//!             println!("Save menu item activated");
//!         },
//!         _ => {}
//!     }
//! }
//! ```
//!
//! ### Note for [winit] or [tao] users:
//!
//! You should use [`MenuEvent::set_event_handler`] and forward
//! the menu events to the event loop by using [`EventLoopProxy`]
//! so that the event loop is awakened on each menu event.
//!
//! ```no_run
//! # use tao::event_loop::EventLoopBuilder;
//! enum UserEvent {
//!   MenuEvent(muda::MenuEvent)
//! }
//!
//! let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
//!
//! let proxy = event_loop.create_proxy();
//! muda::MenuEvent::set_event_handler(Some(move |event| {
//!     proxy.send_event(UserEvent::MenuEvent(event));
//! }));
//! ```
//!
//! [`EventLoopProxy`]: https://docs.rs/winit/latest/winit/event_loop/struct.EventLoopProxy.html
//! [winit]: https://docs.rs/winit
//! [tao]: https://docs.rs/tao

#[cfg(all(
    any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ),
    feature = "gtk4"
))]
extern crate gtk4 as gtk;

pub mod about_metadata;
pub mod accelerator;
mod builders;
mod context_menu;
mod error;
mod icon;
mod items;
mod menu_event;
mod menu_icon;
mod menu_id;
mod platform_impl;
mod sealed;
#[cfg(feature = "snapshot")]
mod snapshot;
mod state_cell;
mod util;

/// Styling for the menus muda builds with GTK 3.
///
/// muda creates ordinary `GtkMenu` widgets, so an application can theme them with CSS the same
/// way it would theme any GTK widget. Two things are needed: the CSS itself, installed for the
/// whole screen with [`set_css`], and a style class that scopes it to muda's menus, set with
/// [`set_menu_css_class`] and added to every popup menu and submenu muda creates after the call.
/// Set the class before building menus; menus that already exist keep the classes they were
/// created with.
///
/// ```no_run
/// # fn main() -> muda::Result<()> {
/// gtk::init().unwrap();
///
/// muda::gtk_style::set_menu_css_class(Some("my-app-menu"))?;
/// muda::gtk_style::set_css(
///     "menu.my-app-menu { background: #202124; border-radius: 8px; padding: 4px; }
///      menu.my-app-menu menuitem { color: #e8eaed; border-radius: 4px; padding: 4px 8px; }
///      menu.my-app-menu menuitem:hover { background: #3c4043; }",
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// CSS is installed at `GTK_STYLE_PROVIDER_PRIORITY_APPLICATION`, above the theme and below user
/// overrides. Invalid CSS is reported as [`Error::InvalidGtkCss`] and leaves the previously
/// installed CSS in place. muda itself gives each item's content box a negative left margin so
/// that icons line up with the theme's menu padding; that margin is not part of the styling
/// surface, so style the `menu`, `menuitem` and `image` nodes instead.
///
/// ## Platform-specific
///
/// Only available with the `gtk3` feature, and not when `gtk4` is also enabled.
#[cfg(all(
    any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ),
    feature = "gtk3",
    not(feature = "gtk4")
))]
pub mod gtk_style {
    pub use crate::platform_impl::style::{clear_css, menu_css_class, set_css, set_menu_css_class};
}

pub use about_metadata::AboutMetadata;
pub use builders::*;
pub use context_menu::ContextMenu;
pub use dpi;
pub use error::*;
pub use icon::{BadIcon, Icon, NativeIcon};
pub use items::*;
pub use menu_event::{MenuEvent, MenuEventReceiver};
pub use menu_icon::{default_menu_icon_size, set_default_menu_icon_size};
pub use menu_id::MenuId;
#[cfg(feature = "snapshot")]
pub use snapshot::*;
pub(crate) use state_cell::{StateCell, WeakStateCell};
