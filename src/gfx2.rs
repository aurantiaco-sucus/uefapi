use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use core::ops::{Add, Sub};
use core::slice;
use baked_font::{Font, Glyph};
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput, Mode};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

pub const fn pos(x: i32, y: i32) -> Pos {
    Pos { x, y }
}

impl From<(i32, i32)> for Pos {
    fn from((x, y): (i32, i32)) -> Self {
        Self { x, y }
    }
}

impl Add for Pos {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Pos {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Pos {
    pub const fn dim(self) -> Dim {
        Dim { w: self.x, h: self.y }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Dim {
    pub w: i32,
    pub h: i32,
}

pub const fn dim(w: i32, h: i32) -> Dim {
    Dim { w, h }
}

impl From<(i32, i32)> for Dim {
    fn from((w, h): (i32, i32)) -> Self {
        Self { w, h }
    }
}

impl Add for Dim {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            w: self.w + other.w,
            h: self.h + other.h,
        }
    }
}

impl Sub for Dim {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            w: self.w - other.w,
            h: self.h - other.h,
        }
    }
}

impl Dim {
    pub const fn pos(self) -> Pos {
        Pos { x: self.w, y: self.h }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Rect {
    pub pos: Pos,
    pub dim: Dim,
}

pub const fn rect(pos: Pos, dim: Dim) -> Rect {
    Rect { pos, dim }
}

impl Rect {
    pub fn area(self) -> Area {
        Area {
            pos1: self.pos,
            pos2: self.pos + self.dim.pos(),
        }
    }
    
    pub fn normalize(self) -> Self {
        Self {
            pos: Pos {
                x: self.pos.x.min(self.pos.x + self.dim.w),
                y: self.pos.y.min(self.pos.y + self.dim.h),
            },
            dim: Dim {
                w: self.dim.w.abs(),
                h: self.dim.h.abs(),
            },
        }
    }
    
    pub fn translate(self, pos: Pos) -> Self {
        Self {
            pos: self.pos + pos,
            dim: self.dim,
        }
    }
    
    pub fn resize(self, dim: Dim) -> Self {
        Self {
            pos: self.pos,
            dim,
        }
    }
    
    pub fn relocate(self, pos: Pos) -> Self {
        Self {
            pos,
            dim: self.dim,
        }
    }
    
    pub fn contains(self, pos: Pos) -> bool {
        pos.x >= self.pos.x && pos.x < self.pos.x + self.dim.w &&
        pos.y >= self.pos.y && pos.y < self.pos.y + self.dim.h
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Area {
    pub pos1: Pos,
    pub pos2: Pos,
}

pub const fn area(pos1: Pos, pos2: Pos) -> Area {
    Area { pos1, pos2 }
}

impl Area {
    pub fn rect(self) -> Rect {
        Rect {
            pos: self.pos1,
            dim: (self.pos2 - self.pos1).dim(),
        }
    }
    
    pub fn normalize(self) -> Self {
        Self {
            pos1: Pos {
                x: self.pos1.x.min(self.pos2.x),
                y: self.pos1.y.min(self.pos2.y),
            },
            pos2: Pos {
                x: self.pos1.x.max(self.pos2.x),
                y: self.pos1.y.max(self.pos2.y),
            },
        }
    }
    
    pub fn map(self, f1: impl Fn(Pos) -> Pos, f2: impl Fn(Pos) -> Pos) -> Self {
        Self {
            pos1: f1(self.pos1),
            pos2: f2(self.pos2),
        }
    }
    
    pub fn map_all(self, f: impl Fn(Pos) -> Pos) -> Self {
        self.map(|x| f(x), |x| f(x))
    }
    
    pub fn intersection(self, other: Self) -> Option<Self> {
        let area = Self {
            pos1: Pos {
                x: self.pos1.x.max(other.pos1.x),
                y: self.pos1.y.max(other.pos1.y),
            },
            pos2: Pos {
                x: self.pos2.x.min(other.pos2.x),
                y: self.pos2.y.min(other.pos2.y),
            },
        };
        if area.pos1.x < area.pos2.x && area.pos1.y < area.pos2.y {
            Some(area)
        } else {
            None
        }
    }
}

#[repr(packed)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Color {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8
}

pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    let r = r as u32 * a as u32 / 255;
    let g = g as u32 * a as u32 / 255;
    let b = b as u32 * a as u32 / 255;
    Color {
        r: r as u8,
        g: g as u8,
        b: b as u8,
        a,
    }
}

pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    rgba(r, g, b, 255)
}

impl Color {
    pub const fn black_alpha(a: u8) -> Self {
        rgba(0, 0, 0, a)
    }
    
    pub const fn white_alpha(a: u8) -> Self {
        rgba(255, 255, 255, a)
    }
    
    pub const BLACK: Self = rgb(0, 0, 0);
    pub const WHITE: Self = rgb(255, 255, 255);
    pub const RED: Self = rgb(255, 0, 0);
    pub const GREEN: Self = rgb(0, 255, 0);
    pub const BLUE: Self = rgb(0, 0, 255);
}

impl Color {
    #[inline]
    pub fn apply_alpha(self, alpha: u8) -> Self {
        let oa = self.a as u32;
        let na = self.a as u32 * alpha as u32 / 255;
        let r = self.r as u32 * na / oa;
        let g = self.g as u32 * na / oa;
        let b = self.b as u32 * na / oa;
        Color {
            r: r as u8,
            g: g as u8,
            b: b as u8,
            a: na as u8,
        }
    }

    #[inline]
    pub fn premultiplied_over(self, other: Self) -> Self {
        Self {
            r: premultiplied_over_ch(self.r, other.r, other.a),
            g: premultiplied_over_ch(self.g, other.g, other.a),
            b: premultiplied_over_ch(self.b, other.b, other.a),
            a: premultiplied_over_ch(self.a, other.a, other.a),
        }
    }
    
    #[inline]
    pub fn additive_over(self, other: Self) -> Self {
        Self {
            r: self.r.saturating_add(other.r),
            g: self.g.saturating_add(other.g),
            b: self.b.saturating_add(other.b),
            a: self.a.saturating_add(other.a),
        }
    }
}

#[inline]
fn premultiplied_over_ch(bg: u8, fg: u8, fg_alpha: u8) -> u8 {
    ((fg as u32 + bg as u32 * (255 - fg_alpha as u32)) / 255) as u8
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Buffer {
    pub data: Vec<Color>,
    pub dim: Dim,
}

impl Buffer {
    pub fn new(dim: Dim) -> Self {
        Self {
            data: vec![Color::BLACK; (dim.w * dim.h) as usize],
            dim,
        }
    }
    
    pub fn new_cleared(dim: Dim, color: Color) -> Self {
        Self {
            data: vec![color; (dim.w * dim.h) as usize],
            dim,
        }
    }
    
    pub fn rect(&self) -> Rect {
        rect(pos(0, 0), self.dim)
    }
    
    pub fn area(&self) -> Area { 
        area(pos(0, 0), self.dim.pos())
    }
    
    pub fn clear(&mut self, color: Color) {
        for pixel in self.data.iter_mut() {
            *pixel = color;
        }
    }
    
    pub fn area_apply(
        &self, other_bounds: Area, other_area: Area, pos: Pos
    ) -> Option<(Area, Pos)> {
        let other_area = other_area.intersection(other_bounds);
        let other_area = if let Some(other_area) = other_area {
            other_area
        } else {
            return None;
        };
        let dst_area = rect(pos, other_area.rect().dim).area()
            .intersection(self.area());
        let dst_area = if let Some(dst_area) = dst_area {
            dst_area
        } else {
            return None;
        };
        Some((other_area, dst_area.pos1))
    }
    
    pub fn apply(
        &mut self, src: &Buffer, src_area: Area, dst_pos: Pos, op: impl FnMut(&mut Color, Color)
    ) {
        let (src_area, dst_pos) = if let Some(x) = 
            self.area_apply(src.area(), src_area, dst_pos) { x } else { return; };
        self.apply_unchecked(src, src_area, dst_pos, op)
    }
    
    pub fn apply_unchecked(
        &mut self, src: &Buffer, src_area: Area, dst_pos: Pos, mut op: impl FnMut(&mut Color, Color)
    ) {
        let dim = src_area.rect().dim;
        let src_pos = src_area.pos1;
        for y in 0..dim.h {
            for x in 0..dim.w {
                let src_pos = src_pos + pos(x, y);
                let dst_pos = dst_pos + pos(x, y);
                let src_idx = src_pos.y as usize * src.dim.w as usize + src_pos.x as usize;
                let dst_idx = dst_pos.y as usize * self.dim.w as usize + dst_pos.x as usize;
                op(&mut self.data[dst_idx], src.data[src_idx]);
            }
        }
    }
    
    pub fn premultiplied_over(&mut self, src: &Buffer, src_area: Area, dst_pos: Pos) {
        self.apply(src, src_area, dst_pos, |dst, src| {
            *dst = dst.premultiplied_over(src);
        });
    }
    
    pub fn additive_over(&mut self, src: &Buffer, src_area: Area, dst_pos: Pos) {
        self.apply(src, src_area, dst_pos, |dst, src| {
            *dst = dst.additive_over(src);
        });
    }
}

static mut SCREEN: Buffer = Buffer {
    data: Vec::new(),
    dim: dim(0, 0),
};

pub struct Screen {}

impl Screen {
    pub fn init() {
        let st = uefi_services::system_table();
        let gop_handle = st.boot_services()
            .get_handle_for_protocol::<GraphicsOutput>().unwrap();
        let gop = st.boot_services()
            .open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
        let res = gop.current_mode_info().resolution();
        unsafe { SCREEN = Buffer::new(dim(res.0 as i32, res.1 as i32)); }
    }

    pub fn modes() -> Vec<Mode> {
        let st = uefi_services::system_table();
        let gop_handle = st.boot_services()
            .get_handle_for_protocol::<GraphicsOutput>().unwrap();
        let gop = st.boot_services()
            .open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
        gop.modes(st.boot_services()).collect()
    }

    pub fn init_mode(mode: &Mode) {
        let st = uefi_services::system_table();
        let gop_handle = st.boot_services()
            .get_handle_for_protocol::<GraphicsOutput>().unwrap();
        let mut gop = st.boot_services()
            .open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
        gop.set_mode(&mode).unwrap();
        drop(gop);
        Self::init();
    }

    pub fn get() -> &'static mut Buffer {
        debug_assert!(unsafe { SCREEN.dim.w != 0 });
        #[allow(static_mut_refs)]
        unsafe { &mut SCREEN }
    }

    pub fn rect() -> Rect {
        rect(pos(0, 0), Self::get().dim)
    }

    pub fn present(rect: Rect) {
        let screen = Self::get();
        let st= uefi_services::system_table();
        let gop_handle = st.boot_services()
            .get_handle_for_protocol::<GraphicsOutput>().unwrap();
        let mut gop = st.boot_services()
            .open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
        let buffer = unsafe {
            slice::from_raw_parts(screen.data.as_ptr() as *const BltPixel, screen.data.len())
        };
        let coord = (rect.pos.x as usize, rect.pos.y as usize);
        gop.blt(BltOp::BufferToVideo {
            buffer,
            src: BltRegion::SubRectangle {
                coords: coord,
                px_stride: screen.dim.w as usize,
            },
            dest: coord,
            dims: (rect.dim.w as usize, rect.dim.h as usize),
        }).unwrap();
    }
}

pub struct StraightGlyphIterator<'a, T: Iterator<Item=Glyph>> {
    font: &'a Font,
    iter: T,
    off: Pos,
}

impl<'a, T: Iterator<Item=Glyph>> StraightGlyphIterator<'a, T> {
    pub fn new(font: &'a Font, iter: T) -> Self {
        Self { font, iter, off: pos(0, 0) }
    }
}

pub trait FontExt {
    fn straight_glyphs<'a, T: Iterator<Item=Glyph>>(&'a self, iter: T)
        -> StraightGlyphIterator<'a, T>;
}

impl FontExt for Font {
    fn straight_glyphs<T: Iterator<Item=Glyph>>(&self, iter: T) -> StraightGlyphIterator<T> {
        StraightGlyphIterator::new(self, iter)
    }
}

impl<'a, T: Iterator<Item=Glyph>> Iterator for StraightGlyphIterator<'a, T> {
    type Item = (Glyph, Rect, Pos);
    
    fn next(&mut self) -> Option<Self::Item> {
        let glyph = self.iter.next()?;
        let fg_pos = pos(glyph.pos.0 as i32, glyph.pos.1 as i32);
        let fg_dim = dim(glyph.size.0 as i32, glyph.size.0 as i32);
        let fg_rect = rect(fg_pos, fg_dim);
        let c_off = self.off;
        self.off.x += fg_dim.w;
        Some((glyph, fg_rect, c_off))
    }
}

pub struct LineWrapGlyphIterator<'a, T: Iterator<Item=Glyph>> {
    iter: StraightGlyphIterator<'a, T>,
    width: i32,
    height: i32,
}

impl<'a, T: Iterator<Item=Glyph>> StraightGlyphIterator<'a, T> {
    pub fn line_wrap(self, width: i32, height: i32) -> LineWrapGlyphIterator<'a, T> {
        LineWrapGlyphIterator { iter: self, width, height }
    }
}

impl<'a, T: Iterator<Item=Glyph>> Iterator for LineWrapGlyphIterator<'a, T> {
    type Item = (Glyph, Rect, Pos);
    
    fn next(&mut self) -> Option<Self::Item> {
        let (glyph, fg_rect, mut c_off) = self.iter.next()?;
        let fg_dim = fg_rect.dim;
        if c_off.x + fg_dim.w > self.width {
            self.iter.off.x = fg_dim.w;
            self.iter.off.y += self.height;
            c_off.x = 0;
        }
        Some((glyph, fg_rect, c_off))
    }
}

pub struct AreaPosIter {
    area: Area,
    pos: Pos
}

impl Area {
    pub fn pos_iter(&self) -> AreaPosIter {
        AreaPosIter {
            area: *self,
            pos: self.pos1
        }
    }
}

impl Iterator for AreaPosIter {
    type Item = Pos;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos.y >= self.area.pos2.y {
            return None;
        }
        let pos = self.pos;
        self.pos.x += 1;
        if self.pos.x >= self.area.pos2.x {
            self.pos.x = self.area.pos1.x;
            self.pos.y += 1;
        }
        Some(pos)
    }
}

impl Buffer {
    pub fn draw_glyph(&mut self, loc: Pos, font: &Font, glyph: Glyph, color: Color) {
        let glyph_off = pos(glyph.pos.0 as i32, glyph.pos.1 as i32) - loc;
        let glyph_dim = dim(glyph.size.0 as i32, glyph.size.1 as i32);
        let area = self.area().intersection(rect(loc, glyph_dim).area());
        let area = if let Some (x) = area { x } else { return; };
        for loc in area.pos_iter() {
            let glyph_loc = glyph_off + loc;
            let alpha = font.bitmap[
                glyph_loc.x as usize + glyph_loc.y as usize * font.width as usize];
            let color = color.apply_alpha(alpha);
            let px = &mut self.data[
                loc.x as usize + loc.y as usize * self.dim.w as usize];
            *px = px.premultiplied_over(color);
        }
    }
}