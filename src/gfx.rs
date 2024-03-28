use alloc::{slice, vec};
use alloc::vec::Vec;

use baked_font::{Font, Glyph};
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
        self.present_partial(0, 0, res.0, res.1);
    }

    pub fn present_partial(&self, x: usize, y: usize, width: usize, height: usize) {
        let st = uefi_services::system_table();
        let gop_handle = st.boot_services()
            .get_handle_for_protocol::<GraphicsOutput>().unwrap();
        let mut gop = st.boot_services()
            .open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
        gop.blt(BltOp::BufferToVideo {
            buffer: unsafe {
                slice::from_raw_parts(self.data.as_ptr() as *const BltPixel, self.data.len())
            },
            src: BltRegion::SubRectangle {
                coords: (x, y),
                px_stride: self.width,
            },
            dest: (x, y),
            dims: (width, height),
        }).unwrap()
    }

    pub fn pos_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width as i32 && y < self.data.len() as i32 / self.width as i32
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
                if !self.pos_in_bounds(tar_x, tar_y) {
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
    let gop = st.boot_services()
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

pub struct ProgressBar {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub progress: f32,
    pub bg_color: Color,
    pub fg_color: Color,
}

impl ProgressBar {
    pub fn render(&self, buffer: &mut Buffer) {
        let progress_width = (self.width as f32 * self.progress) as i32;
        for y in 0..self.height {
            for x in 0..self.width {
                let color = if x < progress_width {
                    self.fg_color
                } else {
                    self.bg_color
                };
                let idx = ((self.y + y) as usize * buffer.width) + (self.x + x) as usize;
                buffer.data[idx] = color;
            }
        }
    }

    pub fn render_spinner(&mut self, buffer: &mut Buffer) {
        let deg = if self.progress >= 0.5 {
            1.0 - self.progress
        } else {
            self.progress
        } * 2.0;
        let color = self.bg_color.apply(self.fg_color, (deg * 255.0) as u8);
        let bh = buffer.data.len() as i32 / buffer.width as i32;
        for y in self.y..(self.y + self.height) {
            if y < 0 || y >= bh { continue; }
            let begin = (y as usize * buffer.width) + self.x as usize;
            buffer.data[begin..(begin + self.width as usize)].fill(color);
        }
        if self.progress >= 0.98 {
            self.progress = 0.0;
        } else {
            self.progress += 0.02;
        }
    }

    pub fn present(&self, buffer: &Buffer) {
        buffer.present_partial(self.x as usize, self.y as usize,
                               self.width as usize, self.height as usize);
    }
}