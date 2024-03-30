use alloc::{slice, vec};
use alloc::vec::Vec;

use baked_font::{Font, Glyph};
use log::{info, warn};
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
    
    pub const fn height(&self) -> usize {
        self.data.len() / self.width
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

pub struct GlyphStraightIterator<'a, 'b> {
    font: &'a Font,
    chars: &'b [char],
    pos: usize,
    offset: i32,
}

impl<'a, 'b> GlyphStraightIterator<'a, 'b> {
    fn from_font_chars(font: &'a Font, chars: &'b [char]) -> Self {
        Self { font, chars, pos: 0, offset: 0 }
    }
}

impl<'a, 'b> Iterator for GlyphStraightIterator<'a, 'b> {
    type Item = (Glyph, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.chars.len() {
            return None;
        }
        let val = self.font.lookup(&self.chars, self.pos)
            .map(|(glyph, is_pair)| {
                self.pos += if is_pair { 2 } else { 1 };
                let x_offset = self.offset;
                self.offset += glyph.size.0 as i32;
                (glyph, x_offset)
            });
        if val.is_none() {
            warn!("Glyph not found: {}", self.chars[self.pos]);
            self.pos += 1;
            return self.next();
        }
        val
    }
}

pub struct GlyphWrappedIterator<'a, 'b> {
    font: &'a Font,
    chars: &'b [char],
    pos: usize,
    offset: i32,
    line_width: i32,
    line_height: i32,
    y_offset: i32,
}

impl<'a, 'b> Iterator for GlyphWrappedIterator<'a, 'b> {
    type Item = (Glyph, i32, i32);
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.chars.len() {
            return None;
        }
        if self.chars[self.pos] == '\n' {
            self.pos += 1;
            self.offset = 0;
            self.y_offset += self.line_height;
            return self.next();
        }
        let val = self.font.lookup(&self.chars, self.pos)
            .map(|(glyph, is_pair)| {
                self.pos += if is_pair { 2 } else { 1 };
                if self.offset + glyph.size.0 as i32 > self.line_width {
                    self.offset = 0;
                    self.y_offset += self.line_height;
                }
                let x_offset = self.offset;
                self.offset += glyph.size.0 as i32;
                (glyph, x_offset, self.y_offset)
            });
        if val.is_none() {
            warn!("Glyph not found: {}", self.chars[self.pos]);
            self.pos += 1;
            return self.next();
        }
        val
    }
}

pub trait FontExt {
    fn straight_iter<'a, 'b>(&'a self, chars: &'b [char]) -> GlyphStraightIterator<'a, 'b>;
    fn wrapped_iter<'a, 'b>(
        &'a self, chars: &'b [char], line_width: i32, line_height: i32
    ) -> GlyphWrappedIterator<'a, 'b>;
    fn wrapped_height(&self, string: &str, line_width: i32, line_height: i32) -> i32;
}

impl FontExt for Font {
    fn straight_iter<'a, 'b>(&'a self, chars: &'b [char]) -> GlyphStraightIterator<'a, 'b> {
        GlyphStraightIterator::from_font_chars(self, chars)
    }
    
    fn wrapped_iter<'a, 'b>(
        &'a self, chars: &'b [char], line_width: i32, line_height: i32
    ) -> GlyphWrappedIterator<'a, 'b> {
        GlyphWrappedIterator {
            font: self, chars, pos: 0, offset: 0,
            line_width, line_height, y_offset: 0,
        }
    }

    fn wrapped_height(&self, string: &str, line_width: i32, line_height: i32) -> i32 {
        let chars = string.chars().collect::<Vec<_>>();
        self.wrapped_iter(&chars, line_width, line_height)
            .map(|(_, _, y_off)| y_off)
            .last().unwrap() + line_height
    }
}

#[inline]
const fn apply_alpha_on_u8(src: u8, alpha: u8) -> u8 {
    ((src as u16) * (alpha as u16) / 255) as u8
}

impl Buffer {
    pub fn render_text_straight(
        &mut self, text: &str, x: i32, y: i32, font: &Font, color: Color
    ) {
        let chars = text.chars().collect::<Vec<_>>();
        for (glyph, x_off) in font.straight_iter(&chars) {
            self.render_glyph(x + x_off, y, font, glyph, color);
        }
    }
    
    pub fn render_text_wrapped(
        &mut self, text: &str, x: i32, y: i32, 
        line_width: i32, line_height: i32, font: &Font, color: Color
    ) {
        let chars = text.chars().collect::<Vec<_>>();
        let iter = font.wrapped_iter(&chars, line_width, line_height);
        for (glyph, x_off, y_off) in iter {
            self.render_glyph(x + x_off, y + y_off, font, glyph, color);
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

pub struct ScrollingScreen {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub buf: Buffer,
    pub pos: i32,
}

impl Buffer {
    pub fn into_scrolling_screen(self) -> ScrollingScreen {
        ScrollingScreen {
            x: 0,
            y: 0,
            width: self.width as i32,
            height: (self.data.len() / self.width) as i32,
            buf: self,
            pos: 0
        }
    }
}

impl ScrollingScreen {
    pub fn render(&self, buffer: &mut Buffer) {
        if self.x >= buffer.width as i32 || 
            self.y >= buffer.height() as i32 || 
            self.x + self.width <= 0 || 
            self.y + self.height <= 0  { return; }
        let clip_x_begin = if self.x < 0 { -self.x } else { 0 };
        let clip_y_begin = if self.y < 0 { -self.y } else { 0 };
        let clip_x_end = if self.x + self.width > buffer.width as i32 { buffer.width as i32 - self.x };
        for dy in 0..self.height {
            let src_y = self.pos + dy;
            if src_y >= self.buf.height() as i32 { continue; }
            let 
        }
    }
    
    pub fn present(&self, buffer: &Buffer) {
        buffer.present();
    }
}

impl Buffer {
    pub fn from_text(
        text: &str, line_width: i32, line_height: i32, font: &Font, color: Color
    ) -> Self {
        let height = font.wrapped_height(text, line_width, line_height);
        let mut buffer = Self::new(line_width as usize, height as usize);
        buffer.render_text_wrapped(
            text, line_width, line_height, 0, 0, font, color);
        buffer
    }
}

pub struct ScrollingScreenController {
    
}