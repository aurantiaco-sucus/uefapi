#![no_main]
#![no_std]

extern crate alloc;

use log::info;
use uefi::prelude::*;
use uefi::table::boot::SearchType;
use uefapi::prelude::*;

const FONT_DATA: &[u8] = include_bytes!("../../baked-font-generator/font.bin");

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    system_table.boot_services().set_watchdog_timer(0, 0x10000, None).unwrap();
    gfx::init_display_mode(gfx::display_modes().iter()
        .filter(|x| x.info().resolution() == (800, 600))
        .next().unwrap());

    let font: baked_font::Font = postcard::from_bytes(FONT_DATA).unwrap();
    let mut screen = gfx::Buffer::new_screen();
    
    let buffer = system_table.boot_services()
        .locate_handle_buffer(SearchType::AllHandles).unwrap();
    info!("Handle count: {}", buffer.len());
    
    screen.present();

    system_table.boot_services().stall(30_000_000);
    Status::SUCCESS
}