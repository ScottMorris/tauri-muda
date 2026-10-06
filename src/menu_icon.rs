// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Platform-independent helpers for sizing menu icons.
//!
//! The pixel maths lives here, rather than in a platform backend, so that it can be unit tested
//! on any host. Not every backend uses every helper.

#![allow(dead_code)]

use std::sync::atomic::{AtomicU32, Ordering};

/// Largest logical icon size, in pixels, that will be honoured.
const MAX_ICON_SIZE: u32 = 256;

/// `0` means "no crate-wide default": each backend keeps its own size.
static DEFAULT_ICON_SIZE: AtomicU32 = AtomicU32::new(0);

/// Sets the logical size, in pixels, at which menu icons are drawn when an item does not set its
/// own with [`IconMenuItemBuilder::icon_size`] or [`IconMenuItem::set_icon_size`].
///
/// Pass `None` to restore the platform default (the default). The value is read when an item is
/// attached to a menu, so call this before building menus.
///
/// ## Platform-specific
///
/// - **Windows / GTK 3**: honoured. Sizes are logical pixels and are multiplied by the display
///   scale factor.
/// - **macOS / GTK 4**: ignored.
///
/// [`IconMenuItemBuilder::icon_size`]: crate::IconMenuItemBuilder::icon_size
/// [`IconMenuItem::set_icon_size`]: crate::IconMenuItem::set_icon_size
pub fn set_default_menu_icon_size(size: Option<u32>) {
    DEFAULT_ICON_SIZE.store(normalize_icon_size(size).unwrap_or(0), Ordering::Relaxed);
}

/// Returns the crate-wide default menu icon size set by [`set_default_menu_icon_size`].
pub fn default_menu_icon_size() -> Option<u32> {
    match DEFAULT_ICON_SIZE.load(Ordering::Relaxed) {
        0 => None,
        size => Some(size),
    }
}

/// Treats `0` as unset and caps absurd values.
pub(crate) fn normalize_icon_size(size: Option<u32>) -> Option<u32> {
    size.filter(|size| *size > 0).map(|s| s.min(MAX_ICON_SIZE))
}

/// The item's own size if it has one, otherwise the crate-wide default.
pub(crate) fn effective_icon_size(item_size: Option<u32>) -> Option<u32> {
    item_size.or_else(default_menu_icon_size)
}

/// Converts a logical size to physical pixels for a display scale factor.
pub(crate) fn physical_size(logical: u32, scale_factor: f64) -> u32 {
    let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };
    ((logical as f64 * scale_factor).round() as u32).max(1)
}

/// The size an image of `width` x `height` takes when fitted inside a `size` x `size` square
/// without changing its aspect ratio. Neither side is ever zero or larger than `size`.
pub(crate) fn fit_within(width: u32, height: u32, size: u32) -> (u32, u32) {
    if size == 0 {
        return (0, 0);
    }
    if width == 0 || height == 0 {
        return (size, size);
    }

    let scale = size as f64 / width.max(height) as f64;
    let fit = |side: u32| ((side as f64 * scale).round() as u32).clamp(1, size);
    (fit(width), fit(height))
}

/// Premultiplies the colour channels of straight-alpha RGBA pixels in place.
pub(crate) fn premultiply_alpha(rgba: &mut [u8]) {
    for pixel in rgba.chunks_exact_mut(4) {
        let alpha = pixel[3] as u32;
        for channel in &mut pixel[..3] {
            *channel = ((*channel as u32 * alpha + 127) / 255) as u8;
        }
    }
}

/// How each destination sample along one axis is built from source samples: an area average when
/// shrinking, so no source pixel is skipped, and a linear blend when growing.
fn axis_weights(src_len: usize, dst_len: usize) -> Vec<Vec<(usize, f32)>> {
    if src_len == dst_len {
        return (0..dst_len).map(|i| vec![(i, 1.0)]).collect();
    }

    let ratio = src_len as f64 / dst_len as f64;
    (0..dst_len)
        .map(|i| {
            if dst_len > src_len {
                let position = (i as f64 + 0.5) * ratio - 0.5;
                let first = position.floor();
                let fraction = (position - first) as f32;
                let clamp = |index: f64| index.clamp(0.0, (src_len - 1) as f64) as usize;
                vec![
                    (clamp(first), 1.0 - fraction),
                    (clamp(first + 1.0), fraction),
                ]
            } else {
                let start = i as f64 * ratio;
                let end = (i as f64 + 1.0) * ratio;
                (start.floor() as usize..(end.ceil() as usize).min(src_len))
                    .map(|j| {
                        let overlap = end.min(j as f64 + 1.0) - start.max(j as f64);
                        (j, (overlap / ratio) as f32)
                    })
                    .collect()
            }
        })
        .collect()
}

/// Resamples 4-channel 8-bit pixels. The channels are averaged independently, so the input must
/// already be premultiplied for semi-transparent edges to blend without dark or light fringes.
pub(crate) fn resize_premultiplied(
    src: &[u8],
    src_width: u32,
    src_height: u32,
    dst_width: u32,
    dst_height: u32,
) -> Vec<u8> {
    let (sw, sh, dw, dh) = (
        src_width as usize,
        src_height as usize,
        dst_width as usize,
        dst_height as usize,
    );
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 || src.len() < sw * sh * 4 {
        return vec![0; dw * dh * 4];
    }

    let columns = axis_weights(sw, dw);
    let rows = axis_weights(sh, dh);

    let mut horizontal = vec![0f32; dw * sh * 4];
    for y in 0..sh {
        for (x, taps) in columns.iter().enumerate() {
            let out = (y * dw + x) * 4;
            for &(source_x, weight) in taps {
                let from = (y * sw + source_x) * 4;
                for c in 0..4 {
                    horizontal[out + c] += src[from + c] as f32 * weight;
                }
            }
        }
    }

    let mut result = vec![0u8; dw * dh * 4];
    for (y, taps) in rows.iter().enumerate() {
        for x in 0..dw {
            let out = (y * dw + x) * 4;
            for c in 0..4 {
                let sum: f32 = taps
                    .iter()
                    .map(|&(source_y, weight)| horizontal[(source_y * dw + x) * 4 + c] * weight)
                    .sum();
                result[out + c] = sum.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    result
}

/// Builds the pixels of a `size` x `size` menu bitmap from straight-alpha RGBA data: fitted inside
/// the square without distortion, centred on a transparent background, premultiplied, and in
/// BGRA byte order, which is what a Windows 32-bit DIB with alpha expects.
pub(crate) fn menu_bitmap_bgra(rgba: &[u8], width: u32, height: u32, size: u32) -> Vec<u8> {
    let side = size as usize;
    let mut canvas = vec![0u8; side * side * 4];
    if size == 0 || width == 0 || height == 0 || rgba.len() < (width * height * 4) as usize {
        return canvas;
    }

    let mut premultiplied = rgba[..(width * height * 4) as usize].to_vec();
    premultiply_alpha(&mut premultiplied);

    let (fit_w, fit_h) = fit_within(width, height, size);
    let fitted = if (fit_w, fit_h) == (width, height) {
        premultiplied
    } else {
        resize_premultiplied(&premultiplied, width, height, fit_w, fit_h)
    };

    let (offset_x, offset_y) = ((size - fit_w) as usize / 2, (size - fit_h) as usize / 2);
    for y in 0..fit_h as usize {
        let from = y * fit_w as usize * 4;
        let to = ((y + offset_y) * side + offset_x) * 4;
        canvas[to..to + fit_w as usize * 4]
            .copy_from_slice(&fitted[from..from + fit_w as usize * 4]);
    }

    for pixel in canvas.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    canvas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_sizes() {
        assert_eq!(normalize_icon_size(None), None);
        assert_eq!(normalize_icon_size(Some(0)), None);
        assert_eq!(normalize_icon_size(Some(24)), Some(24));
        assert_eq!(normalize_icon_size(Some(100_000)), Some(MAX_ICON_SIZE));
    }

    #[test]
    fn physical_size_rounds_and_never_hits_zero() {
        assert_eq!(physical_size(16, 1.0), 16);
        assert_eq!(physical_size(16, 1.25), 20);
        assert_eq!(physical_size(16, 1.5), 24);
        assert_eq!(physical_size(16, 2.0), 32);
        assert_eq!(physical_size(15, 1.25), 19);
        assert_eq!(physical_size(1, 0.1), 1);
        assert_eq!(physical_size(16, f64::NAN), 16);
        assert_eq!(physical_size(16, -2.0), 16);
    }

    #[test]
    fn fit_within_preserves_aspect_ratio() {
        assert_eq!(fit_within(32, 32, 16), (16, 16));
        assert_eq!(fit_within(64, 32, 16), (16, 8));
        assert_eq!(fit_within(32, 64, 16), (8, 16));
        assert_eq!(fit_within(8, 8, 16), (16, 16));
        assert_eq!(fit_within(1000, 1, 16), (16, 1));
        assert_eq!(fit_within(0, 10, 16), (16, 16));
        assert_eq!(fit_within(10, 10, 0), (0, 0));
    }

    #[test]
    fn premultiply_scales_colour_by_alpha() {
        let mut pixels = [
            255, 128, 0, 255, // opaque: unchanged
            255, 128, 0, 128, // half transparent
            255, 255, 255, 0, // fully transparent: colour is dropped
        ];
        premultiply_alpha(&mut pixels);
        assert_eq!(pixels[..4], [255, 128, 0, 255]);
        assert_eq!(pixels[4..8], [128, 64, 0, 128]);
        assert_eq!(pixels[8..], [0, 0, 0, 0]);
    }

    #[test]
    fn resize_keeps_flat_colour_and_alpha() {
        let src: Vec<u8> = std::iter::repeat_n([40, 80, 120, 200], 8 * 8)
            .flatten()
            .collect();
        for (w, h) in [(4, 4), (16, 16), (5, 3), (8, 8)] {
            let out = resize_premultiplied(&src, 8, 8, w, h);
            assert_eq!(out.len(), (w * h * 4) as usize);
            assert!(out.chunks_exact(4).all(|p| p == [40, 80, 120, 200]));
        }
    }

    #[test]
    fn downscale_averages_every_source_pixel() {
        // 2x1: one black and one white opaque pixel, averaged into one grey pixel.
        let src = [0, 0, 0, 255, 255, 255, 255, 255];
        assert_eq!(resize_premultiplied(&src, 2, 1, 1, 1), [128, 128, 128, 255]);
    }

    #[test]
    fn upscale_interpolates_between_pixels() {
        let src = [0, 0, 0, 255, 255, 255, 255, 255];
        let out = resize_premultiplied(&src, 2, 1, 4, 1);
        let first = out[0];
        let last = out[12];
        assert_eq!((first, last), (0, 255));
        assert!(out[4] > first && out[4] < out[8] && out[8] < last);
    }

    #[test]
    fn semi_transparent_edge_has_no_dark_fringe() {
        // A white pixel next to a fully transparent black one. Averaged straight (not
        // premultiplied) the result would be grey; premultiplied it stays white at half alpha.
        let rgba = [255, 255, 255, 255, 0, 0, 0, 0];
        let bgra = menu_bitmap_bgra(&rgba, 2, 1, 1);
        assert_eq!(bgra, [128, 128, 128, 128]);
        // Premultiplied colour never exceeds alpha.
        assert!(bgra[0] <= bgra[3] && bgra[1] <= bgra[3] && bgra[2] <= bgra[3]);
    }

    #[test]
    fn menu_bitmap_is_square_bgra_and_centred() {
        // A 2x1 image of one red and one green opaque pixel, drawn at size 4: fitted to 4x2 and
        // centred vertically.
        let rgba = [255, 0, 0, 255, 0, 255, 0, 255];
        let bgra = menu_bitmap_bgra(&rgba, 2, 1, 4);
        assert_eq!(bgra.len(), 4 * 4 * 4);

        let pixel = |x: usize, y: usize| &bgra[(y * 4 + x) * 4..(y * 4 + x) * 4 + 4];
        assert_eq!(pixel(0, 0), [0, 0, 0, 0]);
        assert_eq!(pixel(3, 3), [0, 0, 0, 0]);
        // Red in RGBA becomes the last colour byte in BGRA.
        assert_eq!(pixel(0, 1), [0, 0, 255, 255]);
        assert_eq!(pixel(3, 2), [0, 255, 0, 255]);
    }

    #[test]
    fn menu_bitmap_survives_bad_input() {
        assert_eq!(menu_bitmap_bgra(&[], 0, 0, 2), vec![0; 16]);
        assert_eq!(menu_bitmap_bgra(&[1, 2, 3], 1, 1, 2), vec![0; 16]);
        assert!(menu_bitmap_bgra(&[0; 4], 1, 1, 0).is_empty());
    }

    #[test]
    fn default_size_round_trips() {
        assert_eq!(effective_icon_size(Some(24)), Some(24));
        set_default_menu_icon_size(Some(20));
        assert_eq!(default_menu_icon_size(), Some(20));
        assert_eq!(effective_icon_size(None), Some(20));
        assert_eq!(effective_icon_size(Some(24)), Some(24));
        set_default_menu_icon_size(None);
        assert_eq!(default_menu_icon_size(), None);
        assert_eq!(effective_icon_size(None), None);
    }
}
