#![no_std]
extern crate alloc;

pub mod gfx;
pub mod gfx2;

pub mod prelude {
    pub use crate::gfx;
}

pub mod prelude_dev {
    pub use crate::gfx2 as gfx;
}