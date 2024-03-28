use alloc::{slice, vec};
use alloc::vec::Vec;
use core::mem;
use core::ops::Add;
use core::ptr::slice_from_raw_parts;

use baked_font::{Font, Glyph};
use log::info;
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput, Mode};

static mut SCREEN_RESOLUTION: (usize, usize) = (0, 0);

pub fn screen_resolution() -> (usize, usize) {
    unsafe { SCREEN_RESOLUTION }
}

pub fn init() {
    let st = uefi_services::system_table();
    let gop_handle = st.boot_services()
        .get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let gop = st.boot_services()
        .open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
    unsafe { SCREEN_RESOLUTION = gop.current_mode_info().resolution(); }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Buffer {
    pub data: Vec<Color>,
    pub width: usize,
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![Color::default(); width * height],
            width,
        }
    }

    pub fn new_screen() -> Self {
        let res = screen_resolution();
        Self::new(res.0, res.1)
    }

    pub fn present(&self) {
        let res = screen_resolution();
        let st = uefi_services::system_table();
        let gop_handle = st.boot_services()
            .get_handle_for_protocol::<GraphicsOutput>().unwrap();
        let mut gop = st.boot_services()
            .open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
        gop.blt(BltOp::BufferToVideo {
            buffer: unsafe { 
                slice::from_raw_parts(self.data.as_ptr() as *const BltPixel, self.data.len()) 
            },
            src: BltRegion::Full,
            dest: (0, 0),
            dims: (res.0, res.1),
        }).unwrap()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Color {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    _reserved: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, _reserved: 0 }
    }
    
    pub const fn gray(value: u8) -> Self {
        Self::rgb(value, value, value)
    }

    #[inline]
    pub const fn alpha(self, alpha: u8) -> Self {
        Self {
            r: apply_alpha_on_u8(self.r, alpha),
            g: apply_alpha_on_u8(self.g, alpha),
            b: apply_alpha_on_u8(self.b, alpha),
            _reserved: 0,
        }
    }

    #[inline]
    pub const fn add_clamped(self, other: Color) -> Self {
        Self {
            r: self.r.saturating_add(other.r),
            g: self.g.saturating_add(other.g),
            b: self.b.saturating_add(other.b),
            _reserved: 0,
        }
    }

    #[inline]
    pub const fn apply(self, other: Color, alpha: u8) -> Self {
        let cur = self.alpha(255 - alpha);
        let other = other.alpha(alpha);
        cur.add_clamped(other)
    }
    
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
}

#[inline]
const fn apply_alpha_on_u8(src: u8, alpha: u8) -> u8 {
    ((src as u16) * (alpha as u16) / 255) as u8
}

impl Buffer {
    pub fn render_text(&mut self, x: i32, y: i32, text: &str, font: &Font, color: Color) {
        let mut off = 0;
        let mut pos = 0;
        let chars = text.chars().collect::<Vec<_>>();
        while pos < chars.len() {
            if let Some((glyph, is_pair)) = font.lookup(&chars, pos) {
                self.render_glyph(x + off, y, font, glyph, color);
                off += glyph.size.0 as i32;
                pos += if is_pair { 2 } else { 1 };
            } else {
                pos += 1;
            }
        }
    }
    
    pub fn render_text_multiline(
        &mut self, x: i32, y: i32, width: i32, line_height: i32, 
        text: &str, font: &Font, color: Color
    ) {
        let mut x_off = 0;
        let mut y_off = 0;
        let mut pos = 0;
        let chars = text.chars().collect::<Vec<_>>();
        while pos < chars.len() {
            if let Some((glyph, is_pair)) = font.lookup(&chars, pos) {
                if x_off + glyph.size.0 as i32 > width {
                    x_off = 0;
                    y_off += line_height;
                }
                self.render_glyph(x + x_off, y + y_off, font, glyph, color);
                x_off += glyph.size.0 as i32;
                pos += if is_pair { 2 } else { 1 };
            } else {
                if chars[pos] == '\n' {
                    x_off = 0;
                    y_off += line_height;
                }
                pos += 1;
            }
        }
    }

    #[inline]
    pub fn render_glyph(&mut self, x: i32, y: i32, font: &Font, glyph: Glyph, color: Color) {
        let bmp_ori_x = glyph.pos.0 as i32;
        let bmp_ori_y = glyph.pos.1 as i32;
        let bmp_height = (font.bitmap.len() / font.width as usize) as i32;
        for glyph_y in 0..glyph.size.1 {
            for glyph_x in 0..glyph.size.0 {
                let bmp_x = bmp_ori_x as usize + glyph_x as usize;
                let bmp_y = bmp_ori_y as usize + glyph_y as usize;
                let bmp_idx = bmp_y * font.width as usize + bmp_x;
                let alpha = font.bitmap[bmp_idx];
                if alpha == 0 {
                    continue;
                }
                let tar_x = x + glyph_x as i32 + glyph.offset.0 as i32;
                let tar_y = y + glyph_y as i32 + glyph.offset.1 as i32;
                if tar_x < 0 || tar_y < 0 || tar_x >= self.width as i32 || tar_y >= bmp_height {
                    continue;
                }
                let tar_idx = (tar_y as usize * self.width) + tar_x as usize;
                self.data[tar_idx] = self.data[tar_idx].apply(color, alpha);
            }
        }
    }
}

pub fn display_modes() -> Vec<Mode> {
    let st = uefi_services::system_table();
    let gop_handle = st.boot_services()
        .get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop = st.boot_services()
        .open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
    gop.modes(st.boot_services()).collect()
}

pub fn init_display_mode(mode: &Mode) {
    let st = uefi_services::system_table();
    let gop_handle = st.boot_services()
        .get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop = st.boot_services()
        .open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
    gop.set_mode(&mode).unwrap();
    drop(gop);
    init();
}