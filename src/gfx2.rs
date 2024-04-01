use alloc::rc::Rc;
use alloc::vec::Vec;
use core::ops::{Add, Sub};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

pub fn pos(x: i32, y: i32) -> Pos {
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
    pub fn dim(self) -> Dim {
        Dim { w: self.x, h: self.y }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Dim {
    pub w: i32,
    pub h: i32,
}

pub fn dim(w: i32, h: i32) -> Dim {
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
    pub fn pos(self) -> Pos {
        Pos { x: self.w, y: self.h }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Rect {
    pub pos: Pos,
    pub dim: Dim,
}

pub fn rect(pos: Pos, dim: Dim) -> Rect {
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

pub fn area(pos1: Pos, pos2: Pos) -> Area {
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

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color { r, g, b, a }
}

impl Color {
    pub fn with_alpha(self, a: u8) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }
    
    pub fn apply_alpha(self, alpha: u8) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: (self.a as u32 * alpha as u32 / 255) as u8,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Buffer {
    data: Rc<Vec<Color>>,
    dim: Dim,
}