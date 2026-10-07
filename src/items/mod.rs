// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{sealed, Icon, MenuId, NativeIcon, WeakStateCell};

mod check;
mod icon;
mod menu;
mod normal;
mod predefined;
mod submenu;

pub use check::*;
pub use icon::*;
pub use menu::*;
pub use normal::*;
pub use predefined::*;
pub use submenu::*;

// ---------------
// Public types
// ---------------

/// A trait that defines a generic item in a menu, which may be one of [`MenuItemKind`]
pub trait IsMenuItem: sealed::Sealed {
    /// Returns a [`MenuItemKind`] associated with this item.
    fn kind(&self) -> MenuItemKind;
    /// Returns a unique identifier associated with this menu item.
    fn id(&self) -> &MenuId;
    /// Convert this menu item into its menu ID.
    fn into_id(self) -> MenuId;
}

/// An enumeration of all available menu types, useful to match against
/// the items returned from [`crate::Menu::items`] or [`Submenu::items`]
#[derive(Clone)]
pub enum MenuItemKind {
    MenuItem(MenuItem),
    Submenu(Submenu),
    Predefined(PredefinedMenuItem),
    Check(CheckMenuItem),
    Icon(IconMenuItem),
}

/// How one part of a menu item's label is rendered.
///
/// Styles are semantic, so each platform maps them to its own conventions instead of
/// the caller picking colors or fonts. That keeps labels correct in light and dark
/// modes, under increased contrast, and when the system menu font changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum TextStyle {
    /// The platform's default menu label treatment.
    #[default]
    Default,
    /// A de-emphasized treatment, for a part of the label that qualifies the rest:
    /// `Preview (default)`, `Speakers (current)`, `Folder (3 items selected)`.
    ///
    /// ## Platform-specific:
    ///
    /// - **macOS**: [`TextStyle::Secondary`] renders in `NSColor.secondaryLabelColor`, the
    ///   same treatment Finder uses for the " (default)" suffix in its "Open with" submenu.
    /// - **Windows / Linux**: every style renders as plain text for now.
    Secondary,
}

// ---------------
// Internal types
// ---------------

/// A thread-bound [`MenuItemKind`] stored inside otherwise thread-safe menu state.
///
/// [`MenuItemKind`] itself is not thread-safe because it contains `Rc` and platform values. This
/// wrapper suppresses automatic destruction so it can cross a thread boundary as part of menu
/// state. Access to the complete value through [`Self::borrow`], [`Self::clone`], or
/// [`Self::unwrap`] is restricted to its originating thread.
///
/// The wrapped value must eventually be recovered with [`Self::unwrap`] and dropped on its
/// originating thread. The thread-bound [`crate::Menu`] and [`Submenu`] implementations uphold that
/// invariant.
pub(crate) struct UnsafeMenuItemKind(std::mem::ManuallyDrop<MenuItemKind>);

// SAFETY: `ManuallyDrop` prevents the wrapped `MenuItemKind` from being destroyed after this
// wrapper crosses a thread boundary. Accessing, cloning, or recovering the complete value requires
// an unsafe call whose originating-thread requirement is upheld by the thread-bound `Menu` and
// `Submenu` implementations. When enabled, the snapshot projection wraps any platform handle it
// captures and queues access and destruction on the platform thread.
unsafe impl Send for UnsafeMenuItemKind {}

impl UnsafeMenuItemKind {
    pub(crate) fn new(local: MenuItemKind) -> Self {
        Self(std::mem::ManuallyDrop::new(local))
    }

    /// Borrows the complete thread-bound menu item.
    ///
    /// # Safety
    ///
    /// The caller must run on the thread where the wrapped [`MenuItemKind`] was created, with no
    /// concurrent access to the wrapped value from another thread.
    pub(crate) unsafe fn borrow(&self) -> &MenuItemKind {
        &self.0
    }

    /// Clones the complete thread-bound menu item.
    ///
    /// # Safety
    ///
    /// The caller must run on the thread where the wrapped [`MenuItemKind`] was created, with no
    /// concurrent access to the wrapped value from another thread. The returned clone must remain
    /// on that thread.
    pub(crate) unsafe fn clone(&self) -> MenuItemKind {
        unsafe { self.borrow() }.clone()
    }

    /// Recovers the complete thread-bound menu item.
    ///
    /// # Safety
    ///
    /// The caller must run on the thread where the wrapped [`MenuItemKind`] was created, with no
    /// concurrent access to the wrapped value from another thread. The recovered value must remain
    /// on that thread and be dropped there.
    pub(crate) unsafe fn unwrap(mut self) -> MenuItemKind {
        unsafe { std::mem::ManuallyDrop::take(&mut self.0) }
    }

    /// Creates a snapshot from thread-safe fields and wrapped platform handles.
    #[cfg(feature = "snapshot")]
    pub(crate) fn snapshot(&self) -> crate::MenuItemKindSnapshot {
        // The `Send` implementation relies on this method reading only the immutable discriminant
        // and wrapping thread-bound platform values before they cross a thread boundary. Wrapped
        // platform values are accessed and destroyed only from the platform main thread.
        crate::MenuItemKindSnapshot::from(&*self.0)
    }
}

impl MenuItemKind {
    /// Returns a thread-safe snapshot handle for this menu item.
    #[cfg(feature = "snapshot")]
    pub fn snapshot(&self) -> crate::MenuItemKindSnapshot {
        self.into()
    }

    /// Returns a unique identifier associated with this menu item.
    pub fn id(&self) -> &MenuId {
        match self {
            MenuItemKind::MenuItem(i) => i.id(),
            MenuItemKind::Submenu(i) => i.id(),
            MenuItemKind::Predefined(i) => i.id(),
            MenuItemKind::Check(i) => i.id(),
            MenuItemKind::Icon(i) => i.id(),
        }
    }

    /// Casts this item to a [`MenuItem`], and returns `None` if it wasn't.
    pub fn as_menuitem(&self) -> Option<&MenuItem> {
        match self {
            MenuItemKind::MenuItem(i) => Some(i),
            _ => None,
        }
    }

    /// Casts this item to a [`MenuItem`], and panics if it wasn't.
    pub fn as_menuitem_unchecked(&self) -> &MenuItem {
        match self {
            MenuItemKind::MenuItem(i) => i,
            _ => panic!("Not a MenuItem"),
        }
    }

    /// Casts this item to a [`Submenu`], and returns `None` if it wasn't.
    pub fn as_submenu(&self) -> Option<&Submenu> {
        match self {
            MenuItemKind::Submenu(i) => Some(i),
            _ => None,
        }
    }

    /// Casts this item to a [`Submenu`], and panics if it wasn't.
    pub fn as_submenu_unchecked(&self) -> &Submenu {
        match self {
            MenuItemKind::Submenu(i) => i,
            _ => panic!("Not a Submenu"),
        }
    }

    /// Casts this item to a [`PredefinedMenuItem`], and returns `None` if it wasn't.
    pub fn as_predefined_menuitem(&self) -> Option<&PredefinedMenuItem> {
        match self {
            MenuItemKind::Predefined(i) => Some(i),
            _ => None,
        }
    }

    /// Casts this item to a [`PredefinedMenuItem`], and panics if it wasn't.
    pub fn as_predefined_menuitem_unchecked(&self) -> &PredefinedMenuItem {
        match self {
            MenuItemKind::Predefined(i) => i,
            _ => panic!("Not a PredefinedMenuItem"),
        }
    }

    /// Casts this item to a [`CheckMenuItem`], and returns `None` if it wasn't.
    pub fn as_check_menuitem(&self) -> Option<&CheckMenuItem> {
        match self {
            MenuItemKind::Check(i) => Some(i),
            _ => None,
        }
    }

    /// Casts this item to a [`CheckMenuItem`], and panics if it wasn't.
    pub fn as_check_menuitem_unchecked(&self) -> &CheckMenuItem {
        match self {
            MenuItemKind::Check(i) => i,
            _ => panic!("Not a CheckMenuItem"),
        }
    }

    /// Casts this item to a [`IconMenuItem`], and returns `None` if it wasn't.
    pub fn as_icon_menuitem(&self) -> Option<&IconMenuItem> {
        match self {
            MenuItemKind::Icon(i) => Some(i),
            _ => None,
        }
    }

    /// Casts this item to a [`IconMenuItem`], and panics if it wasn't.
    pub fn as_icon_menuitem_unchecked(&self) -> &IconMenuItem {
        match self {
            MenuItemKind::Icon(i) => i,
            _ => panic!("Not an IconMenuItem"),
        }
    }

    /// Convert this item into its menu ID.
    pub fn into_id(self) -> MenuId {
        match self {
            MenuItemKind::MenuItem(i) => i.into_id(),
            MenuItemKind::Submenu(i) => i.into_id(),
            MenuItemKind::Predefined(i) => i.into_id(),
            MenuItemKind::Check(i) => i.into_id(),
            MenuItemKind::Icon(i) => i.into_id(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum IconType {
    Custom(Icon),
    Native(NativeIcon),
}

#[derive(Clone)]
pub(crate) enum MenuItemAction {
    Emit(MenuId),
    Toggle(MenuId, WeakStateCell<CheckMenuItemState>),
    Predefined(WeakStateCell<PredefinedMenuItemState>),
}

#[cfg(test)]
mod tests {
    use crate::{
        CheckMenuItem, Icon, IconMenuItem, IconType, MenuId, MenuItem, NativeIcon,
        PredefinedMenuItem, Submenu,
    };

    fn icon() -> Icon {
        Icon::from_rgba(vec![255; 4 * 4 * 4], 4, 4).unwrap()
    }

    #[test]
    #[cfg_attr(
        all(
            miri,
            not(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))
        ),
        ignore
    )]
    fn it_returns_same_id() {
        let id = MenuId::new("1");
        assert_eq!(id, MenuItem::with_id(id.clone(), "", true, None).id());
        assert_eq!(id, Submenu::with_id(id.clone(), "", true).id());
        assert_eq!(
            id,
            CheckMenuItem::with_id(id.clone(), "", true, true, None).id()
        );
        assert_eq!(
            id,
            IconMenuItem::with_id(id.clone(), "", true, None, None).id()
        );
    }

    #[test]
    #[cfg_attr(
        all(
            miri,
            not(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))
        ),
        ignore
    )]
    fn test_convert_from_id_and_into_id() {
        let id = "TEST ID";
        let expected = MenuId(id.to_string());

        let item = CheckMenuItem::with_id(id, "test", true, true, None);
        assert_eq!(item.id(), &expected);
        assert_eq!(item.into_id(), expected);

        let item = IconMenuItem::with_id(id, "test", true, None, None);
        assert_eq!(item.id(), &expected);
        assert_eq!(item.into_id(), expected);

        let item = MenuItem::with_id(id, "test", true, None);
        assert_eq!(item.id(), &expected);
        assert_eq!(item.into_id(), expected);

        let item = Submenu::with_id(id, "test", true);
        assert_eq!(item.id(), &expected);
        assert_eq!(item.into_id(), expected);

        let item = PredefinedMenuItem::separator();
        assert_eq!(item.id().clone(), item.into_id());
    }

    #[test]
    #[cfg_attr(
        all(
            miri,
            not(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))
        ),
        ignore
    )]
    fn check_item_icon_can_be_set_replaced_and_cleared() {
        let item = CheckMenuItem::new("test", true, true, None);
        assert!(item.state.borrow().icon.is_none());

        item.set_icon(Some(icon()));
        assert!(matches!(
            item.state.borrow().icon,
            Some(IconType::Custom(_))
        ));

        item.set_native_icon(Some(NativeIcon::Folder));
        assert!(matches!(
            item.state.borrow().icon,
            Some(IconType::Native(_))
        ));

        item.set_icon(Some(icon()));
        assert!(matches!(
            item.state.borrow().icon,
            Some(IconType::Custom(_))
        ));

        item.set_icon(None);
        assert!(item.state.borrow().icon.is_none());

        // Changing the icon leaves the check state alone.
        assert!(item.is_checked());
    }

    #[test]
    #[cfg_attr(
        all(
            miri,
            not(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))
        ),
        ignore
    )]
    fn check_item_and_submenu_icon_size_is_normalised() {
        let check = CheckMenuItem::new("test", true, false, None);
        let submenu = Submenu::new("test", true);
        assert_eq!((check.icon_size(), submenu.icon_size()), (None, None));

        check.set_icon_size(Some(24));
        submenu.set_icon_size(Some(24));
        assert_eq!(
            (check.icon_size(), submenu.icon_size()),
            (Some(24), Some(24))
        );

        check.set_icon_size(Some(0));
        submenu.set_icon_size(Some(10_000));
        assert_eq!((check.icon_size(), submenu.icon_size()), (None, Some(256)));

        check.set_icon_size(None);
        submenu.set_icon_size(None);
        assert_eq!((check.icon_size(), submenu.icon_size()), (None, None));
    }

    #[test]
    #[cfg_attr(
        all(
            miri,
            not(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))
        ),
        ignore
    )]
    fn builders_set_the_icon_and_its_size() {
        let check = CheckMenuItem::builder()
            .text("test")
            .checked(true)
            .icon(Some(icon()))
            .icon_size(Some(20))
            .build();
        assert!(matches!(
            check.state.borrow().icon,
            Some(IconType::Custom(_))
        ));
        assert_eq!(check.icon_size(), Some(20));
        assert!(check.is_checked());

        // The last of `icon` and `native_icon` wins.
        let check = CheckMenuItem::builder()
            .icon(Some(icon()))
            .native_icon(Some(NativeIcon::Folder))
            .build();
        assert!(matches!(
            check.state.borrow().icon,
            Some(IconType::Native(_))
        ));
        let check = CheckMenuItem::builder()
            .native_icon(Some(NativeIcon::Folder))
            .icon(Some(icon()))
            .build();
        assert!(matches!(
            check.state.borrow().icon,
            Some(IconType::Custom(_))
        ));

        let submenu = Submenu::builder()
            .text("test")
            .icon(icon())
            .icon_size(Some(20))
            .build()
            .unwrap();
        assert!(matches!(
            submenu.state.borrow().icon,
            Some(IconType::Custom(_))
        ));
        assert_eq!(submenu.icon_size(), Some(20));
    }
}
