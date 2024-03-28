#![no_main]
#![no_std]

extern crate alloc;

use uefi::prelude::*;

use uefapi::prelude::*;

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    system_table.boot_services().set_watchdog_timer(0, 0x10000, None).unwrap();
    gfx::init_display_mode(gfx::display_modes().iter()
        .filter(|x| x.info().resolution() == (800, 600))
        .next().unwrap());

    let mut screen = gfx::Buffer::new_screen();
    let mut pbar = gfx::ProgressBar {
        x: 100,
        y: 100,
        width: 200,
        height: 8,
        progress: 0.0,
        bg_color: gfx::Color::gray(0x33),
        fg_color: gfx::Color::gray(0x99),
    };
    for i in 0..=100 {
        pbar.progress = i as f32 / 100.0;
        pbar.render(&mut screen);
        pbar.present(&screen);
        system_table.boot_services().stall(50_000);
    }
    for _ in 0..=100 {
        pbar.render_spinner(&mut screen);
        pbar.present(&screen);
        system_table.boot_services().stall(50_000);
    }

    system_table.boot_services().stall(30_000_000);
    Status::SUCCESS
}
