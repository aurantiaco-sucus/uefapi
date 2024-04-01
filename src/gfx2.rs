use alloc::vec;
use alloc::vec::Vec;
use core::ops::{Add, Sub};

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
    
    pub fn map(self, f1: impl Fn(Pos) -> Pos, f2: impl Fn(Pos) -> Pos) -> Self {
        Self {
            pos1: f1(self.pos1),
            pos2: f2(self.pos2),
        }
    }
    
    pub fn map_all(self, f: impl Fn(Pos) -> Pos) -> Self {
        self.map(|x| f(x), |x| f(x))
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
    
    pub fn clear(&mut self, color: Color) {
        for pixel in self.data.iter_mut() {
            *pixel = color;
        }
    }
}