---
"muda": minor
---

Add control over how menu icons are sized, and a styling hook for GTK 3 menus:

- `IconMenuItemBuilder::icon_size`, `IconMenuItem::set_icon_size` and `IconMenuItem::icon_size` set the logical size, in pixels, an icon is drawn at in a menu, and `set_default_menu_icon_size` sets it crate-wide. Without a size nothing changes: 16 pixels on GTK 3, and 16 physical pixels on Windows. On GTK 3 the icon is rendered at the display scale factor and handed to GTK as a HiDPI image so it stays sharp, and non-square icons are fitted into the square without being stretched. On Windows icons created from pixels are resampled with premultiplied alpha at the size times the DPI scale of the window a context menu is shown for.
- On GTK 3, the new `gtk_style` module installs application CSS for the screen (`gtk_style::set_css`) and adds a style class of the application's choosing to every menu muda creates (`gtk_style::set_menu_css_class`), so menus can be themed with selectors such as `menu.my-app-menu menuitem`. Invalid CSS is returned as the new `Error::InvalidGtkCss`.
