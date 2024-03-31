use alloc::rc::Rc;
use alloc::vec::Vec;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Dim {
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Rect {
    pub pos: Pos,
    pub dim: Dim,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Area {
    pub pos1: Pos,
    pub pos2: Pos,
}

#[repr(packed)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Color {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Buffer {
    data: Rc<Vec<Color>>,
    dim: Dim,
}