// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// taken from https://github.com/rust-windowing/winit/blob/92fdf5ba85f920262a61cee4590f4a11ad5738d1/src/platform_impl/windows/icon.rs

use std::{fmt, io, mem, path::Path, sync::Arc};

use windows_sys::{
    core::PCWSTR,
    Win32::{Graphics::Gdi::*, UI::WindowsAndMessaging::*},
};

use crate::{icon::*, menu_icon::menu_bitmap_bgra};

use super::util;

impl Pixel {
    fn convert_to_bgra(&mut self) {
        mem::swap(&mut self.r, &mut self.b);
    }
}

impl RgbaIcon {
    fn into_windows_icon(self) -> Result<WinIcon, BadIcon> {
        let source = Arc::new(RgbaIcon {
            rgba: self.rgba.clone(),
            width: self.width,
            height: self.height,
        });
        let rgba = self.rgba;
        let pixel_count = rgba.len() / PIXEL_SIZE;
        let mut and_mask = Vec::with_capacity(pixel_count);
        let pixels =
            unsafe { std::slice::from_raw_parts_mut(rgba.as_ptr() as *mut Pixel, pixel_count) };
        for pixel in pixels {
            and_mask.push(pixel.a.wrapping_sub(u8::MAX)); // invert alpha channel
            pixel.convert_to_bgra();
        }
        assert_eq!(and_mask.len(), pixel_count);
        let handle = unsafe {
            CreateIcon(
                std::ptr::null_mut(),
                self.width as i32,
                self.height as i32,
                1,
                (PIXEL_SIZE * 8) as u8,
                and_mask.as_ptr(),
                rgba.as_ptr(),
            )
        };
        if !handle.is_null() {
            Ok(WinIcon {
                rgba: Some(source),
                ..WinIcon::from_handle(handle)
            })
        } else {
            Err(BadIcon::OsError(io::Error::last_os_error()))
        }
    }
}

#[derive(Debug)]
struct RaiiIcon {
    handle: HICON,
}

#[derive(Clone)]
pub(crate) struct WinIcon {
    inner: Arc<RaiiIcon>,
    /// The straight-alpha pixels the icon was created from, when it was created from pixels.
    rgba: Option<Arc<RgbaIcon>>,
}

unsafe impl Send for WinIcon {}

impl WinIcon {
    /// Draws the icon into a 16 x 16 menu bitmap.
    pub unsafe fn to_hbitmap(&self) -> HBITMAP {
        self.draw_hbitmap(16)
    }

    /// Creates a `size` x `size` pixel menu bitmap. Icons made from pixels are resampled and
    /// premultiplied here; any other icon is drawn by the system at that size.
    pub unsafe fn to_hbitmap_sized(&self, size: u32) -> HBITMAP {
        let Some(source) = &self.rgba else {
            return self.draw_hbitmap(size as i32);
        };

        let pixels = menu_bitmap_bgra(&source.rgba, source.width, source.height, size);
        let mut bits = std::ptr::null_mut();
        let hbitmap = create_dib_section(size as i32, &mut bits);
        if !hbitmap.is_null() && !bits.is_null() {
            // A bottom-up DIB stores its last row first.
            let row_len = size as usize * 4;
            let dib = std::slice::from_raw_parts_mut(bits as *mut u8, pixels.len());
            for (row, dib_row) in pixels
                .chunks_exact(row_len)
                .zip(dib.chunks_exact_mut(row_len).rev())
            {
                dib_row.copy_from_slice(row);
            }
        }
        hbitmap
    }

    unsafe fn draw_hbitmap(&self, size: i32) -> HBITMAP {
        let hdc = CreateCompatibleDC(std::ptr::null_mut());

        let mut bits = std::ptr::null_mut();
        let hbitmap = create_dib_section(size, &mut bits);

        let h_bitmap_old = SelectObject(hdc, hbitmap);

        DrawIconEx(
            hdc,
            0,
            0,
            self.inner.handle,
            size,
            size,
            0,
            std::ptr::null_mut(),
            DI_NORMAL,
        );

        SelectObject(hdc, h_bitmap_old);
        DeleteDC(hdc);

        hbitmap
    }

    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
        RgbaIcon::from_rgba(rgba, width, height)?.into_windows_icon()
    }

    pub(crate) fn from_handle(handle: HICON) -> Self {
        Self {
            #[allow(clippy::arc_with_non_send_sync)]
            inner: Arc::new(RaiiIcon { handle }),
            rgba: None,
        }
    }

    pub(crate) fn from_path<P: AsRef<Path>>(
        path: P,
        size: Option<(u32, u32)>,
    ) -> Result<Self, BadIcon> {
        // width / height of 0 along with LR_DEFAULTSIZE tells windows to load the default icon size
        let (width, height) = size.unwrap_or((0, 0));

        let wide_path = util::encode_wide(path.as_ref());

        let handle = unsafe {
            LoadImageW(
                std::ptr::null_mut(),
                wide_path.as_ptr(),
                IMAGE_ICON,
                width as i32,
                height as i32,
                LR_DEFAULTSIZE | LR_LOADFROMFILE,
            )
        };
        if !handle.is_null() {
            Ok(WinIcon::from_handle(handle as HICON))
        } else {
            Err(BadIcon::OsError(io::Error::last_os_error()))
        }
    }

    pub(crate) fn from_resource(
        resource_id: u16,
        size: Option<(u32, u32)>,
    ) -> Result<Self, BadIcon> {
        // width / height of 0 along with LR_DEFAULTSIZE tells windows to load the default icon size
        let (width, height) = size.unwrap_or((0, 0));
        let handle = unsafe {
            LoadImageW(
                util::get_instance_handle(),
                resource_id as PCWSTR,
                IMAGE_ICON,
                width as i32,
                height as i32,
                LR_DEFAULTSIZE,
            )
        };
        if !handle.is_null() {
            Ok(WinIcon::from_handle(handle as HICON))
        } else {
            Err(BadIcon::OsError(io::Error::last_os_error()))
        }
    }
}

impl Drop for RaiiIcon {
    fn drop(&mut self) {
        unsafe { DestroyIcon(self.handle) };
    }
}

impl fmt::Debug for WinIcon {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        (*self.inner).fmt(formatter)
    }
}

/// Creates a bottom-up 32-bit DIB section of `size` x `size` pixels and returns where its pixels
/// live in `bits`.
unsafe fn create_dib_section(size: i32, bits: &mut *mut std::ffi::c_void) -> HBITMAP {
    let mut bitmap_info: BITMAPINFO = std::mem::zeroed();
    bitmap_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as _;
    bitmap_info.bmiHeader.biWidth = size;
    bitmap_info.bmiHeader.biHeight = size;
    bitmap_info.bmiHeader.biPlanes = 1;
    bitmap_info.bmiHeader.biBitCount = 32;
    bitmap_info.bmiHeader.biCompression = BI_RGB as _;

    let h_dc_bitmap = GetDC(std::ptr::null_mut());
    let hbitmap = CreateDIBSection(
        h_dc_bitmap,
        &bitmap_info,
        DIB_RGB_COLORS,
        bits,
        std::ptr::null_mut(),
        0,
    );
    ReleaseDC(std::ptr::null_mut(), h_dc_bitmap);

    hbitmap
}
