#![no_main]
#![no_std]

extern crate alloc;

use uefi::prelude::*;

use uefapi::prelude::*;

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    system_table.boot_services().set_watchdog_timer(0, 0x10000, None).unwrap();
    gfx::Screen::init_mode(gfx::Screen::modes().iter()
        .filter(|x| x.info().resolution() == (800, 600))
        .next().unwrap());

    gfx::Screen::get().clear(gfx::Color::BLACK);

    let mut pb = gfx::ProgressBar {
        area: gfx::rect(gfx::pos(100, 100), gfx::dim(400, 20)).area(),
        progress: 0.0,
        fg: gfx::gray(0xD0),
        bg: gfx::gray(0x60),
    };

    for _ in 0..50 {
        pb.draw_normal(gfx::Screen::get());
        pb.progress += 0.02;
        gfx::Screen::present(pb.area.rect());
        system_table.boot_services().stall(100_000);
    }
    for _ in 0..2000 {
        pb.draw_marquee(gfx::Screen::get());
        pb.progress += 0.02;
        if pb.progress > 1.0 {
            pb.progress -= 1.0;
        }
        gfx::Screen::present(pb.area.rect());
        system_table.boot_services().stall(50_000);
    }

    gfx::Screen::present(gfx::Screen::rect());

    system_table.boot_services().stall(30_000_000);
    Status::SUCCESS
}
