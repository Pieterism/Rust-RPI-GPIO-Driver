mod utils;

#[macro_use]
extern crate simple_error;
extern crate libc;
extern crate time;
extern crate ctrlc;
extern crate shuteye;
extern crate mmap;
extern crate nix;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::fs::OpenOptions;
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use std::time::Duration;
use shuteye::sleep;
use mmap::{MemoryMap, MapOption};
use utils::file_reader;
use utils::image::Image;
use utils::pixel::Pixel;
use utils::frame::Frame;
use utils::gpio_driver::GPIO;
use utils::gpio_driver::mmap_bcm_register;
use utils::time::Timer;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    sanity_check(&args);

    let interrupt_received = Arc::new(AtomicBool::new(false));
    let path = Path::new(&args[1]);
    let image = file_reader::read_ppm_file(&path);
    let mut frame = Frame::new();
    let mut gpio = GPIO::new(1);
    let timer = Timer::new();
    let int_recv = interrupt_received.clone();

    frame.next_image_frame(&image);
    ctrlc::set_handler(move || {
        int_recv.store(true, Ordering::SeqCst);
    }).unwrap();

    gpio.render_frame(interrupt_received, &image, &mut frame, &timer);
}

fn sanity_check(args: &Vec<String>) {
    if nix::unistd::Uid::current().is_root() == false {
        eprintln!("Must run as root to be able to access /dev/mem\nPrepend \'sudo\' to the command");
        std::process::exit(1);
    } else if args.len() < 2 {
        eprintln!("Syntax: {:?} [image]", args[0]);
        std::process::exit(1);
    }
}

