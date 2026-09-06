// use std::mem;

use std::mem;

use peniko::color::PremulRgba8;
use softbuffer::Pixel;

use crate::imaging_vello_cpu::RendererError;

#[inline]
#[allow(
    clippy::cast_possible_truncation,
    reason = "Backend API consumes f32 blur parameters; truncation from finite f64 IR values is acceptable."
)]
pub fn f64_to_f32(v: f64) -> f32 {
    v as f32
}

pub fn checked_size(width: u32, height: u32) -> Result<(u16, u16), RendererError> {
    let width =
        u16::try_from(width).map_err(|_| RendererError::Internal("render width too large"))?;
    let height =
        u16::try_from(height).map_err(|_| RendererError::Internal("render height too large"))?;
    Ok((width, height))
}

pub fn unpremultiply_rgba8_in_place(bytes: &mut [u8]) {
    for rgba in bytes.as_chunks_mut::<4>().0 {
        let alpha = rgba[3];
        if alpha == 0 || alpha == u8::MAX {
            continue;
        }
        rgba[0] = unpremultiply_channel(rgba[0], alpha);
        rgba[1] = unpremultiply_channel(rgba[1], alpha);
        rgba[2] = unpremultiply_channel(rgba[2], alpha);
    }
}

pub fn unpremultiply_channel(channel: u8, alpha: u8) -> u8 {
    if alpha == 0 {
        return 0;
    }
    if alpha == u8::MAX {
        return channel;
    }

    let value = (u32::from(channel) * 255 + u32::from(alpha) / 2) / u32::from(alpha);
    u8::try_from(value.min(u32::from(u8::MAX))).expect("unpremultiplied channel must fit in u8")
}

// #[inline(always)]
// pub fn swap_blue_and_red_channel<S: Simd>(_: S, pixels: &mut [u8]) {
//     for [r, _, b, _] in pixels.as_chunks_mut::<4>().0 {
//         mem::swap(r, b);
//     }
// }

pub fn write_to_buffer<'a>(
    width: usize,
    pixel_in: &mut [PremulRgba8],
    pixels_out: impl DoubleEndedIterator<Item = (u32, u32, &'a mut Pixel)>,
) {
    for (x, y, pixel_mut) in pixels_out {
        let idx = width * y as usize + x as usize;
        if let Some(pix) = pixel_in.get_mut(idx) {
            mem::swap(&mut pixel_mut.a, &mut pix.a);
            mem::swap(&mut pixel_mut.b, &mut pix.b);
            mem::swap(&mut pixel_mut.g, &mut pix.g);
            mem::swap(&mut pixel_mut.r, &mut pix.r);
        } else {
            *pixel_mut = Pixel::default()
        }
    }
}
