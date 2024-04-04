#![no_main]
#![no_std]

extern crate alloc;

use uefi::prelude::*;
use uefi::proto::media::disk::DiskIo;
use uefi::table::boot::SearchType;
use uefi_services::println;
use uefapi::gfx2::{Color, GlyphCoordIteratorExt, GlyphIteratorExt};
use uefapi::prelude_dev::*;

const FONT_DATA: &[u8] = include_bytes!("../../baked-font-generator/font.bin");
const SOME_LONG_TEXT: &str = include_str!("some_long_text.txt");

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    system_table.boot_services().set_watchdog_timer(0, 0x10000, None).unwrap();
    gfx::Screen::init_mode(gfx::Screen::modes().iter()
        .filter(|x| x.info().resolution() == (800, 600))
        .next().unwrap());

    let font: baked_font::Font = postcard::from_bytes(FONT_DATA).unwrap();

    let buffer = system_table.boot_services()
        .locate_handle_buffer(SearchType::from_proto::<DiskIo>()).unwrap();
    
    gfx::Screen::get().clear(Color::RED);
    gfx::Screen::present(gfx::Screen::rect());
    
    font.lookup_string(SOME_LONG_TEXT)
        .zip(SOME_LONG_TEXT.chars())
        .map(|(x, c)| {
            println!("{c}");
            x
        })
        .glyph_coords()
        .line_wrap(780, 18)
        .draw_each(gfx::Screen::get(), gfx::pos(10, 10), &font, gfx::Color::WHITE);
    
    gfx::Screen::present(gfx::Screen::rect());

    system_table.boot_services().stall(30_000_000);
    Status::SUCCESS
}
