---
"muda": minor
---

Let check menu items carry an icon, and let submenu icons be sized like icon menu item icons:

- `CheckMenuItem::set_icon`, `CheckMenuItem::set_native_icon`, `CheckMenuItem::icon_size`, `CheckMenuItem::set_icon_size` and the matching `CheckMenuItemBuilder::icon`, `native_icon` and `icon_size` set, replace or clear the icon of a check menu item, with the same semantics as on `IconMenuItem`. The check state is kept: on Windows the icon goes in the item's bitmap slot, where the system draws a checked state as a frame around the bitmap; on GTK 3 the image sits between the check indicator and the text; on macOS it is drawn next to the text. GTK 4 does not draw it.
- `Submenu::icon_size`, `Submenu::set_icon_size` and `SubmenuBuilder::icon_size` set the logical size a submenu's existing icon is drawn at, and `set_default_menu_icon_size` now covers check menu items and submenus too. `CheckMenuItemSnapshot::icon` reads a check item's icon from a menu snapshot.
- On GTK 3, `set_text` now changes the label of items that hold an icon (icon menu items, submenus and check menu items with an icon), which it did not reach before.
