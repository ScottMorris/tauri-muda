// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Right-click the window to open a context menu showing the same icon at several sizes. On
//! GTK 3 the menu is also themed with application CSS.

#![allow(unused)]

use muda::{ContextMenu, IconMenuItem, Menu, MenuEvent, MenuItem};
#[cfg(all(
    feature = "gtk3",
    any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )
))]
use tao::platform::unix::WindowExtUnix;
#[cfg(target_os = "windows")]
use tao::platform::windows::WindowExtWindows;
use tao::{
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Window, WindowBuilder},
};

fn main() {
    let event_loop = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_title("Menu icons")
        .build(&event_loop)
        .unwrap();

    // Menu icons are drawn at 16 logical pixels unless asked otherwise. Setting a crate-wide
    // default affects every item that does not set its own size.
    // muda::set_default_menu_icon_size(Some(20));

    #[cfg(all(
        feature = "gtk3",
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )
    ))]
    {
        // The class must be set before the menu is built; the CSS can be installed at any time.
        muda::gtk_style::set_menu_css_class(Some("example-menu")).unwrap();
        muda::gtk_style::set_css(
            "menu.example-menu { background: #202124; border-radius: 8px; padding: 4px; }
             menu.example-menu menuitem { color: #e8eaed; border-radius: 4px; padding: 4px 8px; }
             menu.example-menu menuitem:hover { background: #3c4043; }",
        )
        .unwrap();
    }

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/icon.png");
    let icon = load_icon(std::path::Path::new(path));

    let menu = Menu::new();
    menu.append_items(&[
        &IconMenuItem::builder()
            .text("Default size")
            .icon(Some(icon.clone()))
            .build(),
        &IconMenuItem::builder()
            .text("16 pixels")
            .icon(Some(icon.clone()))
            .icon_size(Some(16))
            .build(),
        &IconMenuItem::builder()
            .text("24 pixels")
            .icon(Some(icon.clone()))
            .icon_size(Some(24))
            .build(),
        &MenuItem::new("No icon", true, None),
    ])
    .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Released,
                        button: MouseButton::Right,
                        ..
                    },
                ..
            } => {
                show_context_menu(&window, &menu);
                while let Ok(event) = MenuEvent::receiver().try_recv() {
                    println!("{event:?}");
                }
            }
            _ => (),
        }
    })
}

fn show_context_menu(window: &Window, menu: &dyn ContextMenu) {
    #[cfg(target_os = "windows")]
    unsafe {
        menu.show_context_menu_for_hwnd(window.hwnd() as _, None);
    }
    #[cfg(all(
        feature = "gtk3",
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )
    ))]
    menu.show_context_menu_for_gtk_window(window.gtk_window().as_ref(), None);
}

fn load_icon(path: &std::path::Path) -> muda::Icon {
    let image = image::open(path)
        .expect("Failed to open icon path")
        .into_rgba8();
    let (width, height) = image.dimensions();
    muda::Icon::from_rgba(image.into_raw(), width, height).expect("Failed to open icon")
}
